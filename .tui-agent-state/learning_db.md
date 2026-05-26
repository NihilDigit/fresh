# Fresh Editor — TUI Agent Knowledge Base

## ⚠️ FALSE POSITIVE PATTERNS — READ BEFORE TESTING

The following things look like bugs but are NOT. Run #1 wasted time on all of these.
Check this list before filing any issue.

| Observation | What it actually is |
|-------------|---------------------|
| File opens with `[+]` / asterisk and unread content on fresh launch | **Hot exit.** Fresh restores all unsaved buffers on startup. `hot_exit` is on by default. Use `fresh --no-restore` if you need a clean slate. |
| `Ctrl+W` selects a word instead of closing the tab | **Intentional.** Fresh's `Ctrl+W` = Select word. VS Code's `Ctrl+W` = Close tab. Use command palette → "Close Buffer" to close. |
| `Ctrl+H` deletes a word instead of opening Find & Replace | **Terminal compatibility.** `Ctrl+H` = Find & Replace is intended and documented, but terminals transmit it as ASCII `0x08` = Backspace. Use `Ctrl+R` for reliable Replace. |
| Menu arrow-key navigation appears unresponsive | **Highlight is subtle.** Selection uses `[48;5;25m]` (dark blue). Use `tmux capture-pane -e` (ANSI mode) to verify which item is active — plain `-p` hides it. |
| `File > Revert` triggers "Cannot reload: buffer has unsaved modifications" | **You triggered the wrong menu item.** That error comes from "Reload with Encoding...". `File > Revert` correctly shows a `(r)evert/(c)ancel` confirmation prompt. Verify selection with ANSI capture. |
| `[No Name]*` buffer appears on launch | **Hot exit** restored an unnamed scratch buffer from a previous session. |
| "Split Vertical" command produces stacked (horizontal) panes | **Naming convention.** Fresh's "vertical" refers to the split line direction. Stacked panes = horizontal divider. Not a bug. |
| Arrow keys don't move cursor / navigation seems broken | **DECCKM terminal mode.** Fresh uses application cursor key mode. Must send `$'\033O[A-D]'` NOT tmux `Up`/`Down` key names. See tmux automation section below. |
| `DC` tmux key name doesn't delete forward | **Wrong escape.** Use `$'\033[3~'` for the Delete key in Fresh. |
| Search/Replace panel shows "No matches found" for open file | **Workspace scoping bug (#2112).** The search backend only searches within the git project root. Files from /tmp or outside the workspace fail silently. |
| Shift+F3 does not navigate to previous match | **Terminal compatibility.** `S-F3` in tmux does not send the correct shift+F3 escape. "Find Previous" works via command palette (binding: `Ctrl+Shift+N`, but also problematic in tmux). |
| `Ctrl+Tab` doesn't switch tabs — types Tab character into buffer | **Wrong key.** Tab switching is `Ctrl+PgDn` / `Ctrl+PgUp` (tmux: `C-NPage` / `C-PPage`). Never use `Ctrl+Tab` in tmux. |
| File Explorer arrow keys do nothing after Ctrl+B | **Focus not on explorer.** `Ctrl+B` opens the sidebar but does NOT focus it. Use `Ctrl+E` to give focus to the explorer. Status bar shows "File explorer focused". |
| "Toggle Line Wrap" not found in command palette | **It's in the menu, not the palette.** Toggle Line Wrap is in View menu (`Alt+V` → navigate down 2 from File Explorer to `☑ Line Wrap`). |
| Close buffer prompt requires Enter after the letter key | **Confirmed in Run #3.** When the close-buffer prompt appears as a bottom-line input, you must type the letter (e.g. `d`) AND press Enter to confirm. Just pressing `d` appends to the prompt text. |
| Settings checkboxes not navigable via Tab | **Run #6 concern RESOLVED in Run #7.** Checkboxes ARE reachable: use ↑↓ arrows (DECCKM) in the right panel to navigate to a checkbox, then Enter to toggle. Tab only reaches number/text inputs. |
| "move_to_paragraph" not found in command palette | **By design** (PR #2084): movement-only actions don't get palette entries, same as `move_left`. BUT missing default keybinding is a bug (#2122). These actions exist but require manual keybinding configuration. |
| "Next Window" command shows "Cancelled" | **Single-window behavior** — correct when only 1 window open. Requires Orchestrator-created multi-window sessions to have multiple windows to cycle. |

---

## Application Overview
- **Name:** Fresh — a modern terminal text editor
- **Version:** 0.3.9 (as of Run #7)
- **Binary:** `./target/release/fresh`
- **Build:** `cargo build --release --bin fresh`

---

## MANDATORY PRE-TESTING CHECKLIST
Before filing any bug, the agent MUST:
1. Check `docs/features/` for feature documentation
2. Check `docs/blog/` for release notes (features that look like bugs are often documented)
3. Check `docs/configuration/keyboard.md` for the actual keybinding table
4. Verify menu navigation with `tmux capture-pane -p -e` (ANSI) to confirm the highlighted item
5. Check `CHANGELOG.md` for the relevant version's feature list

---

## ISSUE FILING STANDARDS

### When to open an issue

Open an issue only when you can answer ALL of the following:
- **What is the exact expected behavior?** (cite a reference: VS Code, Sublime, browser, or Fresh's own docs)
- **What is the exact actual behavior?** (observed directly, reproducible)
- **Would a reasonable user be confused or blocked by this?**

Do NOT open an issue when:
- You haven't finished testing ("needs re-test", "not yet verified")
- You're unsure if the behavior is intentional — check docs first, then decide
- You can only describe the symptom but not the expected vs actual contrast

If you observe something suspicious but haven't confirmed it yet, note it in `test_plan.md` as a pending test case. File the issue only after you have clear evidence.

### Two valid issue types

**1. Bug** — behavior is broken or incorrect per Fresh's own documentation
- Example: A documented shortcut does nothing, a save dialog corrupts the file, a crash

**2. Usability issue** — behavior works but contradicts what users coming from VS Code/Sublime/browsers will expect
- Example: F3 silently ignored while search bar is open (#2111), Ctrl+H transmits as Backspace (#2109)
- These are still valid issues. "It's documented" does not mean "users won't be confused."
- Label these as `bug` since they represent a user experience failure

**Not a valid issue:**
- Behavior that matches Fresh's documentation AND matches common editor conventions
- Something you observed once and couldn't reproduce
- Something listed in the FALSE POSITIVE PATTERNS table above

### Required issue structure

Every issue MUST contain all four of these sections. If you can't fill them all in, finish testing first.

```
## Steps to reproduce
1. [exact, numbered steps starting from a clean state]
2. ...

## Expected behavior
[What a user coming from VS Code/Sublime/browser would expect — be specific.
If citing a reference editor, name it: "In VS Code, F3 advances to the next
match without closing the search bar."]

## Actual behavior
[What Fresh actually does — be specific. "Nothing happens" is not enough;
say "The cursor does not move. The search bar remains open but no navigation occurs."]

## Workaround
[If one exists. If none, say "None."]
```

### Issue title rules

- State the problem, not the investigation: **"F3 does not navigate while search bar is open"** not ~~"Search F3 navigation not verified"~~
- Use present tense: **"F3 does not..."** not ~~"F3 didn't..."~~
- Name the specific feature or key: **"Ctrl+H opens backspace instead of Find & Replace"** not ~~"Keyboard shortcut issue"~~
- Never use words like "maybe", "possibly", "needs confirmation", or "not verified" in a title — if you're not sure, don't file yet

### Before filing: search check

Search GitHub with at least 3 query variations. Use: the key name, the symptom, the feature name.
Log your search queries in the issue body so future runs don't repeat the same searches.

---

## Key Bindings (VERIFIED)

### File Operations
| Key | Action | Notes |
|-----|--------|-------|
| `Ctrl+N` | New empty buffer | Tab shows "[No Name]" |
| `Ctrl+O` | Open file dialog | Full file browser |
| `Ctrl+S` | Save | Save-as dialog for new files |
| `Ctrl+Q` | Quit | Hot exit saves state |
| `Ctrl+B` | Toggle File Explorer sidebar | Opens/closes only |
| `Ctrl+E` | Focus/unfocus File Explorer | Switches keyboard focus between editor and explorer |
| `Ctrl+W` | **Select word under cursor** | ⚠️ NOT "close buffer" (different from VS Code!) |
| `Alt+W` | Close current tab | Closes the tab in the focused split; prompts if unsaved (requires letter + Enter) |
| `Ctrl+PgDn` | Next Buffer | tmux: `C-NPage` |
| `Ctrl+PgUp` | Previous Buffer | tmux: `C-PPage` |
| Close Buffer | No default shortcut | Use `Ctrl+P → "Close Buffer"` for buffer-level close |
| Save As | No keyboard shortcut in terminals | Use `Alt+F → navigate to Save As...` |

### Editing
| Key | Action | Notes |
|-----|--------|-------|
| `Ctrl+Z` | Undo | Per-character granularity |
| `Ctrl+Y` | Redo | |
| `Ctrl+C` | Copy | |
| `Ctrl+V` | Paste | |
| `Ctrl+A` | Select All | |
| `Ctrl+D` | Add cursor at next match | Multi-cursor: select word first |
| `Ctrl+H` | ⚠️ BROKEN IN TERMINALS | Intended: Find & Replace. Actual in tmux: Backspace (deletes word). Use `Ctrl+R` instead. |
| `Alt+↑/↓` | Move line up/down | |
| `Alt+U` | Uppercase selection/word | |
| `Alt+L` | Lowercase selection/word | |

### Search & Replace
| Key | Action | Notes |
|-----|--------|-------|
| `Ctrl+F` | Open search bar | Remembers previous term |
| `Ctrl+R` | Open replace bar | |
| `Ctrl+Alt+R` | Query replace (interactive) | y/n/!/q per match |
| `Enter` (in search) | Jump to match, close search | |
| `F3` | Find next (after search closes) | Must be tested with search bar CLOSED |
| `Shift+F3` | Find previous | |
| `Escape` | Cancel search | |
| `Alt+C` | Toggle case-sensitive | In search bar |
| `Alt+W` | Toggle whole word | In search bar |
| `Alt+R` | Toggle regex | In search bar |
| `Alt+A` | Search & Replace in Project | |

### Views & Navigation
| Key | Action | Notes |
|-----|--------|-------|
| `Ctrl+P` | Command Palette | Modes: `>cmd` `:line` `#buffer` `file` |
| `F10` or `Alt+letter` | Open menu bar | Arrow keys navigate; Enter selects |
| `Alt+]` | Next split | |
| `Alt+[` | Previous split | |
| `Ctrl+G` | Go to line number | |
| `F8` | Jump to next error | |
| `Shift+F8` | Jump to previous error | |
| `Alt+←` | Navigate back in history | |

### Close Buffer (no default shortcut)
- Use: `Ctrl+P` → "Close Buffer" → `Enter`
- Prompt: `(s)ave, (d)iscard, (C)ancel?` → press letter then `Enter`

### Utility Dock & Terminal
| Key | Action | Notes |
|-----|--------|-------|
| `Alt+\`` | Open terminal in utility dock | Splits screen; terminal at bottom |
| `Ctrl+Space` | Toggle terminal mode ↔ scrollback mode | "Terminal mode enabled/disabled" in status bar |
| `Ctrl+]` | Exit terminal mode | Same as Ctrl+Space |
| `F9` | Toggle keyboard capture in terminal | All keys go to terminal; UI dims |
| `Ctrl+F` | Search in terminal scrollback | Works when in scrollback (read-only) mode |
| `Alt+J` | Toggle dock focus | Switch focus between editor and bottom dock |

### View Toggles (Run #3)
| Key | Action | Notes |
|-----|--------|-------|
| `Ctrl+P → "Toggle Line Numbers"` | Toggle line numbers | Via command palette |
| `Alt+V → ☑ Line Wrap` | Toggle line wrap | Via View menu ONLY (not palette) |
| `Alt+V → ☐ Mouse Support` | Toggle mouse support | Via View menu |
| `Alt+V → ☑ Vertical Scrollbar` | Toggle scrollbar | Via View menu |

### Settings & Theme (Run #4 + Run #5)
| Action | Method | Notes |
|--------|--------|-------|
| Open Settings UI | `Ctrl+P → "Open Settings"` | Full visual UI with categories |
| Change Theme | `Ctrl+P → "Select Theme"` | Arrow keys (DECCKM) to navigate |
| Edit Theme (visual) | `Ctrl+P → "Edit Theme"` | Select theme → color tree opens |
| Keybinding Editor | `Ctrl+P → "Open Keybinding Editor"` | 843 bindings, / search, r record-key |
| Settings persist | Saved to `/root/.config/fresh/config.json` | theme, keybindings |
| Settings search | `Ctrl+P → "Open Settings"` → `/` key | Searches setting names |
| Whitespace: trailing | Settings → `/` → "whitespace" → "Trailing Spaces" → Enter | Enables ··· dots for trailing spaces |

### Advanced Features (Run #6)
| Feature | Command | Notes |
|---------|---------|-------|
| Theme Editor color edit | `Ctrl+P → "Edit Theme"` → pick theme → navigate to color → Enter → type hex → Enter | Bottom input: "FieldName (#RRGGBB or named):" |
| Theme Editor Save As | `Ctrl+P → "Theme: Save As"` | Saves custom theme to ~/.config/fresh/themes/ as JSON |
| Auto Save | Set in `/root/.config/fresh/config.json`: `"editor": {"auto_save_enabled": true, "auto_save_interval_secs": 5}` | OR navigate Settings > Editor > Recovery; interval 30s default |
| Env Manager | `Ctrl+P → "Env: Show Status"` → `"Env: Activate"` → `"Env: Use System (Deactivate)"` | Status bar shows "Environment active (direnv)" when activated |
| Tour | `Ctrl+P → "Tour: Load Definition..."` → type path (pre-filled as `.fresh-tour.json`) | Steps shown as overlay; Tab to focus "Next →"; Up→focus "Next →"; Enter to advance |
| Review Diff Stage | Review Diff → `n` navigate to hunk → `s` to stage | Hunk moves from UNSTAGED to STAGED; `u` to unstage, `d` broken (BUG #2117) |
| Orchestrator New Session | `Ctrl+P → "Orchestrator: New Session"` OR `Orchestrator: Open` → `Alt+N` | FORM: Project Path, worktree checkbox, session name, agent, branch. Tab×6 to reach "Create Session". Creates git worktree. |
| Workspace Trust (verified set) | `Ctrl+P → "Workspace Trust…"` → T | Status bar: "Workspace trusted — project tooling may run processes" |

### Advanced Features (Run #5)
| Feature | Command | Notes |
|---------|---------|-------|
| Large file mode | Open any file >threshold | Byte offsets in gutter; "Byte 0" in status bar |
| Scan Line Index | `Ctrl+P → "Scan Line Index"` | Builds line index; then `:N` palette nav works |
| Code Folding | `Ctrl+P → "Toggle Fold"` | ▸=folded, ▾=expanded; Down skips folded |
| Vertical Ruler | `Ctrl+P → "Add Ruler"` → type col → Enter | Column tinted with `[48;5;236m]` background |
| Remove Ruler | `Ctrl+P → "Remove Ruler"` | Removes ruler from current buffer |
| Move File Explorer | `Ctrl+P → "Move File Explorer to Other Side"` | Switches left↔right; persists to config |
| Live Diff: vs HEAD | `Ctrl+P → "Live Diff: vs HEAD"` | Green + lines in gutter; status: "Live Diff: comparing against HEAD" |
| Orchestrator | `Ctrl+P → "Orchestrator: Open"` | Session selector popup; Alt+Q may not work via tmux |
| Workspace Trust | `Ctrl+P → "Workspace Trust…"` | Security dialog; T=trust, K=restrict; Esc=cancel |
| Environment Manager | `Ctrl+P → "Env: Show Status"` | Shows active environment; Env: Activate to inject |

### Advanced Features (Run #4)
| Feature | Command | Notes |
|---------|---------|-------|
| Git Log | `Ctrl+P → "Git Log"` | q to quit, arrows navigate, Enter preview |
| Review Diff | `Ctrl+P → "Review Diff"` | n/p hunks, s/u/d stage/unstage/discard |
| Git Blame | `Ctrl+P → "Git Blame"` | Magit-style per-line blame |
| Live Grep | `Ctrl+P → "Live Grep"` | Multi-scope (Files/Buffers/Terminals), streaming |
| Diagnostics Panel | `Ctrl+P → "Toggle Diagnostics Panel"` | Opens in dock; q to close |
| Record Macro | `Ctrl+P → "Record Macro"` → digit → Enter | F5 stops, F4 plays |
| Set Bookmark | `Ctrl+P → "Set Bookmark"` → digit → Enter | Alt+N jumps to bookmark N |
| Markdown Preview | `Ctrl+P → "Markdown: Toggle Compose/Preview"` | ANSI bold/italic rendered |
| Duplicate Line | `Ctrl+P → "Duplicate Line"` | Duplicates current line below |
| Surround selection | Select text, type bracket/quote | `[word]`, `"text"`, etc. |

### Editing (Run #4)
| Feature | Key | Notes |
|---------|-----|-------|
| Select line | `Ctrl+L` | Selects current line, cursor advances |
| Smart Home | `Home` | First: col of first non-whitespace; second: col 1 |
| Position History Back | `Alt+Left` | Navigate back through edit positions across files |
| Auto-close brackets | Type `(` → inserts `()` | Cursor inside; typing `)` skips |
| Binary file handling | Ctrl+O → open .bin | Tab: [BIN], content: `<FF>...`, auto [RO] |

---

## Settings UI Navigation (Run #6 — CRITICAL)

### Navigation Model
The Settings UI has a complex two-panel navigation:
- **Left panel** (category tree): Uses ↑↓ DECCKM arrows. Collapsed categories (`▶`) skip all children on Down. Expand with Right arrow (`→`), collapse with Left (`←`). 
- **Right panel** (settings content): Scrollable. Automatically scrolls to show the selected section when you select it in the left panel AND press Enter.
- **Tab**: Navigates to the next FOCUSABLE widget in the right panel. Only NUMBER FIELDS and TEXT INPUTS are focusable via Tab. CHECKBOXES ARE NOT tab-navigable (confirmed in Run #6).
- **To enter a section**: Left panel → navigate to section → Enter → right panel scrolls to that section
- **To reach a setting**: Left panel → correct section → Enter → Tab to first number/text field → navigate from there

### Step-by-Step: Reach Auto Save Enabled checkbox
The checkbox `Auto Save Enabled: [ ]` is in Settings > Editor (expanded with →) > Recovery.
Currently, the ONLY confirmed working method is:
1. Edit `/root/.config/fresh/config.json` directly: add `"editor": {"auto_save_enabled": true}`
2. Restart Fresh to pick up the config change
3. Verify in Settings UI that `Auto Save Enabled: [v]` is now checked

### Category Navigation Order (Editor expanded)
From Editor (expanded), Down navigates:
1. Bracket Matching
2. Completion  
3. Diagnostics
4. Display
5. Editing
6. Keyboard
7. LSP
8. Mouse
9. Performance
10. Recovery ← contains: auto_recovery_interval, auto_revert_poll, **auto_save_enabled**, auto_save_interval, recovery_enabled
11. Startup
12. Status Bar
13. Whitespace

### Reaching Recovery
1. Navigate: ↑↓ to `▶ Editor` → Right to expand → Down × 10 to `Recovery`
2. Press Enter → right panel scrolls to Recovery content
3. Tab → first focusable item (Auto Recovery Save Interval Secs)
4. Tab again → jumps to `[ User ]` footer button (bypassing checkboxes)

## tmux Automation Notes (CRITICAL — Run #2 Discovery)

### Arrow Keys MUST Use DECCKM Sequences
Fresh operates in DECCKM (application cursor key mode). Standard VT100 arrow sequences are **silently ignored**.

```bash
# CORRECT — DECCKM application sequences
tmux send-keys -t SESSION $'\033OA'   # Up
tmux send-keys -t SESSION $'\033OB'   # Down
tmux send-keys -t SESSION $'\033OC'   # Right
tmux send-keys -t SESSION $'\033OD'   # Left

# WRONG — VT100 sequences (ignored by Fresh)
tmux send-keys -t SESSION Up
tmux send-keys -t SESSION Down
tmux send-keys -t SESSION $'\033[A'   # Also ignored
```

### Delete Key
```bash
tmux send-keys -t SESSION $'\033[3~'   # CORRECT — forward delete
# NOT: tmux send-keys -t SESSION DC    # DC key name is NOT forwarded correctly
```

### Keyboard Timing
- Allow 200-300ms between key sends: `sleep 0.3`
- Allow 1-2s for panel operations (search, replace): `sleep 2`
- Multi-key sequences (multiple BSpace): add delay between each: `sleep 0.1` per key

### Command Palette Mode Switching
The palette opens with `>` (command mode). To switch modes:
- BSpace to remove `>` → file/fuzzy-finder mode (shows project files)
- Type `:` → line mode
- Type `#` → buffer mode
- Ctrl+U does NOT clear the input (sends literal in editor instead)
- Use multiple BSpace presses with delays; watch for input leak bug (#2113)

---

## Features That Look Like Bugs Added in Run #6

| Observation | What it actually is |
|-------------|---------------------|
| Settings checkboxes not reachable via Tab | **UNCONFIRMED** — Tab navigates to number/text inputs + footer buttons; checkboxes appear to be skipped. May require mouse. PENDING investigation. |
| "Tour ended" status message after Exit Tour | **By design** — Confirms the tour was successfully closed. |
| Orchestrator New Session: Enter closes dialog without creating session | **Focus issue** — Enter may toggle the "Create worktree" checkbox if that's focused. Must Tab×6 to reach "Create Session" button. |
| Settings UI: "Enter" on a search result navigates to the SECTION, not the setting | **By design** — Enter scrolls right panel to show the section containing the setting, then you must Tab to interact with individual settings. |
| Review Diff UNSTAGED shows +1/-1 for same content after staging | **Likely line-ending normalization** — git staging may normalize line endings, creating a minor diff. NOT a bug; verify with git diff on the staged content. |

## Features That Look Like Bugs Added in Run #5

| Observation | What it actually is |
|-------------|---------------------|
| Review Diff: "Discard hunk" fails with "Patch failed: error: patch failed..." | **Real bug #2117** — NOT by design. File `git apply --reverse` works fine from shell; Fresh's internal patch application is broken. |
| Alt+Q doesn't open Orchestrator in tmux | **tmux interference** — Alt+Q might not be forwarded correctly by tmux. Use `Ctrl+P → "Orchestrator: Open"` instead. |
| Theme Editor left panel shows `*tree* [RO]` with "Editing disabled" | **By design** — the tree panel is read-only; Tab to switch to the right editing panel to actually change a color value. |
| Review Diff palette shows "Git Blame: Close" before "Git Blame" | **Palette ordering** — sub-commands appear below the main command in fuzzy-match results. Always navigate UP multiple times to find the main "Git Blame" command. |

---

## Features That Look Like Bugs (ARE BY DESIGN)

### Hot Exit (session persistence)
- **What:** All buffers — including unnamed scratch and dirty files — are automatically saved on quit and restored on next launch.
- **Why it looks like a bug:** Files reopen with `[+]`/asterisk even though content matches disk. An extra `[No Name]*` buffer may appear.
- **It's correct behavior.** Config: `hot_exit` (default: on). Docs: `docs/features/session-persistence.md`.
- **UX note to file separately:** Consider showing a "Restored N unsaved changes from previous session" notification to make this self-evident.

### Ctrl+W Selects Word (not close tab)
- Fresh uses `Ctrl+W` for "Select word under cursor."
- VS Code uses `Ctrl+W` for "Close tab." This is an intentional divergence.
- Docs: `docs/features/editing.md` confirms `Ctrl+W = Select word under cursor`.

### Revert = Discard Changes and Reload
- `File > Revert` shows a `(r)evert / (c)ancel?` prompt when the buffer is modified — this is correct.
- `File > Reload with Encoding...` DOES refuse with "Cannot reload: buffer has unsaved modifications" — this is also correct (it prevents overwriting local edits with a re-encoded version).
- ⚠️ Don't confuse these two menu items.

### Search Enter Closes After First Match
- This is by design. `Enter` = jump to match + close search bar.
- `F3` (after search closes) = navigate to next match.
- To cycle: `Ctrl+F` (pre-filled) → `Enter` → `F3` → `F3` ...

---

## UI Structure

### Layout
```
┌─ Menu bar (File / Edit / View / Selection / Go / LSP / Help) ─┐
├─ Tab bar ([Dashboard ×] [filename* ×])                        ─┤
│                                                                 │
│   Editor area (line numbers, ~ for empty lines)               │
│                                                                 │
└─ Status bar: mode | path [±] | Ln N, Col N | msg | enc | ⚠   ─┘
```

### Status Bar Indicators
| Indicator | Meaning |
|-----------|---------|
| `[+]` | Buffer has unsaved changes |
| `[RO]` | Read-only buffer |
| `*` in tab title | Same as `[+]` |
| `⚠ N` | N warnings/errors in diagnostics |
| `[⚠ 1]` on first launch | "Test i18n plugin loaded" (benign) |

### Selection Rendering (ANSI)
- Cursor character: `[48;5;16m` (near-black bg) or `[7m` (reverse video)
- Selected text: `[48;5;17m` (dark blue bg)
- Search match: `[48;5;17m` or `[48;5;226m` depending on theme
- Menu selected item: `[48;5;25m` (subtle dark blue — **easy to miss in non-ANSI capture**)

---

## tmux Interaction Rules

### Key Sending
```bash
# CORRECT — individual send-keys calls
tmux send-keys -t SESSION "S-Left" ""
sleep 0.2
tmux send-keys -t SESSION "S-Left" ""

# WRONG — sends literal text "S-Left S-Left"
tmux send-keys -t SESSION "S-Left S-Left" ""
```

### Key Name Reference
| Intent | tmux name |
|--------|-----------|
| Arrow keys | `Up` `Down` `Left` `Right` |
| Shift+Arrow | `S-Up` `S-Down` `S-Left` `S-Right` |
| Ctrl+key | `C-p` `C-f` `C-z` etc. |
| Alt/Meta+key | `M-f` `M-]` `M-[` etc. |
| Function keys | `F1` … `F12` |
| Home / End | `Home` `End` |
| Ctrl+Home/End | `C-Home` `C-End` |

### Timing
- After launching Fresh: sleep 2s before first capture
- Between selection keystrokes: sleep 0.2s minimum
- After Ctrl+P / menu open: sleep 0.5–1s
- After file operations: sleep 1s

### Verifying Menu Selection
Always use `tmux capture-pane -p -e` (with ANSI) to confirm which menu item is highlighted (`[48;5;25m`). Plain `-p` capture will not show the selection.

---

## Session / Testing Notes
- Fresh stores hot-exit state at `$XDG_RUNTIME_DIR/fresh/` or `/tmp/fresh-$UID/`
- Each test run that makes edits will affect the next run via hot exit
- Use `fresh --no-restore` to launch without restoring previous session state (useful for clean-slate tests)
- "Calibrate Keyboard" in the command palette detects terminal key-translation issues

---

## Lessons Added in Run #7

### Lesson 29: Settings UI — Checkboxes ARE keyboard-navigable (Run #7 resolution)
- Previous run thought checkboxes were NOT reachable by Tab. CORRECTION:
  - ↑↓ arrows (DECCKM: `\033OA`/`\033OB`) navigate ALL items in the right panel, including checkboxes
  - Enter toggles the highlighted checkbox (`[ ]` → `[v]` or vice versa)
  - Tab ONLY reaches number/text inputs (skips checkboxes)
  - The `>` prefix in the right panel shows the selected item
  - Modified-but-unsaved items show `>●` (bullet = pending change)
- Confirmed via: Settings UI → search `/confirm_quit` → Enter → checkboxes are selectable

### Lesson 30: Settings Search — Press `/` in LEFT Panel
- In Settings UI, pressing `/` triggers a SEARCH mode over all setting names/descriptions
- This works from the LEFT panel (not the right panel)
- Search shows: "(N-12 of 38) ↓" result count with ↓ arrows to cycle
- The first match is auto-selected with `▸` arrow in the results
- Press Enter to navigate to the selected result's location in the right panel

### Lesson 31: select_to_paragraph escape sequences (confirmed Run #7)
- `Ctrl+Shift+Down` = CSI `1;6B` = `$'\033[1;6B'` in tmux → triggers `select_to_paragraph_down`
- `Ctrl+Shift+Up` = CSI `1;6A` = `$'\033[1;6A'` in tmux → triggers `select_to_paragraph_up`
- These escape sequences work correctly in Fresh's application terminal mode
- Selection renders as `[48;5;17m]` (dark blue background) on selected text

### Lesson 32: confirm_quit prompt behavior
- When enabled: pressing Ctrl+Q shows `Quit Fresh? (y)es, (N)o:` at the bottom line
- Requires letter + Enter to confirm: `y` then Enter = quits; `N` then Enter = stays
- Just pressing `N` alone appends to the prompt without cancelling (same pattern as Close Buffer)
- Status bar shows "Close cancelled" after pressing N+Enter

### Lesson 33: Live Grep 0.3.9 toolbar and scopes
- New toolbar format: `Search in: [v] Files Alt+L  [ ] Ignored Alt+H  [v] Buffers Alt+U  [v] Terminals Alt+T  [ ] Diagnostics Alt+D`
- Toggle with: Alt+L (Files), Alt+H (Ignored), Alt+U (Buffers), Alt+T (Terminals), Alt+D (Diagnostics)
- Match modes: Alt+O (Word), Alt+G (Regex) — these are toggles shown with [v]/[ ]
- Provider cycle: Alt+P — cycles git-grep → rg → grep → (back)
- Save matches: Alt+M — saves current matches to a buffer
- Result count in header: "1 / 1000" format; "1000+" means capped at 1000
- Buffer results tagged: `[buf]` prefix (e.g., `[buf] /tmp/fresh-test/sample.js:3`)
- Status "Searching…" during search, then result count replaces it

### Lesson 34: move_to_paragraph design context (PR #2084)
- PR #2084 deliberately omitted command palette entries for `move_to_paragraph_*`
- Reasoning: "just like `move left` command is not needed" (movement-only actions don't get palette entries)
- BUT: forgot to add DEFAULT KEYBINDING (unlike `select_to_paragraph_*` which have Ctrl+Shift+↓/↑)
- This is likely an oversight since the select variants DO have bindings
- Filed as BUG #2122
- Workaround: Keybinding Editor → `/paragraph` → select action → Enter → press desired key

---

## Lessons Added in Run #8

### Lesson 35: LSP Status Popup — Enter selects FIRST item (no Down navigation needed for first item)
- `Ctrl+P → "Show LSP Status"` or `Ctrl+P → "LSP: ..."` opens LSP server status popup
- Popup format: `○ rust-analyzer (not running)` / `Start rust-analyzer (always)` / `Start rust-analyzer once` / `Disable LSP for rust` / `View Log` / `Dismiss (Esc)`
- **CRITICAL**: DO NOT use DECCKM sequences (`$'\033OB'`) to navigate LSP popups — they will close the popup (ESC character dismisses it) and the remaining characters get typed into the editor buffer
- Enter selects the FIRST item (currently highlighted — the top option)
- For navigating popup options beyond the first, use plain tmux key names: `Down` / `Up` (NOT `$'\033OB'`/`$'\033OA'`)
- When LSP server not installed: Fresh tries to start it → spawns log file tab → status bar: "LSP (error)" → log tab shows full error backtrace
- LSP log file location: `/root/.local/state/fresh/logs/lsp/<name>-<pid>.log`
- LSP states visible in status bar: `LSP (off)`, `LSP (error)`, `LSP` (running)

### Lesson 36: Command Palette List Navigation — Use plain tmux key names
- For navigating the RESULTS LIST in the command palette (both command and file mode), use plain `Up`/`Down` tmux key names
- DECCKM sequences (`$'\033OA'`/`$'\033OB'`) start with ESC (`\033`) which CLOSES any open overlay/popup/palette
- So: `$'\033OA'` in palette input = ESC (closes palette) + literal `OA` typed somewhere
- Safe rule: DECCKM sequences ONLY for cursor movement within the EDITOR BUFFER itself
- All overlay/popup/dialog navigation: use plain `Up`, `Down`, `Left`, `Right` key names

### Lesson 37: Live Grep Alt+M — Saves to *Quickfix* buffer
- `Alt+M` in Live Grep closes the overlay and saves all current matches to a `*Quickfix*` [RO] buffer
- The Quickfix buffer opens in a new split pane
- Format: `file:line:col  match_content` (one per line)
- Header line: `Quickfix: <query> (N matches)`
- The Live Grep overlay is dismissed when Alt+M is pressed (replaced by the buffer)
- The *Quickfix* buffer is read-only and shows the full path
- To navigate to a match: use Ctrl+PgDn to switch to the Quickfix tab, then navigate to line and Enter to jump

### Lesson 38: C3 Language Support (0.3.9) — Working
- Fresh detects `.c3`, `.c3i`, `.c3t` extensions as C3 language
- Status bar shows `C3` language indicator
- Grammar: `c3.sublime-syntax` — full syntax highlighting for:
  - Keywords (`module`, `fn`, `struct`, `import`, `return`, `const`): cyan `[38;5;51m]`
  - Types (`int`, `double`, `void`, struct names): pink `[38;5;207m]`
  - Function names: yellow `[38;5;226m]`
  - Numbers: blue `[38;5;69m]`
  - Strings: green `[38;5;34m]`
  - Comments (`//`): gray `[38;5;253m]`
- Code folding indicators (▾/▸) appear at `fn` and `struct` declarations
- LSP configuration: `c3lsp` configured but not bundled — shows `LSP (off)` until c3lsp is on PATH
- C3 file can be opened from outside the project (e.g., `/tmp/test.c3`) via Ctrl+O

### Lesson 39: Orchestrator 0.3.9 UI Improvements
- New header format: `ORCHESTRATOR :: Sessions  —  [scope]` (scope = "all projects" or "user/fresh")
- **Alt+P (current only)**: Toggles between all-projects and current-project scope
  - All projects: `Project: [ All ▾   (Alt+P) ]`
  - Current project: `Project: [ fresh ▾   (Alt+P) ]`
  - Header also updates: "all projects" → "user/fresh"
- **Alt+T (show all worktrees)**: Checkbox toggle `[ ] Show all worktrees` / `[v] Show all worktrees`
- **Filter search**: Press `/` to enter the filter input box; type to filter session names
- **Alt+N (New Session)**: Opens session creation form
- Navigation keys (shown in footer): `↑↓ nav  Enter dive  Space select  Alt+P current only  Tab focus  Esc close`
- Session detail panel (right): `[ Visit ]  [ Details ]  [ Stop ]  [ Archive ]  [ Delete ]` action buttons
- BASE session always shown as the current running session

### Lesson 40: Review Diff Discard — FIXED in 0.3.9
- BUG #2117 (discard hunk fails with "patch does not apply") is FIXED in the 0.3.9 dev build
- Discard workflow now works correctly:
  1. Review Diff → `n` to navigate to hunk (status: "Hunk 1 of 1")
  2. Press `d` → confirmation dialog: "Discard this hunk in 'FILE'? This cannot be undone."
  3. Enter to confirm → status: "Review Diff: 0 hunks", panel: "No changes to review."
  4. File is reverted to HEAD state (git diff confirms no changes)
- Confirmed twice in Run #8

### Lesson 41: Live Grep Diagnostics Scope — No results without active LSP
- `Alt+D` toggles Diagnostics scope in Live Grep toolbar
- When only Diagnostics scope is enabled (`[ ] Files`, `[ ] Buffers`, `[ ] Terminals`, `[v] Diagnostics`):
  - Without an active LSP server: "No matches" for any search (even empty)
  - The Diagnostics scope searches LSP-generated diagnostics, not just file content
  - The `[⚠ N]` in status bar (from i18n plugin or LSP errors) does NOT populate Diagnostics scope
  - Provider line disappears when Diagnostics-only (no git-grep/rg needed for diagnostics search)
- When LSP is running and diagnosing code: Diagnostics scope would show matches against diagnostic messages

### Lesson 42: getWorkingDataDir() and getTerminalDir() plugin APIs (0.3.9)
- `editor.getWorkingDataDir()`: Per-working-directory data directory root for plugin storage
  - Different from `getThemesDir()` — scoped to the current project/worktree
  - Used for storing project-specific plugin data
- `editor.getTerminalDir()`: Directory holding terminal scrollback backing files for current working dir
  - Path: `<data_dir>/terminals/<encoded-cwd>/`
  - Used by Live Grep to scope "Terminals" search to the current project's terminals (not all terminals)
  - This is WHY Live Grep "Terminals" scope stays scoped to the current project

### Lesson 43: Workspace Session Isolation
- Sessions ARE correctly scoped per working directory
- Launching Fresh from `/home/user/fresh` restores the session for that directory only
- Launching Fresh from `/tmp/fresh-test-project` restores (or starts fresh) for that directory only
- External files opened from outside the project (e.g., `/tmp/file.txt` from a project session) ARE included in that project's session restore
- No cross-project tab mixing observed — issue #2056 appears resolved
