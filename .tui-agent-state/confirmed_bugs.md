# Confirmed Bugs Registry

## Bug Report Format
Each bug includes:
- Bug ID: BUG-NNN
- Date: discovered
- Severity: Critical/High/Medium/Low
- Description
- Reproduction steps (exact tmux send-keys sequence)
- Expected behavior
- Actual behavior
- GitHub Issue link (if filed)

---

## Open Bugs

### BUG-001 - Revert Command Refuses to Discard Unsaved Changes
- **Date:** 2026-05-26
- **Severity:** High
- **Description:** The "Revert" command (File > Revert) refuses to work when the buffer has unsaved modifications, showing "Cannot reload: buffer has unsaved modifications (save first)". This is the opposite of correct behavior - Revert is meant to DISCARD unsaved changes and reload from disk.
- **Reproduction Steps:**
  ```
  tmux send-keys -t fresh-testing "/path/to/fresh /tmp/test.txt" Enter
  # Make some edits (type text)
  tmux send-keys -t fresh-testing "M-f" ""  # Open File menu
  tmux send-keys -t fresh-testing "Down Down Down Down Down Down" ""  # Navigate to Revert
  # Observe: "Cannot reload: buffer has unsaved modifications (save first)"
  ```
  Alternative: Edit a file, then use command palette to run "Revert" command
- **Expected:** Revert should ask "Are you sure you want to discard unsaved changes?" then reload from disk
- **Actual:** Error: "Cannot reload: buffer has unsaved modifications (save first)"
- **GitHub Issue:** https://github.com/sinelaw/fresh/issues/2108

---

### BUG-002 - Ctrl+H Deletes Word Instead of Opening Find & Replace
- **Date:** 2026-05-26
- **Severity:** Medium (documentation/UX inconsistency)
- **Description:** Ctrl+H is described as the Find & Replace shortcut in VS Code and Sublime Text, which Fresh claims to be compatible with. However, Ctrl+H in Fresh deletes the previous word (like Alt+Backspace in standard bindings). The actual Replace shortcut is Ctrl+R.
- **Reproduction Steps:**
  ```
  # Open Fresh with a file containing text
  # Position cursor after a word
  tmux send-keys -t fresh-testing "C-h" ""
  # Observe: Word before cursor is deleted instead of Replace dialog opening
  ```
- **Expected:** Ctrl+H opens Find & Replace dialog (as in VS Code/Sublime Text)
- **Actual:** Ctrl+H deletes the previous word
- **GitHub Issue:** https://github.com/sinelaw/fresh/issues/2109
- **Notes:** Ctrl+R is the actual Replace shortcut. This contradicts Fresh's claim of "VS Code / Sublime Text familiar keybindings."

---

### BUG-003 - File Opens as Modified on First Launch (Session Restoration)
- **Date:** 2026-05-26
- **Severity:** Medium
- **Description:** When opening Fresh with a file argument after a previous session where that file had (discarded) unsaved changes, the file opens as modified (`[+]` indicator, asterisk in tab title) even though the file content matches what's on disk. This appears to be a hot-exit/session restoration issue where discarded changes are being partially restored.
- **Reproduction Steps:**
  ```
  # Run 1: Open file, modify it, close and DISCARD changes
  ./fresh /tmp/test.txt
  # Type some text
  # Close buffer: use command palette → "Close Buffer"
  # At prompt: press 'd' then Enter to discard
  # Quit: Ctrl+Q
  
  # Run 2: Open same file again
  ./fresh /tmp/test.txt
  # Observe: file shows as modified [+] immediately on open
  ```
- **Expected:** File opens as unmodified (no `[+]` marker, no asterisk) when content matches disk
- **Actual:** File shows as modified even though content matches disk exactly
- **GitHub Issue:** https://github.com/sinelaw/fresh/issues/2110
- **Notes:** Also observed: extra "[No Name]*" buffer opens alongside the file in the restored session.

---

### BUG-004 - Search Enter Navigation Does Not Advance When Cursor Is At Match
- **Date:** 2026-05-26
- **Severity:** Low-Medium (UX issue)
- **Description:** When using Ctrl+F search and the cursor is already positioned AT a search match, pressing Enter does not advance to the next match. Instead it closes the search bar and stays at the current position. Users who reopen search (Ctrl+F) from an existing match position cannot navigate forward.
- **Reproduction Steps:**
  ```
  # Open Fresh with a file containing multiple occurrences of "fox"
  # Search for "fox" with Ctrl+F
  tmux send-keys -t fresh-testing "C-f" ""
  tmux send-keys -t fresh-testing "fox" ""
  tmux send-keys -t fresh-testing "Enter" ""  # Navigate to first match
  # Cursor is now AT the "fox" match
  tmux send-keys -t fresh-testing "C-f" ""  # Reopen search
  # "fox" is pre-filled
  tmux send-keys -t fresh-testing "Enter" ""  # Expect next match, but...
  # Observe: cursor stays at same match
  ```
- **Expected:** Enter should always advance to the NEXT match after current cursor position
- **Actual:** When cursor is already AT a match, Enter stays at same match and closes search
- **GitHub Issue:** https://github.com/sinelaw/fresh/issues/2111

---

## Resolved Bugs

*(None yet)*
