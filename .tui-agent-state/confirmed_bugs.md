# Confirmed Bugs Registry

## Format
Each bug entry:
- **ID:** BUG-NNN
- **Title:** Short description
- **Severity:** Critical / High / Medium / Low
- **Status:** Open / Fixed / Closed
- **GitHub Issue:** #NNN (if filed)
- **Reproduction Steps:** (tmux send-keys sequence)
- **Expected:** What should happen
- **Actual:** What happened (from tmux capture-pane)
- **First Seen:** Date of first occurrence

---

_No formally confirmed bugs yet. Bug candidates from Run #12 below. See session_log.md for earlier bug candidates BC-01 and BC-02. See github_issues.md for filed issues._

---

## Run #12 Bug Candidates (2026-05-27) — Pending GitHub Search Before Filing

### RC12-01: Keyboard Shortcuts Buffer — 'q' Does Not Close Buffer Despite Documentation
- **Severity:** Low (Documentation/UX)
- **Status:** Candidate — needs GitHub search before filing
- **Reproduction:**
  1. `tmux send-keys -t fresh-test '/home/user/fresh/target/release/fresh' Enter` (wait 2s)
  2. `tmux send-keys -t fresh-test 'S-F1'` (wait 2s)
  3. Verify buffer opens with header: "Keyboard Shortcuts"
  4. Verify line 4 reads: "Press 'q' to close this buffer."
  5. `tmux send-keys -t fresh-test 'q'`
  6. `tmux capture-pane -t fresh-test -p | tail -3`
- **Expected:** Buffer closes, returns to previous buffer
- **Actual:** Status bar shows "Editing disabled in this buffer"; buffer stays open
- **Workaround:** Use `Alt+W` to close
- **Reproduction count:** 2 (same run, same result both times)
- **Pre-file checklist:**
  - [ ] Search GitHub: "keyboard shortcuts q close"
  - [ ] Search GitHub: "'q' to close buffer"
  - [ ] Search GitHub: "read only buffer q"
  - [ ] If no match: file with label `tui-agent-auto-bug`

### RC12-02: Edit Menu Shows "Replace..." with Ctrl+Alt+R (Which Invokes Query Replace, Not Basic Replace)
- **Severity:** Low (Documentation/UX)
- **Status:** Candidate — may be by design; needs investigation
- **Reproduction:**
  1. Launch Fresh, open any file
  2. Press `F10` to open menu bar → navigate to "Edit" menu with `Right`
  3. Scroll to "Replace..." — note shortcut shown: `Ctrl+Alt+R`
  4. Press `Escape`, then open Command Palette (`Ctrl+P`)
  5. Search for "Replace" — observe two distinct commands:
     - "Replace" = `Ctrl+R` (no confirm-each)
     - "Query Replace" = `Ctrl+Alt+R` (interactive, with confirm-each)
  6. Press `Ctrl+Alt+R` directly — note "Confirm each" checkbox is checked
- **Expected:** Edit menu "Replace..." should either show `Ctrl+R` (basic Replace) or be labeled "Query Replace..."
- **Actual:** Edit menu shows "Replace..." mapped to `Ctrl+Alt+R` which invokes Query Replace mode (interactive), not the basic Replace. Basic Replace (`Ctrl+R`) has no Edit menu entry.
- **Assessment:** Could be intentional (devs prefer query replace as primary) or a documentation inconsistency
- **Pre-file checklist:**
  - [ ] Search GitHub: "Edit menu Replace shortcut"
  - [ ] Search GitHub: "Replace Ctrl+R menu"
  - [ ] If no match AND confirmed as unintentional: file with `tui-agent-auto-bug`
