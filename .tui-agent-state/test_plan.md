# Fresh Editor - Automated TUI Test Plan

## PROCESS RULES (added after Run #1 false positives)
1. **Read docs FIRST.** Before any test session, skim `docs/features/` and `docs/blog/` for the version under test.
2. **Verify menu navigation with ANSI capture** (`-e` flag) to confirm the highlighted item before asserting behavior.
3. **Check the CHANGELOG** for features that could explain "surprising" behavior before filing a bug.
4. **Test keyboard shortcuts bare** (no tmux shortcuts that might intercept). If a key acts unexpectedly, check for terminal compatibility issues before blaming Fresh.
5. **Never file a bug based on a single observation.** Always reproduce at least twice.
6. **Launch clean for fresh-state tests:** Use `fresh --no-restore` to skip hot-exit restoration when testing initial launch behavior.

---

## Run History
| Run # | Date | Status | Tests Run | Bugs Found |
|-------|------|--------|-----------|------------|
| 1     | 2026-05-26 | COMPLETED | 30+ | 4 filed → 2 real, 2 false positives |

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
- [ ] **TC-025** Save As (Ctrl+Shift+S) → to test next run
- [x] **TC-026** PASSED - "Close Buffer" command prompts `(s)ave, (d)iscard, (C)ancel?` for unsaved
- [ ] **TC-027** Close saved file → closes without dialog (to verify)
- [ ] **TC-028** Open multiple files → tabs appear (to test next run)
- [ ] **TC-029** Switch between tabs (Ctrl+Tab or mouse click) (to test next run)

---

## Priority 3: Editing Features (PARTIALLY COMPLETED - Run #1)
### Objective: Verify editing workflows

- [x] **TC-030** PASSED - Undo (Ctrl+Z) and redo (Ctrl+Y) work across multiple steps
- [x] **TC-031** PASSED - Shift+Left/Right selects text (cursor shown as reversed, selection as blue)
- [x] **TC-032** PASSED - Ctrl+A selects all text
- [x] **TC-033** PASSED - Copy (Ctrl+C) and Paste (Ctrl+V) work correctly
- [ ] **TC-034** Cut (Ctrl+X) - to test next run
- [x] **TC-035** PASSED - Ctrl+D adds cursor at next match, multi-cursor editing confirmed working
- [ ] **TC-036** Block selection mode - to test next run
- [ ] **TC-037** Comment/uncomment line (Ctrl+/) - to test next run
- [ ] **TC-038** Auto-indent - to test next run

---

## Priority 4: Search & Replace (PARTIALLY COMPLETED - Run #1)
### Objective: Verify search and replace workflows

- [x] **TC-040** PASSED - Ctrl+F opens search bar with case-sensitive/whole-word/regex toggles
- [x] **TC-041** PASSED - Search highlights all matches in ANSI colors
- [x] **TC-042** PARTIAL - Enter navigates to first match then CLOSES search bar; re-open Ctrl+F for next
          ⚠️ BUG-004: Enter doesn't advance when cursor IS AT a match
- [ ] **TC-043** Shift+Enter/F3 for previous match - F3 and Shift+Enter NOT CONFIRMED to work
- [x] **TC-044** PASSED - Escape closes search bar
- [x] **TC-045** TERMINAL COMPAT ISSUE - Ctrl+H IS intended to open find & replace (documented)
          but terminals send Ctrl+H as Backspace (0x08). Use Ctrl+R as the reliable Replace shortcut.
          Issue #2109 open: suggests adding Ctrl+H to Calibrate Keyboard wizard and documenting the conflict.
- [x] **TC-046** PASSED (via Ctrl+R) - Replace All works by default
- [x] **TC-047** PASSED - All 3 occurrences replaced simultaneously
- [ ] **TC-048** Case-sensitive toggle (Alt+C shown in search bar) - to test next run
- [ ] **TC-049** Regex toggle (Alt+R shown in search bar) - to test next run

---

## Priority 5: Views & Layout (PARTIALLY COMPLETED - Run #1)
### Objective: Verify split panes, file explorer, terminal

- [x] **TC-050** PASSED - "Split Vertical" via command palette splits horizontally (stacked)
          NOTE: Ctrl+\ not confirmed to work; use command palette → "Split Vertical"
          NOTE: "Split Vertical" creates horizontal layout (two panes stacked)
- [x] **TC-051** PASSED - Alt+] switches to next split pane
- [x] **TC-052** PASSED - "Close Split" command closes the split pane
- [x] **TC-053** PASSED - Ctrl+B toggles File Explorer (NOT Ctrl+E as assumed)
          NOTE: Ctrl+E appears to open file explorer differently
- [x] **TC-054** PASSED - Arrow keys navigate directories; Right expands, Left collapses
          Tab key switches focus to file explorer
- [ ] **TC-055** File Explorer: open file from explorer - to test next run
- [ ] **TC-056** Toggle line numbers - to test next run
- [ ] **TC-057** Toggle line wrap - to test next run
- [ ] **TC-058** Integrated terminal: open/close - to test next run

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

## Immediate Next Action (Run #2)

### FIRST: Documentation Review (mandatory before testing)
- Read `docs/features/editing.md` for complete keybinding table
- Read `docs/features/search-replace.md` for search workflow
- Check `CHANGELOG.md` for 0.3.x features

### Priority Tests to Complete:
1. **TC-BUG004-VERIFY**: Test F3 AFTER closing search bar (press Ctrl+F, Enter, then F3)
2. TC-025: Save As (Ctrl+Shift+S)
3. TC-027/028/029: Multiple tabs (Ctrl+Tab to switch)
4. TC-034: Cut with Ctrl+X
5. TC-036: Block selection (Alt+Shift+Arrow)
6. TC-037: Comment/uncomment line
7. TC-038: Auto-indent
8. TC-048/049: Case-sensitive (Alt+C) and regex (Alt+R) search toggles
9. TC-055: Open file from file explorer (Enter on file in explorer)
10. TC-056/057: Toggle line numbers/wrap (via View menu or command palette)
11. TC-058: Integrated terminal (command palette → "Terminal")
12. TC-NEW-001: Verify `File > Revert` shows `(r)evert/(c)ancel` prompt when buffer modified
13. TC-NEW-002: Test `fresh --no-restore` launches with clean state (no hot exit)
14. TC-NEW-003: Test hot exit: make edits, Ctrl+Q, relaunch — verify changes restored
15. TC-NEW-004: Test Ctrl+W selects word under cursor

### Reminders from Run #1 Lessons:
- Always verify menu item selection with ANSI capture before asserting behavior
- tmux sends Ctrl+H as Backspace — use Ctrl+R for Replace
- Ctrl+W = select word (not close buffer)
- Close Buffer = Ctrl+P → "Close Buffer"
- File Explorer toggle = Ctrl+B
- F3 navigates search AFTER search bar closes
