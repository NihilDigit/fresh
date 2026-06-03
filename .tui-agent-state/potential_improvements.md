# Potential Improvements Backlog

This file accumulates UX, documentation, and feature improvement ideas discovered
during automated testing. These are NOT bugs — the application works as intended —
but they represent friction points that real users are likely to hit.

Each entry records: what confused us, what the correct behavior is, and what
change would make it self-evident without requiring users to read docs.

---

## UI / Discoverability

### IMP-001 — Hot Exit: No Indication That Restoration Happened
- **Observed:** On relaunch, files open with `[+]`/asterisk and no explanation.
- **Correct behavior:** Hot exit intentionally restored unsaved changes from the prior session.
- **Problem:** Users (and the test agent) see `[+]` and think something went wrong or the file is corrupted. Nothing tells them "we restored your previous session."
- **Suggested fix:** On first render after a hot-exit restore, show a dismissible status message or notification banner: *"Restored 2 unsaved buffers from your previous session. [`hot_exit` is on — disable in settings]"*
- **Effort:** Low — one status message in the restore path.
- **Discovered:** Run #1, 2026-05-26

---

### IMP-002 — Search Bar: No Hint for F3 / Shift+F3 Navigation
- **Observed:** The search bar shows `[x] Case Sensitive (Alt+C) | [ ] Whole Word (Alt+W) | [ ] Regex (Alt+R)` but no navigation hint.
- **Correct behavior:** After pressing Enter to jump to a match, `F3`/`Shift+F3` navigate next/previous. This is the correct workflow.
- **Problem:** Users (and the test agent) expect Enter to cycle through matches (VS Code behavior). Nothing in the UI signals that Enter closes the bar and F3 takes over.
- **Suggested fix:** Extend the hint line to: `[x] Case Sensitive (Alt+C)  [ ] Whole Word (Alt+W)  [ ] Regex (Alt+R)  |  Enter: jump · F3: next · Shift+F3: prev`
- **Effort:** Very low — add text to the search bar footer.
- **Discovered:** Run #1, 2026-05-26

---

### IMP-003 — Ctrl+H Terminal Compatibility Not Surfaced
- **Observed:** `Ctrl+H` is documented as Find & Replace but in terminals (including tmux) it is transmitted as ASCII `0x08` = Backspace. Pressing it silently deletes text.
- **Correct behavior:** `Ctrl+R` is the reliable Find & Replace shortcut; `Ctrl+H` is the *intended* shortcut but unreliable in terminals.
- **Problem:** Fresh markets "familiar VS Code/Sublime keybindings." VS Code users reach for Ctrl+H and destroy text with no warning.
- **Suggested fixes (pick one or combine):**
  1. Add `Ctrl+H` to the **Calibrate Keyboard** wizard's detection list, with a warning: *"Your terminal sends Ctrl+H as Backspace. The Find & Replace shortcut Ctrl+H may not work — use Ctrl+R instead."*
  2. Add a note to the keyboard reference doc next to the `Ctrl+H` entry: *"Note: many terminals transmit Ctrl+H as Backspace. If this doesn't open Find & Replace, use Ctrl+R."*
  3. On the first occurrence of a "delete previous word" action triggered by `0x08`, offer a one-time tooltip: *"Ctrl+H was received as Backspace. Did you mean Find & Replace? Use Ctrl+R."*
- **Effort:** Low–Medium.
- **Discovered:** Run #1, 2026-05-26

---

### IMP-004 — Menu Selection Highlight Too Subtle
- **Observed:** Navigating the menu bar with arrow keys works, but the selection highlight (`[48;5;25m` dark blue background) is nearly invisible in many terminal themes, making the menu appear unresponsive to keyboard input.
- **Correct behavior:** Arrow key navigation works correctly.
- **Problem:** Users (and the test agent's plain-text captures) cannot tell which menu item is highlighted. We initially reported this as "menu navigation doesn't work" before checking the ANSI output.
- **Suggested fix:** Use a higher-contrast selection color in the menu, or invert text color on selection, consistent with how the command palette highlights items.
- **Effort:** Low — theme/color change.
- **Discovered:** Run #1, 2026-05-26

---

### IMP-005 — Ctrl+W Diverges From VS Code Without Warning
- **Observed:** `Ctrl+W` selects the word under cursor. In VS Code/Sublime, `Ctrl+W` closes the current tab.
- **Correct behavior:** This is an intentional design choice in Fresh (documented in editing.md).
- **Problem:** VS Code users repeatedly pressing Ctrl+W to close tabs instead select words, which is invisible on some lines and confusing otherwise. There is no "close buffer" keyboard shortcut at all by default.
- **Suggested fixes:**
  1. Add a default `Ctrl+W` → "Close Buffer" keybinding, aliasing it alongside the word-select behavior (or make it context-sensitive: close if nothing is selected, select word if cursor is in a word).
  2. OR: In the "Getting Started" / welcome dashboard, call out this specific divergence from VS Code.
  3. OR: When a user presses Ctrl+W 3+ times in quick succession with no effect, show a hint: *"Ctrl+W selects the word under cursor. To close a buffer, use the command palette (Ctrl+P → 'Close Buffer')."*
- **Effort:** Medium (binding change) or Low (documentation/hint).
- **Discovered:** Run #1, 2026-05-26

---

### IMP-006 — "Reload with Encoding" Error Message Could Guide User Better
- **Observed:** `File > Reload with Encoding...` shows "Cannot reload: buffer has unsaved modifications (save first)" when the buffer is dirty.
- **Correct behavior:** The error is intentional — reloading with a different encoding would discard local edits.
- **Problem:** The error message is a dead end. The user knows they have unsaved changes but doesn't know what to do next.
- **Suggested fix:** Extend the message to: *"Cannot reload: buffer has unsaved modifications. Save first (Ctrl+S), or discard changes via Close Buffer → (d)iscard, then reopen."*
- **Effort:** Very low — improve error string.
- **Discovered:** Run #1, 2026-05-26

---

## Documentation Gaps

### IMP-007 — Session Persistence / Hot Exit Needs Prominent Mention on First Launch
- **Observed:** The Dashboard shows git/disk info but no mention of hot exit or session restore.
- **Suggested fix:** On the Dashboard (or a "first run" panel), add one line: *"Your editor state — open files, unsaved changes, and terminal sessions — is automatically saved on quit and restored on relaunch. Configure with `hot_exit` in settings."*
- **Effort:** Low.
- **Discovered:** Run #1, 2026-05-26

---

### IMP-008 — "Split Vertical" Command Creates Horizontal Layout
- **Observed:** The command palette entry "Split Vertical" creates a horizontal layout (two panes stacked, divided by a horizontal line).
- **Correct behavior:** This is consistent with many editors where "vertical split" = a vertical *divider* (side by side), but Fresh's "Split Vertical" creates a *horizontal divider* (stacked). The naming is the reverse of what VS Code users expect.
- **Suggested fix:** Rename to "Split Horizontally" and add a "Split Vertically" (side by side) command. Or add parenthetical descriptions: "Split Vertical (stacked)" vs "Split Horizontal (side by side)".
- **Effort:** Low — rename + add second variant.
- **Discovered:** Run #1, 2026-05-26

---

## Testing Infrastructure

### IMP-010 — "Toggle Line Wrap" Not Available in Command Palette
- **Observed:** "Toggle Line Numbers" is in the command palette, but "Toggle Line Wrap" is not.
- **Correct behavior:** Line wrap can be toggled via View menu (`Alt+V → Line Wrap`).
- **Problem:** Inconsistency — some view toggles are in the palette (line numbers) and others aren't (line wrap). Users reaching for Ctrl+P to toggle wrap won't find it.
- **Suggested fix:** Add "Toggle Line Wrap" to the command palette alongside "Toggle Line Numbers".
- **Effort:** Very low.
- **Discovered:** Run #3, 2026-05-26

---

### IMP-011 — Shift+F3 Key Binding Documentation May Be Inconsistent
- **Observed:** `docs/features/editing.md` documents `Shift+F3` = Find Previous. But the command palette shows "Find Previous" bound to `Ctrl+Shift+N`.
- **Correct behavior:** Unknown without testing in a proper terminal — both may exist simultaneously.
- **Problem:** If `Shift+F3` doesn't actually work (even outside tmux), users will be confused when the documented shortcut does nothing.
- **Suggested fix:** Test `Shift+F3` in a proper xterm/Kitty terminal. If it works, document both. If only `Ctrl+Shift+N` works, update editing.md.
- **Effort:** Low (just testing + doc update).
- **Discovered:** Run #3, 2026-05-26

---

### IMP-009 — No `--headless` or Scriptable Test Mode
- **Observed:** The TUI agent must use tmux to interact with Fresh. This works but is fragile — timing-dependent, ANSI parsing is complex, and key send errors produce hard-to-diagnose bugs (e.g., "S-Left S-Left" sent as literal text).
- **Suggested fix:** A `fresh --test-mode` or pipe-based command interface that accepts structured input (JSON events) and produces structured output (cursor position, buffer content, status) would make automated testing far more reliable.
- **Effort:** High — new subsystem. But would significantly improve the quality of Fresh's own e2e test suite as well.
- **Discovered:** Run #1, 2026-05-26

---

### IMP-012 — Package Manager: Install/Uninstall Buttons Not Reliably Keyboard-Accessible
- **Observed (Run #16):** In the Package Manager panel, selecting a package and pressing Tab reaches an "Enter Activate" position that shows `[ Install ]` brackets in the detail pane. However, pressing Enter at that position opens the Search field rather than triggering the Install action. The Tab cycle order is: Search → filter tabs (All/Installed/Plugins/Themes/Languages/Bundles) → Sync → list items → back to Search.
- **Correct behavior:** The package list and Install/Uninstall buttons should be fully keyboard-operable for accessibility. Currently, installing requires the "Package: Install from URL" command (palette workaround) and uninstalling requires manual `rm -rf` of the plugin directory.
- **No "Package: Uninstall" command exists in the command palette** — users cannot uninstall via keyboard alone without knowing the filesystem path.
- **Problem:** A fully mouse-free workflow (essential for a TUI editor's core audience) is blocked. Power users who rely entirely on keyboards cannot install or uninstall plugins.
- **Suggested fixes:**
  1. Fix the "Enter Activate" binding in the Package Manager so pressing Enter on an Install/Uninstall button actually triggers it.
  2. Add a `Package: Uninstall` (or `Package: Remove`) command to the command palette that operates on the currently-selected/installed package.
  3. Alternatively, expose Install/Uninstall as direct keybindings (e.g., `i` to install, `u` to uninstall) within the Package Manager panel, consistent with how other panels use single-key shortcuts.
- **Effort:** Medium — requires fixing focus/event routing in the Package Manager widget and adding a palette command.
- **Discovered:** Run #16, 2026-05-31

---

### IMP-013 — clangd LSP Does Not Auto-Start Despite `enabled: true` in Config
- **Observed (Run #18):** After installing clangd and adding `{"lsp": {"c": {"command": "clangd", "args": [], "enabled": true}}}` to `~/.config/fresh/config.json`, Fresh shows `LSP (off)` on launch and the LSP Status popup shows "○ clangd (not running)". The user must manually click "Start clangd (always)" to start the server.
- **Correct behavior (per docs):** `docs/features/lsp.md` states "Install the server and Fresh will use it automatically" in the Built-in LSP Support table. The same doc lists C/C++ with clangd. If `enabled: true` is set, the server should auto-start when a matching file is opened.
- **Problem:** Users who configure clangd per the documentation and then open a `.c` file expect LSP to start automatically. Instead, they see `LSP (off)` with no indication of why or what to do. The LSP Status popup is not discoverable — users won't know to click on `LSP (off)` or run "Show LSP Status".
- **Possible cause:** Fresh may be treating `enabled: true` as "not disabled" rather than "auto-start"; the auto-start may require a project-level trust/consent step. Or the built-in config for C/clangd may have `enabled: false` by default, and the custom config's `enabled: true` conflicts rather than overrides.
- **Suggested fix:** Either (a) auto-start the LSP when `enabled: true` is in config and the binary is on PATH, or (b) show a more prominent notification: *"clangd is configured but not running. [Start LSP (Alt+L)] [Dismiss]"* in the status bar or as a notification banner.
- **Effort:** Low to Medium — behavioral change in LSP startup logic or UX notification.
- **Discovered:** Run #18, 2026-06-03
