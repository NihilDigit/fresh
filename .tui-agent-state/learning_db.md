# Fresh Editor — TUI Agent Knowledge Base

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

## Key Bindings (VERIFIED)

### File Operations
| Key | Action | Notes |
|-----|--------|-------|
| `Ctrl+N` | New empty buffer | Tab shows "[No Name]" |
| `Ctrl+O` | Open file dialog | Full file browser |
| `Ctrl+S` | Save | Save-as dialog for new files |
| `Ctrl+Q` | Quit | Hot exit saves state |
| `Ctrl+B` | Toggle File Explorer sidebar | NOT Ctrl+E |
| `Ctrl+W` | **Select word under cursor** | ⚠️ NOT "close buffer" (different from VS Code!) |

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
