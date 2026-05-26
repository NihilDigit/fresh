# Fresh Editor - Automated TUI Test Plan

## PROCESS RULES (added after Run #1 false positives)
1. **Read docs FIRST.** Before any test session, skim `docs/features/` and `docs/blog/` for the version under test.
2. **Verify menu navigation with ANSI capture** (`-e` flag) to confirm the highlighted item before asserting behavior.
3. **Check the CHANGELOG** for features that could explain "surprising" behavior before filing a bug.
4. **Test keyboard shortcuts bare** (no tmux shortcuts that might intercept). If a key acts unexpectedly, check for terminal compatibility issues before blaming Fresh.
5. **Never file a bug based on a single observation.** Always reproduce at least twice.
6. **Launch clean for fresh-state tests:** Use `fresh --no-restore` to skip hot-exit restoration when testing initial launch behavior.
7. **Before filing an issue, you must be able to state:**
   - The exact expected behavior (and why — cite VS Code/docs/convention)
   - The exact actual behavior
   - That you've reproduced it at least twice
   If you can't state all three, add a pending test case here instead and file later.
8. **Issue titles must state the problem, not the investigation.** "F3 does not navigate while search bar is open" ✓ — "F3 navigation not verified" ✗. See `learning_db.md` → ISSUE FILING STANDARDS for the full template and rules.

---

## Run History
| Run # | Date | Status | Tests Run | Bugs Found |
|-------|------|--------|-----------|------------|
| 1     | 2026-05-26 | COMPLETED | 30+ | 4 filed → 2 real, 2 false positives |
| 2     | 2026-05-26 | COMPLETED | 20+ | 2 filed → 2 real, 0 false positives |
| 3     | 2026-05-26 | COMPLETED | 20+ | 0 filed → 0 confirmed new bugs |
| 4     | 2026-05-26 | COMPLETED | 30+ | 0 filed → 0 confirmed new bugs |
| 5     | 2026-05-26 | COMPLETED | 15+ | 1 filed → 1 real bug (#2117) |
| 6     | 2026-05-26 | COMPLETED | 7   | 0 filed → 0 confirmed new bugs; 1 PENDING investigation |
| 7     | 2026-05-26 | COMPLETED | 12  | 1 filed → 1 real bug (#2122) |

---

## Priority 1: Core Editor Launch & Basic UX (COMPLETED - Run #1)
### Objective: Verify basic launch, UI elements, and fundamental key bindings work.

- [x] **TC-001** PASSED - Launch fresh with no arguments → Shows dashboard with git/disk info
- [x] **TC-002** PASSED - Launch fresh with file argument → File loads correctly (hot exit restores previous session state — BY DESIGN)
- [x] **TC-003** PASSED - Menu bar visible, keyboard navigable (F10 or Alt+letter), subtle highlight
- [x] **TC-004** PASSED - Status bar visible with file info, cursor position, and indicators
- [x] **TC-005** PASSED - Ctrl+P opens command palette with full command list
- [x] **TC-006** PASSED - Escape closes command palette
- [x] **TC-007** PASSED - Typing text works, cursor position shown in status bar
- [x] **TC-008** PASSED - Ctrl+Z undo works; Ctrl+Y redo also works
- [x] **TC-009** PASSED - Ctrl+S on new file opens Save As dialog with file browser
- [x] **TC-010** PASSED - Close Buffer with unsaved changes prompts `(s)ave, (d)iscard, (C)ancel?`
           NOTE: Ctrl+W is "Select word under cursor" (NOT close buffer — different from VS Code!)
           NOTE: Close Buffer has no default shortcut. Use: Ctrl+P → "Close Buffer"
- [x] **TC-011** PASSED - Ctrl+Q exits Fresh cleanly

---

## Priority 2: File Operations (PARTIALLY COMPLETED - Run #1)
### Objective: Verify open, save, new, close workflows

- [x] **TC-020** PASSED - Ctrl+N creates blank editor with "[No Name]" tab
- [x] **TC-021** PASSED - Ctrl+O opens file browser dialog
- [x] **TC-022** PASSED - Can type path and open existing file
- [x] **TC-023** PASSED - Ctrl+S on new file prompts Save As
- [x] **TC-024** PASSED - Ctrl+S on saved file saves immediately (status: "Saved")
- [x] **TC-025** PASSED - Save As via File menu (Alt+F → Save As); pre-fills path; no palette command
          NOTE: Ctrl+Shift+S is NOT reliable in terminals (shift stripped, becomes Ctrl+S)
- [x] **TC-026** PASSED - "Close Buffer" command prompts `(s)ave, (d)iscard, (C)ancel?` for unsaved
          NOTE: In Run #3, prompt required letter + Enter to confirm (not just the letter)
- [x] **TC-027** PASSED - Close saved file (Alt+W): closes immediately without dialog
- [x] **TC-028** PASSED - Multiple files open → tabs shown in tab bar
- [x] **TC-029** PASSED - Ctrl+PgDn / Ctrl+PgUp = Next/Previous Buffer (NOT Ctrl+Tab)
          NOTE: Ctrl+Tab in tmux sends Tab character to buffer — DO NOT use

---

## Priority 3: Editing Features (PARTIALLY COMPLETED - Run #1)
### Objective: Verify editing workflows

- [x] **TC-030** PASSED - Undo (Ctrl+Z) and redo (Ctrl+Y) work across multiple steps
- [x] **TC-031** PASSED - Shift+Left/Right selects text (cursor shown as reversed, selection as blue)
- [x] **TC-032** PASSED - Ctrl+A selects all text
- [x] **TC-033** PASSED - Copy (Ctrl+C) and Paste (Ctrl+V) work correctly
- [x] **TC-034** PASSED - Cut (Ctrl+X): cuts selected text; Ctrl+V pastes correctly
- [x] **TC-035** PASSED - Ctrl+D adds cursor at next match, multi-cursor editing confirmed working
- [x] **TC-036** PASSED - Block selection: Alt+Shift+Down extends column downward, Alt+Shift+Right extends right
          Typing replaces block simultaneously across all affected rows
- [x] **TC-037** PASSED - Ctrl+/ toggles line comment for JS/language files; no effect on .txt (no language)
- [x] **TC-038** PASSED - Auto-indent: Enter after `{` inserts indented line at correct level

---

## Priority 4: Search & Replace (PARTIALLY COMPLETED - Run #1)
### Objective: Verify search and replace workflows

- [x] **TC-040** PASSED - Ctrl+F opens search bar with case-sensitive/whole-word/regex toggles
- [x] **TC-041** PASSED - Search highlights all matches in ANSI colors
- [x] **TC-042** PARTIAL - Enter navigates to first match then CLOSES search bar
          ⚠️ BUG-004 (confirmed): F3 silently ignored while search bar is open. Correct workflow:
          Enter → closes bar → F3 navigates next. But this contradicts VS Code/browser behavior.
- [x] **TC-043** PARTIAL - Shift+F3 for previous match: NOT recognized in tmux (S-F3 not forwarded)
          Find Previous works via command palette (binding shown as Ctrl+Shift+N, but also broken in tmux)
          → PENDING: test in proper terminal to confirm if Shift+F3 works natively
- [x] **TC-044** PASSED - Escape closes search bar
- [x] **TC-045** TERMINAL COMPAT ISSUE - Ctrl+H IS intended to open find & replace (documented)
          but terminals send Ctrl+H as Backspace (0x08). Use Ctrl+R as the reliable Replace shortcut.
          Issue #2109 open: suggests adding Ctrl+H to Calibrate Keyboard wizard and documenting the conflict.
- [x] **TC-046** PASSED (via Ctrl+R) - Replace All works by default
- [x] **TC-047** PASSED - All 3 occurrences replaced simultaneously
- [x] **TC-048** PASSED - Case-sensitive toggle (Alt+C): status bar confirms toggle on/off
- [x] **TC-049** PASSED - Regex toggle (Alt+R): regex mode confirmed; actual regex matching works (e.g. `line\..*`)

---

## Priority 5: Views & Layout (PARTIALLY COMPLETED - Run #1)
### Objective: Verify split panes, file explorer, terminal

- [x] **TC-050** PASSED - "Split Vertical" via command palette splits horizontally (stacked)
          NOTE: Ctrl+\ not confirmed to work; use command palette → "Split Vertical"
          NOTE: "Split Vertical" creates horizontal layout (two panes stacked)
- [x] **TC-051** PASSED - Alt+] switches to next split pane
- [x] **TC-052** PASSED - "Close Split" command closes the split pane
- [x] **TC-053** PASSED - Ctrl+B toggles File Explorer
          NOTE: Ctrl+E switches focus between editor and file explorer (does NOT toggle open/close)
- [x] **TC-054** PASSED - DECCKM arrow keys navigate directories; Right expands, Left collapses
- [x] **TC-055** PASSED - File Explorer: arrow navigation auto-previews files; Enter opens as permanent tab
          NOTE: Focus workflow: Ctrl+B to open, Ctrl+E to focus, DECCKM arrows to navigate, Enter to open
- [x] **TC-056** PASSED - Toggle line numbers via command palette "Toggle Line Numbers"
- [x] **TC-057** PASSED - Toggle line wrap via View menu (☑ = on, ☐ = off); status bar confirms
          NOTE: "Toggle Line Wrap" is NOT in command palette — use View menu (Alt+V)
- [x] **TC-058** PASSED - Integrated terminal (more features):
          - Ctrl+Space: toggles terminal mode ↔ scrollback (read-only) mode
          - Ctrl+F: searches in terminal scrollback
          - Status bar shows "Terminal mode enabled/disabled" and "Terminal [capture]" for F9 capture mode

---

## Priority 6: Command Palette (COMPLETED - Run #4)
### Objective: Verify command palette completeness

- [x] **TC-060** PASSED - Ctrl+P opens palette with `>` prefix, shows all commands
- [x] **TC-061** PASSED - "File" search shows New File, Open File, Git Find File, Copy File Path, etc.
- [x] **TC-062** PASSED - "Theme" search shows Edit Theme, Inspect Theme at Cursor, Select Theme
- [x] **TC-063** PASSED - Execute command (Toggle Line Numbers) → line numbers hidden; status bar confirmed
- [x] **TC-064** PASSED - Fuzzy search: "tog num" → "Toggle Line Numbers" as top result
- [x] **TC-065** PASSED - Buffer switch: `#sample` → Enter switches to sample.txt tab

---

## Priority 7: Settings & Configuration (COMPLETED - Run #4)
### Objective: Verify settings access and persistence

- [x] **TC-070** PASSED - `Ctrl+P → "Open Settings"` opens full settings UI with category nav panel + rich controls
- [x] **TC-071** PASSED - `Ctrl+P → "Select Theme"` → theme picker with 8 built-in themes; "Theme changed to 'dark'" in status bar
- [x] **TC-072** PASSED - `Ctrl+P → "Keybinding Editor"` opens 843-binding table with / search, r record-key search, filter by context/source
- [x] **TC-073** PASSED - Theme persisted after quit+relaunch: /root/.config/fresh/config.json shows `"theme": "builtin://dark"`

---

## Priority 8: Edge Cases & Stress Tests (COMPLETED - Run #5)
### Objective: Find stability issues

- [x] **TC-080** PASSED - Large file (159MB text): byte offsets in gutter, "Scan Line Index" builds line index, navigation to line 1,000,000 works
- [x] **TC-081** PASSED - Binary file: opens as `[BIN]` tab, content as `<FF><FE>..`, marked `[RO]`, no crash
- [x] **TC-082** PASSED - Empty file: opens with single blank line, `~` for empty buffer, editable
- [x] **TC-083** PASSED - Rapid key presses: burst text input received all characters; 100+ rapid undo stable
- [x] **TC-084** PASSED - 12 simultaneous tabs (4 original + 8 new): tab bar scrolls, Ctrl+PgDn/Up switches correctly
- [x] **TC-085** PASSED - Resize 200x50 → 40x12 → 200x50: graceful reflow, no crash

---

## Backlog (Future Runs)
- LSP features (go to definition, hover, diagnostics) — requires LSP server installed
- Plugin system testing: installing a plugin from URL
- Scroll Sync (split view with same buffer)
- Current line/column highlight toggle
- Auto-save behavior
- Theme editor: complete color editing workflow (requires mouse or precise keyboard navigation)
- Review Diff: verify BUG #2117 is fixed when a fix is released
- Environment manager: test `Env: Activate` on a project with `.envrc` or `.venv`
- Workspace Trust: test setting trust level (T to trust, K to restrict) and verifying LSP behavior changes
- Tour feature: `Ctrl+P → "Tour: Load Definition..."` — test `.fresh-tour.json`
- Diagnostics panel: test inline diagnostics toggle (enable `diagnostics_inline_text`)
- `confirm_quit` setting: enable and verify quit prompt appears
- `auto_save_enabled`: test auto-save interval behavior
- Multi-window: test Orchestrator "New Session" spawning a second window
- "Review Range (Commit or Branch)": test reviewing a specific git range

---

## Completed Tests (Run #5)
- [x] **TC-080** PASSED - Large file (159MB), byte offsets, Scan Line Index, line 1,000,000 navigation
- [x] **TC-RUST** PASSED - Rust syntax highlighting (keywords, functions, strings, numbers in ANSI)
- [x] **TC-PYTHON** PASSED - Python syntax highlighting confirmed
- [x] **TC-JS** PASSED - JavaScript syntax highlighting confirmed  
- [x] **TC-FOLD** PASSED - Code folding: ▸/▾, Toggle Fold, navigation skips folded regions
- [x] **TC-BLAME** PASSED - Git Blame: commit blocks, b/q navigation
- [x] **TC-REVIEWDIFF** PASSED (partial) - Review Diff opens, shows hunks, n/p nav; discard BUGS (#2117)
- [x] **TC-WHITESPACE** PASSED - Trailing spaces show as ··· indicators
- [x] **TC-ENCODING** PASSED - Latin-1 file: Windows-1252 auto-detected, éàñ decoded
- [x] **TC-THEME-EDITOR** PARTIAL - Opens, shows colors, field selection via ANSI; editing keyboard workflow unclear
- [x] **TC-MOVE-EXPLORER** PASSED - "Move File Explorer to Other Side" works (0.3.8)
- [x] **TC-LIVE-DIFF** PASSED - Live Diff: vs HEAD shows green + lines, status confirms mode
- [x] **TC-RULERS** PASSED - Add Ruler at col 80 shows tinted column
- [x] **TC-ORCHESTRATOR** PASSED - Orchestrator: Open shows session selector
- [x] **TC-WORKSPACE-TRUST** PASSED - Workspace Trust dialog: ⚠ warning, T/K options, .envrc detected

## Completed Tests (Run #6)
- [x] **TC-THEME-EDITOR** PASSED (complete) - Color editing: navigate → Enter → type hex → confirm; Save As creates ~/.config/fresh/themes/my-test-theme.json
- [x] **TC-AUTO-SAVE** PASSED - Enable in config; edit file; wait >8s; tab loses asterisk + status bar loses [+]
- [x] **TC-ENV-MANAGER** PASSED - Show Status → Activate (direnv) → Deactivate; "Environment active (direnv)" status confirmed
- [x] **TC-TOUR** PASSED - Load .fresh-tour.json; all 4 steps navigate correctly; Exit Tour works; status: "Tour ended"
- [x] **TC-REVIEWDIFF-STAGE** PASSED - Stage hunk: 3 lines moved from UNSTAGED to STAGED; 'n' navigates hunk; 's' stages
- [x] **TC-ORCHESTRATOR-NEW** PASSED - Orchestrator: New Session; Alt+N opens form; Tab×6 to Create Session; worktree created
- [x] **TC-WORKSPACE-TRUST-SET** PASSED - Press T to trust in new session; status: "Workspace trusted — project tooling may run processes"

---

## Completed Tests (Run #7)
- [x] **TC-DASHBOARD-DEFAULT** CONFIRMED - Fresh 0.3.9 no longer opens dashboard automatically with `--no-restore`
- [x] **TC-PARA-SELECT** PASSED - select_to_paragraph_down/up work (CSI 1;6B / CSI 1;6A escape sequences)
- [x] **TC-SETTINGS-CHECKBOX** RESOLVED - Checkboxes ARE keyboard-navigable: ↑↓ arrows in right panel, Enter to toggle
- [x] **TC-CONFIRM-QUIT** PASSED - `Quit Fresh? (y)es, (N)o:` prompt appears when enabled; letter+Enter to confirm
- [x] **TC-SCROLL-SYNC** PASSED - Both panes scroll synchronously with same buffer open in each
- [x] **TC-AUTO-REVERT** PASSED - External file modification detected and auto-reverted within ~3s
- [x] **TC-NEXT-WINDOW** TESTED - "Cancelled" when single window open (correct); multi-window requires Orchestrator
- [x] **TC-LIVE-GREP-0.3.9** PASSED - Scope toggles (Files/Buffers/Terminals/Diagnostics), provider cycle, Word/Regex toggles all working
- [x] **TC-PAGEDOWN-OVERSHOOT** BASIC-PASS - PageDown/PageUp navigate correctly on wrapped file; overshoot repro hard to construct
- [x] **TC-COMPLETION-AUTO-SHOW** LIMITED - Setting toggles correctly; popup requires LSP (currently off)
- [x] **TC-PARA-MOVE-BUG** BUG FILED - move_to_paragraph_down/up inaccessible; Issue #2122 opened

---

## Immediate Next Action (Run #8)

### FIRST: State Check
- Version is 0.3.9 (Cargo.toml) — built from master
- BUG #2117 (Review Diff discard): still open, not fixed in 0.3.9
- BUG #2122 (move_to_paragraph no keybinding): newly filed
- Config state: auto_save OFF, confirm_quit OFF, completion_popup_auto_show OFF (all restored to defaults)

### Priority Tests for Run #8:
1. **LSP status indicator deep-dive**
   - Open a Rust file from the codebase (e.g., `crates/fresh-core/src/action.rs`)
   - Observe LSP indicator in status bar — does it offer to download/configure an LSP?
   - Try `Ctrl+P → "LSP: ..."` commands and document what's available without a server

2. **Orchestrator: Attach to existing worktree** (new 0.3.9 feature)
   - `Ctrl+P → "Orchestrator: Open"` → check if existing worktrees are listed (session-1 from Run #6)
   - Test "Show all worktrees" toggle and bulk actions (multi-select)
   - Document the new UI improvements

3. **Live Grep: Diagnostics scope** 
   - `Ctrl+P → "Live Grep"` → enable Diagnostics scope (Alt+D)
   - Search for something; verify diagnostics scope works or note behavior

4. **Live Grep: Alt+M save matches**
   - Run Live Grep with results → press Alt+M → verify matches saved to a buffer
   - Document format of saved matches

5. **Review Diff discard re-test**
   - Make a small edit, save, `Ctrl+P → "Review Diff"`, navigate to hunk, press `d` (discard)
   - Verify BUG #2117 is still present in 0.3.9 and update the issue

6. **C3 language support** (new 0.3.9 feature)
   - Create a file with `.c3` extension containing basic C3 syntax
   - Verify syntax highlighting activates

7. **Plugin API: getWorkingDataDir / getTerminalDir** (new 0.3.9)
   - Check if any built-in plugins expose these APIs or if they're only for custom plugins
   - Document behavior in learning_db

8. **Workspace restore: multiple projects** (0.3.9 fix — #2056)
   - Verify that reopening Fresh after viewing files from the /tmp directory doesn't mix tabs from multiple projects

### CRITICAL Reminders for Run #8:
- **Settings checkboxes**: ↑↓ arrows in the right panel → Enter to toggle (NOT Tab)
- **Settings search**: Press `/` in Settings UI left panel to search settings
- **Paragraph navigation**: move_to_paragraph_* has no default binding; select_to_paragraph uses CSI 1;6B/6A
- **Auto-save**: Currently OFF in config. Enable ONLY for auto-save testing; disable when done.
- **BUG #2117**: Review Diff `d` (discard) is broken — use `s` (stage) instead
- **BUG #2122**: move_to_paragraph keybinding — watch for fix in future versions
- **Orchestrator worktree**: session-1 at /root/.local/share/fresh/orchestrator/home_user_fresh/session-1 (may be stale)
