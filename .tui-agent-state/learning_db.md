# Fresh TUI Editor — Knowledge Base

## Application Overview
- **Name:** Fresh — a modern terminal text editor
- **Binary:** `/opt/node22/bin/fresh` (installed via npm @fresh-editor/fresh-editor)
- **Version:** 0.3.9
- **Launch:** `fresh [file]` — opens with optional file argument
- **Session Persistence:** Fresh restores all previously open buffers on next launch (built-in session memory)

## tmux Session Setup
- Session: `fresh-tui-test`
- Size: 220 columns × 50 rows
- Create: `tmux new-session -d -s fresh-tui-test -x 220 -y 50`
- Kill: `tmux kill-session -t fresh-tui-test`
- Send keys: `tmux send-keys -t fresh-tui-test 'text' Enter`
- Read output: `tmux capture-pane -t fresh-tui-test -p`
- Read with colors: `tmux capture-pane -t fresh-tui-test -p -e`

## Critical tmux send-keys Notes
- Spaces in `send-keys` arguments are interpreted as KEY SEPARATORS
  - BAD: `tmux send-keys -t s "Split H"` → sends "Split" (invalid key) and "H" (capital H)
  - GOOD: `tmux send-keys -t s 'S' 'p' 'l' 'i' 't' ' ' 'H'` → sends each char individually
- Special key names: Enter, Escape, BSpace (backspace), Space, Up/Down/Left/Right, Home/End
- Ctrl keys: `C-a`, `C-p`, `C-z`, etc.
- Alt keys: `M-w`, `M-a`, etc.  
- Shift+arrow: `S-Down`, `S-Up`, `S-Left`, `S-Right`
- F-keys: `F1`, `F3`, `F10` — but F10 reliability is timing-dependent (can insert `[21~]` escape sequence)

## Menu Bar Navigation
- **F10** — opens File menu (first menu)
- **Right/Left** — navigate between menus while one is open
- **Down/Up** — navigate within a menu (WARNING: Down from File menu switches to Edit; can confuse)
- **Escape** — close menu
- **Enter** — execute menu item
- Menu order: File | Edit | View | Selection | Go | LSP | Help
- Menu navigation with 3+ separate Right presses correctly navigates: File→Edit→View→Selection

## Key Bindings (Confirmed)
### File Operations
- **Ctrl+N** — New File (new empty buffer tab)
- **Ctrl+O** — Open File (file picker with browser + sizes/timestamps)
- **Ctrl+S** — Save (prompts Save As for new buffers)
- **Ctrl+Q** — Quit (no prompt for unsaved new buffers)
- **Alt+W** — Close Tab (but ALSO toggles whole-word search depending on context!)
- **Close Buffer** (command palette) — prompts save/discard for modified buffers

### Save/Discard Dialog
- Prompt: `'[filename]' modified. (s)ave, (d)iscard, (C)ancel? `
- Input: Type letter + Enter (NOT single keypress)
- 's' + Enter → save
- 'd' + Enter → discard
- 'C' + Enter or Escape → cancel

### Editing
- **Ctrl+Z** — Undo (single character/action granularity)
- **Ctrl+Y** — Redo
- **Ctrl+X** — Cut
- **Ctrl+C** — Copy (shows "Copied" status)
- **Ctrl+V** — Paste
- **Ctrl+A** — Select All
- **Ctrl+W** — Select Word (at cursor)
- **Ctrl+L** — Select Line
- **Ctrl+D** — Add cursor at next match (multi-cursor)
- **Ctrl+/** — Toggle Comment (works on code files; no effect on plain text)
- **Ctrl+Alt+↑/↓** — Add cursor above/below
- **Alt+Shift+I** — Add cursors to line ends

### Navigation
- **Ctrl+G** — Go to Line (prompt at bottom: type number + Enter)
- **Ctrl+Right/Left** — Word movement
- **Shift+Down/Up/Left/Right** — Selection extend
- **Ctrl+PgDn / Ctrl+NPage** — Next Buffer/Tab
- **Ctrl+PgUp / Ctrl+PPage** — Previous Buffer/Tab
- **Home/End** — Beginning/End of line
- **F3** — Find Next (uses last search)
- **Shift+F3** — Find Previous

### Search & Replace
- **Ctrl+F** — Find (bar at bottom with options)
  - **Alt+C** — Toggle Case Sensitive
  - **Alt+W** — Toggle Whole Word  
  - **Alt+R** — Toggle Regex
  - **Enter** — Jump to current match and CLOSE search bar
  - **F3** — Next match WITHOUT closing search bar
- **Ctrl+Alt+R** — Query Replace
  - First field: search term (Enter to advance)
  - Second field: replacement (Enter to start)
  - Per-match: y=yes, n=skip, a=all remaining, c=cancel
- **Alt+A** — Search and Replace in Project (cross-file)

### Views
- **Ctrl+B** — Toggle File Explorer (sidebar with directory tree)
- **F10** — Access menu bar
- **Ctrl+`** / **Alt+`** — Open Terminal in Utility Dock

### Command Palette
- **Ctrl+P** — Open Command Palette
  - Default mode: `>command` (command search)
  - Delete `>` → `file` mode (fuzzy file search)
  - `:` → line mode
  - `#` → buffer mode
  - Type characters with spaces: use single-char `send-keys` format

### Terminal
- **Alt+`** — Open Terminal in Utility Dock
- **Ctrl+Space** — Toggle terminal input mode (enter/exit interactive terminal)
- When terminal mode active: editor keyboard shortcuts are captured by terminal
- Close terminal split via "Close Split" command

## UI Layout
- Menu bar: top row (File/Edit/View/Selection/Go/LSP/[Explorer]/Help)
- Tab bar: second row (shows all open buffers, * = modified)
- Editor area: main area with line numbers + content
- Status bar: bottom-1 (filename, cursor pos, status message, encoding, filetype, hint)
- Dialogs/prompts: appear at very bottom (BELOW status bar) — easy to miss!

## Status Bar Format
`Local | [filename] [+] | Ln N, Col N | [message] | encoding | filetype | hint`
- `[+]` = unsaved changes (same as `*` in tab)
- Encoding: UTF-8, ASCII, LF
- Filetype: Text, TOML, Rust, etc.
- Hint: "Palette: Ctrl+P" = normal mode; "Terminal mode..." = in terminal

## Important Behaviors

### Session Persistence
Fresh remembers all open buffers across sessions. Each launch restores the previous state. This means test buffers accumulate. Explicitly close/discard them before each run.

### Multi-cursor
1. Position cursor at first word
2. Ctrl+W to select word
3. Ctrl+D to add cursor at NEXT occurrence
4. Repeat Ctrl+D for more cursors
5. Type to edit all cursors simultaneously

### Save/Discard Dialog
The save prompt appears at the VERY BOTTOM of the screen, below the status bar. Typing after the dialog opens inputs into the dialog's text field (not the editor). Always check for this dialog before typing.

### Alt+W Quirk
Alt+W behaves differently depending on context:
- In search bar: toggles Whole Word search option
- On a tab (when not in search): closes the tab ("Close Tab")
- Outside search, while in editing: may toggle whole-word setting (BUG CANDIDATE)
Safer alternative: Use "Close Buffer" or "Close Tab" commands via Ctrl+P.

### F10 Reliability
F10 can sometimes insert escape sequence `[21~]` into the active buffer instead of opening the menu. This is timing-dependent with tmux. If this happens, undo with Ctrl+Z. Prefer using command palette (Ctrl+P) for menu actions.

### Large File Mode
Fresh enters a "virtual byte-offset mode" for large files (observed at 49MB / 500K lines):
- Status bar shows: `Byte N` instead of `Ln N, Col N`
- Gutter shows BYTE OFFSETS (0, 98, 196...) instead of line numbers
- Navigation (Ctrl+End to bottom) is immediate
- Search still works (found 2 matches in <2s for 49MB file)

### Binary File Mode
Fresh opens binary files gracefully:
- Tab title shows `[BIN]` tag (e.g., `ls [BIN] ×`)
- Non-printable bytes rendered as `<XX>` hex notation
- No crash, no freeze
- Line wrap applies to binary content

### Go to Matching Bracket
- Command: "Go to Matching Bracket" in command palette
- Shortcut: `Ctrl+]` (but `Ctrl+]` = ASCII 0x1D may not transmit correctly via tmux send-keys)
- Use command palette invocation for testing: `Ctrl+P` → type "matching" → Enter
- Works on: `(` → `)`, `{` → `}`, and presumably `[` → `]`

### Position History
- `Alt+Left` = Go Back in position history (previous cursor positions)
- `Alt+Right` = Go Forward in position history
- History is built by navigating (Ctrl+G jumps, bracket matches, etc.)
- Works across lines (tested: lines 5 → line 1 via Alt+Left)

### Alt+W Confirmed Behavior (NOT a bug)
- Normal editing mode (not in search bar): `Alt+W` = **Close Tab** ("Tab closed" status)
- Search bar active: `Alt+W` = **Toggle Whole Word** (search option)
- This is correct context-sensitive behavior, not a bug

### Line Wrap Toggle
- View menu → `☑ Line Wrap` item toggles wrapping
- No keyboard shortcut (use View menu or command palette if it exists)
- Default state: ON (confirmed by visual wrap of long lines at startup)
- No "Toggle Line Wrap" found in command palette — must use View menu

## Alt+A — Project-wide Search & Replace Panel

### Panel Layout
- Opens at the bottom of the screen as a split panel
- Two fields: `Search:` and `Replace:` (Tab to navigate between them)
- Scope checkbox: `[v] All Files` (when checked, searches all project files)
- Options: `Case (Alt+C)`, `Regex (Alt+R)`, `Whole Word (Alt+W)`
- Replace button changes context-sensitively to "Replace All in [filename] (Alt+Ret)" when scoped
- Match list: grouped by file with counts (e.g. `▼ [v] TX tmp_test_files/test_file1.txt (3/3)`)
- Hint line: `Tab next  Space include/exclude  Enter open  Alt+Ret replace selected  Esc close`

### Key Behaviors
- **Scope via Space:** Pressing Space on a file group header toggles its inclusion
  - When pressed on a non-current-file group, it deselects "All Files" and scopes to current file only
  - Shows `Only in: [filename]` indicator when scoped
- **Replace confirmation:** "Replace N match(es) in M file(s)? Undo only covers files still open. Press Enter to confirm, Esc to cancel."
- **Outside-workspace files:** Fixed in commit b7e7e64 — files outside the workspace root (e.g., /tmp) ARE now included in search results
- **Status**: Shows match count in status bar and "Replaced N occurrences in M files" after replace

## Calibrate Keyboard Wizard

### Overview
- Command: "Calibrate Keyboard" in command palette
- Steps: 24 steps across 5 groups
- Groups: Basic Editing, Line Navigation, Word Navigation, Document Navigation, Emacs-Style
- Controls: `[s]` Skip key, `[b]` Back, `[g]` Skip group, `[a]` Abort

### Groups and Keys Tested
1. **Basic Editing**: BACKSPACE, DELETE, TAB, SHIFT+TAB
2. **Line Navigation**: HOME, END, SHIFT+HOME, SHIFT+END (and others)
3. **Word Navigation**: ALT+LEFT/RIGHT, ALT+SHIFT+LEFT/RIGHT, CTRL+LEFT/RIGHT, CTRL+SHIFT+LEFT/RIGHT
4. **Document Navigation**: PAGE UP, PAGE DOWN (and others)
5. **Emacs-Style**: CTRL+A, CTRL+E, CTRL+K, CTRL+Y

### Key Finding
- **Does NOT test CTRL+H** — the terminal compatibility issue (#2109, Ctrl+H = Backspace) is NOT addressed by the wizard
- The wizard would need a group for "Troublesome Keys" to detect this (see IMP-003)
- Final screen: "Input Calibration - Verify" with `[y] Save [a] Abort`
- Summary shows: "All Keys Working!" if all captured keys matched expected

### Rapid Input Behavior
- Fresh handles rapid keystrokes without dropped input (tested: 50 chars at full tmux speed)
- Rapid Ctrl+Z (20 × with no delay) correctly undoes char-by-char with no corruption
- All keys in fast-burst sequences are received and processed correctly

### Resize Reflow
- Fresh adapts layout correctly when tmux window is resized (220×50 → 80×24 → 180×40)
- Status bar truncates gracefully: `Ln 1, Col 1` becomes `Ln 1, ...` at narrow widths
- Resize mid-typing produces no corruption or dropped characters
- Editor remains fully responsive after resize

## Flash: Jump Plugin
- Command: "Flash: Jump" in command palette
- Behavior: Overlays single-character hint labels on every visible word/position in the editor
- Pressing a hint character jumps the cursor to that position
- Status bar shows "Flash[]" while active
- Works correctly — cursor moves to the position labeled by the typed hint char

## Package Manager
- Commands: "Package: Packages" and "Package: Install from URL" (source: `pkg`)
- **Package: Packages** opens a package browser buffer with:
  - Package list (left panel): 13 available packages with type tags [P]=Plugin, [T]=Theme, [L]=Language
  - Detail panel (right): version, author, license, description, tags, repo URL, [Install] button
  - Filter tabs at top: All | Installed | Plugins | Themes | Languages | Bundles | Sync
  - `/` opens search input; Enter confirms; search filters list by name/description
  - Status bar: "Registry synced (N/N sources)"
  - Navigation: ↑↓ navigate, Tab next, / search, Enter select, Esc close
- **Package: Install from URL** shows "Git URL or local path:" prompt at bottom

## Live Diff Plugin
- Commands all prefixed "Live Diff:" in command palette (source: `live_diff`)
- Available modes: vs HEAD, vs Disk (unsaved changes), vs Branch..., vs Default Branch, Refresh, Toggle (Buffer), Toggle (Global), Set Default Mode
- **vs HEAD**: Green `│` gutter markers (ANSI 38;5;78) on added lines; green background (48;5;22) on added content. Status: "Live Diff: comparing against HEAD"
- **vs Disk**: `+` marker (ANSI 38;5;71) on lines in buffer not yet saved. Status: "Live Diff: comparing against file on disk"
- **vs Branch...**: Prompts "Branch or ref" input with "main" pre-filled. Status: "Live Diff: comparing against [branch]"
- All modes confirmed working and switching via status bar confirmation

## Block Selection (Alt+Shift+Arrow)
- **Keys (confirmed working in Run #15):**
  - Block select down: `M-S-Down` (Alt+Shift+↓)
  - Block select up: `M-S-Up` (Alt+Shift+↑)
  - Block select left: `M-S-Left` (Alt+Shift+←)
  - Block select right: `M-S-Right` (Alt+Shift+→)
- **Behavior:** Rectangular (column) selection — same columns selected on every row
- **Confirmed by:** Selecting "Line " (5 chars, cols 1-5) on rows 1-4, then typing '>' replaced it on all rows simultaneously
- **Note:** Run #12 reported M-S-Down did NOT work. Run #15 confirms it DOES work on this build. The key sequences are correct.

## Dev Container Features
- Commands all prefixed "Dev Container:" in command palette (source: `devcontainer`)
- **Create Config**: Creates `.devcontainer/devcontainer.json` with minimal Ubuntu base template `{"name": "...", "image": "mcr.microsoft.com/devcontainers/base:ubuntu"}`
- **Show Info**: Opens `*Dev Container*` panel showing container name, image, action buttons (Run Lifecycle, Open Config, Rebuild, Close). Controls: Tab cycle, Enter activate, Alt+r run, Alt+o open, Alt+b rebuild, **q close** (q works here, unlike *Keyboard Shortcuts*)
- **Show Features**: Returns "No features configured" if devcontainer.json has no features section
- **Show Forwarded Ports**: Opens panel with "No configured or runtime ports to show."; controls: r refresh, q/Esc close
- **Without devcontainer.json**: "No devcontainer.json found" status message
- These Dev Container panels properly close with 'q' — showing the 'q' issue is specific to certain [RO] buffer types

## Live Grep Provider Cycling
- Alt+P cycles providers: git-grep → rg → grep → (back to git-grep)
- Only 3 providers available in this environment (command palette says rg, ag, git-grep, ack, fff, grep — but only 3 installed)
- Search results appear for all providers with match count ("1000+" for common terms)
- Provider shown in toolbar as "[ git-grep ]", "[ rg ]", "[ grep ]"

## Run History
- Run 1 (2026-05-26): First run, built binary, tested ~35 test cases across all sprints
  - Sprints 1-9 largely completed
  - One bug candidate identified (Alt+W inconsistency)
  - Session cleanup: fresh exited cleanly via Ctrl+Q
- Run 13 (2026-05-27): Bug verification + new feature tests
  - TB01 confirmed (BUG-001), TB02 confirmed (BUG-002), TB03 resolved (not a bug)
  - T28 (bracket match), T30 (pos history), T37 (line wrap), T45 (large file), T46 (binary) all PASSED
  - Filed issue #2135; added comment to #2125
  - Binary built with debug profile (target/debug/fresh)
- Run 14 (2026-05-27): T47/T48/Alt+A/Calibrate Keyboard + bug recheck
  - T47 PASS (rapid input), T48 PASS (resize reflow)
  - Alt+A PASS (project-wide search/replace, scoping, confirmation dialog)
  - Calibrate Keyboard TESTED (24 steps/5 groups, no Ctrl+H)
  - #2125 PARTIAL FIX: Diagnostics panel q/a confirmed fixed; *Keyboard Shortcuts* 'q' still broken
  - #2112 CONFIRMED FIXED: /tmp files now found in Search/Replace panel
  - Binary built from release profile (target/release/fresh) on branch claude/awesome-clarke-c7jCY
- Run 15 (2026-05-27): Plugin features + bug rechecks
  - Flash:Jump PASS, Package Manager PASS (Packages + Install from URL), Live Diff PASS (all modes), Live Grep Cycle Provider PASS, Block Selection PASS (M-S-Down/M-S-Right confirmed working), Dev Container PASS (4 commands tested)
  - *Keyboard Shortcuts* 'q' STILL BROKEN; #2117 STILL BROKEN
  - Binary built from release profile (target/release/fresh) on branch claude/awesome-clarke-cN0ma
- Run 16 (2026-05-31): Bug rechecks + new features in v0.3.10
  - #2117 CONFIRMED FIXED (v0.3.10): Review Diff discard hunk works correctly
  - *Keyboard Shortcuts* 'q' STILL BROKEN → new #2165 filed (parent #2125 closed)
  - Diagnostics 'q' STILL WORKING (confirmed)
  - Git Blame PASS (blame buffer, commit info, 'b' go-back, 'q' close)
  - Live Diff: Set Default Mode PASS (all modes accepted)
  - Orchestrator features PASS (Alt+P, Alt+T, Details, filter search)
  - Package install (via "Package: Install from URL") PASS; Color Highlighter plugin PASS (swatches)
  - Uninstall via file deletion works (rm -rf plugin dir); package removed immediately
  - Dev Container: Attach "CLI Not Found" dialog PASS (shows npm install command)
  - Binary built from release profile (target/release/fresh) on branch claude/awesome-clarke-jWgGn

## Git Blame Plugin
- Command: "Git Blame" in command palette (source: `git_blame`)
- Opens `*blame:<filename>*` [RO] buffer with per-line commit annotations
- Format: `── <hash> (<author>, <time>) "<commit message>" ──` as block header
- Status bar: "Git blame: N blocks | b: blame at parent | q: close"
- **'b'** — go back to parent commit of current line's commit
  - If file was added in that commit (initial), shows: "Cannot get blame at SHA^ (may be initial commit or file didn't exist)"
- **'q'** — closes the blame panel correctly (unlike *Keyboard Shortcuts* buffer)
- Tested on: single-commit files and README.md (monorepo context)

## Package Manager: Install via URL
- **Install command:** "Package: Install from URL" (command palette)
- **Prompt:** "Git URL or local path:" at the bottom
- **URL format for monorepo plugins:** `https://github.com/owner/repo#subfolder`
  - Example: `https://github.com/sinelaw/fresh-plugins#color-highlighter`
- **After install:** Status shows "Installed and activated <name> v<version>"
- **Package location:** `/root/.config/fresh/plugins/packages/<name>/`
- **Package files:** `<name>.ts`, `<name>.i18n.json`, `package.json`
- **Uninstall:** No "Package: Uninstall" command exists. Must delete package directory manually: `rm -rf /root/.config/fresh/plugins/packages/<name>/`
  - Fresh detects removal in real-time (no restart needed)
  - Plugin deactivates immediately when directory deleted

## Package Manager: UI Navigation Notes
- Package browser Tab cycle (from list item): 
  - Tab 1: "Enter Select" (list navigation)
  - Tab 2: "Enter Activate" (shows `[ Install ]`/`[ Uninstall ]` brackets in right panel)  
  - Tab 3-4: "Enter Search" (search field)
  - Tab 5-9+: "Enter Filter" (filter tabs: All, Installed, Plugins, Themes, Languages, Bundles)
  - Tab 10-11: "Enter Sync" (Sync tab)
- ⚠️ Tab 2 "Enter Activate" MAY activate the search field rather than the Install button (inconsistent behavior observed — further testing needed)
- For reliable install: use "Package: Install from URL" command in palette
- For uninstall: manually delete the plugin directory

## Color Highlighter Plugin
- Install URL: `https://github.com/sinelaw/fresh-plugins#color-highlighter`
- Commands: "Color Highlighter: Enable", "Color Highlighter: Disable", "Color Highlighter: Toggle"
- **Enable behavior:** Shows filled block `█` character before each color code
  - `#ff0000` → `[38;5;196m█` (red, ANSI 256-color approximation)
  - `rgb(0, 128, 255)` → `[38;5;33m█` (blue)
  - `hsl(120, 100%, 50%)` → `[38;5;46m█` (green)
  - `#abc` → `[38;5;145m█` (light blue/lavender = #aabbcc)
- Supports CSS files and any file with color expressions
- Plugin activates immediately on install (no restart needed)
- Plugin deactivates in real-time on file deletion (no restart needed)

## Dev Container: Attach Error Handling
- When devcontainer CLI is NOT installed:
  - Dialog: "Dev Container CLI Not Found"
  - Message: "The devcontainer CLI is needed for rebuild. Copy the install command below, or dismiss."
  - Shows: "Copy: npm i -g @devcontainers/cli"
  - Dismiss with ESC
- When devcontainer.json is NOT found: status shows "No devcontainer.json found" (no dialog)
- Docker daemon not running does NOT affect this error path (CLI check happens before docker check)

## File Explorer (Run #17)
- **Toggle sidebar:** `Ctrl+B` — shows/hides file explorer sidebar
- **Focus explorer:** `Ctrl+E` — moves keyboard focus from editor to file explorer
- **Navigation:** Up/Down = move cursor; Right = expand directory; Left = collapse directory; Enter = open file (permanent)
- **Auto-preview:** Cursor movement auto-previews files in editor without permanently opening them
- **New file:** `Ctrl+N` when explorer is focused (NOT when editor is focused) — creates file in current dir
- **Delete file:** `Delete` key when file is highlighted → confirmation `y`/`n` → "Moved to trash" on confirm
- **IMPORTANT:** `Ctrl+N` when editor is focused = new buffer, NOT explorer new file. Must press `Ctrl+E` first to focus explorer.
- The `>` prefix in file names (or tree node markers) distinguishes folders from files.

## Settings Panel Navigation (Run #17)
- **Open settings:** Command palette `Ctrl+P` → "Open Settings"
- **Panel structure:** 3-panel overlay: [Categories list | Settings items | Right content panel]
- **Tab cycle:** Categories → Settings → Footer → Categories (one Tab per panel)
- **Categories panel:** When focused (blue `[48;5;25m` bg + `>` prefix), Up/Down navigates categories; right panel updates to show that category's settings
- **Settings panel:** When focused, Up/Down navigates between individual setting controls. Each control shows a description box below it.
- **Footer panel:** Shows `[ Edit ]  [ User ]  [ Reset ]  [ Save ]  [ Cancel ]`. Tab cycles: Save → Cancel → Edit → wrap to Categories (Layer/Reset buttons skipped by Tab per known usability bug).
- **Enter in Settings panel:** For Toggle = toggle value; for Dropdown = open dropdown; for Number = enter edit mode; for TextList = enters editing mode (Up/Down navigate items, Delete removes, Enter confirms/adds)
- **TextList interaction model:**
  1. Navigate to TextList header (e.g., "Sources:") with Up/Down in Settings panel
  2. Press Up to navigate INTO existing items (cursor moves to items list)
  3. Item focused state: item text shown with blue bracket color `[38;5;25m`, hint text "Del:remove  Enter:edit" appears
  4. Press Delete to remove focused item; Enter opens inline editor for item text
  5. Tab exits TextList (jumps to next panel in Tab cycle)
- **TextList [x] buttons:** Visual-only for mouse users; NOT keyboard-accessible via Tab. Keyboard deletion = Delete key.
- **Escape behavior:** Closes settings and DISCARDS unsaved changes (no confirmation dialog when changes are pending — contrary to expectation)
- **Selection highlight:** Blue `[48;5;25m` background = keyboard focus; `[48;5;17m]` darker blue = modified/active indicator
- **Modification indicator:** `●` appears next to setting label when value differs from default

## LSP: Pyright Status (Run #17)
- **pyright-langserver** (from `pip install pyright`): Initialize handshake SUCCEEDS ("LSP (python) ready")
- BUT all request-based features TIMEOUT after 30s: hover, definition, completion, signatureHelp, diagnostics
- Log evidence: warnings log shows 10 timeouts; `LSP initialize result: position_encoding=None` (suspicious)
- pyright processes (Python wrapper + Node child) both running but idle (sleeping state = waiting for input)
- Bug filed: #2197
- **Suspected cause:** Position encoding mismatch — `position_encoding=None` may mean Fresh defaults to UTF-16 but pyright expects UTF-8, causing it to silently discard all subsequent requests
- **Alternative to try next run:** clangd on a tiny C project (avoid rust-analyzer = too heavy)
