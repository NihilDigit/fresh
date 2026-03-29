//! Archive extraction: zip, tar.gz, tar.xz, and raw gz.

use anyhow::{bail, Context, Result};
use std::path::Path;

/// Extracts archives in various formats.
pub struct Extractor;

impl Extractor {
    /// Extract an archive to `dest_dir`, stripping `strip_components` leading
    /// path components (like `tar --strip-components`).
    ///
    /// The archive format is detected from the file extension.
    pub fn extract(archive_path: &Path, dest_dir: &Path, strip_components: u32) -> Result<()> {
        std::fs::create_dir_all(dest_dir)
            .with_context(|| format!("Failed to create directory {}", dest_dir.display()))?;

        let name = archive_path.to_string_lossy().to_lowercase();

        if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            Self::extract_tar_gz(archive_path, dest_dir, strip_components)
        } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
            Self::extract_tar_xz(archive_path, dest_dir, strip_components)
        } else if name.ends_with(".zip") {
            Self::extract_zip(archive_path, dest_dir, strip_components)
        } else if name.ends_with(".gz") {
            Self::extract_gz(archive_path, dest_dir)
        } else {
            bail!("Unsupported archive format: {}", archive_path.display());
        }
    }

    fn extract_tar_gz(archive_path: &Path, dest_dir: &Path, strip_components: u32) -> Result<()> {
        let file = std::fs::File::open(archive_path)
            .with_context(|| format!("Failed to open {}", archive_path.display()))?;
        let decoder = flate2::read::GzDecoder::new(file);
        Self::extract_tar(decoder, dest_dir, strip_components)
    }

    fn extract_tar_xz(archive_path: &Path, dest_dir: &Path, strip_components: u32) -> Result<()> {
        let file = std::fs::File::open(archive_path)
            .with_context(|| format!("Failed to open {}", archive_path.display()))?;
        let decoder = xz2::read::XzDecoder::new(file);
        Self::extract_tar(decoder, dest_dir, strip_components)
    }

    fn extract_tar<R: std::io::Read>(
        reader: R,
        dest_dir: &Path,
        strip_components: u32,
    ) -> Result<()> {
        let mut archive = tar::Archive::new(reader);
        for entry in archive.entries().context("Failed to read tar entries")? {
            let mut entry = entry.context("Failed to read tar entry")?;
            let original_path = entry
                .path()
                .context("Failed to read entry path")?
                .into_owned();

            // Strip leading path components
            let stripped: std::path::PathBuf = original_path
                .components()
                .skip(strip_components as usize)
                .collect();

            if stripped.as_os_str().is_empty() {
                continue;
            }

            let dest_path = dest_dir.join(&stripped);

            // Safety: prevent path traversal
            if !dest_path.starts_with(dest_dir) {
                bail!(
                    "Path traversal detected in archive: {}",
                    original_path.display()
                );
            }

            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            entry
                .unpack(&dest_path)
                .with_context(|| format!("Failed to extract {}", original_path.display()))?;
        }
        Ok(())
    }

    fn extract_zip(archive_path: &Path, dest_dir: &Path, strip_components: u32) -> Result<()> {
        let file = std::fs::File::open(archive_path)
            .with_context(|| format!("Failed to open {}", archive_path.display()))?;
        let mut archive = zip::ZipArchive::new(file)
            .with_context(|| format!("Failed to read zip {}", archive_path.display()))?;

        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .with_context(|| format!("Failed to read zip entry {i}"))?;

            let original_path = match entry.enclosed_name() {
                Some(p) => p.to_owned(),
                None => continue, // skip invalid paths
            };

            // Strip leading path components
            let stripped: std::path::PathBuf = original_path
                .components()
                .skip(strip_components as usize)
                .collect();

            if stripped.as_os_str().is_empty() {
                continue;
            }

            let dest_path = dest_dir.join(&stripped);

            // Safety: prevent path traversal
            if !dest_path.starts_with(dest_dir) {
                bail!(
                    "Path traversal detected in archive: {}",
                    original_path.display()
                );
            }

            if entry.is_dir() {
                std::fs::create_dir_all(&dest_path)?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = std::fs::File::create(&dest_path)
                    .with_context(|| format!("Failed to create {}", dest_path.display()))?;
                std::io::copy(&mut entry, &mut outfile)
                    .with_context(|| format!("Failed to extract {}", original_path.display()))?;

                // Set executable permission on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = entry.unix_mode() {
                        std::fs::set_permissions(
                            &dest_path,
                            std::fs::Permissions::from_mode(mode),
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Extract a single gzip-compressed file (e.g., `rust-analyzer-x86_64-unknown-linux-gnu.gz`).
    /// The output file name is the archive name without `.gz`.
    fn extract_gz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
        let file = std::fs::File::open(archive_path)
            .with_context(|| format!("Failed to open {}", archive_path.display()))?;
        let mut decoder = flate2::read::GzDecoder::new(file);

        // Derive output filename from archive name minus .gz
        let stem = archive_path
            .file_stem()
            .context("Archive has no file stem")?;
        let dest_path = dest_dir.join(stem);

        let mut outfile = std::fs::File::create(&dest_path)
            .with_context(|| format!("Failed to create {}", dest_path.display()))?;
        std::io::copy(&mut decoder, &mut outfile)
            .with_context(|| format!("Failed to extract {}", archive_path.display()))?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&dest_path, std::fs::Permissions::from_mode(0o755))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    /// Create a tar.gz archive containing a single file.
    fn create_tar_gz(dir: &Path, archive_name: &str, inner_path: &str, content: &[u8]) -> PathBuf {
        let archive_path = dir.join(archive_name);
        let file = std::fs::File::create(&archive_path).unwrap();
        let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(enc);

        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        builder
            .append_data(&mut header, inner_path, content)
            .unwrap();
        builder.into_inner().unwrap().finish().unwrap();

        archive_path
    }

    /// Create a zip archive containing a single file.
    fn create_zip(dir: &Path, archive_name: &str, inner_path: &str, content: &[u8]) -> PathBuf {
        let archive_path = dir.join(archive_name);
        let file = std::fs::File::create(&archive_path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip_writer.start_file(inner_path, options).unwrap();
        zip_writer.write_all(content).unwrap();
        zip_writer.finish().unwrap();
        archive_path
    }

    /// Create a gz-compressed single file.
    fn create_gz(dir: &Path, archive_name: &str, content: &[u8]) -> PathBuf {
        let archive_path = dir.join(archive_name);
        let file = std::fs::File::create(&archive_path).unwrap();
        let mut enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        enc.write_all(content).unwrap();
        enc.finish().unwrap();
        archive_path
    }

    #[test]
    fn test_extract_tar_gz() {
        let dir = tempfile::tempdir().unwrap();
        let content = b"#!/bin/sh\necho hello\n";
        let archive = create_tar_gz(dir.path(), "tool.tar.gz", "bin/tool", content);

        let dest = dir.path().join("output");
        Extractor::extract(&archive, &dest, 0).unwrap();

        let extracted = std::fs::read(dest.join("bin/tool")).unwrap();
        assert_eq!(extracted, content);
    }

    #[test]
    fn test_extract_tar_gz_strip_components() {
        let dir = tempfile::tempdir().unwrap();
        let content = b"binary data";
        let archive = create_tar_gz(
            dir.path(),
            "tool.tar.gz",
            "gopls-v0.21.1/bin/gopls",
            content,
        );

        let dest = dir.path().join("output");
        Extractor::extract(&archive, &dest, 1).unwrap();

        // After stripping 1 component, "gopls-v0.21.1/bin/gopls" → "bin/gopls"
        let extracted = std::fs::read(dest.join("bin/gopls")).unwrap();
        assert_eq!(extracted, content);
    }

    #[test]
    fn test_extract_zip() {
        let dir = tempfile::tempdir().unwrap();
        let content = b"zip file content";
        let archive = create_zip(dir.path(), "tool.zip", "tool.exe", content);

        let dest = dir.path().join("output");
        Extractor::extract(&archive, &dest, 0).unwrap();

        let extracted = std::fs::read(dest.join("tool.exe")).unwrap();
        assert_eq!(extracted, content);
    }

    #[test]
    fn test_extract_zip_strip_components() {
        let dir = tempfile::tempdir().unwrap();
        let content = b"nested zip content";
        let archive = create_zip(dir.path(), "tool.zip", "prefix/inner/tool.exe", content);

        let dest = dir.path().join("output");
        Extractor::extract(&archive, &dest, 1).unwrap();

        let extracted = std::fs::read(dest.join("inner/tool.exe")).unwrap();
        assert_eq!(extracted, content);
    }

    #[test]
    fn test_extract_gz_single_file() {
        let dir = tempfile::tempdir().unwrap();
        let content = b"rust-analyzer binary";
        let archive = create_gz(dir.path(), "rust-analyzer.gz", content);

        let dest = dir.path().join("output");
        Extractor::extract(&archive, &dest, 0).unwrap();

        // gz extraction uses the archive stem as filename
        let extracted = std::fs::read(dest.join("rust-analyzer")).unwrap();
        assert_eq!(extracted, content);
    }

    #[test]
    fn test_extract_unsupported_format() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("tool.rar");
        std::fs::write(&archive, b"fake").unwrap();

        let dest = dir.path().join("output");
        let result = Extractor::extract(&archive, &dest, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported"));
    }

    #[test]
    fn test_extract_creates_dest_dir() {
        let dir = tempfile::tempdir().unwrap();
        let content = b"data";
        let archive = create_gz(dir.path(), "tool.gz", content);

        let dest = dir.path().join("deeply/nested/output");
        assert!(!dest.exists());

        Extractor::extract(&archive, &dest, 0).unwrap();
        assert!(dest.exists());
    }
}
