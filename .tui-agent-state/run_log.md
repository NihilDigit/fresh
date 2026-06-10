# TUI Agent Run Log

---

## Run #23 — 2026-06-10

### Status: COMPLETED

### What Was Done
- Synced state; built release binary from **origin/master @ f4ee3630** (**v0.3.12**, ~6 min) in a `/tmp/fresh-build` worktree (state branch stays checked out). master moved past Run #22's b022a7fc.
- **Preflight:** GitHub MCP auth OK (read #2291). Playbook integrity OK; lessons continuity OK.
- **Resumed an interrupted prior Run #23:** a previous invocation today (08:25Z, same f4ee3630 commit) already rechecked #2291 and commented "confirmed fixed" but never committed its state (run_log/github_issues still said Run #22). I completed the run without re-commenting on #2291.
- Created tmux session `fresh-r23` (220×50) on a real git C project (`/tmp/trust_dive23`, 2 commits, `compile_commands.json` trust trigger, clangd 18 installed).
- **WORKSPACE TRUST DEEP-DIVE (priority #1) — full 3-state enforcement matrix mapped:**
  - **Dialog is richer than Run #22 documented:** now has explicit `[ OK ]` / `[ Quit (Ctrl+Q) ]` buttons and per-option descriptions that spell out the enforcement contract (Restricted = "Runs system tools on PATH (git, ripgrep, system python); Blocks: project-local executables ./gradlew/.venv/bin/python/node_modules/.bin/*, env activation .env/.envrc/mise, and language servers"; Block All = "Nothing runs"). Letter (T/K/B) selects radio; **Enter confirms** the selected radio (no need to Tab to OK).
  - **Restricted (default):** LSP gated OFF (`not auto-started: workspace is not trusted`); **git ALLOWED** (git blame → 3 correct multi-commit blocks; git_explorer/git_gutter/merge_conflict spawn fine); **ripgrep ALLOWED** (Live Grep found "hello" with both `git-grep` and `rg` providers). Status bar: `Restricted`.
  - **Block All Execution (B):** EVERYTHING blocked — process layer returns `Process error: workspace trust is set to Blocked — no processes may run` (exit -1) for every git spawn; git blame → "No blame information available"; Live Grep → "No matches"; LSP off. Status bar: `Blocked`. Confirming Block All triggers the same editor restart (File Explorer auto-opens) but **preserves the open file** (#2291 fix holds here too).
  - **Trusted (T):** ungates tooling — `clangd-lsp` plugin loads & registers commands (clangdProjectSetup/SwitchSourceHeader) which it does NOT in Restricted/Blocked; "not trusted" gate messages stop. clangd binary stays dormant only because `auto_start` defaults false (IMP-013), so status still reads `LSP (off)`. Status bar: `Trusted`. trust.json → `{"level":"trusted"}`.
  - **Palette surface:** only ONE command — "Workspace Trust…" (opens the dialog). No direct trust/restrict/block palette commands; the `workspace_trust_block` action is not palette-exposed.
- **#2291 recheck (via UI):** CONFIRMED FIXED — directly observed main.c surviving TWO trust-triggered restarts (→Block All, →Trusted). Prior Run #23 already commented; did not duplicate. Marked resolved in state.

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| Trust dialog UI (buttons + option semantics) | **NEW DETAIL** | `[OK]`/`[Quit]` buttons; full enforcement contract in option descriptions |
| Restricted: LSP gating | **PASS** (blocked) | `not auto-started: workspace is not trusted` |
| Restricted: git blame | **PASS** (allowed) | 3 multi-commit blocks, correct attribution |
| Restricted: Live Grep (git-grep + rg) | **PASS** (allowed) | both providers return matches |
| Block All: git blame | **PASS** (blocked) | "No blame information available"; process denied in log |
| Block All: Live Grep | **PASS** (blocked) | "No matches"; process denied in log |
| Block All: process layer | **PASS** | `workspace trust is set to Blocked — no processes may run` |
| Block All: file preservation across restart | **PASS** | main.c kept (#2291 fix) |
| Trusted: tooling ungated | **PASS** | clangd-lsp plugin loads/registers; gate msgs stop |
| #2291 restart data-loss | **CONFIRMED FIXED** (v0.3.12) | file survives restart; prior Run #23 comment stands |

### Issues Filed / Comments
- None this run. No new behavioral bug (enforcement matches the dialog's documented contract). One low-severity UX note → potential_improvements (IMP-015): Blocked-mode tool failures show generic messages ("not a git file or error", "No matches") instead of "blocked by workspace trust".

### Key Findings
1. **Workspace Trust is a 3-state, process-layer enforcement** (Trusted/Restricted/Blocked) and it works correctly. Restricted is the interesting middle: PATH tools (git, ripgrep) run, but project-local executables, env activation, and language servers are gated.
2. **Block All denies at the spawn layer** for ALL processes with a clear log message; user-facing tools degrade gracefully but without telling the user WHY (the "Blocked" status bar is the only hint).
3. **Trusting ungates the LSP plugin layer**, but actual LSP start is still governed by `auto_start` (so trusting alone doesn't start clangd in this config).
4. **#2291 fix verified independently** by watching the open file survive two trust restarts.

### Version
- Binary: v0.3.12 built from origin/master @ f4ee3630 (2026-06-10)

### Cleanup
- tmux `fresh-r23` killed; `/tmp/trust_dive23` removed; `~/.local/share/fresh/workspaces/*` (trust.json) removed; `/tmp/fresh-build` worktree retained for next run's incremental build (optional cleanup).

---

## Run #22 — 2026-06-09

### Status: COMPLETED

### What Was Done
- Synced state; built release binary from **origin/master @ b022a7fc** (**v0.3.12**, ~5 min). NOTE: state branch builds only v0.3.8 — always build from origin/master now.
- Created tmux session `fresh-test-run22` (220×50)
- **Preflight:** GitHub MCP auth OK. Playbook integrity OK. Discovered #2165, #2212 closed by maintainer (completed) and #2113 closed (not_planned) since Run #21.
- **#2165 recheck** — CONFIRMED FIXED in v0.3.12: 'q' in *Keyboard Shortcuts* closes the buffer ("Tab closed"). Comment added.
- **#2212 recheck** — CONFIRMED FIXED in v0.3.12: clangd repro project → Alt+. at unused-include diagnostic shows "remove #include directive quickfix"; applying it edits the buffer ("Applied: remove #include directive (1 change(s))"). Comment added.
- **NEW FEATURE discovered: Workspace Trust prompt** (enforcement now ON; was "groundwork, off by default" in 0.3.10 CHANGELOG). Dialog: Trust(T)/Keep Restricted(K)/Block(B), Enter confirms, Esc does NOT dismiss. Persisted in `~/.local/share/fresh/workspaces/<encoded-path>/trust.json`.
- **NEW BUG FOUND + FILED #2291**: choosing "Trust folder & Allow Tooling" triggers a full editor restart (`Restart requested with new working directory` in log). With `--no-restore`, the CLI-opened file AND unsaved edits are silently discarded (no save prompt; recovery chunk written but never offered). Default mode survives via session restore (File Explorer auto-opens though). 3/3 reproducible.
- **SSH scp-style END-TO-END** — Installed openssh-server, ran sshd on localhost:22 with key auth. `fresh --no-restore root@localhost:/tmp/file.txt` → **FULL PASS**: status bar shows `root@localhost`, remote content loads, edit + Ctrl+S writes through ("Saved", on-disk file updated).
- **#2221 recheck (ssh:// URL form)** — STILL BROKEN in v0.3.12 even with working sshd: opens empty buffer, "Local | ssh://...". Comment added with the ssh-binary-present data point.
- **Keybinding editor: Delete binding** — PASS. Added F9→save (verified working), then 'd' on the row → "Custom binding removed" → Ctrl+S → `keybindings` key cleanly removed from config.json → F9 verified inert again.
- **Keybinding editor: Record Key Search** — PASS. 'r' → press key → filters list (Ctrl+S → 3/866 across contexts; F9 → 1/866). CAVEAT: record mode captures arrows too, so you cannot navigate results while in it; use '/' text search + Enter to act on results.
- **#2122 recheck** — STILL OPEN in v0.3.12: move_to_paragraph_down/up have empty Key column; select_to_paragraph_* still Ctrl+Shift+↓/↑.
- **#2113** — closed by maintainer as not_planned → monitoring item retired.

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| #2165 *Keyboard Shortcuts* 'q' | **CONFIRMED FIXED** (v0.3.12) | "Tab closed"; comment added |
| #2212 Code Actions diagnostics | **CONFIRMED FIXED** (v0.3.12) | Fix popup + apply work end-to-end; comment added |
| Workspace Trust prompt | **NEW FEATURE** | T/K/B radio + Enter; trust.json persistence; LSP gated on trust |
| Trust → file/edits dropped (--no-restore) | **BUG (#2291 filed)** | Editor restart discards CLI file + unsaved edits, silently |
| SSH scp-style end-to-end | **PASS** | Real sshd; open/edit/save round-trip verified |
| #2221 ssh:// URL form | **STILL BROKEN** | Treated as local path even with working sshd; comment added |
| Keybinding editor: Add (F9→save) | **PASS** | Tab×2 from Action needed to reach [Save] button |
| Keybinding editor: Delete binding | **PASS** | 'd' + Ctrl+S; config.json key removed; F9 inert |
| Keybinding editor: Record Key Search | **PASS** | Filters by pressed key; arrows captured while active |
| #2122 move_to_paragraph binding | **STILL OPEN** | No binding in v0.3.12 |

### Issues Filed / Comments
- **#2291** (new): "Workspace Trust: choosing 'Trust folder & Allow Tooling' restarts the editor and silently discards the opened file and unsaved edits (with --no-restore)"
- Comments: #2165 (confirmed fixed), #2212 (confirmed fixed), #2221 (still broken with working sshd)

### Key Findings
1. **Workspace Trust enforcement is live in v0.3.12** and gates LSP auto-start ("LSP for 'cpp' not auto-started: workspace is not trusted"). compile_commands.json is a trust trigger.
2. **Trust confirm = full editor restart.** Relies on session persistence to rebuild the session; with --no-restore that means silent data loss (#2291).
3. **SSH remote editing via scp-style works end-to-end** against a real sshd (status-bar origin, content, write-back all correct).
4. **Keybinding editor add/delete/record all functional**; quirks noted in potential_improvements (focus path to Save button, transient unresponsiveness after Add dialog, record-mode arrow capture).
5. **Observation (unconfirmed, low priority):** keybinding editor total count differed between opens in one session (866 vs 548 bindings) — possibly plugin lazy registration; re-observe before treating as a bug.

### Version
- Binary: v0.3.12 built from origin/master @ b022a7fc (2026-06-09)

### Cleanup
- tmux session `fresh-test-run22` killed; sshd stopped; /tmp test dirs removed; config.json reset to `{}`; trust.json + recovery chunks for test workspaces removed

---

## Run #21 — 2026-06-03

### Status: COMPLETED

### What Was Done
- Synced state from `tui-automated-testing-state`; built release binary from `claude/awesome-clarke-fYwrE` (**v0.3.10**, ~6.5 min build)
- Created tmux session `fresh-test-run21` (220×50)
- **Preflight:** GitHub MCP auth confirmed (9 open/filed issues after filing #2221). Playbook integrity confirmed.
- **SSH features** — Tested both URI forms. URL-style `ssh://` DOES NOT WORK (treated as local path, BUG #2221). scp-style `user@host:/path` works correctly (triggers SSH path, fails at ssh-binary-not-found).
- **Keybinding editor** — Full workflow tested: open editor, search, add binding (F9→save, normal context), save with Ctrl+S, verify F9 triggers save. All PASS.
- **Search in selection** — NOT IMPLEMENTED. Find bar has only Case/Whole Word/Regex options. No "In Selection" toggle.
- **Multi-root workspaces** — PASS. Workspace scoping correct (file picker shows only CWD files). Cross-workspace file opens and appears in project-wide search with full path.
- **#2165 recheck** — CONFIRMED STILL OPEN in v0.3.10 (`claude/awesome-clarke-fYwrE`): 'q' in *Keyboard Shortcuts* still shows "Editing disabled".
- **#2113 recheck** — NOT REPRODUCED in 8 more attempts (16 total across all runs).

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| SSH URL-style `ssh://` | **BUG (#2221)** | Treated as local relative path; no connection; no error |
| SSH scp-style `user@host:/path` | **PASS** (partial) | Correctly detects SSH; fails at ssh-not-found (no ssh binary installed) |
| Keybinding editor: Open | **PASS** | 852 bindings, Config path shown |
| Keybinding editor: Search (/) | **PASS** | "10/852 shown" filter works for 'save' |
| Keybinding editor: Add binding (F9→save) | **PASS** | Dialog: key capture → action autocomplete → context cycle → Save |
| Keybinding editor: Save (Ctrl+S) | **PASS** | "Keybinding changes saved"; config.json updated correctly |
| Keybinding editor: Binding works | **PASS** | F9 triggers file save ("Saved" in status bar) |
| Search in selection | **NOT IMPLEMENTED** | No "In Selection" toggle in find bar |
| Multi-root: workspace scoping | **PASS** | File picker shows only CWD files |
| Multi-root: cross-workspace file open | **PASS** | Ctrl+O opens outside workspace; full path shown |
| Multi-root: project-wide search | **PASS** | Alt+A includes all open buffers (in- and out-of-workspace) |
| #2165 *Keyboard Shortcuts* 'q' | **CONFIRMED STILL OPEN** | v0.3.10 / claude/awesome-clarke-fYwrE |
| #2113 race condition | **NOT REPRODUCED** | 8 more attempts; 16 total without reproduction |

### Issues Filed / Comments
- **#2221** (new): "SSH URL-style URI (`ssh://host/path`) treated as local file path instead of triggering SSH connection"

### Key Findings
1. **SSH URL-style form is broken**: `fresh ssh://host/path` silently opens an empty local file. Log confirms path resolved as CWD + URI. scp-style correctly triggers SSH (fails gracefully if `ssh` binary missing). Filed as #2221.
2. **Keybinding editor fully functional**: Add/edit/save/test cycle all work. Autocomplete for action names works. Context field cycles with ←/→. Ctrl+S saves to config.json. Added F9→save binding verified working.
3. **Search in selection not implemented**: Fresh's search bar has no "In Selection" option. Documented as IMP-014.
4. **Multi-root workspaces work correctly**: Workspace scoping (file picker), cross-workspace file opening, and project-wide search all work as expected.

### Version
- Binary: v0.3.10 built from `claude/awesome-clarke-fYwrE` (2026-06-03)

### Cleanup
- tmux session `fresh-test-run21` killed
- Config reset to `{}`
- Temp files: /tmp/multiroot_a/, /tmp/multiroot_b/, /tmp/search_in_sel_test.txt

---

## Run #20 — 2026-06-03

### Status: COMPLETED

### What Was Done
- Synced state from `tui-automated-testing-state`; built release binary from `claude/awesome-clarke-57Uge` (**v0.3.10**, ~6.5 min build)
- Created tmux session `fresh-test-run20` (220×50)
- **Preflight:** GitHub MCP auth confirmed (8 open/filed issues). Playbook integrity confirmed. All sections of AGENT_INSTRUCTIONS.md present.
- **#2165 recheck** — *Keyboard Shortcuts* 'q' CONFIRMED STILL OPEN in v0.3.10 ("Editing disabled in this buffer")
- **text-actions plugin** — Installed from GitHub URL (network available). Tested ALL decode commands. Discovered new decode commands not previously documented.
- **#2212 recheck on v0.3.10** — CONFIRMED STILL OPEN. LSP log shows `"context":{"diagnostics":[]}` still empty in v0.3.10. Comment added to GitHub issue #2212.
- **Bookmarks (Alt+0-9)** — Full test of all slots: set bookmarks 0, 1, 5, 9; tested jumping with Alt+0/1/5/9; tested unset slot (Alt+2 → "Bookmark '2' not set").
- **Keyboard macros** — Recorded complex 5-action macro (slot 3): SmartHome + InsertChar('#') + InsertChar(' ') + MoveDown + SmartHome. Played back on 5 lines. Verified via List Macros.
- **Markdown preview** — Toggled compose mode. Verified bold/italic ANSI rendering, inline code, code blocks with syntax highlighting, blockquotes, lists, HR. Editing inside code blocks works.

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| #2165 *Keyboard Shortcuts* 'q' | **STILL OPEN** | "Editing disabled" in v0.3.10 |
| text-actions: Decode Base64 | **PASS** | "SGVsbG8gV29ybGQ=" → "Hello World" |
| text-actions: Decode URI Component | **PASS** | "Hello%20World%21" → "Hello World!" |
| text-actions: Decode JSON String | **PASS** | `"Hello\nWorld\t!"` → multiline with newline+tab |
| text-actions: Decode Hex to JSON | **PASS** | "48656c6c6f" → "[72,101,108,108,111]" |
| text-actions: Encode→Decode round-trip | **PASS** | "Fresh Editor 2026" → Base64 → back = correct |
| #2212 Code Actions (v0.3.10) | **STILL OPEN** | `context.diagnostics` still empty; comment added to #2212 |
| Bookmarks: Set (0, 1, 5, 9) | **PASS** | "Bookmark 'N' set" for each |
| Bookmarks: Jump (Alt+0/1/5/9) | **PASS** | "Jumped to bookmark 'N'" at correct lines |
| Bookmarks: Unset slot (Alt+2) | **PASS** | "Bookmark '2' not set" |
| Keyboard macros: Record (slot 3) | **PASS** | 5-action macro; "Macro '3' saved (5 actions)" |
| Keyboard macros: Playback (F4) | **PASS** | Applied "# " prefix to 5 lines correctly |
| Keyboard macros: List Macros | **PASS** | `*Macros*` buffer shows SmartHome/InsertChar/MoveDown |
| Markdown: Toggle Compose mode | **PASS** | "Markdown Compose: ON (soft breaks, centered)" |
| Markdown: Bold/Italic ANSI | **PASS** | `**bold**` → `[1m` bold; `*italic*` → `[3m` italic; markers hidden |
| Markdown: Inline code | **PASS** | `` `code` `` → colored, backticks stripped |
| Markdown: Code blocks | **PASS** | Fence markers visible; code syntax-highlighted inside |
| Markdown: Blockquotes | **PASS** | `>` colored with teal; rendering correct |
| Markdown: Lists + HR | **PASS** | Both ordered and unordered lists; `---` HR visible |
| Markdown: Edit inside code block | **PASS** | New line added inside Python block; compose mode updates correctly |

### Issues Filed / Comments
- Comment on **#2212**: "Reproduced in v0.3.10 — `context.diagnostics` still sent as empty"

### Key Findings
1. **text-actions plugin has more decode commands than documented in learning_db.md**: Decode Base64 to String, Decode Hex String to JSON Byte Array, Decode JSON String to String are all available and work correctly. Previously only Decode URI Component and Decode URI Encoded were documented.
2. **All text-actions decode+encode round-trips correct**: Base64, URI Component, JSON String, Hex all verified correct against independent reference values.
3. **#2212 still unfixed in v0.3.10**: `context.diagnostics` is always `[]` in codeAction requests. Updated GitHub issue with v0.3.10 confirmation.
4. **Bookmarks fully functional**: Alt+0 through Alt+9 all work; unset slots give informative message; setting via "Set Bookmark" command works.
5. **Keyboard macros work for complex multi-step operations**: 5-step macro (comment prefix + move to next line) recorded, played, and listed correctly. `*Macros*` buffer shows action-level detail.
6. **Markdown Compose mode fully functional**: Bold `[1m`, italic `[3m` ANSI attributes applied; inline code stripped of backticks; code blocks get syntax highlighting inside fences; editing inside code blocks works in compose mode.
7. **clangd auto-starts in v0.3.10** with `"enabled": true` (no `auto_start` needed) — behavior changed vs v0.3.8. UPDATE: needs verification — may have started automatically due to the new build or config change.

### Version
- Binary: v0.3.10 built from `claude/awesome-clarke-57Uge` (2026-06-03)

### Cleanup
- tmux session `fresh-test-run20` killed
- Temp files removed: /tmp/cpp_test_v2/, /tmp/bookmark_test.txt, /tmp/markdown_test.md, /tmp/text_actions_test.txt
- Config reset to `{}`
- text-actions plugin NOT removed (was in /root/.config/fresh/plugins/ but config dir was clean start)

---

## Run #19 — 2026-06-03

### Status: COMPLETED

### What Was Done
- Synced state from `tui-automated-testing-state`; built release binary from `tui-automated-testing-state` (**v0.3.8**, ~7 min build)
- Created tmux session `fresh-test-run19` (220×50)
- **Preflight:** GitHub MCP auth confirmed (7 open/filed issues). Playbook integrity confirmed. All sections of AGENT_INSTRUCTIONS.md present.
- **LSP Code Actions (Alt+.)** — Definitive root cause found via LSP log: Fresh always sends `"context":{"diagnostics":[]}` (empty) in codeAction requests. clangd requires diagnostics to provide fix-based code actions. Filed new issue #2212.
- **#2113 race condition** — 8 rapid attempts across 3 patterns; not reproduced. Consistent with "timing-sensitive, reproduced once" history.
- **Encoding handling** — Latin-1 file: auto-detected as Windows-1252, Reload with Encoding, Set Encoding all work. UTF-8 round-trip confirmed by hex inspection.
- **Themes** — All 8 themes (dark, dracula, high-contrast, light, nord, nostalgia, solarized-dark, terminal) tested. Colors confirmed distinct via ANSI. "nord" is new compared to v0.3.9 test.
- **Clangd auto-start** — Confirmed: `enabled: true` does NOT auto-start; `auto_start` setting exists (default: false). Docs say "automatically" but mean "config is pre-built" not "auto-launches". Updated IMP-013 with this finding.
- **text-actions decode** — BLOCKED: GitHub network unavailable. git clone hangs; Fresh shows "Failed to install..." correctly after process killed.

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| LSP Code Actions (Alt+.) | **BUG (#2212)** | Fresh sends empty `context.diagnostics` always; clangd needs them for fix-based actions |
| #2113 race condition | **NOT REPRODUCED** | 8 attempts, 3 patterns; timing-sensitive per original report |
| Encoding: auto-detect Latin-1 | **PASS** | Detected as Windows-1252 (correct superset); all chars render properly |
| Encoding: Reload with Encoding | **PASS** | 8-encoding picker; current marked; navigation works with ANSI verify |
| Encoding: Set Encoding | **PASS** | Switches buffer encoding, marks modified, UTF-8 round-trip correct on save |
| Themes: dark/dracula/high-contrast | **PASS** | Color codes confirm distinct themes |
| Themes: light | **PASS** | Light background (`48;5;254m`); correct for light theme |
| Themes: nord | **PASS** | New in v0.3.8; distinct blue-grey palette (`188/237` codes) |
| Themes: nostalgia/solarized-dark/terminal | **PASS** | All 8 themes apply and produce different colors |
| Clangd auto_start investigation | **IMP-013 UPDATED** | `auto_start` setting exists, default `false`; docs misleading but not a bug |
| text-actions decode | **BLOCKED** | GitHub network unavailable; documented |

### Issues Filed / Comments
- **#2212** — NEW: "Alt+. shows 'No code actions available' for diagnostic-based fixes even when clangd reports '(fix available)'" — LSP log evidence: empty `context.diagnostics` in every codeAction request

### Key Findings
1. **Code Actions root cause confirmed**: Fresh always sends `"context":{"diagnostics":[]}` in `textDocument/codeAction`. clangd published 7 diagnostics with "(fix available)" but returns empty `[]` without the diagnostic context. This is the "TODO: Implement diagnostic retrieval when needed" left from closed issue #1915. Filed as new dedicated issue #2212.
2. **Encoding feature fully functional**: Detection, reload, set-encoding, and save all work correctly. Latin-1 ↔ UTF-8 round-trip confirmed via hex. 8-encoding picker with "current" marker and ANSI-confirmable navigation.
3. **All 8 themes work**: Including new "nord" theme (not present in v0.3.9 tests). Navigation in theme picker requires ANSI verify (no plain-text indicator of selected item).
4. **auto_start LSP setting discovered**: Config schema has `auto_start: boolean, default: false`. Users who want clangd to auto-start must set `"auto_start": true`. Docs saying "use it automatically" refer to pre-built config, not auto-launch.
5. **text-actions decode BLOCKED**: No GitHub network in this environment. Fresh plugin install handles failure gracefully ("Failed to install...").

### Version
- Binary: v0.3.8 built from `tui-automated-testing-state` (2026-06-03)

### Cleanup
- tmux session `fresh-test-run19` killed
- Temp files removed: /tmp/cpp_lsp_test/, /tmp/latin1_test.txt, /tmp/test_palette_leak.txt, /tmp/claude-0/fresh-pkg-clone-*
- Config reset to `{}`
- clangd stopped (fresh exited)

---

## Run #18 — 2026-06-03

### Status: COMPLETED

### What Was Done
- Synced state from `tui-automated-testing-state`; built release binary from `tui-automated-testing-state` (**v0.3.8**, ~8 min build from scratch)
- Installed clangd-18 via `apt-get install clangd` (not installed by default)
- Created tmux session `fresh-test-run18` (220×50)
- **Preflight:** GitHub MCP auth confirmed (7 open/filed issues verified). Playbook integrity confirmed. All 3 AGENT_INSTRUCTIONS.md sections present.
- **LSP: clangd on C project** — Set up small C project in `/tmp/c_lsp_test/` with compile_commands.json; configured clangd in Fresh config. Tested all major LSP features.
- **text-actions plugin** — Installed from GitHub URL and tested encoding/decoding commands.
- **Git Blame: multi-commit history** — Tested 'b' navigation on CHANGELOG.md (399 blocks, multiple commits). Confirmed depth tracking.
- **#2122 recheck** — Confirmed move_to_paragraph_down/up still has no keybinding in v0.3.8 (keybinding editor shows empty for those actions).
- **#2165 recheck** — Confirmed *Keyboard Shortcuts* 'q' still shows "Editing disabled" in v0.3.8.

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| clangd: auto-start | **NEEDS MANUAL START** | Even with `"enabled": true` in config, shows "not running" — needed "Start clangd (always)" from LSP Status popup |
| LSP: Hover (Alt+K) | **PASS** | Shows function signature popup: "int add(int a, int b)" |
| LSP: Go to Definition (F12) | **PASS** | Jumped to definition at main.c:9, status "Jumped to definition at /tmp/c_lsp_test/main.c:9" |
| LSP: Completions (Ctrl+Space) | **PASS** | Showed "make_point(int x, int y) Point" suggestion for "mak" prefix |
| LSP: Find References (Shift+F12) | **PASS** | Found 2 references to 'add' (definition + call site) |
| LSP: Rename Symbol (F2) | **PASS** | Renamed 'add' → 'sum' at definition and all call sites simultaneously |
| LSP: Inlay hints | **PASS** | Parameter names shown in call sites: "add(a: 3, b: 4)", "make_point(x: 10, y: 20)" |
| LSP: Code Actions (Alt+.) | **NOT AVAILABLE** | "No code actions available" even at error location (malloc undeclared). Likely clangd limitation for this error type, not a Fresh bug. |
| text-actions plugin: install | **PASS** | "Installed and activated fresh-text-actions-plugin v0.1.0" |
| text-actions plugin: commands | **PASS** | 6+ commands: Base64/JSON/URI encode+decode |
| text-actions plugin: Base64 | **PASS** | "Hello World" → "SGVsbG8gV29ybGQ=" (correct) |
| Git Blame: multiple commits | **PASS** | CHANGELOG.md shows 399 blocks with multiple distinct commit hashes |
| Git Blame: 'b' go to parent | **PASS** | bc11f2b → 059f4ab → 60d0ba2; depth counter shown in status |
| Git Blame: 'q' close | **PASS** | "Git blame closed" status |
| #2122 move_to_paragraph keybinding | **CONFIRMED STILL OPEN** | No keybinding in v0.3.8 (same as #2122 report) |
| #2165 *Keyboard Shortcuts* 'q' | **CONFIRMED STILL OPEN** | "Editing disabled in this buffer" in v0.3.8 |

### Issues Filed / Comments
- No new issues filed — all findings either PASS or match known open issues
- Note: clangd auto-start behavior is a potential UX issue (docs say "auto", but requires manual start). Logged in potential_improvements.md as IMP-013.

### Key Findings
1. **clangd LSP fully functional** once started: hover, definition, completions, references, rename all work. Inlay hints shown automatically.
2. **Code Actions (Alt+.)** returned "No code actions available" even at diagnostic error locations. This may be clangd's behavior for C "undeclared function" errors (no quick-fix available), not a Fresh bug. Future run should test with C++ or a different error type.
3. **text-actions plugin** installs cleanly from external GitHub URL. All 6+ encoding commands appear in palette. Base64 encoding verified correct.
4. **Git Blame multi-commit history** navigation works: 'b' goes to parent, depth counter shown, multiple commits verified. First commit shows "Cannot get blame at SHA^ (may be initial commit)".
5. **clangd auto-start**: Despite `"enabled": true` in config.json, clangd shows as "not running" on fresh launch. Requires manual "Start clangd (always)" from LSP Status popup. This contradicts the docs which say LSP auto-starts when installed. Documented as IMP-013.

### Version
- Binary: v0.3.8 built from `tui-automated-testing-state` branch (2026-06-03)

### Cleanup
- fresh exited cleanly via Ctrl+Q
- tmux session `fresh-test-run18` killed
- text-actions plugin removed: `rm -rf /root/.config/fresh/plugins/packages/fresh-text-actions-plugin`
- LSP config reset to `{}`
- /tmp/c_lsp_test/ removed

---

## Run #17 — 2026-06-02

### Status: COMPLETED

### What Was Done
- Synced state from `tui-automated-testing-state`; built release binary from `claude/awesome-clarke-VmLci` (**v0.3.10**, ~8 min)
- Created tmux session `fresh-test-run17` (220×50)
- **Preflight:** Confirmed AGENT_INSTRUCTIONS.md updated per user instructions (real LSP preference added; forget previous issues instruction enacted by resetting test priority to coverage-first).
- **User overrides this run:**
  1. "forget previous issues; move on to testing completely other UX aspects or features or user flows"
  2. "prefer real-world use cases and tools" instruction added to AGENT_INSTRUCTIONS.md
  3. Removed fake-pylsp symlink; switched to real pyright
  4. Avoided rust-analyzer; used pyright on small Python project in /tmp
- **File Explorer (Ctrl+B / Ctrl+E):** Tested full keyboard-only navigation
- **LSP with pyright:** Set up real pyright on a small Python project in `/tmp/py_lsp_test/`; discovered major LSP timeout bug
- **Settings panel:** Tested navigation model, TextList [x] delete keyboard accessibility
- **Bug filed:** #2197 — pyright LSP all request-based features timeout after 30s

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| File Explorer: Ctrl+B toggle | **PASS** | Shows/hides sidebar |
| File Explorer: Ctrl+E focus | **PASS** | Moves focus from editor to explorer |
| File Explorer: Up/Down navigate | **PASS** | Moves cursor through files/dirs |
| File Explorer: Right expand dir | **PASS** | Expands directory |
| File Explorer: Left collapse dir | **PASS** | Collapses directory |
| File Explorer: Enter opens file | **PASS** | Opens file permanently (not preview) |
| File Explorer: auto-preview on navigate | **PASS** | Files auto-preview as cursor moves |
| File Explorer: New file (Ctrl+N) | **PASS** | Creates file when explorer focused |
| File Explorer: Delete file (Delete key) | **PASS** | Confirms with y/n; "Moved to trash" |
| Settings: Tab cycle (Cat→Settings→Footer→Cat) | **PASS** | Blue `[48;5;25m` highlight confirms focus |
| Settings: TextList navigate Up/Down to items | **PASS** | Up/Down navigates from header into items |
| Settings: TextList Delete removes item | **CONFIRMED** | Hint "Del:remove" shown when item focused |
| Settings: TextList [x] keyboard-accessible | **CONFIRMED NOT** | Tab exits TextList; [x] is mouse-only |
| Settings: Escape discards unsaved changes | **PASS** | No confirmation dialog; changes discarded |
| pyright LSP: initialize | **PASS** | Shows "LSP (python) ready" in status bar |
| pyright LSP: hover (Alt+K) | **FAIL** | Timeout after 30s (10/10 requests) |
| pyright LSP: definition (F12) | **FAIL** | Timeout after 30s |
| pyright LSP: completion (Ctrl+Space) | **FAIL** | Timeout after 30s |
| pyright LSP: signatureHelp | **FAIL** | Timeout after 30s |
| pyright LSP: diagnostics | **FAIL** | 0 items (no code diagnostics published) |

### Issues Filed
- **#2197** (new): "Pyright LSP: all request-based features (hover, definition, completions) timeout after 30s while LSP shows 'ready'"

### Key Findings
1. File Explorer fully functional with keyboard-only navigation including file creation and deletion.
2. Settings panel uses Tab cycle: Categories → Settings → Footer → Categories. Arrow keys in Categories panel navigate categories; Tab switches focus to the Settings panel.
3. Settings TextList [x] buttons are MOUSE-ONLY. Keyboard deletion uses Delete key while item focused (confirmed via "Del:remove" hint text).
4. pyright LSP integration broken — initialize succeeds but ALL subsequent LSP requests (hover, definition, completion, signatureHelp, diagnostics) silently timeout after 30s. Position encoding mismatch suspected (log: `LSP initialize result: position_encoding=None`).

### Version
- Binary: v0.3.10 built from `claude/awesome-clarke-VmLci` (same as Run #16 branch, new commit)

### Cleanup
- tmux session `fresh-test-run17` killed
- /tmp/py_lsp_test/ removed

---

## Run #16 — 2026-05-31

### Status: COMPLETED

### What Was Done
- Synced state from `tui-automated-testing-state`; built release binary from `claude/awesome-clarke-jWgGn` (**v0.3.10**, ~2.5 min)
- Created tmux session `fresh-test-run16` (220×50)
- **Preflight:** GitHub MCP auth confirmed. Playbook integrity verified. Discovered #2117 and #2125 both CLOSED by maintainer since Run #15.
- **Bug recheck — *Keyboard Shortcuts* 'q':** STILL BROKEN in 0.3.10 ("Editing disabled in this buffer"). Filed new issue #2165 since #2125 is closed.
- **Bug recheck — #2117 Review Diff discard hunk:** CONFIRMED FIXED in 0.3.10! Created review_diff_test16.txt (+4 lines), opened Review Diff, navigated to hunk, pressed 'd' → confirmed dialog → Enter → "Review Diff: 0 hunks". File reverted to original. Manual git apply --reverse no longer needed.
- **Diagnostics panel 'q' recheck:** CONFIRMED STILL FIXED — "Diagnostics panel closed" on 'q' press. Consistent with #2125 closure.
- **Git Blame plugin:** PASS — `*blame:README.md*` buffer opens with commit info (commit hash, author, time, message). Status bar shows "Git blame: N blocks | b: blame at parent | q: close". 'b' correctly returns "Cannot get blame at SHA^ (may be initial commit)" for file at initial commit. 'q' closes with "Git blame closed".
- **Live Diff: Set Default Mode:** PASS — prompt "Default mode (head, disk, or branch:<ref>)head" appears. Accepted "disk", "branch:main", and "head" — all showed "Live Diff: default mode updated". Note: prompt always pre-fills "head" regardless of current setting.
- **Orchestrator features (0.3.10):** PASS — Alt+P toggles project scope (All → user/fresh), Alt+T toggles show-all-worktrees checkbox, Tab focuses detail panel buttons (blue highlight), Details view shows "ACT Xs in-place" + working dir + file preview, "/" filter input works, Escape closes. All 0.3.9+ features confirmed working.
- **Package: Install + Uninstall + Color Highlighter:**
  - Install via "Package: Install from URL" → `https://github.com/sinelaw/fresh-plugins#color-highlighter` → "Installed and activated color-highlighter v1.0.0" ✅
  - Package browser shows INSTALLED (1) with ✓ checkmark ✅
  - Color Highlighter: Enable command adds `█` swatches before hex/rgb/hsl values in CSS (ANSI confirms actual colors: `[38;5;196m` red, `[38;5;33m` blue, `[38;5;46m` green) ✅
  - Uninstall via `rm -rf /root/.config/fresh/plugins/packages/color-highlighter` → package browser shows AVAILABLE (13), swatches immediately removed ✅
  - ⚠️ NOTE: Package UI Install/Uninstall button navigation is complex (Tab through 8+ elements to reach). "Enter Activate" at Tab position shows `[ Install ]`/`[ Uninstall ]` but pressing Enter activates search field. Documented in potential_improvements.md.
- **Dev Container: Attach (no CLI):** PASS — dialog "Dev Container CLI Not Found: The devcontainer CLI is needed for rebuild. Copy the install command below, or dismiss. Copy: npm i -g @devcontainers/cli / Dismiss (ESC)". Clear, helpful error with actionable install command.

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| *Keyboard Shortcuts* 'q' close | **STILL BROKEN** | Filed new #2165 (parent #2125 was closed) |
| #2117 Review Diff discard hunk | **CONFIRMED FIXED** | Works in 0.3.10 — 0 hunks after discard |
| #2125 Diagnostics panel 'q' | **STILL FIXED** | "Diagnostics panel closed" confirmed |
| Git Blame plugin | **PASS** | Blame buffer, 'b' go-back, 'q' close all work |
| Live Diff: Set Default Mode | **PASS** | head/disk/branch:main all accepted |
| Orchestrator features | **PASS** | Alt+P/T, Details, filter search all work |
| Package: Install from URL | **PASS** | "Installed and activated color-highlighter v1.0.0" |
| Color Highlighter plugin | **PASS** | Swatches for hex/rgb/hsl with correct colors |
| Package: Uninstall (file delete) | **PASS** | Package removed, swatches gone in real-time |
| Dev Container: Attach error handling | **PASS** | "CLI Not Found" dialog with npm install command |

### Issues Filed / Comments
- Filed new issue **#2165**: "*Keyboard Shortcuts* buffer: pressing 'q' shows 'Editing disabled' despite in-buffer documentation" (since #2125 closed)
- Updated `github_issues.md` and `confirmed_bugs.md`

### Version
- Binary: v0.3.10 built from `claude/awesome-clarke-jWgGn` (new version vs Run #15's 0.3.9)

### Cleanup
- Fresh exited via Ctrl+Q (d = discard and quit)
- tmux session `fresh-test-run16` killed
- review_diff_test16.txt committed + removed from dev branch
- /tmp/test_colors.css removed
- .devcontainer/ directory removed

---

## Run #15 — 2026-05-27

### Status: COMPLETED

### What Was Done
- Synced state from `tui-automated-testing-state`; built release binary from `claude/awesome-clarke-cN0ma` (v0.3.9, ~80s)
- Created tmux session `fresh-test-run15` (220×50)
- **Preflight:** GitHub MCP auth confirmed (listed issues). Playbook integrity verified.
- **Bug recheck — *Keyboard Shortcuts* 'q':** STILL BROKEN ("Editing disabled in this buffer"). Same as Run #14.
- **Bug recheck — #2117 (Review Diff discard hunk):** STILL BROKEN. Created review_diff_test.txt with +3 lines, triggered discard — "Patch failed: error: patch failed: review_diff_test.txt:2error: review_diff_test.txt: patch does not apply". Manual `git apply --reverse --check` succeeds (confirming it's Fresh's bug).
- **Flash: Jump plugin:** PASS — opened via command palette, jump-hint overlay activated (letters replace visible chars), pressed 'n' hint to jump from Ln 7 Col 18 → Ln 7 Col 6.
- **Package Manager (Package: Packages):** PASS — shows 13 available packages with categories [P/T/L], detail panel, filter tabs (All/Installed/Plugins/Themes/Languages/Bundles/Sync). Search by "/" filters: "theme" → 3 results. Registry synced (1/1 sources).
- **Package Manager (Package: Install from URL):** PASS — prompts "Git URL or local path:" input dialog.
- **Live Diff: vs HEAD:** PASS — green `│` gutter markers (ANSI 38;5;78) and green bg (48;5;22) on added lines. Status: "Live Diff: comparing against HEAD".
- **Live Diff: vs Disk:** PASS — `+` marker on unsaved line. Status: "Live Diff: comparing against file on disk".
- **Live Diff: vs Branch...:** PASS — "Branch or ref" prompt pre-filled "main". Status: "Live Diff: comparing against main".
- **Live Grep: Cycle Provider:** PASS — Alt+P cycles: git-grep → rg → grep → git-grep. All 3 providers available. Search "Test" returned 1000+ matches.
- **Block selection (Alt+Shift+Arrow):** PASS — M-S-Down and M-S-Right work! Block selected "Line " (cols 1-5) across rows 1-4. Typing '>' replaced selection on all 4 rows simultaneously. NOTE: Run #12 reported M-S-Down didn't work — it DOES work in this build.
- **Dev Container features:** PASS — Create Config creates minimal .devcontainer/devcontainer.json; Show Info displays container config with action buttons; Show Features shows "No features configured"; Show Forwarded Ports shows "No configured or runtime ports to show."; all Dev Container panels close with 'q' (unlike *Keyboard Shortcuts* buffer).

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| *Keyboard Shortcuts* 'q' close | **STILL BROKEN** | "Editing disabled in this buffer" — same as Runs 12-14 |
| #2117 Review Diff discard hunk | **STILL BROKEN** | Patch failed error persists; manual git apply --reverse works |
| Flash: Jump plugin | **PASS** | Hint overlay activates, pressing hint char jumps cursor |
| Package: Packages browser | **PASS** | 13 packages, search, filter tabs, detail panel, Install button |
| Package: Install from URL | **PASS** | "Git URL or local path:" prompt appears |
| Live Diff: vs HEAD | **PASS** | Green gutter markers on added lines; status confirmed |
| Live Diff: vs Disk | **PASS** | `+` marker on unsaved content; status confirmed |
| Live Diff: vs Branch... | **PASS** | Branch prompt, "comparing against main" confirmed |
| Live Grep: Cycle Provider | **PASS** | git-grep → rg → grep cycling; search works with all providers |
| Block selection (Alt+Shift+Arrow) | **PASS** | M-S-Down and M-S-Right work; rectangular edit confirmed |
| Dev Container: Create Config | **PASS** | Creates .devcontainer/devcontainer.json with template |
| Dev Container: Show Info | **PASS** | Shows config, action buttons, q closes correctly |
| Dev Container: Show Features | **PASS** | "No features configured" |
| Dev Container: Show Forwarded Ports | **PASS** | "No configured or runtime ports" panel with q close |

### Issues Filed / Comments
- No new issues filed (all tests passed or are known bugs with open issues)
- Note: *Keyboard Shortcuts* 'q' bug persists — already tracked via #2125 comment

### Cleanup
- Fresh exited via Ctrl+Q (d = discard and quit)
- tmux session `fresh-test-run15` killed
- review_diff_test.txt commit reverted on dev branch; .devcontainer removed

---

## Run #14 — 2026-05-27

### Status: COMPLETED

### What Was Done
- Synced state from `tui-automated-testing-state`; built release binary from `claude/awesome-clarke-c7jCY`
- Created tmux session `fresh-test-run14` (220×50)
- **T47 Rapid keystrokes:** 50-char burst intact; 20 rapid Ctrl+Z all undone correctly. PASS.
- **T48 Resize reflow:** 220×50 → 80×24 → 180×40 all reflow; mid-typing resize safe. PASS.
- **Alt+A project-wide Search & Replace:** Panel opened; 9 matches in 4 files found; Space scoping (deselected source files to scope to test_file1.txt); Replace All with confirmation ("Replaced 3 occurrences in 1 files"). PASS.
- **Calibrate Keyboard wizard:** 24 steps/5 groups (Basic Editing, Line Navigation, Word Navigation, Document Navigation, Emacs-Style). Does NOT test Ctrl+H. s/b/g/a controls all work.
- **#2125 recheck (Diagnostics panel):** q CONFIRMED FIXED (commit 89caf72). `*Keyboard Shortcuts*` 'q' STILL BROKEN ("Editing disabled"). Comment posted on #2125.
- **#2112 recheck (outside-workspace search):** CONFIRMED FIXED (commit b7e7e64). /tmp files now found in Search/Replace panel. Comment posted on #2112.

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| T47: Rapid keystrokes | **PASS** | 50-char burst intact, 20 rapid undos clean |
| T48: Resize reflow | **PASS** | All size transitions smooth, mid-typing resize safe |
| Alt+A: Project-wide Search | **PASS** | 9 matches/4 files, scoping, replace all with confirmation |
| Calibrate Keyboard wizard | **TESTED** | 24 steps/5 groups; Ctrl+H NOT tested by wizard |
| #2125 Diagnostics 'q' fix | **CONFIRMED FIXED** | commit 89caf72 verified via UI |
| #2125 *Keyboard Shortcuts* 'q' | **STILL BROKEN** | Shows "Editing disabled in this buffer" |
| #2112 Outside-workspace search | **CONFIRMED FIXED** | commit b7e7e64 verified via UI |

### Issues Filed / Comments
- No new issues filed
- Comment on #2125: Diagnostics panel fixed; *Keyboard Shortcuts* still broken
- Comment on #2112: Confirmed fixed with test procedure

### Cleanup
- Fresh exited via Ctrl+Q; tmux session `fresh-test-run14` killed
- Test files removed: `tmp_test_files/`, `/tmp/rapid_test.txt`, `/tmp/outside_workspace_test.txt`

---

## Run #13 — 2026-05-27

### Status: COMPLETED

### What Was Done
- Loaded state from `tui-automated-testing-state` branch
- Built fresh debug binary from source (`cargo build --bin fresh --features runtime`, ~3.5 min)
  - Binary: `target/debug/fresh`
- Created tmux session `fresh-test` (220×50)
- **Bug Verification (Sprint 12):**
  - TB01: CONFIRMED — `*Keyboard Shortcuts*` 'q' close non-functional (BUG-001)
  - TB02: CONFIRMED — Edit menu "Replace..." mislabeled (BUG-002)
  - TB03: RESOLVED — Alt+W behavior IS correct (context-sensitive, not a bug)
- **GitHub Actions:**
  - Searched for RC12-01: Already covered by issue #2125 → Added comment with Keyboard Shortcuts buffer info
  - Filed new issue #2135 for RC12-02 (Edit menu label mismatch)
- **New Feature Tests:**
  - T28: PASS — Go to Matching Bracket (via command palette; `(` → `)`, `{` → `}`)
  - T30: PASS — Position History (Alt+Left back, Alt+Right forward)
  - T37: PASS — Toggle Line Wrap (View menu ☑ Line Wrap)
  - T45: PASS — Large file (49MB / 500K lines) opens instantly, navigation immediate, search <2s
  - T46: PASS — Binary file (/bin/ls) opens gracefully with [BIN] tag and hex notation

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TB01: Keyboard Shortcuts 'q' close | **CONFIRMED BUG** | "Editing disabled" — same root cause as #2125 |
| TB02: Edit menu Replace label | **CONFIRMED BUG** | Mislabeled "Replace..." → filed #2135 |
| TB03: Alt+W inconsistency | **RESOLVED - NOT A BUG** | Context-sensitive behavior is correct |
| T28: Go to Matching Bracket | **PASS** | Works via command palette |
| T30: Position History | **PASS** | Alt+Left/Right navigate back/forward |
| T37: Toggle Line Wrap | **PASS** | View menu ☑ toggle works both ways |
| T45: Large File Performance | **PASS** | 49MB opened instantly; byte-offset mode |
| T46: Binary File Handling | **PASS** | [BIN] tag; hex notation for non-printable |

### Issues Found / Filed
- Issue #2135 filed: "Edit menu 'Replace...' label maps to Ctrl+Alt+R (Query Replace)"
- Comment on #2125: Keyboard Shortcuts buffer also affected by same root cause

### Key Learnings
- Fresh uses "byte offset mode" for large files (gutter shows bytes, not line numbers)
- Binary files get `[BIN]` tab tag + `<XX>` hex notation for non-printable bytes  
- `Ctrl+]` (ASCII 0x1D) doesn't transmit reliably via tmux send-keys; use command palette for bracket matching
- Alt+W = Close Tab (outside search) is CORRECT behavior; not a bug
- Line Wrap is in View menu (no command palette entry found in this search)

### Cleanup
- Fresh exited via Ctrl+Q
- tmux session `fresh-test` killed
- Test files /tmp/test_brackets.js, /tmp/test_long_line.txt, /tmp/large_test_file.txt deleted

---

## Run #12 — 2026-05-27

### Status: COMPLETED

### What Was Done
- Attempted to load existing state (no local state found → pulled from remote)
- Built fresh 0.3.9 binary from source: `cargo build --release --bin fresh` (~60s)
  - Binary path: `target/release/fresh` (Note: previous runs used `/opt/node22/bin/fresh` via npm)
- Created tmux session `fresh-test` (220×50)
- Executed comprehensive re-verification of Sprints 1-9 (most already tested in Runs 1-11)
- Investigated 2 new potential bugs

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| Sprint 1 (Launch & UI) | **PASS** | All confirmed working as documented |
| Sprint 2 (File Ops) | **PASS** | Ctrl+N/O/S, Alt+W, save dialog all work |
| Sprint 3 (Editing) | **PASS** | Ctrl+Z/Y/C/X/V/A/W/L/D//, all working |
| Sprint 4 (Search/Replace) | **PASS** | Ctrl+F search, Ctrl+R replace, Ctrl+Alt+R query replace all work |
| Sprint 5 (Navigation) | **PASS** | Ctrl+G go-to-line, Command Palette, menu bar |
| Sprint 6 (Command Palette) | **PASS** | All modes (file/>command/:line/#buffer) verified |
| Sprint 7 (Views/Layout) | **PASS** | Split Vertical/Horizontal, File Explorer, Theme Selection |
| Sprint 8 (Tabs/Buffers) | **PASS** | Multi-tab, next/prev buffer, close with confirm dialog |
| Sprint 9 (Terminal) | **PASS** | Integrated terminal, Ctrl+Space toggle, Close Split |
| Settings UI | **PASS** | All categories visible, General settings confirmed |
| Help System | **PASS** | F1 manual, Shift+F1 keyboard shortcuts both open |

### Issues Found This Run

#### BUG-CANDIDATE-RC12-01: Keyboard Shortcuts Buffer 'q' Close Does Not Work
- Buffer text at line 4: "Press 'q' to close this buffer."
- **Actual behavior:** Pressing 'q' shows "Editing disabled in this buffer" in status bar, buffer stays open
- **Workaround:** Use Alt+W
- **Severity:** Low
- **Note:** Check if this is already filed under existing issues before filing new issue
- **Filing blocked:** GitHub MCP token expired this run; file in Run #13

#### BUG-CANDIDATE-RC12-02: Edit Menu "Replace..." Shows Ctrl+Alt+R (Query Replace, Not Basic Replace)
- Edit menu item "Replace..." shortcut = `Ctrl+Alt+R` = opens Query Replace (interactive mode)
- Basic "Replace" (Ctrl+R) is NOT in the Edit menu at all
- Command palette clearly shows two distinct commands: Replace (Ctrl+R) vs Query Replace (Ctrl+Alt+R)
- **Assessment:** May be intentional design, or documentation inconsistency
- **Note:** Already documented in learning_db.md as known behavior; re-verify whether it's a real bug
- **Filing blocked:** GitHub MCP token expired; assess in Run #13

### Key Learnings / Corrections
- Binary can be built from source via `cargo build --release --bin fresh`; binary is `target/release/fresh` not `fresh-editor`
- Binary installed by npm is at `/opt/node22/bin/fresh` (from previous runs); source build works too
- Session persistence confirmed: Unsaved buffers restored on relaunch (hot exit)
- Save/discard dialog confirmed: letter + Enter (not single keypress)
- Keyboard shortcuts buffer cannot be closed with 'q' despite the docs saying so
- Alt+W and Whole Word toggle conflict documented: Alt+W in search bar = toggle whole word; outside search = close tab
- Block selection tmux keys: `M-S-Down` appears to NOT trigger block select reliably in this tmux version (investigation needed)

### Cleanup
- tmux sessions `fresh-test` and `quit-test` both killed
- No test files left behind on disk (all were in /tmp)

---

## Run #11 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh 0.3.9 binary from `claude/ecstatic-mayer-5DivD` branch (6.5 min build)
- Checked out `tui-automated-testing-state` branch, loaded all prior state
- Launched tmux session `fresh-qa` (200×50)
- Executed 10+ test objectives covering bookmarks, Settings add/delete/reset, and LSP with fake-pylsp

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TC-BOOKMARKS | **PASSED** | Alt+1/2/etc jump to bookmarks 1/2/etc; "not set" for missing; Ctrl+P → "Set Bookmark" |
| TC-SETTINGS-ADD-NEW | **PASSED** | Typing while focused on list header activates [+] Add new inline input; Enter confirms |
| TC-SETTINGS-CTRL-R | **RESOLVED/PARTIAL** | Ctrl+R is a NO-OP for field reset; Escape from field reverts pending changes; [ Reset ] button via Tab works |
| TC-SETTINGS-DEL-X | **PENDING** | [x] buttons appear mouse-only; keyboard navigation to sub-list items not confirmed |
| TC-FAKE-LSP | **PASSED** | fake-pylsp recognized as `pylsp`; LSP starts; connection handshake logged |
| TC-LSP-GOTO-DEF | **PASSED** | F12 Go to Definition works; navigates to LSP-returned location |
| TC-LSP-HOVER | **PARTIAL** | Alt+K shows "No hover info available" (expected with fake-pylsp null response) |
| TC-LSP-REFERENCES | **PASSED** | Find References opens dock panel with clickable results; Enter navigates correctly |
| TC-REFERENCES-NAV | **CONFIRMED** | References panel Enter WORKS (unlike *Quickfix* BUG #2124) |

### Issues Found This Run
- **0 new bugs filed**
- **1 important distinction**: References panel (from LSP Find References) correctly handles Enter navigation — this is DIFFERENT from *Quickfix* buffer (BUG #2124 which is from Live Grep Alt+M)
- **Ctrl+R in Settings**: Does NOT reset number fields — CHANGELOG claim may be incorrect for 0.3.9

### Key Learnings
- Binary 0.3.9 confirmed from `fresh --version`
- Bookmarks: `Ctrl+P → Set Bookmark → digit → Enter`; jump with `Alt+N`
- Settings list [+] Add new: type text directly while header is focused (no Enter needed to start)
- Settings [x] delete: likely mouse-only (no keyboard path found)
- Escape from Settings pending field: REVERTS changes (useful as keyboard reset)
- fake-pylsp setup: symlink `scripts/fake-lsp/bin/fake-pylsp` → `/usr/local/bin/pylsp`; set `FAKE_DEVCONTAINER_STATE` env
- LSP Find References panel IS keyboard-navigable (Enter works); bug is specific to *Quickfix*

---

## Run #10 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh binary from source (tui-automated-testing-state base = `88883dc`, v0.3.8)
- Launched tmux session `fresh-test` (200×50)
- Executed 7 test objectives: Alt+/, Markdown Preview, Keyboard Macros, Settings Ctrl+R, Review Diff regression check

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TC-ALT-SLASH | **PASSED** | `M-/` opens Live Grep; 375 results for "fn main"; preview split works |
| TC-MARKDOWN | **PASSED** | Markdown Compose mode: ANSI bold/italic/headings; status "Markdown Compose: ON (soft breaks, centered)" |
| TC-MACRO-RECORD | **PASSED** | "Record Macro" prompt (0-9), F5 stops, macro saved with action count |
| TC-MACRO-PLAYBACK | **PASSED** | F4 plays macro correctly; all 3 test lines got " [MACRO]" appended |
| TC-MACROS-LIST | **PASSED** | "List Macros" opens `*Macros*` buffer; WARNING: buffer is editable (not strict RO) |
| TC-SETTINGS-CTRL-R | **PARTIAL** | Ctrl+R when field highlighted does NOT reset; `[ Reset ]` button reachable via Tab; full test inconclusive |
| TC-REVIEW-DIFF-CONTROLS | **FALSE POSITIVE CORRECTED** | All controls broken BY DESIGN — per `docs/internal/review-diff-feature-restoration-plan.md` (Status: Planned) |

### Issues Found This Run
- **0 new bugs filed**
- **1 false positive corrected**: Run #8 TC-REVIEW-DIFF-DISCARD "PASSED" was wrong; Review Diff panel controls were never implemented in this codebase version

### Key Learnings
- Version is 0.3.8 (not 0.3.9 as previously logged)
- Review Diff panel controls are planned-but-not-implemented features
- DECCKM `$'\033OB'` must be UNQUOTED in bash (not inside double quotes)
- `Explorer` menu item appears in menu bar when File Explorer is used
- `*Macros*` buffer is editable (different from strictly-RO Quickfix/Diagnostics)

---

## Run #9 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh 0.3.9 binary from source (`cargo build --release --bin fresh`, ~3 min)
- Checked out `tui-automated-testing-state` branch, loaded state from 8 prior runs
- Launched tmux session `fresh-test` (200×50)
- Executed 8+ test objectives covering LSP popup navigation, Quickfix navigation, shell commands, multi-cursor, diagnostics panel, and backlog items

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TC-LSP-POPUP-NAV-2 | **CONFIRMED** | Plain `Up`/`Down` keys navigate popup; ANSI `[48;5;25m]` highlight confirms selection |
| TC-QUICKFIX-ENTER | **BUG FOUND** | Enter → "Editing disabled"; no navigation keybindings for Quickfix; BUG #2124 filed |
| TC-DIAG-PANEL-SHORTCUTS | **BUG FOUND** | q/a/Enter all → "Editing disabled"; status hints are non-functional; BUG #2125 filed |
| TC-SETTINGS-CTRL-R | **PARTIAL** | Ctrl+R in Settings closes the overlay; `[ Reset ]` footer button not reachable via Tab cycling |
| TC-SHELL-CMD | **PASSED** | `Alt+|` → "Shell command:" prompt → sort → `*Shell: sort*` tab with sorted output |
| TC-SHELL-CMD-REPLACE | **PASSED** | `Shell Command (Replace)` via palette → `sort -r` → in-place replacement confirmed |
| TC-MULTICURSOR-LINE-ENDS | **PASSED** | `M-I` (Alt+Shift+I) on 5 lines → "6 cursors | Added cursors to line ends (6)" |
| TC-BUG2122-RECHECK | **STILL OPEN** | `move_to_paragraph_down/up` still have no keybinding; select variants have `Ctrl+Shift+↓/↑` |

### Issues Found This Run
- **BUG #2124 filed**: Quickfix buffer `Enter` shows "Editing disabled" — no jump-to-match behavior despite design spec requiring it
- **BUG #2125 filed**: Diagnostics panel `q/a/RET` shortcuts are non-functional — status bar hints are misleading

### Key Discoveries This Run
1. **Quickfix buffer has no navigation keybindings**: Searching Keybinding Editor for `/quickfix` only shows export bindings (Alt+M, Alt+Q in `prompt` context). The design doc says Enter should navigate but this was never implemented.
2. **Diagnostics panel shortcuts don't work**: The `q: close | a: toggle filter | RET: goto` hints in the status bar and `Enter:select | Esc:close` panel body text are misleading — these shortcuts are not bound.
3. **Shell Command feature fully confirmed**: Both `Alt+|` (output to new buffer) and `Shell Command (Replace)` (output replaces selection) work correctly. Tested with `sort` and `sort -r`.
4. **Add Cursors to Line Ends (`M-I`) confirmed working**: 5-line selection → 6 cursors at line ends. Status bar shows confirmation message.
5. **Fake LSP (`scripts/fake-lsp/bin/fake-pylsp`) discovered**: Requires `FAKE_DEVCONTAINER_STATE` env var. Could unlock LSP feature testing in future runs.
6. **Settings UI Ctrl+R investigation**: The `Ctrl+R` key closes Settings overlay (routes to global Find & Replace). The `[ Reset ]` button is in the footer but not reachable via Tab cycling in the tested workflow. Needs further investigation.
7. **Settings keystroke leak confirmed**: Navigating Settings with Tab and search can leak keystrokes into editor. Config file was accidentally modified during testing (restored manually).

### Lessons Learned
See learning_db.md for additions: Lesson 44–50

---

## Run #8 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh 0.3.9 binary from source (`cargo build --release --bin fresh`, ~3 min)
- Pulled state from `tui-automated-testing-state` branch (7 prior runs)
- Launched tmux session `fresh-test` (200×50)
- Executed 10 test objectives covering 0.3.9 features, bug regression checks, and new discoveries

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TC-LSP-STATUS | **PASSED** | LSP status popup shows server state; auto-opens log tab on failure; states: (off)/(error)/running |
| TC-LSP-POPUP-NAV | **DISCOVERED** | DECCKM sequences (ESC prefix) CLOSE popups; use plain Up/Down for popup nav |
| TC-LIVE-GREP-DIAG | **PASSED** | Alt+D toggles Diagnostics scope; "No matches" without LSP (expected); provider disappears |
| TC-LIVE-GREP-ALTM | **PASSED** | Alt+M saves to `*Quickfix*` [RO] buffer in split; format: `file:line:col  content` |
| TC-ORCHESTRATOR-0.3.9 | **PASSED** | New UI: Alt+P project scope, Alt+T show worktrees, `/` filter, session detail buttons |
| TC-C3-LANGUAGE | **PASSED** | C3 syntax highlighting fully working; `C3` status bar; folding at fn/struct |
| TC-REVIEW-DIFF-DISCARD | **BUG FIXED** | BUG #2117 CONFIRMED FIXED in 0.3.9 — discard works correctly; comment on GH issue |
| TC-WORKSPACE-RESTORE-2056 | **PASSED** | Session isolation by working directory confirmed; no cross-project tab mixing |
| TC-PLUGIN-API-DATADIRS | **DOCUMENTED** | getWorkingDataDir() and getTerminalDir() documented from API types |

### Issues Found This Run
- **None filed** — BUG #2117 resolved; all other behaviors working as expected or documented

### Key Discoveries This Run
1. **BUG #2117 (Review Diff discard) FIXED**: Confirmed working in 0.3.9 dev build. Tested twice. Comment posted on GitHub.
2. **Popup navigation insight**: DECCKM sequences (`$'\033OA'`, `$'\033OB'`) start with ESC which CLOSES any active overlay/popup. For popup list navigation, use plain tmux key names (`Up`, `Down`). DECCKM only applies to cursor movement inside the editor buffer.
3. **C3 language support**: Full syntax highlighting with Sublime syntax grammar. `.c3`, `.c3i`, `.c3t` extensions. c3lsp configured but not bundled.
4. **Orchestrator 0.3.9 UI**: New project scope filter (Alt+P), show-all-worktrees toggle (Alt+T), `/` filter search, session detail action buttons (Visit/Details/Stop/Archive/Delete).
5. **Live Grep Alt+M Quickfix buffer**: Saves all matches to `*Quickfix*` [RO] buffer with `file:line:col  content` format, 249 matches saved correctly.
6. **LSP (error) state**: When LSP binary missing: Fresh tries to start it, immediately opens the log file as a [RO] tab, status bar shows `LSP (error)`. Log shows the exact error (e.g., `Unknown binary 'rust-analyzer' in official toolchain`).

### Lessons Learned
See learning_db.md for additions: Lesson 35–43

---

## Run #7 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh 0.3.9 binary from source (`cargo build --release --bin fresh`)
- Pulled state from `tui-automated-testing-state` branch (6 prior runs)
- Launched tmux session `fresh-test` (200×50)
- Executed 12 test objectives covering 0.3.9 new features and backlog items

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TC-DASHBOARD-DEFAULT | **CONFIRMED** | 0.3.9: Dashboard no longer opens by default with `--no-restore` |
| TC-PARA-SELECT | **PASSED** | select_to_paragraph_down/up work via Ctrl+Shift+↓/↑ (CSI 1;6B / CSI 1;6A escape sequences) |
| TC-SETTINGS-CHECKBOX | **RESOLVED** | Checkboxes ARE reachable: ↑↓ arrows navigate to them in the right panel; Enter toggles them |
| TC-CONFIRM-QUIT | **PASSED** | Enable in Settings → "Confirm Quit: [ ]" → Enter → Save; Ctrl+Q shows `Quit Fresh? (y)es, (N)o:` |
| TC-SCROLL-SYNC | **PASSED** | Both splits scroll together when Scroll Sync enabled; confirmed with CHANGELOG.md in both panes |
| TC-AUTO-REVERT | **PASSED** | External file append detected and reverted within ~3s (auto_revert_poll = 2000ms default) |
| TC-NEXT-WINDOW | **TESTED** | "Next Window" returns "Cancelled" when only 1 window open — correct single-window behavior |
| TC-LIVE-GREP-0.3.9 | **PASSED** | New toolbar working: scope toggles (Files/Buffers/Terminals), provider cycle, [buf] tag, Word mode |
| TC-PAGEDOWN-OVERSHOOT | **PASSED** | Basic PageDown/PageUp navigation correct; targeted fix hard to confirm without bug repro file |
| TC-COMPLETION-AUTO-SHOW | **PARTIAL** | Setting toggles correctly; popup requires LSP (off) — not testable without LSP server |
| TC-PARA-MOVE-BUG | **BUG CONFIRMED** | move_to_paragraph_down/up have NO default keybinding and are NOT in command palette → GitHub #2122 filed |
| TC-BUG-2117-CHECK | **STILL OPEN** | Review Diff discard bug NOT fixed in 0.3.9 (not in changelog fixes) |

### Issues Found This Run
- **BUG #2122 filed**: `move_to_paragraph_down/up` actions (0.3.9 feature) have no default keybinding and no command palette entry. Users cannot invoke the feature without manually binding it. Inconsistent with `select_to_paragraph_*` which have `Ctrl+Shift+↓/↑`.

### Key Discoveries This Run
1. **Settings checkboxes via keyboard**: Navigate with ↑↓ arrows (DECCKM) in the right panel, press Enter to toggle. This DOES work — previous run's concern was unfounded. Tab only reaches number/text inputs.
2. **select_to_paragraph escape sequences**: CSI 1;6B = Ctrl+Shift+Down, CSI 1;6A = Ctrl+Shift+Up — confirmed working
3. **Live Grep 0.3.9**: Provider shows as `[ git-grep ]`, `[ rg ]`, `[ grep ]` when cycling with Alt+P. File scope results untagged; Buffer scope results show `[buf]` prefix.
4. **confirm_quit prompt format**: Shows `Quit Fresh? (y)es, (N)o:` at bottom line, requires letter + Enter (N+Enter = stays open, Y+Enter = quits).
5. **Settings search**: Press `/` in Settings UI while in the LEFT panel to trigger search across all setting names (not just visible category).
6. **move_to_paragraph design intent** (from PR #2084): Author intentionally omitted palette commands but appears to have overlooked adding default keybindings — `select_to_paragraph` has bindings but the new `move_to_paragraph` does not.

### Lessons Learned
See learning_db.md for additions: Lesson 29–34

---

## Run #6 — 2026-05-26

### Status: COMPLETED

### What Was Done
- Built Fresh binary from source (`cargo build --release --bin fresh`, ~50s)
- Checked out `tui-automated-testing-state` branch, loaded state from 5 prior runs
- Launched tmux session `fresh_test` (200×50)
- Executed 7 test objectives covering theme editor, auto-save, env manager, tour, review diff, orchestrator, workspace trust

### Test Results Summary
| Test | Result | Notes |
|------|--------|-------|
| TC-THEME-EDITOR (complete) | **PASSED** | Color edit + Save As → custom theme created in ~/.config/fresh/themes/ |
| TC-AUTO-SAVE | **PASSED** | Enable in config; file auto-saved within 8s (5s interval); tab loses asterisk |
| TC-ENV-MANAGER | **PASSED** | Show Status → Activate (direnv) → Deactivate: all 3 commands working |
| TC-TOUR | **PASSED** | Load .fresh-tour.json; navigate Step 1→2→3→4→Exit; each step opens correct file |
| TC-REVIEWDIFF-STAGE | **PASSED** | Stage hunk with `s`: 3 added lines moved to STAGED section |
| TC-ORCHESTRATOR-NEW | **PASSED** | Alt+N → form → Tab×6 to Create Session → session-1 worktree created |
| TC-WORKSPACE-TRUST | **PASSED** | T to trust → status bar confirms "Workspace trusted" |

### Issues Found This Run
- **PENDING BUG INVESTIGATION**: Settings UI checkboxes NOT reachable via Tab key. Tab navigates to number/text inputs and footer buttons, skipping checkboxes (e.g., "Auto Save Enabled"). Needs investigation whether this is by design or a bug.
- **NOTE**: Orchestrator "Create Session" button requires exactly 6 Tab presses from the dialog open state to reach the button. More than 6 = cycles back to checkbox. Important UX discovery.
- **NOTE**: Tour panel button navigation: Tab focuses buttons, Up/Down navigates within tour panel. Pressing Enter when "Next →" is focused advances the tour.

### False Positive Rate: 0% (0 of 0 bugs filed)

### Settings Navigation Discovery
The Settings UI uses a complex navigation model:
- `↑↓` in left panel: navigate sections
- `Tab`: jump to next focusable widget IN THE RIGHT PANEL (number inputs and text inputs only; checkboxes are NOT tab-navigable)
- `Enter` on section: scrolls right panel to show that section
- Auto-save was enabled by directly editing /root/.config/fresh/config.json (demonstrated it persists and works)

---

## Run #1 — 2026-05-26

### Status: COMPLETED (with post-run self-correction)

### What Was Done
- Built Fresh binary from source (`cargo build --release --bin fresh`, 16s)
- Initialized all state files for the first time
- Launched tmux session, executed 30+ test cases across core launch, file ops, editing, search/replace, and views
- Filed 4 GitHub issues
- **Post-run:** Reviewed documentation, discovered 2 of 4 issues were false positives
- Closed #2108 and #2110, updated #2109 and #2111

### Test Results Summary
| Category | Passed | Failed | Notes |
|----------|--------|--------|-------|
| Core launch (TC-001–011) | 11 | 0 | |
| File operations (TC-020–026) | 7 | 0 | |
| Editing (TC-030–035) | 6 | 0 | |
| Search & replace (TC-040–049) | 8 | 1 | TC-043 Shift+F3 broken in tmux (terminal compat) |
| Views & layout (TC-050–058) | 9 | 0 | |
| Issues filed | 4 | — | 2 real (#2109, #2111); 2 false positives (#2108, #2110) |

### Lessons Learned (Run #1)
- Arrow key DECCKM requirement discovered
- Menu highlight verification requires `-e` ANSI capture
- Hot exit causes file restoration on re-launch — not a bug
- "Revert" vs "Reload with Encoding" distinction
