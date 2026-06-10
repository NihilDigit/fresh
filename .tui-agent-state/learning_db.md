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

## LSP: clangd Integration (Run #18)
- **Install:** `apt-get install clangd` (not pre-installed; installs clangd-18 via Ubuntu package)
- **Config for clangd:** `{"lsp": {"c": {"command": "clangd", "args": ["--log=verbose"], "enabled": true}}}`
- **compile_commands.json:** Place in project root to help clangd find include paths
- **Auto-start behavior:** Despite `enabled: true` in config, clangd shows as "not running" on Fresh launch. Must manually start via LSP Status popup (click/run "Show LSP Status" → "Start clangd (always)"). After "Start clangd (always)", LSP shows "ready".
- **Confirmed working features:**
  - Hover (Alt+K): Shows function signature popup
  - Go to Definition (F12): Jumps to definition, "Jumped to definition at <file>:<line>" in status
  - Completions (Ctrl+Space): Shows typed-function-matching completions with signature
  - Find References (Shift+F12): Opens popup with all reference locations
  - Rename Symbol (F2): Prompts "Rename to:", renames at definition + all call sites
  - Inlay hints: Auto-shown (parameter names in call sites, e.g., "add(a: 3, b: 4)")
- **Code Actions (Alt+.):** "No code actions available" at C error location (undeclared malloc). May be clangd limitation — C++ errors typically have more code actions. Needs further testing with C++ or different error type.
- **Language ID:** Use "c" (lowercase) in config.json for C files (Fresh detects .c files as language "C")
- **LSP Status popup format:** `○ clangd (not running)` then options: "Start clangd (always)", "Start clangd once", "Disable LSP for c", "View Log", "Dismiss"

## text-actions Plugin (Run #18)
- **Install URL:** `https://github.com/PavelLoparev/fresh-text-actions-plugin`
- **No subfolder needed** (standalone repo, not monorepo)
- **Version:** v0.1.0
- **Status after install:** "Installed and activated fresh-text-actions-plugin v0.1.0"
- **Commands added (source: "plugin"):**
  - "Encode String to Base64"
  - "Encode String to JSON String"
  - "Encode String to URI Component Encoded"
  - "Encode String to URI Encoded"
  - "Decode URI Component Encoded to String"
  - "Decode URI Encoded to String"
  - "Encode JSON Byte Array to Hex String"
- **Usage:** Select text BEFORE opening command palette. Ctrl+L (select line) + Ctrl+P works reliably. Ctrl+A + Ctrl+P may lose selection.
- **Base64 verified:** "Hello World" → "SGVsbG8gV29ybGQ=" (correct)
- **Uninstall:** `rm -rf /root/.config/fresh/plugins/packages/fresh-text-actions-plugin`

## Encoding Handling (Run #19)
- **Auto-detection:** Fresh auto-detects encoding on file open. Latin-1 encoded files are detected as "Windows-1252" (a superset — correct and reasonable).
- **Status bar encoding:** Shows current encoding (e.g., `LF  Windows-1252  Text`)
- **Reload with Encoding...** command (command palette or File menu):
  - Opens an encoding picker with 8+ encodings: UTF-8, UTF-8 BOM, UTF-16 LE, UTF-16 BE, ASCII, Latin-1, Windows-1252 (+ more)
  - Current encoding marked with "current" label in the picker
  - Bottom prompt shows selected encoding name: "Reload with encoding: Latin-1"
  - Navigation: Up/Down; use ANSI capture (`-e`) to confirm highlighted item (`[48;5;25m` background)
  - Pressing Enter reloads the file from disk with the selected encoding
  - Characters render correctly after reload with correct encoding
- **Set Encoding** command (command palette):
  - Same encoding picker as Reload
  - Changes the BUFFER encoding (how Fresh interprets/saves the bytes) without reloading from disk
  - Marks buffer as modified `[+]` with asterisk `*` in tab title
  - Status: "Encoding set to UTF-8"
  - Saving after Set Encoding writes the file in the new encoding
  - **Round-trip verified:** Latin-1 file → Set Encoding to UTF-8 → Save → file bytes are valid UTF-8 (confirmed by hex)
- **Encoding picker navigation tip:** Arrow keys work; plain-text capture doesn't show selection — use `capture-pane -e` and grep for `48;5;25m` to find highlighted item

## Themes (Run #19)
- **8 themes in v0.3.8:** dark, dracula, high-contrast, light, nord, nostalgia, solarized-dark, terminal
- **"nord" is NEW** compared to the list observed in earlier tests (v0.3.9 had 7 themes without nord)
- **Select Theme** command in command palette; no default keyboard shortcut
- **Theme picker navigation:** Arrow keys; current theme marked "(current)" in list; ANSI `-e` capture needed to see selection highlight (`48;5;25m`)
- **Theme apply confirmation:** Status bar shows "Theme changed to '[name]'"
- **ANSI color evidence (selected samples):**
  - high-contrast: menu bar `38;5;231m` fg, `48;5;236m` bg
  - dark: menu bar `38;5;252m` fg, `48;5;237m` bg
  - light: menu bar `38;5;234m` fg, `48;5;254m` bg (near-white background)
  - nord: menu bar `38;5;188m` fg, `48;5;237m` bg (light blue-grey text)

## LSP: auto_start Setting (Run #19)
- **Config schema defines:** `auto_start: boolean, default: false`
  - Description: "Whether to auto-start this LSP server when opening matching files. If false (default), the server must be started manually via command palette"
- **`enabled: true`** ≠ auto-start. It means "this server is configured and not disabled."
- **To auto-start:** Set `"auto_start": true` in LSP config. Example:
  ```json
  {"lsp": {"cpp": {"command": "clangd", "enabled": true, "auto_start": true}}}
  ```
- **Built-in LSP config docs:** "Fresh will use it automatically" = the CONFIG is pre-built (no manual JSON needed). NOT that the server auto-launches.
- **Without `auto_start: true`:** User must open LSP Status popup and click "Start [server] (always)" to start it

## LSP: Code Actions Root Cause (Run #19)
- **Bug #2212:** Fresh always sends `"context":{"diagnostics":[]}` (empty array) in all `textDocument/codeAction` requests
- **Effect:** clangd's fix-based code actions (triggered by diagnostics) are never returned — clangd returns `[]` when `context.diagnostics` is empty
- **LSP log evidence:**
  - Incoming: `publishDiagnostics` with N diagnostics including `"(fix available)"` markers
  - Outgoing: `codeAction` with `{"context":{"diagnostics":[]},"range":...}`
  - Reply: `{"result":[]}`
- **Root cause (from closed issue #1915 source comment):** `// TODO: Implement diagnostic retrieval when needed` in `app/lsp_requests.rs`
- **Not a clangd limitation:** This is a Fresh implementation gap. Clangd DOES return fix actions — but requires the diagnostic objects in the context.
- **Affected:** All diagnostic-based code actions for any LSP server (clangd, potentially others). Non-diagnostic refactoring actions may also be absent for other reasons.

## Git Blame: Multi-Commit History Navigation (Run #18)
- **'b' key behavior with multi-commit files:** Navigates to PARENT commit of the commit on the current cursor line
- **Depth tracking:** Status bar shows "Git blame at <SHA>^ | depth: N | b: go deeper | q: close"
  - depth: 0 = HEAD blame (initial state)
  - depth: 1 = one parent commit back
  - depth: 2 = two parent commits back
  - etc.
- **Confirmed on CHANGELOG.md (399 blame blocks):**
  - Line 3 at HEAD: bc11f2b (1 week ago)
  - After 'b': 059f4ab (2 weeks ago) at depth: 1
  - After 'b' again: 60d0ba2 (2 weeks ago) at depth: 2
- **When at initial commit:** "Cannot get blame at <SHA>^ (may be initial commit or file didn't exist)"
- **Important:** Fresh must be launched from the git repo directory for git blame to work. If launched from /tmp, git blame returns "No blame information available (not a git file or error)"

## text-actions Plugin: Full Decode Command Set (Run #20 — updated)

Learning_db.md previously documented only 7 commands. The plugin v0.1.0 has MORE commands:

**Encode commands:**
- "Encode String to Base64"
- "Encode String to JSON String"
- "Encode String to URI Component Encoded"
- "Encode String to URI Encoded"
- "Encode JSON Byte Array to Hex String"

**Decode commands (previously MISSING from docs):**
- "Decode Base64 to String" — `"SGVsbG8gV29ybGQ="` → `"Hello World"` ✅
- "Decode URI Component Encoded to String" — `"Hello%20World%21"` → `"Hello World!"` ✅
- "Decode URI Encoded to String" (also present)
- "Decode JSON String to String" — `"Hello\nWorld\t!"` → multiline with ANSI newline+tab ✅
- "Decode Hex String to JSON Byte Array" — `"48656c6c6f"` → `"[72,101,108,108,111]"` ✅

**Round-trip verified:** "Fresh Editor 2026" → Base64 → decode → "Fresh Editor 2026" (exact match to `echo -n "Fresh Editor 2026" | base64`)

**Usage:** Select text with Ctrl+L (select line) BEFORE opening command palette. All commands work on selected text (replace selection with result).

## Bookmarks (Run #20 — full slot test)
- **Set:** Ctrl+P → "Set Bookmark" → Enter → type digit (0-9) + Enter → "Bookmark 'N' set"
- **Jump:** Alt+N (where N = 0-9). Status: "Jumped to bookmark 'N'" on success
- **Unset slot:** "Bookmark 'N' not set" (stays on current line)
- **Slots 0-9 all confirmed working** (Run #20 tested 0, 1, 5, 9 + unset slot 2)
- Bookmarks persist per-session (session restore carries them)

## Keyboard Macros (Run #20 — complex macro confirmed)
- **Record:** Ctrl+P → "Record Macro" → slot digit (0-9) + Enter → "Recording macro 'N' (F5 or Ctrl+P → Stop Recording)"
- **During recording:** All keystrokes are captured LIVE (macro actions are actually applied to the buffer during recording)
- **Stop:** F5 → "Macro 'N' saved (X actions) - F4 → Play Last Macro"
- **Play:** F4 → "Played macro 'N' (X actions)". Plays the LAST recorded macro.
- **Named play:** Ctrl+P → "Play Macro" (if available) to play a specific slot
- **List:** Ctrl+P → "List Macros" → `*Macros*` buffer showing action-level detail per slot
  - Actions shown: SmartHome, InsertChar('X'), MoveDown, MoveUp, etc.
- **Complex macro confirmed working** (Run #20): 5-action macro = Home + "#" + " " + Down + Home. Played on 5 consecutive lines, all received "# " prefix correctly.
- **Slots:** 0-9 (tested slot 3 in Run #20; slots 0-9 in Run #10 earlier)

## SSH Remote Editing (Run #21)

### Forms Documented (docs/features/ssh.md)
- **scp-style:** `user@host:path[:line[:col]]` — WORKS (correctly triggers SSH connection path)
- **URL-style:** `ssh://[user@]host[:port]/path[:line[:col]]` — BUG (#2221): treated as local path

### scp-style Behavior
- `fresh user@host:/path` → shows "Connecting via SSH to user@host..." in terminal stdout
- Fails at SSH-spawn step if `ssh` binary not installed: "Failed to spawn SSH process (No such file or directory)"
- Error message is actionable: "Is the `ssh` command installed and in your PATH?"
- Requires Python 3 on remote host for the agent component

### URL-style Bug (#2221)
- `fresh ssh://host/path` is treated as a **local relative path** (CWD + URI string)
- Log evidence: `open_file_no_focus: path="/home/user/fresh/ssh://localhost/etc/hosts"`
- Empty local file opened; no error shown; status bar shows "Local | ssh://..."
- Workaround: use scp-style form (`user@host:/path`)

### Requirements for SSH
- `ssh` binary must be in PATH (error message tells you if not)
- Python 3 on remote host (for the remote agent)
- Status bar shows `[SSH:user@host]` when connected (per docs)

## Keybinding Editor (Run #21 — full workflow tested)

### Opening
- Command palette → "Open Keybinding Editor" (or Edit → Keybinding Editor...)
- Shows 852 bindings by default; Config path shown at top

### Adding a New Binding
1. Press `a` → "Add Keybinding" dialog
2. Press `Enter` → key capture mode → press any key → key captured (e.g., F9)
3. `Tab` → Action field: type action name; autocomplete popup with Up/Down/Tab/Enter navigation
4. `Tab` → Context field: `[normal]  ←/→ to change` (cycles: global, normal, prompt, popup, file_explorer, menu, terminal)
5. `Tab` → Save/Cancel buttons at bottom
6. `Enter` on Save → dialog closes, binding count increments, `[modified]` shown

### Saving Changes
- `Ctrl+S` in keybinding editor → saves to config.json, status bar: "Keybinding changes saved"
- Config format: `{"keybindings": [{"key": "F9", "action": "save", "when": "normal"}]}`

### Verified Working (Run #21)
- Add F9 → save → normal → Save → Ctrl+S → "Keybinding changes saved" ✅
- F9 triggers file save correctly after adding binding ✅
- config.json written correctly ✅
- Search (`/`): filters by action name/description/key; shows count (e.g., "10/852 shown") ✅

## Search in Selection (Run #21)
- **NOT IMPLEMENTED** — Find bar has no "In Selection" toggle
- Search always spans entire buffer regardless of active selection
- See IMP-014 in potential_improvements.md

## Multi-Root Workspaces (Run #21)
- **Workspace scoping:** Ctrl+P file mode shows ONLY files from the CWD (workspace root)
- **Cross-workspace:** Files outside workspace root opened via Ctrl+O appear with full absolute paths
- **Project-wide search (Alt+A):** Includes ALL open buffers — workspace files (relative paths) AND out-of-root files (full paths)
- **Session isolation:** Workspace root = CWD at launch. Different CWDs have separate sessions.

## Markdown Compose Mode (Run #20 — full verification)
- **Toggle:** Ctrl+P → "Markdown: Toggle Compose/Preview" → prompts "Compose width: None" → press Enter for viewport width
  - Second toggle = OFF
  - Status on ON: "Markdown Compose: ON (soft breaks, centered)"
  - Status: "Markdown compose width: using viewport width" (first activation message)
- **Rendering in compose mode:**
  - `**bold**` → ANSI `[1m` bold attribute; asterisks HIDDEN ✅
  - `*italic*` → ANSI `[3m` italic attribute; asterisks HIDDEN ✅
  - `` `inline code` `` → colored `[38;5;69m`; backticks STRIPPED ✅
  - `# Heading` → heading color `[38;5;51m`; `#` prefix STILL VISIBLE
  - `## Heading` → heading color; `##` STILL VISIBLE
  - ` ```python ... ``` ` code blocks → fence markers STILL VISIBLE; code syntax-highlighted ✅
  - `> blockquote` → `>` colored `[38;5;6m` (teal); quote marker STILL VISIBLE
  - `- lists`, `1. ordered lists`, `---` HR → all visible, normal rendering
- **Line numbers:** HIDDEN in compose mode (no `N │` prefix in display)
- **Editing inside code blocks works in compose mode** — new lines added correctly; display updates immediately
- **Toggle workflow quirk:** First Ctrl+P → "Toggle" activation shows a "Compose width" prompt. Second activation (same command) DIRECTLY toggles ON (no prompt). Third activation toggles OFF.

## Workspace Trust (Run #22 — NEW in v0.3.12)
- **Enforcement is ON** in v0.3.12 (was "groundwork, off by default" in 0.3.10 CHANGELOG). A folder with tooling markers (e.g. `compile_commands.json`) triggers a "⚠ SECURITY WARNING" dialog on launch.
- **Dialog options:** `(*)` radio list — "Trust folder & Allow Tooling (T)", "Keep Restricted (Default) (K)", "Block All Execution (B)". Letter selects the radio, **Enter confirms**. **Esc does NOT dismiss the dialog.**
- **Persistence:** `~/.local/share/fresh/workspaces/<percent-encoded-path>/trust.json` → `{"level": "trusted"}`. Delete the dir to reset for re-testing.
- **LSP is gated on trust:** log shows `LSP for 'cpp' not auto-started: workspace is not trusted`. After trusting, `auto_start: true` works.
- **Trust confirm = FULL EDITOR RESTART** (log: `Restart requested with new working directory`). Session restore rebuilds buffers in default mode; with `--no-restore` the CLI file + unsaved edits are silently lost → BUG #2291.
- "Keep Restricted" + Enter does NOT restart; file stays open; status: "Workspace restricted — repo-controlled execution is blocked".
- Palette command to reopen dialog: **"Workspace Trust…"** (type `Trust` — searching `Trust Lev` finds nothing; old "Set Workspace Trust Level" command is gone).

## SSH Loopback Test Recipe (Run #22)
- `apt-get update && apt-get install -y openssh-server` (404s without update first)
- `ssh-keygen -A; ssh-keygen -t ed25519 -N "" -f ~/.ssh/id_ed25519 -q; cat ~/.ssh/id_ed25519.pub >> ~/.ssh/authorized_keys; chmod 600 ~/.ssh/authorized_keys; mkdir -p /run/sshd; /usr/sbin/sshd -p 22`
- scp-style works END-TO-END (v0.3.12): `fresh --no-restore root@localhost:/tmp/file.txt` → status-bar origin segment becomes `root@localhost`, content loads, Ctrl+S writes through ("Saved").
- ssh:// URL form still broken (#2221) — status stays `Local`, empty buffer.

## Keybinding Editor: Add/Delete/Record (Run #22 — v0.3.12)
- Footer: `Enter:Edit  a:Add  d:Delete  /:Search  r:Record Key  c:Context  s:Source  ?:Help  Ctrl+S:Save  Esc:Close`
- **Add dialog focus path:** Key field → Enter (start capture) → press key → Tab → type action (autocomplete popup; Enter accepts) → **Tab lands on Context** (←/→ to change) → **Tab again lands on [Save]** → Enter. Pressing Enter while on Context CLOSES the dialog WITHOUT saving (silent — caused a lost add in Run #22).
- After the Add dialog closes there can be a transient state where r//, Esc are ignored until an arrow key is pressed (similar to #2143 item 5 "invisible dialog steals input"). Buffer does NOT receive the keys.
- **Delete:** select custom row (Source column = `custom`) → `d` → "Custom binding removed" + [modified] → Ctrl+S → "Keybinding changes saved"; the `keybindings` key is removed from config.json entirely when last custom binding deleted.
- **Record Key Search:** `r` → "Record Key: Press a key..." → pressing any key filters (e.g. Ctrl+S → 3/866 across normal/merge/theme contexts). CAVEAT: arrows/Esc are CAPTURED as search keys — you cannot navigate results in record mode. To act on a result use `/` text search → Enter (keeps filter, returns focus to list) → arrows → d/Enter.
- Quirk to re-observe: total binding count differed between two opens in one session (866 vs 548).

## Workspace Trust — Full 3-State Enforcement Matrix (Run #23 — v0.3.12, master @ f4ee3630)
Supersedes/extends the Run #22 notes above. Enforcement is process-layer and works correctly.
- **Dialog (richer than Run #22):** now has explicit `[ OK ]` and `[ Quit (Ctrl+Q) ]` buttons. Three radios with descriptions that ARE the documented enforcement contract:
  - `(*) Trust folder & Allow Tooling (T)` — "Runs everything: language servers, build scripts, tasks, env activation."
  - `(*) Keep Restricted (Default) (K)` — "Runs system tools found on your PATH (git, ripgrep, the system python). Blocks: executables & scripts located inside this project (./gradlew, .venv/bin/python, node_modules/.bin/*), env activation (.env/.envrc/mise), and language servers."
  - `(*) Block All Execution (B)` — "Nothing runs — no system tools, language servers, scripts, or tasks."
  - Letter (T/K/B) selects the radio; **Enter confirms** the selected one (don't need to Tab to [OK]). Use the test reference: assert against these descriptions.
- **Status-bar word reflects state:** `Restricted` / `Blocked` / `Trusted` (leftmost segment).
- **RESTRICTED (default):** LSP gated OFF (log: `LSP for '<lang>' not auto-started: workspace is not trusted`). **git ALLOWED** — git blame works (multi-commit blocks), git_explorer/git_gutter/merge_conflict all spawn. **ripgrep/git-grep ALLOWED** — Live Grep returns matches with both `git-grep` and `rg` providers. Does NOT restart on confirm.
- **BLOCK ALL:** every subprocess denied at spawn — log: `Process error: workspace trust is set to Blocked — no processes may run` (exit_code -1). User-facing: git blame → "No blame information available (not a git file or error)"; Live Grep → "No matches" (misleading — see IMP-017). LSP off. **Confirm triggers an editor restart** (File Explorer auto-opens) but **preserves the open file** (#2291 fix).
- **TRUSTED:** ungates the tooling layer — the `clangd-lsp` plugin loads & registers commands (clangdProjectSetup/SwitchSourceHeader) that it does NOT register in Restricted/Blocked; "not trusted" gate messages stop. BUT actual LSP server start is still governed by `auto_start` (default false → status still `LSP (off)` even when trusted). Confirm triggers a restart. trust.json → `{"level":"trusted"}` (levels seen: `trusted`; restricted/blocked presumably analogous).
- **Palette surface:** only ONE command, "Workspace Trust…" (opens the dialog). No direct trust/restrict/block palette commands; `workspace_trust_block` is not palette-exposed.
- **Test recipe:** real git C project in /tmp with `compile_commands.json` (trust trigger) + 2 commits; `rm -rf ~/.local/share/fresh/workspaces/*` to reset trust; tail `~/.local/state/fresh/logs/fresh-*.log` for the `not trusted` / `Blocked — no processes may run` evidence. NOTE git in this env enforces commit signing — use `-c commit.gpgsign=false` for throwaway repos.

## Orchestrator Dock (Run #24 — v0.3.12, master @ f4ee3630)
The dock is a persistent, non-modal LEFT-column session switcher (CHANGELOG headline feature). Fully tested; works as documented, NO bugs.
- **Toggle/focus:** `Alt+O` opens the dock and focuses it; `Alt+O` again toggles. Header shows underlined `Orchestrator` title (Alt+O mnemonic).
- **Dock layout (top→bottom):** `[ + New Alt+N ]` `[ Manage ]` · `[ view: card|compact ]` `[ All ▾ ]` (project dropdown) · `[ ] all worktrees` · `[ ] show empty` · `Filter [/ to search]` · separator · session rows.
- **Session row (card view):** bordered box, 2 lines — `· <project>   ▣ <session>` / `▸ <branch>   <gitstatus>` (e.g. "clean"). Leading glyph = working/idle indicator (`·` idle, `*` active/current session).
- **Focus model inside dock:** `Tab`/`BTab` cycle controls; focused control gets blue bg `48;2;0;100;200`. **Arrows (Up/Down) LIVE-SWITCH the active session** — retarget the editor pane to the selected session with NO restart (bidirectional; confirmed editor pane swaps main.rs ↔ remote terminal instantly).
- **View toggle:** focus `[ view: card ]` + Enter → `[ view: compact ]`. Compact = 1-line rows (`· <session>  clean`); card = bordered 2-line boxes.
- **Project dropdown `[ All ▾ ]`:** Enter opens a popup: `[ ● All projects ]` + one row per project (`[ tmp/orch_test24 ]`). Radio scope filter.
- **Filter:** press `/` (when dock focused) to activate, type to narrow session list live (e.g. "test24-1" → only that row shows). Escape clears/closes.
- **Manage button** → opens the FULL modal Orchestrator dialog (`ORCHESTRATOR :: Sessions — all projects`): left Sessions panel (New / Project dropdown Alt+P / Show all worktrees Alt+T / Show empty Alt+I / Filter / session list) + right detail panel with `[ Visit ] [ Details ] [ Stop ] [ Archive ] [ Delete ]` and a live file preview. Same dialog as prior runs' Orchestrator tests.
- **Right-click context menu:** inject SGR right-click on a session row (`\e[<2;COL;ROWM` press + `m` release). Popup anchored at cursor: header = session name, `[ Visit… ]` `[ Archive ]` `[ Delete ]`, "Esc to close". Matches CHANGELOG.
  - **Archive** (left-click `\e[<0;COL;ROWM`/`m`) → centered "Confirm Archive" dialog: "This will: • SIGKILL all session processes • close the editor session • move the worktree to .archived/  Reversible via Unarchive." `[ Cancel ] [ Confirm Archive ]`. Escape cancels.
- **New Session dialog (`Alt+N`):** `ORCHESTRATOR :: New Session`. **Type selector** `Run in: [ Local ] [ SSH ] [ Kubernetes ] [ Devcontainer ]` — focus the "Run in" row (it's ABOVE Project Path; the dialog OPENS with focus on Project Path, so `BTab` once to reach it) then ←/→ live-switches type AND reflows the whole form:
  - **Local:** Project Path (auto-detects cwd) · `[v] Create a new git worktree` (auto-disabled "(disabled — non-git)" when path isn't a git repo) · Session Name (auto `<proj>-N`) · Agent `[ terminal ][ claude ↻ ][ aider ↻ ][ custom… ]` · Agent Command · Branch (auto "HEAD (no origin configured)").
  - **SSH:** Host `[user@]host[:port]` (placeholder mentions "or paste ssh://host/path") · Remote Path · Identity file · SSH options (`-J jump-host`/ProxyCommand) · Session Name · Agent · Agent Command "remote login shell".
  - **Kubernetes:** Target · Context (kubeconfig) · Namespace (default) · Pod · Workspace path (/workspace) · info "ⓘ kubectl exec into the pod — any cluster (EKS/GKE/AKS/k3d)".
  - **Devcontainer:** Project Path · info "ⓘ runs `devcontainer up`, then attaches (docker exec)". Create with no devcontainer config → graceful error "Devcontainer: open the project and run 'Dev Containers: Reopen in Container'".
- **Create Session button focus quirk (NOT a bug — verified):** the primary `[ Create Session ]` button shows NO highlight when UNfocused (plain cyan `38;2;0;255;255`), but DOES get the blue bg `48;2;0;100;200` when Tab-focused — and Enter then creates the session. Tab order (Local): Project Path → worktree → Session Name → Agent → Agent Command → Branch → Cancel → Create Session. Cancel is the 6th Tab (blue), Create Session the 7th. Keyboard creation WORKS; my earlier "Enter does nothing" was measurement error (was on the Branch text field, not the button). Mouse left-click on the button also works.
- **Footer:** `Tab next / accept  S-Tab prev  ↑↓ suggest / history  Space toggle  Enter advance / act  Esc cancel`. NOTE: in `tmux send-keys`, use `BTab` for Shift+Tab — `S-Tab` is inserted as the LITERAL text "S-Tab" into the focused field in this tmux build (corrupts the form).
- **Creating a Local worktree session** spawns a real git worktree under `~/.local/share/fresh/orchestrator/<sanitized_proj>/<session>/` (slashes→`_`) and switches the editor to it (terminal agent → bash prompt in the worktree). Clean up with `git worktree remove --force <path>` from the parent repo + `git worktree prune`.

## Go to LSP Symbol — Document Symbol Finder (Run #25, 0.3.12)
- **Palette command:** "Go to LSP Symbol" (source `lsp_navigation`, desc "List document symbols from LSP and navigate to selected"). No default keybinding. Sibling LSP nav cmds: Rename Symbol (F2), Find References (Shift+F12), Go to Definition (F12), Show Hover Info (Alt+K).
- **Scope = DOCUMENT symbols only** (current file). clangd `textDocument/documentSymbol`, NOT workspace symbols. Filtering for a symbol that lives in another open-but-inactive file (e.g. `helper` in helpers.c) → "No matches". This matches its description; NOT a bug.
- **List format:** `[kind] name    <source line preview>`. Kinds seen on C: `[class]` (struct/typedef — clangd emits each struct TWICE: the typedef + the struct tag), `[field]`, `[var]` (file-scope global), `[fn]` (function). Prompt at bottom: `Go to symbol:`.
- **Live filtering:** type to narrow the list live (substring match on name). `make` → make_rectangle; `print` → print_rectangle; `main` → main.
- **LIVE PREVIEW (the marquee behavior):** as you filter/arrow-navigate, the editor pane scrolls to and highlights the selected symbol's location WITHOUT committing. Editor shows the symbol name highlighted yellow (`38;5;226m`); the selected list row has blue bg (`48;5;25m`). Up/Down arrows move the selection and re-preview.
- **Enter = commit jump:** cursor lands correctly on the symbol (verified via `tmux display-message -p '#{cursor_y}'` — physical terminal cursor on the symbol's editor line).
- **Escape = cancel + RESTORE:** cursor returns to the exact pre-open position (e.g. open from Ln 16 → preview around → Esc → back at Ln 16, Col 1). Good live-preview/restore model, matches VS Code.
- **BUG #2301 (filed Run #25, low-sev):** after Enter-jump, the status bar **line number is stale** — it keeps the pre-jump line; only `Col` updates. Self-corrects on the next cursor move. Reproduced 3/3 (start Ln30 → jump `main`@44 → status `Ln 30, Col 5` while cursor really on 44). **Feature-specific:** Ctrl+G and F12 both update `Ln` immediately (F12 also prints `Jumped to definition at …:N`). So it's the symbol-jump path that fails to refresh the line component.

## Workspace Trust pre-seed: encoding does NOT match (Run #25)
- Tried pre-seeding `~/.local/share/fresh/workspaces/<urlencoded path>/trust.json` with `%2Ftmp%2F...` (Python `urllib.parse.quote`). Fresh did NOT recognize it — trust dialog still appeared on launch. Fresh's workspace-dir encoding scheme differs from straight percent-encoding. **Just trust via the dialog (`T` then Enter)** — simpler and reliable. (Trust confirm triggers a restart but in default mode session-restore brings the file back; File Explorer auto-opens — IMP-015.)
- **clangd auto-start DID work this run:** config `{"lsp":{"c":{"command":"clangd","args":["--background-index","--log=verbose"],"enabled":true,"auto_start":true}}}` + trusting the workspace → on restart, status bar shows `LSP (on)` and inlay hints render (`make_rectangle(x1: 0.0, …)`, `{.x= 0.0}`) with no manual "Start clangd". Confirms IMP-013/Run#19: `auto_start:true` + Trusted = clangd launches automatically.

## Rainbow Brackets (Run #26) — 0.3.12, on by default, PASS
- **Feature:** "Rainbow bracket colorization for matching brackets across the viewport" (CHANGELOG 0.3.12, #1088). Built-in, **on by default — no config needed**; no `docs/configuration` entry (only `docs/internal/*` mention a `rainbow_brackets: bool` theme field).
- **How it colors:** by **nesting depth**, not bracket type. Verified `tmux capture-pane -p -e` foreground SGR codes:
  - Depth→color cycle (6 colors, then repeats): `0=38;5;6`(cyan) `1=38;5;2`(green) `2=38;5;3`(yellow) `3=38;5;126/127`(magenta) `4=38;5;15`(white) `5=38;5;27`(blue).
  - **Matching open/close pairs share the same color** (e.g. `((((( deep )))))` opens 6,2,3,126,15 → closes mirror 15,126,3,2,6).
  - **Works across bracket types:** `[ { ( [ ( { } ) ] ) } ]` colored 6,2,3,126,15,27 then exact mirror — `(`/`[`/`{` at the same depth get the same color.
  - **Across the whole viewport** (all visible lines colored regardless of cursor position), not just near the cursor.
- **Deep nesting:** 11-level `(((((((((((` cycles `[6,2,3,126,15,27]` and repeats (depth6→6 …), and all 11 closers mirror their openers precisely. No clamp/breakage.
- **Unbalanced handling (robust):** unmatched open `( [ { a } ] ;` → the stray `(` stays its depth color, inner matched pairs unaffected (no cascade). Stray closers `a ) b ] c } ;` → all rendered at depth-0 color (6), no crash, no negative-depth glitch.
- Minor: depth-3 magenta is sometimes `126` vs `127` between lines — both indistinguishable magenta; NOT a fileable bug.

## Terminal Auto-Naming (Run #26) — 0.3.12, on by default, PASS
- **Feature:** "Terminal tab auto-naming: tabs follow the foreground process and OSC title. Setting `editor.terminal_auto_title` (on by default)" (CHANGELOG 0.3.12).
- **Tab name format:** `<foreground process> — <OSC title>`. On opening "Open Terminal" (palette; opens in current split as a buffer tab), the tab read `bash — root@vm: /home/user/fresh` (the `root@vm: /home/user/fresh` part is bash's own `\u@\h: \w` OSC title from PROMPT_COMMAND).
- **Follows foreground process:** ran `python3` → tab became `python3 — root@vm: /home/user/fresh`; `exit()` → reverted to `bash — root@vm: /home/user/fresh`.
- **Follows OSC title:** setting a manual `printf '\033]0;HELLO-FROM-OSC\007'` initially appeared to "not stick" — that was bash's PROMPT_COMMAND overwriting the title on the very next prompt (standard shell behavior, reproducible in any terminal emulator). After `PROMPT_COMMAND=""; PS1="\$ "` then the printf, the tab correctly read `bash — HELLO-FROM-OSC`. So OSC title IS followed; the apparent failure was a bash-side overwrite, NOT a Fresh bug.
- **tmux gotcha (re-confirmed):** `M-grave` is NOT parsed by `tmux send-keys` for Alt+backtick — it inserts the literal text "M-grave" into the buffer. Use the command palette ("Open Terminal" / "Open Terminal in Utility Dock") instead of the Alt+` accelerator when driving via tmux.

## Open File From a Diff (Run #27) — 0.3.12, PASS
- **Feature (CHANGELOG 0.3.12):** "Open file from a diff: in the side-by-side and review-diff views, Enter opens the working-tree file (NEW pane) or the read-only HEAD version (OLD pane) at that line."
- **Entry path (black-box):** Palette → **Review Diff** opens the *unified* `*Review Diff*` buffer (file list + hunks, two line-number columns OLD|NEW). Header legend: `[Enter] jump  [Alt+o] open file`. Pressing **Enter on a hunk line opens the SIDE-BY-SIDE `*Diff: <file>*` view** (NOT the file directly) — that side-by-side view is where the actual open-file actions live.
- **Side-by-side `*Diff: <file>*` view:** two panes — `OLD (HEAD)` (left) / `NEW (Working)` (right), word-level aligned with blank-row padding for inserted/deleted lines. Header legend: OLD `[Enter] open this version  [n/p] hunks  [q] close`; NEW `[Enter/Alt+o] open file`. It is a read-only composite buffer (`[RO]`); `q` closes it.
- **OLD pane Enter** → opens a NEW read-only tab `*HEAD:<file>*` `[RO]`, status `Opened HEAD version (read-only) at line N`. Cursor **lands on the correct HEAD line** (verified: highlighted row `48;5;233` + `tmux display-message -p '#{cursor_y}'` on the right buffer line). Content = `git show HEAD:<file>`. Each Enter opens a *fresh* HEAD tab (e.g. `*HEAD:calc.py* 1`, `*HEAD:calc.py* 2`) rather than reusing — minor.
- **Alt+o** → opens the **working-tree file** (the real editable `<file>` tab), status `Opened <file>`, cursor at the line. This is the NEW/Working action and works regardless of which pane has focus.
- **tmux gotcha:** focusing the NEW (Working) pane via **Tab is unreliable** over tmux — Tab pushed `cursor_x` to the far-right (~219, the COMMENTS column) instead of cleanly switching panes. For the working-file path just use **Alt+o** (universal), don't fight Tab.
- **Related glitch (NOT a new issue):** status bar shows stale `Ln 1, Col 1` for ~one keypress immediately after the diff→open jump (cursor is physically on the right line). Self-corrects on any cursor movement. **Same family as #2301** — confirmed it is NOT LSP-specific; commented on #2301 (Run #27) instead of re-filing. When testing any "jump to location" command, verify the *physical* cursor (`display-message cursor_y` / highlighted row), not just the status bar, which can lag.
