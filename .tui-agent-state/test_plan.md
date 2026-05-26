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

## Priority 6: Command Palette
### Objective: Verify command palette completeness

- [ ] **TC-060** Open command palette (Ctrl+P)
- [ ] **TC-061** Search for "File" commands → relevant commands appear
- [ ] **TC-062** Search for "Theme" → theme selector appears
- [ ] **TC-063** Execute a command from palette
- [ ] **TC-064** Fuzzy search works (partial matches)
- [ ] **TC-065** Switch buffer via command palette

---

## Priority 7: Settings & Configuration
### Objective: Verify settings access and persistence

- [ ] **TC-070** Access Settings UI
- [ ] **TC-071** Change theme via settings/command palette
- [ ] **TC-072** Keybinding editor is accessible
- [ ] **TC-073** Settings changes persist after restart

---

## Priority 8: Edge Cases & Stress Tests
### Objective: Find stability issues

- [ ] **TC-080** Open a very large file (100MB+)
- [ ] **TC-081** Open a binary file
- [ ] **TC-082** Open empty file
- [ ] **TC-083** Rapid key presses don't cause crashes
- [ ] **TC-084** Open 10+ files simultaneously
- [ ] **TC-085** Resize terminal window while editor is open

---

## Backlog (Future Runs)
- LSP features (go to definition, hover, diagnostics)
- Git integration (git log, git grep, diff view)
- Plugin system testing
- Macro recording and playback
- Bookmark navigation
- Markdown preview
- Multi-language syntax highlighting

---

## Immediate Next Action (Run #4)

### FIRST: Documentation Review (mandatory before testing)
- Check `docs/features/lsp.md` for LSP feature documentation
- Check `docs/features/git.md` for git integration docs
- Check `CHANGELOG.md` for any new versions

### Priority Tests for Run #4:
1. TC-060-065: Command Palette (complete coverage — partial in Run #3)
   - TC-063: Execute command from palette
   - TC-064: Fuzzy search (partial match like "tog num" → "Toggle Line Numbers")
   - TC-065: Buffer switch via palette # mode
2. TC-070-073: Settings & Configuration
   - TC-070: Access Settings UI (View > Settings...)
   - TC-071: Change theme via "Select Theme" palette command
   - TC-072: Keybinding editor access
   - TC-073: Settings persist after restart
3. TC-080-085: Edge cases
   - TC-081: Open binary file
   - TC-082: Open empty file
   - TC-083: Rapid key presses for stability
   - TC-085: Resize terminal window while editor open
4. Backlog items (from earlier runs):
   - TC-NEW-002: Hot exit with --no-restore verification
   - TC-NEW-003: Ctrl+W selects word (quick verify)
   - Pending: Shift+F3 in proper xterm terminal (not tmux) — does it work?
5. Advanced features (new):
   - Git log panel (via command palette "Git Log")
   - LSP diagnostics panel
   - Markdown preview
   - Macro recording (F5 to stop, F4 to play)
   - Bookmarks (Ctrl+Shift+0-9 to set, Alt+0-9 to jump)

### CRITICAL Reminders for Run #4:
- **Tab switching**: `C-NPage` / `C-PPage` (NOT Ctrl+Tab)
- **File Explorer**: Ctrl+B to toggle, Ctrl+E to focus, DECCKM arrows to navigate
- **Save As**: File menu only (Alt+F), NOT command palette
- **Close buffer prompt**: type letter + Enter (not just the letter)
- **Arrow keys**: DECCKM sequences `$'\033O[A-D]'`
- **Delete key**: `$'\033[3~'`
- **Line Wrap toggle**: View menu (Alt+V), NOT command palette
- **Terminal**: Alt+\` to open, Ctrl+Space to toggle mode, Ctrl+F to search scrollback
- **Find Previous**: palette command works; Shift+F3 and Ctrl+Shift+N have tmux issues
- Do NOT use `Ctrl+Tab` (sends Tab character to buffer)
