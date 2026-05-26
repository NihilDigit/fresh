# Confirmed Bugs Registry

---

## Open Bugs

### BUG-002 - Ctrl+H Does Not Open Find & Replace in Terminals
- **Date:** 2026-05-26
- **Severity:** Medium (UX friction for VS Code/Sublime users)
- **Root Cause:** Terminal compatibility — `Ctrl+H` is transmitted as ASCII `0x08` (Backspace) in most terminals including tmux. Fresh receives a backspace, not Ctrl+H.
- **GitHub Issue:** https://github.com/sinelaw/fresh/issues/2109 (updated)
- **Workaround:** Use `Ctrl+R` for Replace (reliable in all terminals).
- **Notes:** Fresh's "Calibrate Keyboard" wizard exists for exactly this class of issue. The `Ctrl+H` → Backspace collision is particularly harmful because it silently deletes text.

---

### BUG-004 - F3 Does Not Navigate While Search Bar Is Open
- **Date:** 2026-05-26
- **Severity:** Medium (UX friction — contradicts VS Code/Sublime/browser behavior)
- **Description:** Pressing F3 while the search bar is open has no effect. The current workflow requires pressing Enter first (which closes the bar), then F3 to navigate. In VS Code, Sublime Text, and all major browsers, F3 navigates to the next match while the search bar remains visible. This is the dominant paradigm Fresh users will expect.
- **Expected:** F3 advances to next match without closing the search bar
- **Actual:** F3 is silently ignored while search bar is open; must press Enter to close first
- **Workaround:** Enter → closes bar, then F3 works (but this is non-obvious and nothing in the UI explains it)
- **GitHub Issue:** https://github.com/sinelaw/fresh/issues/2111 (updated with clear expected vs actual)
- **Status:** Confirmed usability bug; no further verification needed

---

## Closed / Retracted

### ~~BUG-001~~ — FALSE POSITIVE (closed #2108)
- We accidentally triggered "Reload with Encoding..." instead of "Revert" via imprecise menu navigation.
- `File > Revert` works correctly: it shows a `(r)evert / (c)ancel` confirmation prompt when the buffer is modified.
- **Lesson:** Always verify menu selection with ANSI capture before asserting behavior.

### ~~BUG-003~~ — BY DESIGN (closed #2110)
- File opening as modified is the **hot exit** feature: Fresh preserves all unsaved buffer state on quit and restores it on the next startup (`hot_exit` config, default: on).
- **Lesson:** Read the docs and CHANGELOG before filing issues. Hot exit is documented in `docs/features/session-persistence.md` and announced in the 0.2.18 release.

---

### BUG-005 - Search/Replace Panel Returns "No Matches Found" for Files Outside Git Workspace
- **Date:** 2026-05-26
- **Severity:** High (silently returns wrong result with misleading UI)
- **GitHub Issue:** https://github.com/sinelaw/fresh/issues/2112 (filed Run #2)
- **Status:** Open
- **Root Cause:** The search/replace backend appears to only index/search within the git project root. Files opened from outside (e.g. `/tmp`) are not searchable.
- **Reproduction:**
  1. Open a file from `/tmp` (not in the git workspace): `fresh --no-restore /tmp/sample.txt`
  2. Open Search/Replace panel: `Ctrl+P → "Search and Replace in Current File"`
  3. Type any text that exists in the file, press Tab
  4. After ~3s: panel shows "No matches found" despite the text existing
- **Key detail:** The "Only in: /tmp/sample.txt" label in the panel misleads the user into expecting file-scoped search, but it silently fails. In-project files work correctly.
- **Inconsistent UI state:** The Matches area permanently shows "Searching…" while the status bar shows "No matches found" — these contradict each other.
- **Workaround:** Copy the file into the project directory before using Search/Replace.
- **Confirmed reproduced:** Yes, twice in the same session.

---

### BUG-006 - Command Palette Keystrokes Leak into Editor Buffer When Switching to File Mode
- **Date:** 2026-05-26
- **Severity:** Medium (silent data corruption — user may not notice immediately)
- **GitHub Issue:** https://github.com/sinelaw/fresh/issues/2113 (filed Run #2)
- **Status:** Open (but not reproduced in Run #3)
- **Root Cause:** Likely a focus/input-routing race condition during command palette mode transition.
- **Reproduction:**
  1. Open command palette: `Ctrl+P` (opens in command mode with `>` prefix)
  2. Clear the `>` by pressing BSpace 3-5 times (switches to file/fuzzy-finder mode)
  3. Type a fuzzy search query (e.g. "cargo") to find project files
  4. Select a file with Up + Enter
  5. Return to original buffer — it now contains characters from the search query (e.g. "Cargo" inserted into file content)
- **Run #3 Status:** NOT REPRODUCED in 2 fresh attempts. Possible the bug was fixed in the current build, or is highly timing-dependent. Continue monitoring in future runs.
- **Workaround:** After using the fuzzy file finder, check the source buffer and Ctrl+Z if characters were leaked.

---

## Resolved Bugs

*(None yet)*
