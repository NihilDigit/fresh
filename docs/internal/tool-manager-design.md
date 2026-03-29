# Tool Manager: Architecture & UX Specification

Integrated system for discovering, installing, updating, and managing external development tools (LSP servers, formatters, linters, DAP servers) directly from within Fresh.

See also: [tool-manager-research.md](tool-manager-research.md) for prior art analysis.

## Table of Contents

- [1. Design Goals](#1-design-goals)
- [2. Architectural Alternatives](#2-architectural-alternatives)
- [3. System Architecture (Recommended)](#3-system-architecture-recommended)
- [4. Core API (Rust)](#4-core-api-rust)
- [5. Plugin API (TypeScript)](#5-plugin-api-typescript)
- [6. Tool Recipe Format](#6-tool-recipe-format)
- [7. User Flows](#7-user-flows)
- [8. TUI Components](#8-tui-components)
- [9. Config-Driven Installation](#9-config-driven-installation)
- [10. Edge Cases & Error Handling](#10-edge-cases--error-handling)
- [11. Relationship to Existing Systems](#11-relationship-to-existing-systems)

---

## 1. Design Goals

### Sandboxing & Isolation

Tools are installed locally to the Fresh environment, never polluting the global system PATH:

| Platform | Install root |
|---|---|
| Linux/macOS | `~/.local/share/fresh/tools/` |
| Windows | `%LOCALAPPDATA%\fresh\tools\` |

Each tool gets its own versioned directory: `{root}/{tool-name}/{version}/`. A top-level `bin/` directory contains shims (symlinks on Unix, `.cmd` wrappers on Windows) that Fresh prepends to `PATH` when spawning tool processes.

### First-Class Cross-Platform Support

The system abstracts away:
- Path separators (`/` vs `\`)
- Executable extensions (append `.exe` on Windows)
- Execution permissions (`chmod +x` on Unix)
- Archive formats (`.tar.gz`/`.tar.xz` on Unix, `.zip` on Windows)
- Target architecture mapping (Node's `process.arch` → Rust target triples)
- Shell semantics (interactive shell wrapping on Unix, direct exec on Windows)
- Libc variants (glibc vs musl/Alpine)

### Separation of Concerns

**Rust Core** handles the heavy, security-sensitive, platform-specific operations:
- Secure HTTPS downloading with progress reporting
- Checksum and signature verification
- Cross-platform archive extraction (zip, tar.gz, tar.xz)
- Binary shimming (symlinks on Unix, .cmd wrappers on Windows)
- Process spawning with correct platform semantics
- Tool inventory persistence and cleanup

**TypeScript Plugins** provide the "recipes" — declarative metadata that tells the core what to do:
- Tool metadata (name, description, homepage, categories)
- Download URL construction based on OS/arch
- Installation strategy selection (binary download, npm, pip, cargo, go, system)
- Version discovery (query GitHub API, npm registry, etc.)
- Post-install validation

### Extensibility

Third-party plugin authors can add support for new tools by publishing a package containing recipe definitions. No Rust changes required for new tools.

---

## 2. Architectural Alternatives

This section presents fundamentally different approaches, then variations within the recommended approach. Each is evaluated against: complexity, cross-platform reliability, extensibility, and user experience.

### Approach A: Pure Rust Core (Mason-like)

All tool management logic lives in Rust. Recipes are declarative data (YAML/JSON files), not executable code. The Rust core interprets recipes and performs all operations.

```
recipes/gopls.yaml  ──→  Rust recipe parser  ──→  Rust downloader/installer
recipes/ruff.yaml   ──→  Rust recipe parser  ──→  Rust downloader/installer
```

**Tradeoffs:**

| Dimension | Assessment |
|---|---|
| Complexity | High Rust complexity. Every new install strategy (npm, pip, cargo, go, GitHub release) requires new Rust code. |
| Cross-platform | Excellent. All platform logic compiled and tested at build time. No runtime dependency on JS. |
| Extensibility | Low. Adding a tool with a novel install method requires a Fresh release. Third-party authors write YAML, not code. |
| Performance | Best. No JS overhead for installation logic. |
| Maintenance | Heavy. Every upstream change in how tools are distributed may require Rust changes. |
| Prior art | Mason.nvim (Lua, not Rust, but same concept: data-driven recipes interpreted by core). |

**When to choose:** If the tool set is small and stable, or if minimizing runtime dependencies is paramount (e.g., embedded/WASM targets where QuickJS isn't available).

### Approach B: Pure Plugin (Current Marketplace Direction)

All tool management logic lives in TypeScript plugins. The Rust core provides only the primitives it already has (`spawnProcess`, file I/O). The tool manager plugin handles downloading, extraction, shimming — everything.

```
tool-manager.ts plugin  ──→  editor.spawnProcess("curl", [...])
                        ──→  editor.spawnProcess("tar", [...])
                        ──→  fs operations via plugin API
```

**Tradeoffs:**

| Dimension | Assessment |
|---|---|
| Complexity | Low Rust complexity. High JS complexity. |
| Cross-platform | Poor. TypeScript must handle all platform quirks: Windows tar (bsdtar vs GNU), chmod, path separators, .cmd shim generation. The QuickJS sandbox has limited fs/process APIs. |
| Extensibility | Excellent. Anyone can write a plugin. No gatekeeping. |
| Performance | Adequate for installation (not a hot path). |
| Maintenance | Fragile. Platform edge cases accumulate in JS. No type-safe platform abstraction. |
| Prior art | VS Code extensions (each manages its own tools via Node.js APIs). Plugin marketplace design doc. |

**When to choose:** If the plugin API already has rich enough fs/process primitives and you want zero new Rust code.

### Approach C: Hybrid — Rust Core Primitives + TypeScript Recipes (Recommended)

Rust provides a focused set of cross-platform primitives (download, extract, shim, verify). TypeScript plugins provide the recipes (metadata + URL construction logic). The recipe is executable code, not just data — it can query APIs, handle version logic, and adapt to upstream changes.

```
recipe.ts (JS)  ──→  editor.downloadFile(url, ...)   ──→  Rust HTTPS + checksum
                ──→  editor.extractArchive(path, ...) ──→  Rust zip/tar
                ──→  editor.createToolShim(name, ...) ──→  Rust symlink/.cmd
```

**Tradeoffs:**

| Dimension | Assessment |
|---|---|
| Complexity | Moderate in both layers. Rust API is small and focused. JS recipes are straightforward. |
| Cross-platform | Good. Rust handles the hard platform differences. JS never touches chmod, path separators, or .cmd generation. |
| Extensibility | Good. New tools = new JS recipes. New install strategies may need new Rust primitives (but the set is finite). |
| Performance | Same as B (installation is not a hot path). |
| Maintenance | Balanced. Platform bugs are in Rust (compiled, tested). Tool-specific bugs are in JS (easy to update). |
| Prior art | Zed (Rust core + extension-defined language servers). |

**When to choose:** When you want the cross-platform reliability of Rust with the flexibility of scripted recipes. This is the recommended approach and is detailed in the remaining sections.

### Approach D: External Tool Manager (Delegate to Mason/aqua/mise)

Fresh doesn't manage tools at all. Instead, it delegates to an external tool manager and detects tools on PATH.

```
User installs Mason/aqua/mise/asdf externally
Fresh detects tools on PATH at startup
Fresh prompts "gopls not found — install via: mason install gopls"
```

**Tradeoffs:**

| Dimension | Assessment |
|---|---|
| Complexity | Minimal. Fresh only needs PATH detection and documentation. |
| Cross-platform | Depends on external tool. Mason requires Neovim. aqua/mise are standalone but have their own quirks. |
| Extensibility | N/A — delegated entirely. |
| UX | Poor. User must learn and manage a separate tool. Onboarding friction is high. |
| Maintenance | None for Fresh. All burden on external tool. |
| Prior art | Helix (system PATH only). |

**When to choose:** Early in development when the tool manager isn't a priority, or for power users who already have a setup. Could be a Phase 0 stopgap: ship PATH detection and prompts now, build the real tool manager later.

### Variation Matrix (Within Approach C)

Even within the recommended hybrid approach, there are significant design variations:

#### V1: Recipe Format — Executable Code vs Declarative Data

**Option A: Executable recipes (TypeScript functions)** — as shown in the spec below.

```typescript
const gopls: ToolRecipe = {
  getDownloadInfo(version, platform) {
    // arbitrary logic here — can query APIs, handle edge cases
    const target = targetMap[platform.os]?.[platform.arch];
    return { url: `https://.../${target}/...`, ... };
  }
};
```

**Option B: Declarative recipes (JSON/YAML with template expressions)** — Mason-like.

```yaml
name: gopls
source:
  id: pkg:golang/golang.org/x/tools/gopls@v0.21.1
bin:
  gopls: golang:gopls
```

| | Executable (A) | Declarative (B) |
|---|---|---|
| Flexibility | Can handle any edge case | Limited to what the template language supports |
| Validation | Hard to statically validate | Schema-validated, lintable |
| Security | Runs arbitrary JS in sandbox | No code execution — pure data |
| Authoring DX | Familiar (just TypeScript) | New DSL to learn; expression syntax |
| Debugging | Standard JS debugging | Opaque template expansion |
| Machine-readable | Requires executing to inspect | Tools can parse without executing |

**Recommendation:** Executable recipes (Option A). The edge cases in real-world tool distribution (version-dependent URL format changes, platform quirks, API queries for version resolution) make declarative templates brittle. The QuickJS sandbox already provides security isolation.

#### V2: Registry Architecture — Centralized vs Federated vs Embedded

**Option A: Embedded registry** — Built-in recipes ship with Fresh.

- Recipes live in `plugins/fresh-tools/` in the Fresh repo
- Updated with Fresh releases
- Third-party recipes via plugin packages

**Option B: External registry** — Git repo pulled at runtime (Mason-like).

- Recipes live in a separate `fresh-tools-registry` repo
- Fresh fetches/caches the registry periodically
- Decoupled release cycle

**Option C: Fully federated** — No central registry. Only plugin packages.

- Each language community maintains their own recipe plugin
- No single point of failure
- Harder to discover tools

| | Embedded (A) | External Registry (B) | Federated (C) |
|---|---|---|---|
| Freshness | Tied to Fresh releases | Independent update cycle | Community-paced |
| Discovery | Immediate — all built in | Query registry at runtime | Must install recipe plugins first |
| Offline | Works fully offline | Needs initial fetch | Works after plugin install |
| Governance | Fresh team controls | Can accept community PRs | Fully decentralized |
| Bootstrapping | Zero-config | Need registry fetch on first run | Need to know which plugins to install |

**Recommendation:** Start with **Embedded (A)** for the initial release — ship 20-30 common tool recipes built into Fresh. Add **External Registry (B)** as a follow-up when the recipe count grows beyond what's practical to ship. The recipe registration API (`editor.registerToolRecipes`) supports all three models — the source of recipes is orthogonal to the runtime architecture.

#### V3: LSP Integration — Auto-wire vs Manual Config

**Option A: Auto-wire** — Installing a tool automatically configures it as the LSP/formatter for its languages.

- Zero-config experience after install
- May surprise users who want fine-grained control
- Could conflict with user's existing LSP config

**Option B: Suggest-only** — Installing a tool makes it available but doesn't change config. Shows a follow-up prompt: "Add gopls as Go language server? [Yes] [No]"

- User stays in control
- Extra step, more friction
- Safer when user has custom config

**Option C: Auto-wire with override** — Auto-wire into config unless the user has an explicit entry for that language's LSP. If `config.json` already has `"lsp": { "go": { ... } }`, don't touch it.

| | Auto-wire (A) | Suggest-only (B) | Auto-wire + override (C) |
|---|---|---|---|
| Zero-config UX | Best | Worst | Good |
| User control | Low | High | High |
| Surprise factor | High | None | Low |
| Implementation | Simple | Simple | Moderate (must detect user overrides) |

**Recommendation:** **Auto-wire with override (C)**. This gives the zero-config experience for new users while respecting existing configuration. The rule: if the user or project config layer has an explicit LSP entry for that language, the tool manager doesn't modify it.

#### V4: Update Strategy

**Option A: Manual only** — User must open tool manager and press `u` to update.

**Option B: Check on startup, prompt** — Fresh checks for updates on startup (or daily), shows a non-intrusive notification.

**Option C: Auto-update** — Fresh downloads and installs updates automatically in the background.

| | Manual (A) | Check + Prompt (B) | Auto-update (C) |
|---|---|---|---|
| Stability | Best — tools never change unexpectedly | Good — user decides | Risk of breaking changes |
| Freshness | User may run outdated tools | Good balance | Always latest |
| Network usage | None unless user acts | Periodic version checks | Periodic downloads |
| UX friction | Must remember to check | Low — notification is passive | None, but surprises possible |

**Recommendation:** **Check + Prompt (B)** as default, with **Auto-update (C)** as opt-in setting. Check frequency configurable: "daily" (default), "weekly", "never".

#### V5: Tool Storage — Versioned vs Single-Active

**Option A: Versioned** — Multiple versions coexist: `tools/gopls/v0.21.1/`, `tools/gopls/v0.22.0/`. Shim points to active version.

- Supports project-level version pinning
- Higher disk usage
- Rollback is instant (just re-point shim)

**Option B: Single-active** — Only one version at a time: `tools/gopls/`. Update replaces in-place.

- Simpler model, less disk usage
- No version pinning per-project
- Rollback requires re-download

**Recommendation:** **Versioned (A)**. The disk cost is minimal (most tools are <100MB) and version pinning via `.fresh/tools.json` is a key use case for teams.

---

## 3. System Architecture (Recommended)

```
┌──────────────────────────────────────────────────────────────────┐
│                          Fresh Editor                             │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    Tool Manager Service                     │  │
│  │                      (Rust, async)                          │  │
│  │                                                             │  │
│  │  ┌──────────┐ ┌───────────┐ ┌──────────┐ ┌─────────────┐  │  │
│  │  │Downloader│ │ Extractor │ │ Shimmer  │ │  Inventory  │  │  │
│  │  │(reqwest) │ │(zip/tar)  │ │(bin link)│ │  (JSON DB)  │  │  │
│  │  └──────────┘ └───────────┘ └──────────�� └─────────────┘  │  │
│  │                                                             │  │
│  │  ┌────────────────┐  ┌────────────────┐                    │  │
│  │  │Platform Context│  │Process Spawner │                    │  │
│  │  │(os/arch/libc)  │  │(tool execution)│                    │  │
│  │  └────────────��───┘  └────────────────┘                    │  │
│  └──────────────────────┬─────────────────────────────────────┘  │
│                          │ PluginCommand / PluginResponse         │
│  ┌───────────────────────┴────��───────────────────────────────┐  │
│  │                  Plugin Runtime (QuickJS)                    │  │
│  │                                                              │  │
│  │  ┌──────────────────────────────────────────────────────┐   │  │
│  │  │               Tool Recipe Registry                    │   │  │
│  │  │                                                       │   │  │
│  │  │  Built-in recipes (gopls, rust-analyzer, pyright...)  │   │  │
│  │  │  Third-party recipes (via packages)                   │   │  │
│  │  └──────────────────────────────────────────────────────┘   │  │
│  └───────────────────────────────────��─────────────────────────┘  │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │              LspManager / FormatterRunner                    │  │
│  │         (uses shim PATH to find tool binaries)               │  │
│  └���────────────────────────────────────────────────────────────┘  │
└────────────────────────────────��─────────────────────────────────┘
```

### Data Flow: Installing a Tool

1. Plugin resolves recipe for the requested tool
2. Recipe's `getDownloadInfo(platform)` returns URL, checksum, archive type
3. Plugin sends `InstallTool` command to Rust core via `PluginCommand`
4. Rust core downloads, verifies checksum, extracts archive
5. Rust core creates shims in `bin/` directory
6. Rust core updates inventory database
7. Rust core sends `ToolInstalled` response via `PluginResponse`
8. Plugin updates LspManager config if applicable (auto-wires the `command` field)

---

## 4. Core API (Rust)

### Platform Context

Detected once at startup. Available to both Rust and TypeScript.

```rust
/// Compile-time + runtime platform information.
///
/// Exposed to plugins as `editor.platform`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PlatformContext {
    /// Operating system: "linux", "macos", "windows"
    pub os: String,
    /// CPU architecture: "x86_64", "aarch64", "arm", "x86"
    pub arch: String,
    /// C library variant (Linux only): "gnu", "musl", or null
    pub libc: Option<String>,
    /// Executable file extension: "" on Unix, ".exe" on Windows
    pub exe_ext: String,
    /// Archive preference: "tar.gz" on Unix, "zip" on Windows
    pub archive_ext: String,
    /// Rust target triple: e.g. "x86_64-unknown-linux-gnu"
    pub target_triple: String,
}

impl PlatformContext {
    pub fn detect() -> Self {
        let os = if cfg!(target_os = "linux") { "linux" }
            else if cfg!(target_os = "macos") { "macos" }
            else if cfg!(target_os = "windows") { "windows" }
            else { "unknown" };

        let arch = if cfg!(target_arch = "x86_64") { "x86_64" }
            else if cfg!(target_arch = "aarch64") { "aarch64" }
            else if cfg!(target_arch = "arm") { "arm" }
            else if cfg!(target_arch = "x86") { "x86" }
            else { "unknown" };

        let libc = detect_libc(); // reads /etc/os-release for Alpine, or None

        PlatformContext {
            os: os.to_string(),
            arch: arch.to_string(),
            libc,
            exe_ext: if cfg!(windows) { ".exe" } else { "" }.to_string(),
            archive_ext: if cfg!(windows) { "zip" } else { "tar.gz" }.to_string(),
            target_triple: env!("TARGET").to_string(), // set by build.rs
        }
    }
}
```

### New PluginCommand Variants

Added to the existing `PluginCommand` enum in `api.rs`:

```rust
pub enum PluginCommand {
    // ... existing variants ...

    /// Download a file from a URL with progress reporting.
    /// The Rust core handles HTTPS, redirects, timeouts, and retries.
    DownloadFile {
        /// Unique ID for correlating progress events
        download_id: u64,
        /// Source URL (HTTPS required)
        url: String,
        /// Local destination path
        dest: PathBuf,
        /// Expected SHA-256 hex digest (optional but recommended)
        expected_sha256: Option<String>,
        /// Callback ID for completion notification
        callback_id: JsCallbackId,
    },

    /// Extract an archive to a directory.
    /// Supports: .zip, .tar.gz, .tar.xz, .gz (single file)
    ExtractArchive {
        /// Path to the archive file
        archive_path: PathBuf,
        /// Destination directory
        dest_dir: PathBuf,
        /// Strip N leading path components (like tar --strip-components)
        strip_components: u32,
        /// Callback ID for completion notification
        callback_id: JsCallbackId,
    },

    /// Create an executable shim in the tool bin directory.
    /// Unix: creates a symlink. Windows: creates a .cmd wrapper.
    CreateToolShim {
        /// Name of the shim (e.g., "gopls") — extension added automatically
        shim_name: String,
        /// Absolute path to the actual executable
        target_path: PathBuf,
    },

    /// Remove a tool's shim and optionally its installation directory.
    RemoveTool {
        tool_name: String,
        version: String,
        callback_id: JsCallbackId,
    },

    /// Set file as executable (chmod +x). No-op on Windows.
    SetExecutable {
        path: PathBuf,
    },

    /// Register a tool in the inventory database.
    RegisterToolInstallation {
        tool_name: String,
        version: String,
        install_dir: PathBuf,
        installed_by: String, // recipe name
    },

    /// Query the inventory for installed tools.
    GetInstalledTools {
        callback_id: JsCallbackId,
    },
}
```

### New PluginResponse Variants

```rust
pub enum PluginResponse {
    // ... existing variants ...

    /// Download progress update (fired multiple times during download)
    DownloadProgress {
        download_id: u64,
        bytes_downloaded: u64,
        total_bytes: Option<u64>,
    },

    /// Download completed
    DownloadComplete {
        request_id: u64,
        result: Result<PathBuf, String>,
    },

    /// Archive extraction completed
    ExtractComplete {
        request_id: u64,
        result: Result<(), String>,
    },

    /// Tool removal completed
    ToolRemoved {
        request_id: u64,
        result: Result<(), String>,
    },

    /// Installed tools query result
    InstalledTools {
        request_id: u64,
        tools: Vec<InstalledToolInfo>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct InstalledToolInfo {
    pub name: String,
    pub version: String,
    pub install_dir: String,
    pub installed_by: String,
    pub installed_at: String, // ISO 8601
    pub shim_path: Option<String>,
}
```

### Tool Inventory

Persisted as JSON at `{tools_root}/inventory.json`:

```json
{
  "version": 1,
  "tools": {
    "gopls": {
      "version": "0.21.1",
      "install_dir": "/home/user/.local/share/fresh/tools/gopls/0.21.1",
      "installed_by": "fresh-tools-go",
      "installed_at": "2026-03-30T14:22:00Z",
      "shim": "gopls"
    },
    "rust-analyzer": {
      "version": "2026-03-23",
      "install_dir": "/home/user/.local/share/fresh/tools/rust-analyzer/2026-03-23",
      "installed_by": "fresh-tools-core",
      "installed_at": "2026-03-28T09:15:00Z",
      "shim": "rust-analyzer"
    }
  }
}
```

### Shim Implementation

**Unix** — Relative symlink:
```
~/.local/share/fresh/tools/bin/gopls → ../gopls/0.21.1/gopls
```

**Windows** — `.cmd` wrapper (same pattern as Mason/npm):
```batch
@ECHO off
SETLOCAL
SET dp0=%~dp0
"%dp0%..\gopls\0.21.1\gopls.exe" %*
```

When spawning LSP servers or formatters, Fresh prepends `{tools_root}/bin` to the child process `PATH`. This means tool commands "just work" without absolute paths in config.

---

## 5. Plugin API (TypeScript)

### Platform Context (read-only)

```typescript
/** Available as editor.platform — detected once at startup. */
interface PlatformContext {
  /** "linux" | "macos" | "windows" */
  os: "linux" | "macos" | "windows";
  /** "x86_64" | "aarch64" | "arm" | "x86" */
  arch: "x86_64" | "aarch64" | "arm" | "x86";
  /** "gnu" | "musl" | null (Linux only) */
  libc: "gnu" | "musl" | null;
  /** "" on Unix, ".exe" on Windows */
  exeExt: string;
  /** "tar.gz" on Unix, "zip" on Windows */
  archiveExt: string;
  /** e.g. "x86_64-unknown-linux-gnu" */
  targetTriple: string;
}
```

### Tool Recipe Interface

The heart of the plugin API. Each recipe describes how to install one tool across all supported platforms.

```typescript
/** A tool category that determines how Fresh integrates with it. */
type ToolCategory = "lsp" | "formatter" | "linter" | "dap" | "runtime";

/** Installation strategy — determines what the Rust core needs to do. */
type InstallStrategy =
  | { type: "binary"; download: PlatformDownload }
  | { type: "npm"; package: string; binaries: string[] }
  | { type: "pip"; package: string; binaries: string[]; extras?: string[] }
  | { type: "cargo"; crate: string; binaries: string[] }
  | { type: "go"; module: string; binaries: string[] }
  | { type: "system" };  // rely on system PATH, no installation

/**
 * Platform-specific download descriptor.
 *
 * Recipes return this from getDownloadInfo(). The Rust core uses it
 * to download, verify, and extract the correct binary.
 */
interface PlatformDownload {
  /** Full download URL for this platform */
  url: string;
  /** Expected SHA-256 hex digest (strongly recommended) */
  sha256?: string;
  /** Archive type: "tar.gz", "tar.xz", "zip", "gz", "raw" */
  archiveType: "tar.gz" | "tar.xz" | "zip" | "gz" | "raw";
  /** Path to the executable within the extracted archive */
  binaryPath: string;
  /** Strip N leading path components during extraction */
  stripComponents?: number;
}

/**
 * A tool recipe — the declarative spec for installing one tool.
 *
 * Plugin authors implement this interface to add support for a tool.
 * The tool manager calls these methods; the Rust core executes the
 * actual download/extract/shim operations.
 */
interface ToolRecipe {
  /** Unique identifier (lowercase, hyphens) */
  name: string;
  /** Human-readable name */
  displayName: string;
  /** Short description */
  description: string;
  /** Homepage URL */
  homepage: string;
  /** Tool categories */
  categories: ToolCategory[];
  /** Languages this tool serves (e.g., ["go", "gomod"]) */
  languages: string[];

  /**
   * Determine the installation strategy for the current platform.
   *
   * Return null if this tool doesn't support the current platform.
   * The platform parameter is always editor.platform.
   */
  getInstallStrategy(platform: PlatformContext): InstallStrategy | null;

  /**
   * Query the latest available version.
   *
   * Implementations typically hit GitHub API, npm registry, etc.
   * Return null if version discovery is unavailable (e.g., offline).
   */
  getLatestVersion(): Promise<string | null>;

  /**
   * For the "binary" strategy: resolve the download info for a specific version.
   *
   * This is where OS/arch-specific URL construction happens.
   */
  getDownloadInfo?(version: string, platform: PlatformContext): PlatformDownload | null;

  /**
   * Validate that the tool is working after installation.
   *
   * Typically runs `tool --version` and checks the output.
   * Return the detected version string, or null on failure.
   */
  validate?(installDir: string): Promise<string | null>;

  /**
   * LSP integration: return the config to auto-wire into Fresh's LSP system.
   *
   * Only applicable when categories includes "lsp".
   */
  getLspConfig?(installDir: string): LspAutoConfig | null;
}

interface LspAutoConfig {
  /** The command to start the LSP (relative to install dir or just the shim name) */
  command: string;
  /** Arguments */
  args?: string[];
  /** Initialization options */
  initializationOptions?: Record<string, unknown>;
  /** Root markers for workspace detection */
  rootMarkers?: string[];
}
```

### Example Recipe: gopls

```typescript
const gopls: ToolRecipe = {
  name: "gopls",
  displayName: "gopls",
  description: "Official Go language server",
  homepage: "https://github.com/golang/tools/tree/master/gopls",
  categories: ["lsp"],
  languages: ["go", "gomod", "gowork", "gotmpl"],

  getInstallStrategy(platform) {
    // gopls is installed via `go install` — requires Go toolchain
    return {
      type: "go",
      module: "golang.org/x/tools/gopls",
      binaries: ["gopls"],
    };
  },

  async getLatestVersion() {
    // Query the Go module proxy for latest version
    const resp = await editor.fetch(
      "https://proxy.golang.org/golang.org/x/tools/gopls/@latest"
    );
    if (!resp.ok) return null;
    const data = JSON.parse(resp.body);
    return data.Version; // e.g. "v0.21.1"
  },

  async validate(installDir) {
    const result = await editor.spawnProcess("gopls", ["version"], { cwd: installDir });
    if (result.exitCode !== 0) return null;
    const match = result.stdout.match(/v(\d+\.\d+\.\d+)/);
    return match ? match[1] : null;
  },

  getLspConfig() {
    return {
      command: "gopls",  // shim in bin/ dir
      rootMarkers: ["go.mod", "go.work"],
    };
  },
};
```

### Example Recipe: rust-analyzer (binary download)

```typescript
const rustAnalyzer: ToolRecipe = {
  name: "rust-analyzer",
  displayName: "rust-analyzer",
  description: "Rust language server",
  homepage: "https://rust-analyzer.github.io/",
  categories: ["lsp"],
  languages: ["rust"],

  getInstallStrategy(platform) {
    return {
      type: "binary",
      download: this.getDownloadInfo!("latest", platform)!,
    };
  },

  async getLatestVersion() {
    const resp = await editor.fetch(
      "https://api.github.com/repos/rust-lang/rust-analyzer/releases/latest"
    );
    if (!resp.ok) return null;
    const data = JSON.parse(resp.body);
    return data.tag_name; // e.g. "2026-03-23"
  },

  getDownloadInfo(version, platform) {
    // Map Fresh platform to rust-analyzer's naming convention
    const targetMap: Record<string, Record<string, string>> = {
      linux: {
        x86_64: "x86_64-unknown-linux-gnu",
        aarch64: "aarch64-unknown-linux-gnu",
      },
      macos: {
        x86_64: "x86_64-apple-darwin",
        aarch64: "aarch64-apple-darwin",
      },
      windows: {
        x86_64: "x86_64-pc-windows-msvc",
        aarch64: "aarch64-pc-windows-msvc",
      },
    };

    const target = targetMap[platform.os]?.[platform.arch];
    if (!target) return null;

    const ext = platform.os === "windows" ? ".zip" : ".gz";
    const binName = `rust-analyzer-${target}`;
    const url = `https://github.com/rust-lang/rust-analyzer/releases/download/${version}/${binName}${ext}`;

    return {
      url,
      archiveType: platform.os === "windows" ? "zip" : "gz",
      binaryPath: platform.os === "windows" ? "rust-analyzer.exe" : binName,
    };
  },

  async validate(installDir) {
    const result = await editor.spawnProcess(
      "rust-analyzer", ["--version"], { cwd: installDir }
    );
    if (result.exitCode !== 0) return null;
    return result.stdout.trim();
  },

  getLspConfig() {
    return {
      command: "rust-analyzer",
      rootMarkers: ["Cargo.toml", "rust-project.json"],
    };
  },
};
```

### Example Recipe: typescript-language-server (npm)

```typescript
const tsserver: ToolRecipe = {
  name: "typescript-language-server",
  displayName: "TypeScript Language Server",
  description: "TypeScript/JavaScript language server wrapping tsserver",
  homepage: "https://github.com/typescript-language-server/typescript-language-server",
  categories: ["lsp"],
  languages: ["typescript", "javascript", "typescriptreact", "javascriptreact"],

  getInstallStrategy(platform) {
    return {
      type: "npm",
      package: "typescript-language-server",
      binaries: ["typescript-language-server"],
    };
  },

  async getLatestVersion() {
    const resp = await editor.fetch(
      "https://registry.npmjs.org/typescript-language-server/latest"
    );
    if (!resp.ok) return null;
    const data = JSON.parse(resp.body);
    return data.version;
  },

  getLspConfig() {
    return {
      command: "typescript-language-server",
      args: ["--stdio"],
      rootMarkers: ["tsconfig.json", "jsconfig.json", "package.json"],
    };
  },
};
```

### Recipe Registration

Plugins register recipes during initialization:

```typescript
// In a tool recipe plugin's entry point:
editor.registerToolRecipes([gopls, rustAnalyzer, tsserver, /* ... */]);
```

The built-in `fresh-tools` plugin ships with recipes for ~30 common tools. Third-party packages can register additional recipes via the same API.

---

## 6. Tool Recipe Format

### Built-in Recipe Plugin Structure

```
plugins/fresh-tools/
├── package.json
├── plugin.ts          # registers all recipes
├── recipes/
│   ├── go.ts          # gopls, gofumpt, delve
│   ├── rust.ts        # rust-analyzer
���   ├── typescript.ts  # typescript-language-server, prettier, eslint
│   ├── python.ts      # pyright, ruff, black
│   ├── c-cpp.ts       # clangd, clang-format
│   ├── lua.ts         # lua-language-server
│   ├── zig.ts         # zls
│   └── ...
└── lib/
    └── github.ts      # shared: GitHub Release version discovery
```

### Third-party Recipe Packages

A third-party package providing tool recipes:

```json
{
  "name": "fresh-tools-ruby",
  "type": "plugin",
  "fresh": {
    "entry": "plugin.ts",
    "min_api_version": 2
  }
}
```

```typescript
// plugin.ts
import { rubyLsp, solargraph, rubocop } from "./recipes";
editor.registerToolRecipes([rubyLsp, solargraph, rubocop]);
```

---

## 7. User Flows

### Flow 1: Automatic Discovery

User opens a `.go` file on a fresh machine.

```
1. User opens main.go
2. LspManager detects language "go"
3. LspManager checks config → no LSP configured for "go"
4. LspManager queries tool registry → finds "gopls" recipe with language "go"
5. Tool manager checks inventory → gopls not installed
6. Editor shows inline notification:

   ┌─────────────────────────────────────────────────────────┐
   │  Go language server (gopls) is not installed.            │
   │  [Install]  [Configure Manually]  [Dismiss]             │
   └─────────────────────────────────────────────────────────┘

7. User presses [Install] (or hits Enter — Install is focused by default)
8. Recipe's getInstallStrategy() returns { type: "go", ... }
9. Rust core checks: is `go` available on PATH?
   - YES: Runs `go install golang.org/x/tools/gopls@v0.21.1`
          in the tool's install directory with GOBIN set
   - NO:  Shows error: "Go toolchain required. Install Go from https://go.dev"
10. On success:
    - Creates shim: bin/gopls → ../gopls/v0.21.1/gopls
    - Updates inventory.json
    - Auto-wires LSP config via getLspConfig()
    - Starts gopls for the open buffer
    - Status bar shows: "gopls v0.21.1 ✓"

11. If multiple tools match (e.g., gopls + gofumpt):
    Shows grouped prompt:

   ┌─────────────────────────────────────────────────────────┐
   │  Recommended tools for Go:                               │
   │                                                          │
   │  ☑ gopls       Language server          [not installed]  │
   │  ☑ gofumpt     Formatter                [not installed]  │
   │  ☐ delve       Debugger                 [not installed]  │
   │                                                          │
   │  [Install Selected]  [Skip]                              │
   └────────────────���────────────────────────────────────────┘
```

If the user selects [Dismiss], the tool is added to a "declined" list stored in the user config. Fresh will not prompt again for that tool unless the user explicitly opens the Tool Manager.

### Flow 2: Manual Management (Tool Manager TUI)

User opens the tool dashboard via command palette or keybinding.

**Command:** `Tool Manager: Open` (default: unbound, available in palette)

```
┌─ Tool Manager ──────────────────────────────────────────────────────┐
│                                                                      │
│  Filter: [_______________]             [Installed ▾]  [All ▾]       │
│                                                                      │
│  ── Installed ───────────────────────────────────────────────────── │
│                                                                      │
│  ● rust-analyzer    LSP       2026-03-23    ✓ up to date            │
│  ● gopls            LSP       v0.21.1       ⬆ v0.22.0 available    │
│  ● pyright          LSP       1.1.408       ✓ up to date            │
│  ● ruff             Linter    0.15.7        ✓ up to date            │
│                                                                      │
│  ── Available ───────────────────────────────────────────────────── │
│                                                                      │
│  ○ clangd           LSP       C, C++                                │
│  ○ lua-language-server LSP    Lua                                   │
│  ○ zls              LSP       Zig                                   │
│  ○ prettier         Formatter JS, TS, CSS, HTML                     │
│  ○ black            Formatter Python                                │
│  ○ delve            DAP       Go                                    │
│                                                                      │
│  [i]nstall  [u]pdate  [U]pdate all  [x] uninstall  [?] help        │
└──────────────────────────────────────────────────────────────────────┘
```

**Keystrokes:**

| Key | Action |
|---|---|
| `j`/`k` or `↑`/`↓` | Navigate list |
| `Enter` or `i` | Install selected tool |
| `u` | Update selected tool |
| `U` | Update all installed tools |
| `x` | Uninstall selected tool |
| `/` | Focus filter input |
| `Tab` | Toggle Installed/Available sections |
| `q` or `Esc` | Close tool manager |
| `?` | Show help |

**Detail View** — pressing `Enter` on an installed tool expands inline:

```
│  ▼ gopls            LSP       v0.21.1       ⬆ v0.22.0 available    │
│    Language server for Go                                            │
│    Homepage: https://github.com/golang/tools/tree/master/gopls       │
│    Installed: 2026-03-28 09:15 UTC                                   │
│    Location: ~/.local/share/fresh/tools/gopls/v0.21.1/               │
│    Strategy: go install                                              │
│                                                                      │
│    [u]pdate to v0.22.0  [x] uninstall  [c]opy path                  │
```

**Installation Progress:**

When a tool is being installed, the list item shows a spinner and progress:

```
│  ⠋ rust-analyzer    LSP       Downloading... 45% (12.3/27.1 MB)     │
```

Then on completion:
```
│  ● rust-analyzer    LSP       2026-03-23    ✓ installed              │
```

### Flow 3: Headless / Config-Driven

A project contains `.fresh/tools.json`:

```json
{
  "tools": {
    "gopls": { "version": "v0.21.1" },
    "gofumpt": {},
    "delve": { "version": "v1.24.0" }
  }
}
```

When a user opens this project:

```
1. Fresh reads .fresh/tools.json
2. Compares against inventory → identifies missing/outdated tools
3. Shows notification:

   ┌─────────────────────────────────────────────────────────┐
   │  This project requires 3 tools (2 not installed):        │
   │                                                          │
   │  ✓ gopls v0.21.1          already installed              │
   │  ✗ gofumpt                not installed                  │
   │  ✗ delve v1.24.0          not installed                  │
   │                                                          │
   │  [Install Missing]  [Skip]  [Always Skip for Project]   │
   └─────────────────────────────────────────────────────────┘

4. On [Install Missing]: installs gofumpt and delve in parallel
5. Progress shown in status bar: "Installing tools: 2/3 ⠋"
```

The `tools.json` format is deliberately OS-agnostic — the recipe's `getInstallStrategy()` handles platform differences. The same file works on macOS, Linux, and Windows without modification.

**Generating tools.json from current state:**

Command: `Tool Manager: Export Project Tools`

Writes the current installed tools relevant to the project's languages to `.fresh/tools.json`, ready to commit.

---

## 8. TUI Components

### Notification Bar (Inline)

Appears at the top of the editor area, below the menu bar. Auto-dismisses after action or timeout.

```
┌─ editor area ──────────────────────────────────────────────────────┐
│ ┌────────────────────────────────────────────────────────────────┐ │
│ │ ℹ  gopls is not installed. [Install] [Dismiss]                 │ │
│ └────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  1  package main                                                    │
│  2  ...                                                             │
```

Reuses the existing popup system with `PopupKind::Action` and `PopupPosition::Centered` or a new notification area.

### Progress Indicator

Displayed in the status bar during tool installation:

```
── status bar ─────────────────────────────────────────────────────
  main.go  [Go]  gopls ⠋ installing...  Ln 42, Col 8  UTF-8
```

For multi-tool installs:

```
  Installing tools: gofumpt ⠋  (2/3)
```

### Tool Manager Panel

Full-screen overlay (like the existing settings panel), rendered using the existing control patterns:
- `TextInput` for the filter field
- `Dropdown` for category filter
- Custom list component with scroll, selection highlighting
- Inline expansion for detail view

---

## 9. Config-Driven Installation

### `.fresh/tools.json` Schema

```json
{
  "$schema": "https://fresh.dev/schemas/tools.json",
  "tools": {
    "<tool-name>": {
      "version": "<optional: pin to specific version>",
      "enabled": true
    }
  }
}
```

Minimal example:
```json
{
  "tools": {
    "gopls": {},
    "gofumpt": {}
  }
}
```

With version pinning:
```json
{
  "tools": {
    "rust-analyzer": { "version": "2026-03-23" },
    "clippy": {}
  }
}
```

### Integration with Config Layers

`.fresh/tools.json` is read at the **project config layer** — it does not override user-level tool installations, only ensures the listed tools are present. If a user already has a newer version of a pinned tool, the pinned version is installed alongside (version isolation).

### Automatic Bootstrapping

When `tools.json` is detected on project open:

1. Check if all listed tools are installed at the required versions
2. If any are missing, prompt the user (never install silently)
3. If the user opts in, install in parallel with progress
4. Store the user's choice ("always install for this project" / "never ask") in the session layer

---

## 10. Edge Cases & Error Handling

### Network Failures

- Downloads use exponential backoff with 3 retries
- Partial downloads are cleaned up (no corrupt files left behind)
- If offline, show: "Network unavailable. Tools can be installed when connected."
- Cached version info (from last successful query) used for offline display

### Corrupted Downloads

- SHA-256 verification after download (when checksum provided in recipe)
- Post-extract validation: run `tool --version` to confirm the binary works
- If validation fails: delete the extracted files, show error with the tool's homepage for manual installation

### Missing Runtime Dependencies

When a recipe requires a runtime (npm, pip, cargo, go) that isn't installed:

```
┌─────────────────────────────────────────────────────────────┐
│  Cannot install typescript-language-server:                   │
│  Node.js is required but was not found on PATH.              │
│                                                              │
│  Install Node.js: https://nodejs.org/                        │
│  Or specify path: Settings → Tools → Node.js Path            │
│                                                              │
│  [Open Settings]  [Dismiss]                                  │
└─────────────────────────────────────────────────────────────┘
```

The tool manager checks for runtime availability **before** starting installation, not after. Runtime paths can be configured in user settings:

```json
{
  "tools": {
    "runtimes": {
      "node": "/usr/local/bin/node",
      "go": "/usr/local/go/bin/go",
      "python3": "/usr/bin/python3",
      "cargo": "~/.cargo/bin/cargo"
    }
  }
}
```

### Unsupported Platform/Architecture

If a recipe's `getInstallStrategy()` returns `null`:

```
┌─────────────────────────────────────────────────────────────┐
│  clangd does not provide a pre-built binary for             │
│  linux/arm (32-bit ARM).                                     │
│                                                              │
│  You can install it manually from your system package        │
│  manager and set the path in config.json.                    │
│                                                              │
│  [Open Documentation]  [Dismiss]                             │
└─────────────────���───────────────────────────────────────────┘
```

### Concurrent Installs

- Multiple tools can install in parallel (each gets its own directory)
- Same tool cannot be installed twice concurrently (deduplicated by name, like vscode-zig's pattern)
- Progress for each tool tracked independently

### Disk Space

- Before downloading, check available disk space against `Content-Length`
- If insufficient: "Not enough disk space. Need ~27 MB, have ~12 MB free in ~/.local/share/fresh/tools/"

### Stale Shims

On startup, the tool manager validates that all shims point to existing binaries. Broken shims (e.g., after manual deletion of a tool directory) are removed and the inventory updated.

### Permission Errors

- On Unix: if `chmod +x` fails, show the error and suggest checking directory permissions
- On Windows: if `.cmd` wrapper creation fails, suggest running Fresh as administrator or checking antivirus

### Version Conflicts

When `.fresh/tools.json` pins a version that differs from the user's installed version:
- Install the pinned version in a separate directory
- Point the project-level shim to the pinned version
- User's global installation remains untouched

---

## 11. Relationship to Existing Systems

### Existing Package System

The tool manager **complements** the existing package system (`services/packages.rs`), not replaces it:

| Concern | Package System | Tool Manager |
|---|---|---|
| **What** | Fresh plugins, themes, grammars, language packs | External binaries (LSP servers, formatters, linters) |
| **Format** | `package.json` with `fresh` block | Tool recipes (TypeScript interfaces) |
| **Distribution** | Git repos (per marketplace design) | Direct download, npm, pip, cargo, go |
| **Storage** | `~/.config/fresh/{plugins,languages,bundles}/` | `~/.local/share/fresh/tools/` |
| **Runtime** | QuickJS (TypeScript) | Native executables |

Language packs can declare an LSP `command` in their manifest. If that command is not found, the tool manager steps in to offer installation.

### Existing LspManager

The tool manager wires into `LspManager` via the existing `LspServerConfig`:

1. Tool recipe provides `getLspConfig()` → returns command, args, rootMarkers
2. Tool manager writes this into the user config's `lsp` section (or injects it at runtime)
3. LspManager picks it up on next config reload
4. LspManager spawns the server using the shim in `bin/`

No changes to `LspManager`'s spawning logic are needed — it already uses `tokio::process::Command` with configurable `command` and `args`. The only addition is prepending the tool `bin/` directory to the child process PATH.

### Existing Plugin Marketplace Design

The plugin marketplace (`plugin-marketplace-design.md`) uses git-based distribution for plugins/themes. Tool recipes are **delivered via this same mechanism** — a recipe plugin is a regular Fresh plugin that calls `editor.registerToolRecipes()`. The tool manager's Rust core handles the actual binary installation after the recipe tells it what to do.

### Config Integration

Tool manager settings live in the existing config hierarchy:

```json
{
  "tools": {
    "auto_install_prompt": true,
    "check_updates": "daily",
    "install_root": null,
    "declined": ["delve", "golangci-lint"],
    "runtimes": {
      "node": null,
      "go": null,
      "python3": null,
      "cargo": null
    }
  }
}
```

These follow the existing 4-layer config merge (system → user → project → session).
