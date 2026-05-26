# TUI Agent Run Log

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
