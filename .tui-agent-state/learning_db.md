# Fresh Editor - TUI Agent Knowledge Base

## Application Overview
- **Name:** Fresh - a modern terminal text editor
- **Version:** 0.3.8
- **Language:** Rust
- **Binary name:** `fresh`
- **Location:** `target/release/fresh` (after build)

## Launch Commands
```bash
# Open fresh with no files
./target/release/fresh

# Open a specific file
./target/release/fresh <filename>

# Open in headless/test mode (if available)
# TBD - explore options
```

## Key Bindings (VERIFIED through testing)

### Navigation
- Arrow keys: Move cursor (tmux: use "Up", "Down", "Left", "Right")
- Ctrl+Home: Go to beginning of file
- Ctrl+End: Go to end of file
- Ctrl+Left / Ctrl+Right: Word movement (unverified)
- S-Left, S-Right, S-Up, S-Down: Select text with Shift+Arrow keys

### File Operations
- Ctrl+N: New empty buffer
- Ctrl+O: Open file dialog (file browser dialog)
- Ctrl+S: Save (shows Save As dialog for new files; saves silently for existing)
- Ctrl+Q: Quit application
- Ctrl+B: Toggle File Explorer panel (NOT Ctrl+E!)
- ⚠️ Ctrl+W: NOT bound to Close Buffer (no default shortcut for Close Buffer)
- Close Buffer: Use command palette → "Close Buffer" (prompts for unsaved changes)

### Editing
- Ctrl+Z: Undo (individual characters)
- Ctrl+Y: Redo
- Ctrl+C: Copy (selection or line)
- Ctrl+V: Paste
- Ctrl+A: Select All
- Ctrl+D: Multi-cursor - add cursor at next occurrence of selection
- Ctrl+H: ⚠️ DELETES PREVIOUS WORD (NOT Replace!)

### Search & Replace
- Ctrl+F: Find (search bar opens at bottom)
  - Alt+C: Toggle case-sensitive
  - Alt+W: Toggle whole word
  - Alt+R: Toggle regex
  - Enter: Jump to next match AFTER cursor (closes search bar)
  - Escape: Cancel search
  - ⚠️ NO F3 or Shift+Enter navigation confirmed working
- Ctrl+R: Replace (correct shortcut!)
  - Replaces ALL occurrences by default
  - Alt+I: Toggle "Confirm each" mode for selective replacement
- ⚠️ Ctrl+H is NOT Replace (it's Delete Previous Word)

### Views & Panels
- Ctrl+P: Command Palette (opens with all commands)
- Ctrl+B: Toggle File Explorer sidebar
- F10 or Alt+letter: Open menu bar
- "Split Vertical" command: Creates horizontal split (two panes stacked)
- Alt+]: Next split pane
- Alt+[: Previous split pane
- "Close Split" command: Closes current split pane
- "Toggle Maximize Split" command: Maximize/restore split

### Command Palette Notes
- Ctrl+P opens palette with multiple modes: `file | >command | :line | #buffer`
- Type `>` prefix for commands, `:` for line numbers, `#` for buffer names
- Previous search terms are remembered

## UI Structure
- **Menu bar** at top (File, Edit, View, Selection, Go, LSP, Help)
  - Opens with F10 or Alt+letter (underlined letters are shortcuts)
  - Keyboard navigation with arrow keys WORKS (but highlight is subtle - dark blue)
  - Enter activates selected menu item
  - Menu items show keyboard shortcuts on right
- **Tab bar** below menu (shows open buffers)
  - Asterisk (*) in tab = unsaved changes
  - `[+]` in status bar = file modified but not saved
  - `[RO]` = read-only buffer
- **Editor area** (main content with line numbers and ~ for empty lines)
- **Status bar** at bottom: `mode | filename [status] | Ln N, Col N | message | encoding | type | alerts | hint`

## Selection Rendering (ANSI)
- Cursor position: `[48;5;16m` (very dark background) or `[7m` (reverse video)
- Selected text: `[48;5;17m` (selection blue/dark blue)
- Search match highlight: `[48;5;17m` or `[48;5;226m` depending on theme

## Confirmed Quirks
1. **Close Buffer** has no default keyboard shortcut (no Ctrl+W binding)
2. **Revert** fails with unsaved modifications (BUG-001 - should discard them)
3. **Ctrl+H** deletes word (BUG-002 - users expect Find & Replace)
4. **File opens as modified** after previous session with discarded changes (BUG-003)
5. **Search Enter** doesn't advance when cursor is already at a match (BUG-004)
6. **"Split Vertical"** actually creates horizontal layout (naming inconsistency)
7. Menu navigation highlight is subtle (dark blue `[48;5;25m`) - may appear unnavigable
8. Dashboard shows on first launch with git/disk status info
9. Session restoration: Fresh remembers previous session state (hot exit)
10. Warning `[⚠ N]` in status bar - meaning/context needs investigation

## tmux Interaction Notes
- Always send keys individually for special sequences, not combined in one string
  - WRONG: `tmux send-keys -t session "S-Left S-Left S-Left" ""`  (sends literal text!)
  - RIGHT: Send three separate tmux send-keys calls for S-Left
- Use proper key names: "Up", "Down", "Left", "Right", "S-Left", "S-Right"
- For modifier combos: "C-p" (Ctrl+P), "M-f" (Alt+F), "S-Left" (Shift+Left)
- Sleep at least 0.2s between key presses when doing selection
- Sleep 1-2s after launching the editor before capturing

## Testing Notes
- Application requires a proper terminal emulator
- tmux sessions work well for interaction
- Use `tmux capture-pane -p -e` for ANSI output when checking highlights/colors
- Fresh binary: `./target/release/fresh`
- Build command: `cargo build --release --bin fresh`

## UI Structure
- Menu bar at top (File, Edit, View, etc.)
- Tab bar below menu
- Editor area (main content)
- Status bar at bottom
- Optional: File explorer panel (left), Terminal panel (bottom)

## Known Quirks
- (None yet - to be populated from testing)

## Testing Notes
- Application requires a proper terminal emulator
- tmux sessions work for interaction
- ANSI color codes present in output
