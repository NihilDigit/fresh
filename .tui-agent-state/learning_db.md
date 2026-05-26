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

---

## Application Overview
- **Name:** Fresh — a modern terminal text editor
- **Version:** 0.3.8
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

---

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
