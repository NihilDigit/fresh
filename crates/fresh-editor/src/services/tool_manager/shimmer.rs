//! Binary shimming: creates lightweight launchers for installed tools.
//!
//! - **Unix**: relative symlinks in `bin/`
//! - **Windows**: `.cmd` wrapper scripts

use anyhow::{Context, Result};
use std::path::Path;

/// Creates and removes tool shims.
pub struct Shimmer;

impl Shimmer {
    /// Create a shim named `shim_name` that points to `target_path`.
    ///
    /// On Unix, this creates a symlink: `bin/<shim_name> → <target_path>`
    /// On Windows, this creates a `.cmd` wrapper: `bin\<shim_name>.cmd`
    pub fn create(bin_dir: &Path, shim_name: &str, target_path: &Path) -> Result<()> {
        std::fs::create_dir_all(bin_dir)
            .with_context(|| format!("Failed to create bin directory {}", bin_dir.display()))?;

        #[cfg(unix)]
        {
            Self::create_unix_shim(bin_dir, shim_name, target_path)
        }

        #[cfg(windows)]
        {
            Self::create_windows_shim(bin_dir, shim_name, target_path)
        }
    }

    /// Remove a shim by name.
    pub fn remove(bin_dir: &Path, shim_name: &str) -> Result<()> {
        #[cfg(unix)]
        {
            let shim_path = bin_dir.join(shim_name);
            if shim_path.exists() || shim_path.symlink_metadata().is_ok() {
                std::fs::remove_file(&shim_path)
                    .with_context(|| format!("Failed to remove shim {}", shim_path.display()))?;
            }
        }

        #[cfg(windows)]
        {
            let shim_path = bin_dir.join(format!("{shim_name}.cmd"));
            if shim_path.exists() {
                std::fs::remove_file(&shim_path)
                    .with_context(|| format!("Failed to remove shim {}", shim_path.display()))?;
            }
        }

        Ok(())
    }

    #[cfg(unix)]
    fn create_unix_shim(bin_dir: &Path, shim_name: &str, target_path: &Path) -> Result<()> {
        let shim_path = bin_dir.join(shim_name);

        // Remove existing shim if present
        if shim_path.exists() || shim_path.symlink_metadata().is_ok() {
            std::fs::remove_file(&shim_path).with_context(|| {
                format!("Failed to remove existing shim {}", shim_path.display())
            })?;
        }

        // Create symlink (use absolute path for reliability)
        std::os::unix::fs::symlink(target_path, &shim_path).with_context(|| {
            format!(
                "Failed to create symlink {} → {}",
                shim_path.display(),
                target_path.display()
            )
        })?;

        Ok(())
    }

    #[cfg(windows)]
    fn create_windows_shim(bin_dir: &Path, shim_name: &str, target_path: &Path) -> Result<()> {
        let shim_path = bin_dir.join(format!("{shim_name}.cmd"));
        let target_str = target_path.to_string_lossy();

        let content = format!("@ECHO off\r\nSETLOCAL\r\n\"{}\" %*\r\n", target_str);

        std::fs::write(&shim_path, content)
            .with_context(|| format!("Failed to write shim {}", shim_path.display()))?;

        Ok(())
    }

    /// Set a file as executable (chmod +x). No-op on Windows.
    pub fn set_executable(path: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(path)
                .with_context(|| format!("Failed to read metadata for {}", path.display()))?;
            let mut perms = metadata.permissions();
            let mode = perms.mode() | 0o111; // Add execute for user/group/other
            perms.set_mode(mode);
            std::fs::set_permissions(path, perms).with_context(|| {
                format!("Failed to set executable permission on {}", path.display())
            })?;
        }
        #[cfg(windows)]
        {
            let _ = path; // no-op
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_remove_shim() {
        let dir = tempfile::tempdir().unwrap();
        let bin_dir = dir.path().join("bin");
        let target = dir.path().join("tools/gopls/v0.21.1/gopls");

        // Create a fake target binary
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, b"fake binary").unwrap();

        // Create shim
        Shimmer::create(&bin_dir, "gopls", &target).unwrap();

        #[cfg(unix)]
        {
            let shim_path = bin_dir.join("gopls");
            assert!(shim_path.symlink_metadata().is_ok(), "shim should exist");
            let link_target = std::fs::read_link(&shim_path).unwrap();
            assert_eq!(link_target, target);
        }

        #[cfg(windows)]
        {
            let shim_path = bin_dir.join("gopls.cmd");
            assert!(shim_path.exists(), "shim .cmd should exist");
            let content = std::fs::read_to_string(&shim_path).unwrap();
            assert!(content.contains("gopls"));
        }

        // Remove shim
        Shimmer::remove(&bin_dir, "gopls").unwrap();

        #[cfg(unix)]
        assert!(!bin_dir.join("gopls").exists());
        #[cfg(windows)]
        assert!(!bin_dir.join("gopls.cmd").exists());
    }

    #[test]
    fn test_create_shim_replaces_existing() {
        let dir = tempfile::tempdir().unwrap();
        let bin_dir = dir.path().join("bin");

        let target_v1 = dir.path().join("v1/tool");
        let target_v2 = dir.path().join("v2/tool");
        std::fs::create_dir_all(target_v1.parent().unwrap()).unwrap();
        std::fs::create_dir_all(target_v2.parent().unwrap()).unwrap();
        std::fs::write(&target_v1, b"v1").unwrap();
        std::fs::write(&target_v2, b"v2").unwrap();

        Shimmer::create(&bin_dir, "tool", &target_v1).unwrap();
        Shimmer::create(&bin_dir, "tool", &target_v2).unwrap();

        #[cfg(unix)]
        {
            let link_target = std::fs::read_link(bin_dir.join("tool")).unwrap();
            assert_eq!(link_target, target_v2);
        }
    }

    #[test]
    fn test_remove_nonexistent_shim_is_ok() {
        let dir = tempfile::tempdir().unwrap();
        // Should not error
        Shimmer::remove(dir.path(), "nonexistent").unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn test_set_executable() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("tool");
        std::fs::write(&file, b"binary").unwrap();

        // Start with no execute permission
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();

        Shimmer::set_executable(&file).unwrap();

        let mode = std::fs::metadata(&file).unwrap().permissions().mode();
        assert!(mode & 0o111 != 0, "file should be executable");
    }

    #[test]
    fn test_create_shim_creates_bin_dir() {
        let dir = tempfile::tempdir().unwrap();
        let bin_dir = dir.path().join("deeply/nested/bin");
        let target = dir.path().join("tool");
        std::fs::write(&target, b"binary").unwrap();

        assert!(!bin_dir.exists());
        Shimmer::create(&bin_dir, "tool", &target).unwrap();
        assert!(bin_dir.exists());
    }
}
