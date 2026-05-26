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

## Priority 8: Edge Cases & Stress Tests (PARTIALLY COMPLETED - Run #4)
### Objective: Find stability issues

- [ ] **TC-080** Open a very large file (100MB+) — NOT YET TESTED
- [x] **TC-081** PASSED - Binary file: opens as `[BIN]` tab, content as `<FF><FE>..`, marked `[RO]`, no crash
- [x] **TC-082** PASSED - Empty file: opens with single blank line, `~` for empty buffer, editable
- [x] **TC-083** PASSED - Rapid key presses: burst text input received all characters; 100+ rapid undo stable
- [x] **TC-084** PASSED - 12 simultaneous tabs (4 original + 8 new): tab bar scrolls, Ctrl+PgDn/Up switches correctly
- [x] **TC-085** PASSED - Resize 200x50 → 40x12 → 200x50: graceful reflow, no crash

---

## Backlog (Future Runs)
- LSP features (go to definition, hover, diagnostics) — requires LSP server installed
- Git integration: git grep, git blame, Review Diff with actual changes
- Plugin system testing: installing a plugin from URL
- Multi-language syntax highlighting (Rust, Python, JS files)
- TC-080: Very large file (100MB+)
- Theme editor (Edit Theme command)
- Whitespace indicators (toggle in Settings UI)
- Code folding (gutter fold indicators, Toggle Fold command)
- Scroll Sync (split view with same buffer)
- Vertical rulers (Add Ruler command)
- Current line/column highlight toggle
- Auto-save behavior
- File encoding handling (non-UTF8 files)

---

## Immediate Next Action (Run #5)

### FIRST: Documentation Review (mandatory before testing)
- Check `docs/features/lsp.md` for LSP setup requirements
- Check `docs/features/editing.md` for code folding and whitespace docs
- Check `docs/features/themes.md` for theme editor docs
- Check CHANGELOG.md for any new version since 0.3.9

### Priority Tests for Run #5:
1. TC-080: Very large file (100MB+)
   - Create a 100MB file with `dd` or similar
   - Open it in Fresh and verify: no hang, byte offsets shown, "Scan Line Index" available
   - Test search within large file
2. Theme Editor
   - `Ctrl+P → "Edit Theme"` — opens visual color editor
   - Navigate sections (Editor, UI, Search, Diagnostics, Syntax)
   - Try editing a color value; verify live preview
   - Verify "Save As" creates a custom theme file
3. Multi-language syntax highlighting
   - Create small Rust (.rs), Python (.py), JS (.js) files
   - Verify syntax highlighting activates (ANSI colors on keywords)
   - Verify fold indicators appear for Rust (which uses tree-sitter)
4. Code Folding
   - Open a Rust or JS file
   - `Ctrl+P → "Toggle Fold"` to fold a block
   - Verify gutter indicator changes (▸ = folded, ▾ = expanded)
   - Test Up/Down navigation skips folded regions
5. Git Blame
   - `Ctrl+P → "Git Blame"` on a tracked file
   - Verify blame panel opens with author/date info per line
6. Review Diff with actual changes
   - Make a small edit to a tracked file
   - `Ctrl+P → "Review Diff"` — should show the hunk
   - Test `n` / `p` navigation, then `d` to discard the change
7. File encoding
   - Create a Latin-1 encoded file (with `echo -e '\xe9\xe0\xf1' > /tmp/latin1.txt`)
   - Open via File > Open, toggle "Detect Encoding" (Alt+E in dialog)
   - Verify encoding shown in status bar changes accordingly
8. Whitespace indicators
   - `Ctrl+P → "Open Settings"` → Editor → Whitespace Indicators
   - Toggle "show trailing" and verify trailing spaces shown as ·
9. Live Diff
   - Edit a git-tracked file
   - `Ctrl+P → "Live Diff: vs Disk"` → verify diff opens in split

### CRITICAL Reminders for Run #5:
- **Tab switching**: `C-NPage` / `C-PPage` (NOT Ctrl+Tab)
- **Arrow keys**: DECCKM sequences `$'\033O[A-D]'`
- **Settings persist**: config at /root/.config/fresh/config.json
- **Macro**: palette → "Record Macro" → digit+Enter → actions → F5 stop → F4 play
- **Bookmarks**: palette → "Set Bookmark" → digit+Enter; Alt+N to jump
- **Binary file**: opens [BIN], [RO], hex-escaped content
- **Smart Home**: first press → non-whitespace col; second → col 1
- **Position History**: Alt+Left navigates back; Alt+Right forward
- **Surround**: select text, type bracket/quote → wraps selection
- **Live Grep**: palette → "Live Grep" → type to search; Buffers/Files/Terminals scoped
- **Review Diff**: n/p hunks, s/u/d stage/unstage/discard; Enter to jump to file
- **Git Log**: q to close, arrows to navigate, Enter to preview commit diff
