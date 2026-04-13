# LSP Status-Bar UX — Outstanding Research Notes

Companion to `lsp-install-hints-design.md`. Covers Priorities 2–5
from the status-bar UX improvement backlog. Priority 1 has its own
design doc and is implemented in the same branch.

## Priority 2 — "Start" can still spawn-and-fail on a missing binary

### Current behaviour

The `start:<language>` action in `handle_lsp_status_action` calls
`LspManager::manual_restart(language, file_path)`. That method
**re-spawns every configured server for the language** (it removes
non-universal handles for the language, then calls `force_spawn`).
The popup generates only one `start:` key per language even when
multiple servers are configured, so a click starts them all.

`manual_restart_server(language, server_name, file_path)` already
exists and starts a single named server. It's used by the `restart`
row, not by `start`.

### Question

Should the popup's `Start` row scope to one server instead of the
whole language so we can skip missing binaries cleanly?

### Options

**(A) Per-server Start rows.** For each dormant configured server,
emit a `start_one:<language>/<server_name>` action. Missing-binary
servers get the install-hint rows (from P1) instead of a Start.
Installed servers get a Start of their own.

- Pro: matches the user's mental model when they see two server
  rows (`pylsp`, `ruff-lsp`) — each can start independently.
- Pro: with missing-binary filtering from P1, no spawn-and-fail ever.
- Con: adds one more row per dormant server in a popup with limited
  height. For single-server languages (the majority) nothing changes
  visually.

**(B) Keep all-language Start, filter out missing binaries.** Single
row; clicking starts only the installed servers. Missing ones are
silently skipped (already flagged in the popup above).

- Pro: no UX change for the common single-server case.
- Pro: smaller diff.
- Con: if a language has two servers and one is missing, a user who
  later installs the missing one has no way to start just that one —
  they get a full restart. (Actually: they click the server row's
  not-yet-existing-Start, which doesn't exist → they have to
  manual-restart the whole language.)
- Con: "Start all" semantics is only surprising when a user expects
  single-server granularity, which is the very thing we're improving.

**(C) Defer.** The P1 install-hint rows already keep the missing
server from being silently started by a language-level Start — the
`Start` row only appears for languages that have at least one
installed-but-dormant server. Worst case today is a noisy error in
the status line; no stale indicators, no dangling processes.

### Decision

**Implement (B) in this pass.** Keep `start:<language>` semantics but
filter `force_spawn` callers to skip servers whose binary can't be
found. This is a small change inside the popup builder (already
computing `missing_by_server`) — just gate the `start:` emission on
"at least one server is installed-and-dormant", and use the existing
`manual_restart_server` to start them by name when the user clicks.

Rationale: the noise is real (`LSP error (python): No such file...`
after a Start click is the canonical "why isn't this working"
question) but the fix is a one-line guard, not a UX redesign. If
users report that per-server Start would have helped, upgrade to (A)
then — with install hints visible now, the case for per-server
granularity is weaker than it was before P1.

Implementation sketch (defer to a follow-up commit if this PR is
already large):

```rust
// In build_and_show_lsp_status_popup, when emitting the Start row:
let installed_dormant: Vec<&String> = all_servers
    .iter()
    .filter(|n| !missing_by_server.get(*n).copied().unwrap_or(false))
    .filter(|n| !running_statuses.contains_key(*n))
    .collect();
if !installed_dormant.is_empty() {
    // emit Start row that enumerates only installed_dormant names;
    // handler calls manual_restart_server for each.
}
```

This requires a new action key shape (since today `start:<lang>`
triggers a blanket `manual_restart`) — e.g. `start_installed:<lang>`
that the handler resolves back to the subset via the same
`missing_by_server` check it just made.

### Verify

- `LspManager::manual_restart` is confirmed single-language-scope
  (manager.rs:1179). `manual_restart_server` is confirmed
  single-server-scope (manager.rs:1220).
- `force_spawn` iterates `self.config.get(language)` and spawns each
  enabled server — no "skip missing" logic. Restart-throttle is keyed
  on language, not server, so skipping a missing server doesn't
  break the throttle.

## Priority 3 — Dismissal doesn't persist across sessions

### Current behaviour

`Editor::user_dismissed_lsp_languages: HashSet<String>` is runtime
only. Dismissal is lost on restart. Dismissals are global (not
per-project).

### Design sketch

**Storage.** Use `$XDG_STATE_HOME/fresh/lsp-dismissed.json` (under
`log_dirs::log_dir()` — see `services/log_dirs.rs`). State-layer,
not config-layer: dismissals are machine-local UX preferences, not
the kind of thing a user would commit to a dotfiles repo. Shape:

```json
{ "version": 1, "languages": ["python", "go"] }
```

Load once at `Editor::new`; persist on every `dismiss_lsp_language`
/ `undismiss_lsp_language` (fire-and-forget write, logged on error).

**Scoping.** Global, not per-project. Rationale: per-project
dismissal means a user who never has `gopls` installed still sees a
fresh yellow pill in every new Go project. Global matches the
"I don't want this language, stop telling me" intent. If the user
opens a project where they DO want LSP, they rediscover the pill by
looking at the config / command palette (see P4) — explicitly
un-dismiss, same surface as the current `enable:<language>` row
(still shown when dismissed).

**Migration.** Empty file on first launch; existing users with the
runtime-only HashSet see identical behaviour until they trigger a
dismiss, which writes the file for the first time.

**Implementation order.**
1. Add `services::lsp_dismissal::{load, save}` functions alongside
   `log_dirs.rs`.
2. `Editor::new` loads into `user_dismissed_lsp_languages` (already
   a `HashSet<String>` — just populate it from disk).
3. `dismiss_lsp_language` / `undismiss_lsp_language` append the save
   call.
4. Test via a tempdir override of `log_dir()` (OnceLock initialised
   once per process — may need a test-only reset or injection point;
   check whether the existing tests pass XDG_STATE_HOME in env).

**Size:** ~80 lines + one test.

**Not implementing now** — queued behind P1. Low UX cost of inaction
(session-scoped dismissal is still useful today).

## Priority 4 — Re-enable surface for `enabled=false` servers

### Current behaviour

When `LspServerConfig::enabled=false`, no pill renders for that
language (`compose_lsp_status` returns `None` or skips the language
entirely). The only way to flip it back is editing `config.json`.

### Research: what peers do

- **VS Code**: surfaces disabled extensions in a dedicated extensions
  sidebar. Not analogous — VS Code's LSP config is plugin-driven.
- **Helix**: `:lsp-restart` from the command palette re-enables.
  There is no status-bar surface.
- **Zed**: per-language settings UI lists all configured servers
  including disabled ones; no status-bar surface.
- **Neovim + mason.nvim**: `:Mason` window lists all servers with
  install/uninstall buttons. Again, separate UI; nothing in the
  statusline by default.

### Decision

**Leave as "config edit required" — but add a command-palette entry.**
None of the reference editors surface disabled servers in their
status bar. Rendering a pill for a user-disabled server would
contradict the intent ("I turned this off"). Adding a command
palette action `LSP: Re-enable for <language>` costs one string and
one function call and gives power users a fast way back without
editing JSON.

Implementation pointer: `prompt_actions.rs` already has LSP-related
palette actions (`lsp.server_not_found` at line 1131). Drop in a new
action that iterates `config.lsp`, finds `enabled=false` servers,
and flips them to `true` + persists the config.

**Not implementing now.** Queued.

## Priority 5 — Per-server dismissal granularity

### Research

Grepped the codebase + docs/internal/ for user-surface signal.
Current dismissal handles one common case (language-level) and not
one niche case (mute one server in a multi-server language).

Languages with multiple configured servers in the default config are
sparse. In the built-in config at `populate_lsp_config`, only a few
languages have `LspLanguageConfig::Multi` with more than one entry
(primarily via user extension). The common Python / TS / Rust
configs ship with one server each.

Users who explicitly configure two servers for one language (e.g.
`pylsp` + `ruff-lsp`, `tsserver` + `eslint`) generally want _both_
to be active. The "mute one" case is real but not widely reported.

### Decision

**Defer indefinitely. Document the decision.**

If we ever need it, the schema change is incremental:
`HashSet<String>` becomes `HashSet<(String, Option<String>)>` keyed
by `(language, server_name_or_none)`. Action keys gain a variant
`dismiss_server:<language>/<server>`. The popup row only appears
when the language has >1 configured server.

We'll revisit only on concrete user reports asking for it.

## Cross-cutting nits (tracked, not done)

- **`missing_servers` payload shape.** Contains commands. Plugins
  that match by display name can't. Keep `missing_servers` as-is;
  add `missing_server_display_names: Vec<String>` alongside. Deferred
  until a plugin needs it.

- **Stale popup after install.** Accept staleness; re-open refreshes.

- **Plugin install-helper boilerplate.** After P1 lands, ~15
  `*-lsp.ts` files duplicate install copy that's now also in config.
  Consolidation into `lib/lsp-install-helper.ts` was discussed; the
  better move is to deprecate the plugin side and let config-carried
  hints be the single source of truth. Not doing in this pass; file
  as a follow-up.
