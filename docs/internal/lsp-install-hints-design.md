# LSP Install Hints — Design Note (Priority 1)

Status: accepted. Scope: ship in the same branch as PR #1558's follow-up
(`claude/lsp-statusbar-research-0bLSe`).

## Problem

The LSP status popup already detects "binary not on `$PATH`" and replaces
the actionable "Start …" row with a disabled `Install <name> to enable`
advisory. Users see the dead end but are given no way to act on it
unless a language plugin is loaded AND the plugin happened to be
triggered by a prior spawn failure. Today:

- Plugins (`python-lsp.ts`, `typescript-lsp.ts`, and ~13 others) each
  hardcode their own install copy and surface it through a separate
  `showActionPopup` flow, driven by `lsp_server_error` after a failed
  spawn — not by the pre-click probe.
- Users running without the relevant plugin, or who never clicked
  "Start", never see install help.
- Install knowledge is already in the codebase, but as **comments** in
  `config.rs::populate_lsp_config` ("installed via pip", "installed via
  npm install -g vscode-langservers-extracted", …) — close to the
  config itself, but not machine-readable.

## Goals

1. Make install help a first-class citizen of the status popup so
   it's visible immediately on the "binary not in PATH" path, no
   plugin required.
2. Let users override / extend hints through their `config.json`,
   same as any other LSP server field.
3. Minimise plugin-compat disruption: existing `lsp_server_error`
   handlers keep working; new popup rows are additive.
4. Don't regress WASM (`cargo check --no-default-features`).

## Non-goals

- Rewriting the `*-lsp.ts` plugins. Convergence with plugins is
  deferred (cross-cutting nit; see `lsp-status-research-notes.md`).
- Executing install commands for the user. Copy-to-clipboard only —
  the editor doesn't run arbitrary shell as privileged operations.
- URL-opening in-browser. Out of scope for this pass; copying the
  docs URL is enough.

## Design

### 1. Data model (config-first)

New field on `LspServerConfig`:

```rust
#[serde(default)]
pub install_hints: Vec<LspInstallHint>,
```

```rust
pub struct LspInstallHint {
    /// Short label — what package manager or source this hint uses
    /// ("pipx", "pip", "npm", "brew", "docs"). Surfaced in the row.
    pub label: String,

    /// Copyable text. Typically a shell command, but may be a URL
    /// (when `label == "docs"`). The popup copies this verbatim.
    pub command: String,

    /// Platform filter. `None` means "show on all platforms".
    /// The popup prefers filtered matches and falls back to "show all"
    /// if nothing on the current OS matched.
    #[serde(default)]
    pub platform: Option<InstallPlatform>,
}

#[serde(rename_all = "lowercase")]
pub enum InstallPlatform { Linux, Macos, Windows }
```

**Why on `LspServerConfig` and not a side registry:**

- Built-in defaults travel with the config used by `config::default`.
  No plugin load, no runtime registry bootstrap — a first-run user
  clicking a missing-binary pill gets install help.
- Users who already override `command` / `args` per-server can
  override hints alongside them, in the same block.
- Serde handles backwards-compat: absent field ⇒ empty Vec ⇒
  behaves exactly like today (fallback row).

**Why not JSON/TOML-only:** would require shipping a data file with
the binary and a loader, for no runtime gain. `populate_lsp_config`
already runs at startup; adding `install_hints: vec![...]` per entry
is one line per hint.

### 2. Defaults

Ship hints for the high-traffic languages whose plugins currently
duplicate this copy: `python`, `typescript`/`javascript`, `rust`,
`cpp`/`c`, `go`. Each gets 1–3 platform-labelled hints; a `docs` hint
with the upstream README URL is the universal fallback.

Languages without hints in the default config keep today's behaviour
(disabled "Install … to enable" row). This is intentional: the
config-first approach means users still see a clear indicator of the
dead end, and the new feature only _adds_ actionable rows when we
actually have something to say.

### 3. Popup UX

When a configured server's binary is missing, in the server's section:

```
○ pylsp (binary not in PATH)
    Install pylsp — copy a command:
    Copy pipx · pipx install python-lsp-server      [action]
    Copy pip  · pip install python-lsp-server       [action]
    Copy docs · https://…/python-lsp-server         [action]
```

- Header row (disabled): `Install <name> — copy a command:` replaces
  today's `Install … to enable`. Still disabled; acts as a section
  label so the hint rows below have context.
- Each hint row: `    Copy <label> · <command>` — indented to match
  existing action rows, truncated to the popup's 50-cell width.
  Action key: `copy_install:<language>/<server_name>/<hint_idx>`.
- **Fallback** when `install_hints.is_empty()`: show a single
  `    Copy binary name · <command>` action — gives the user a
  pasteable token for a web search. This replaces today's
  "Install … to enable" dead-end entirely.

### 4. Action handler

Extend `handle_lsp_status_action` with one more prefix:

```rust
} else if let Some(rest) = action_key.strip_prefix("copy_install:") {
    // rest = "<lang>/<server>/<idx>"
    // Look up the hint, call self.clipboard.copy(cmd),
    // set_status_message("Copied: <cmd>")
}
```

No plugin involvement. No outgoing hook. `self.clipboard.copy` is
already in use (`handle_set_clipboard`, `plugin_commands.rs:1599`).

### 5. Platform filter

At build-popup time:
1. Compute `current = InstallPlatform::current()` from `cfg!(target_os)`.
2. `hints_for_os = hints.iter().filter(|h| h.platform == None || h.platform == Some(current))`
3. If `hints_for_os.is_empty() && !hints.is_empty()` — show everything
   with a suffix (e.g. `· (other OS)`). Better to surface a wrong-OS
   hint than nothing; the user can still evaluate it.

### 6. Spawn-failure convergence (decision: leave alone)

`async_handler.rs`'s `LspError` path fires `lsp_server_error` with
`error_type=not_found` today. Short-circuiting that to suppress the
existing plugin popup would break every `*-lsp.ts` plugin that
already subscribes. We leave the existing hook untouched.

Convergence point: both the click-probe and the spawn-failure land
the user at the same status-bar pill. The next click on that pill
opens the popup with the new install rows, regardless of whether the
plugin also fired its own action popup. The two paths are
complementary, not redundant — plugins can continue to offer richer
interactive flows; the core popup guarantees a baseline.

### 7. Staleness when installing with popup open

Deferred to a cross-cutting nit. `command_exists` is uncached, so a
re-open picks up newly-installed binaries naturally. Live re-probe
while the popup is open would require a filesystem watcher on
`$PATH`'s bin dirs, which is a lot of machinery for a one-off user
workflow. Ship accept-staleness; revisit if users report.

### 8. Plugin hook payload

Not modified in this pass. The existing `missing_servers: Vec<String>`
(commands) is enough for plugins to detect which servers are missing.
Display names can be derived by plugins from config if they need
them. Expanding the payload is tracked as a cross-cutting nit.

## Trade-offs considered

| Choice | Picked | Trade-off |
|---|---|---|
| Config-carried vs. plugin registry vs. side registry | Config-carried | Slightly bloats `LspServerConfig`'s JSON schema. Acceptable: `install_hints` has a sane default (empty), doesn't change semantics of any existing field, and users get zero-plugin install help. |
| Inline popup rows vs. secondary modal | Inline rows | Popup width stays at 50 cells; row count grows by ~2–4 per missing server. Trivially scrollable. A modal would bury the hints behind another click. |
| Copy command vs. execute command | Copy | Executing arbitrary `pip`/`npm install -g` requires elevation prompts and a background process UI we don't have. Copy is the common denominator across every platform and shell. |
| URL-as-hint vs. browser-open | URL-as-hint (copy-only) | Opening URLs pulls in an external opener crate and security review. Copy + paste is zero-risk and already-one-keystroke for most users. |
| Expand `missing_servers` now | Defer | Breaking plugin payloads without a converted consumer adds risk for no near-term gain. Tracked separately. |

## Test plan

`tests/e2e/lsp_missing_binary_and_dismiss.rs` already exercises the
popup row shape. Add:

1. `test_missing_binary_popup_shows_install_hints` — config with
   `install_hints` ⇒ popup contains a row per hint with the expected
   action key; no disabled "Install … to enable" dead-end row.
2. `test_missing_binary_popup_no_hints_falls_back_to_copy_name` —
   empty `install_hints` ⇒ popup contains the `Copy binary name`
   fallback action.
3. `test_copy_install_action_copies_to_clipboard` — dispatches
   `copy_install:<lang>/<server>/<idx>`, asserts clipboard contains
   the expected command and a status message was set.
4. `test_install_hints_platform_filter` — hints with mixed platforms;
   only the current-OS ones show up.

Existing tests must continue to pass unchanged.

## Rollout

Single branch, one commit (design doc first, then implementation in a
follow-up commit) per working-style guidance. CHANGELOG entry under
the same `## 0.2.24` block as PR #1558.
