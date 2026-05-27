# Fresh TUI Editor — Test Plan

> **Process:** Read `AGENT_INSTRUCTIONS.md` first. Filing standards, the
> pre-testing checklist, and the anti-drift rules (R1–R4) live there. In
> particular: **R1 — do not re-verify `[x]` sprints; R2 — advance a `[ ]` item
> every run.** Sprints 1–9 are DONE; work the priority order below, not them.

## Status Legend
- [ ] Not started
- [~] In progress
- [x] Completed
- [!] Blocked / Bug found

---

## RUN #13+ PRIORITY ORDER (work top-down; these are the unfound-bug frontier)

1. **Sprint 10 — Edge cases & stress** (T45 large file, T46 binary, T47 rapid
   input, T48 resize reflow). Deferred since Run #1; highest bug-yield.
2. **Project-wide Search & Replace (Alt+A)** — cross-file replace, untested.
3. **Sprint 11 — Navigation edge cases** (T28 bracket-match, T30 position
   history Alt+←/→, T37 line-wrap toggle).
4. **Calibrate Keyboard wizard** — does it detect the Ctrl+H/`0x08` case (#2109)?
5. **Bug verification** (Sprint 12: TB01–TB03) — batch low-severity findings into
   `potential_improvements.md` per R3, do not file individually.

Everything in "Backlog (Future Runs)" below feeds this order; promote items up
as they're picked.

---

## Sprint 1: Basic Launch & UI (Run 1 — 2026-05-26)
- [x] **T01** — Launch fresh with no arguments; verify welcome/empty state
  - PASS: Launches with menu bar, empty [No Name] buffer, status bar with hints
- [x] **T02** — Launch fresh with a file argument; verify file opens correctly
  - PASS: File opens as new tab, content visible, status bar shows filename
  - NOTE: Session persistence active — previous session buffers auto-restore on launch
- [x] **T03** — Verify menu bar is present and accessible
  - PASS: F10 opens File menu; Right/Left navigate; All 7 menus confirmed (File/Edit/View/Selection/Go/LSP/Help)
- [x] **T04** — Verify status bar shows file name, cursor position, mode
  - PASS: Status shows "Local | [filename] | Ln X, Col Y | [message] | encoding | filetype | hint"
- [x] **T05** — Verify title bar / window title updates with open file
  - PASS: Tab title updates with filename; asterisk (*) indicates unsaved changes
- [x] **T06** — Quit with Ctrl+Q; verify clean exit
  - PASS: Exits immediately without prompting (new unsaved buffer skipped)

## Sprint 2: File Operations
- [x] **T07** — Create new file (Ctrl+N); verify empty buffer opens
  - PASS: Ctrl+N opens new [No Name] tab with empty buffer
- [x] **T08** — Open existing file (Ctrl+O); verify file dialog appears
  - PASS: File picker opens with directory browser, file sizes, timestamps
- [x] **T09** — Type text into buffer; verify characters appear
  - PASS: Characters appear at cursor position
- [x] **T10** — Save file (Ctrl+S); verify save confirmation
  - PASS: New buffer → Save dialog with path. Status bar confirms "Saved as: [path]"
- [x] **T11** — Save As; verify prompts for new filename
  - PASS: "Save File As" in command palette opens dialog with current path pre-filled
- [x] **T12** — Close file (Alt+W or "Close Buffer"); verify tab closes, unsaved prompt if dirty
  - PASS: Close Buffer triggers `'[No Name]' modified. (s)ave, (d)iscard, (C)ancel?` prompt
  - NOTE: Dialog requires typing letter + Enter (not single keypress). Alt+W = "Close Tab" (different from "Close Buffer")
  - NOTE: Alt+W outside search bar toggles whole-word search setting, NOT close tab!
  - BUG CANDIDATE: Alt+W behavior inconsistent (sometimes closes tab, sometimes toggles search setting)

## Sprint 3: Editing Features
- [x] **T13** — Type text, use Ctrl+Z to undo; verify revert
  - PASS: Each Ctrl+Z undoes one character (char-by-char granularity)
- [x] **T14** — Undo, then Ctrl+Y to redo; verify restored
  - PASS: Ctrl+Y redoes last undone action
- [x] **T15** — Multi-cursor: Ctrl+D to add cursor at next match; type and verify simultaneous edits
  - PASS: Ctrl+W selects current word, then Ctrl+D adds cursor at next match; typing edits all cursors
- [x] **T16** — Block selection (Shift+arrows); verify selection
  - PASS: Shift+Down extends selection line by line; ANSI codes confirm highlight
- [x] **T17** — Copy (Ctrl+C), Paste (Ctrl+V); verify clipboard operations
  - PASS: Ctrl+C copies selection (with "Copied" status); Ctrl+V pastes into new buffer
- [x] **T18** — Cut (Ctrl+X); verify text removed and pasteable
  - PASS: Select line with Ctrl+L, then Ctrl+X removes and stores in clipboard
- [x] **T19** — Select All (Ctrl+A); verify entire buffer selected
  - PASS: All text highlighted (confirmed via ANSI)
- [x] **T20** — Comment/Uncomment line (Ctrl+/); verify toggle
  - PASS: Works on TOML/code files; no effect on plain .txt files (expected)

## Sprint 4: Search & Replace
- [x] **T21** — Open Find (Ctrl+F); verify search bar appears
  - PASS: Find bar at bottom with Case Sensitive/Whole Word/Regex options
- [x] **T22** — Type search query; verify incremental match highlighting
  - PASS: Matches highlighted in yellow (confirmed via ANSI codes)
- [x] **T23** — Navigate matches with F3; verify cursor jumps with "Match N of M"
  - PASS: F3 = next match, Shift+F3 = prev. Status shows "Match N of M". Wrap-around works.
  - NOTE: Enter CLOSES search dialog and jumps to match. F3 navigates without closing.
- [x] **T24** — Open Find & Replace (Ctrl+Alt+R); verify replace dialog
  - PASS: Query Replace dialog opens; options include Confirm Each (Alt+I)
- [x] **T25** — Replace one occurrence; verify correct substitution
  - PASS: Press 'y' to replace current match
- [x] **T26** — Replace all; verify all occurrences replaced
  - PASS: Press 'a' to replace all remaining. Previous 'n' skips are respected.

## Sprint 5: Navigation
- [x] **T27** — Go to line (Ctrl+G); verify line jump
  - PASS: Dialog at bottom, type line number + Enter
- [x] **T28** — Go to bracket/matching bracket; verify jump
  - PASS (Run #13): Command "Go to Matching Bracket" (Ctrl+]) works via command palette. '(' → ')' and '{' → '}' jumps confirmed. NOTE: Ctrl+] via tmux send-keys is unreliable (Ctrl+] = 0x1D, may not transmit correctly in tmux). Use command palette instead.
- [x] **T29** — Word movement (Ctrl+Left/Right); verify word-by-word nav
  - PASS: Ctrl+Right jumps to end of each word/token
- [x] **T30** — Position history (Alt+Left/Right); verify back/forward nav
  - PASS (Run #13): Alt+Left goes back in position history; Alt+Right goes forward. Tested with Ctrl+G jumps building history.

## Sprint 6: Command Palette
- [x] **T31** — Open command palette (Ctrl+P); verify fuzzy finder appears
  - PASS: Full command list with name, shortcut, description, source (builtin/plugin)
  - Modes: `file | >command | :line | #buffer`
- [x] **T32** — Type partial filename; verify filtered results
  - PASS: Delete '>' to switch to file mode; fuzzy search works across repository
- [x] **T33** — Select file from palette; verify file opens
  - PASS: Enter opens selected file as new tab

## Sprint 7: Views & Layout
- [x] **T35** — Split view (Split Horizontal via command palette); verify two panes visible
  - PASS: Horizontal split shows two panes with divider; "Split pane horizontally" status
- [x] **T36** — Toggle line numbers (via command palette "Toggle Line Numbers"); verify display change
  - PASS: Line number gutter shows/hides correctly
- [x] **T37** — Toggle line wrap; verify wrap behavior
  - PASS (Run #13): View menu → ☑ Line Wrap toggles. OFF = long line on 1 row (truncated). ON = long line wraps to multiple rows. Toggle is bidirectional.
- [x] **T38** — Switch theme (Select Theme → dracula); verify color changes
  - PASS: Theme picker shows dark/dracula/high-contrast/light/nostalgia/solarized-dark/terminal
  - PASS: Dracula theme applied; confirmed via ANSI color code changes
  - BONUS: File Explorer (Ctrl+B) opens directory sidebar — tested and confirmed

## Sprint 8: Tabs & Buffers
- [x] **T39** — Open multiple files; verify tabs appear
  - PASS: Multiple tabs visible simultaneously (tested with 4+ tabs)
- [x] **T40** — Click between tabs; verify buffer switch
  - PASS: Ctrl+PgDn/PgUp navigates between tabs
- [x] **T41** — Close one tab; verify others remain
  - PASS: Alt+W or "Close Tab" command closes individual tabs

## Sprint 9: Integrated Terminal
- [x] **T42** — Open terminal (via "Open Terminal in Utility Dock" Alt+`); verify terminal panel appears
  - PASS: Terminal opens in bottom dock split; shows shell prompt; "Terminal N opened" status
- [x] **T43** — Run a command in terminal; verify output
  - PASS: Commands execute and output displays correctly
- [x] **T44** — Close terminal; verify editor returns to full size
  - PASS: "Close Split" closes terminal dock; Ctrl+Space toggles terminal input mode

## Sprint 10: Edge Cases & Stress
- [x] **T45** — Open a large file; verify performance
  - PASS (Run #13): 49MB / 500K line file opened instantly. Navigation to end: immediate. Search "499999" found 2 matches in <2s. Gutter shows BYTE OFFSETS (not line numbers) for large files — Fresh's virtual view mode.
- [x] **T46** — Open a binary file; verify graceful handling
  - PASS (Run #13): /bin/ls opened gracefully. Tab shows [BIN] tag. Non-printable bytes shown as <XX> hex notation. No crash/freeze.
- [ ] **T47** — Rapid keystrokes; verify no dropped input
- [ ] **T48** — Resize tmux window while editing; verify layout reflow

---

## Backlog (Future Runs)
- T47: Rapid keystrokes stress test
- T48: Resize tmux window while editing; verify layout reflow
- LSP features — tested in Run #11 with fake-pylsp; could test more LSP commands
- Plugin system testing (TypeScript plugins)
- Git integration features — tested partially; more edge cases possible
- Markdown preview — tested in Run #10; verify behavior with embedded code blocks
- Keyboard macros — tested in Run #10; verify complex multi-step macros
- Bookmarks — tested in Run #11; verify all bookmark slots (Alt+0-9)
- F10 reliability: Sometimes inserts `[21~]` escape sequence instead of opening menu (timing-dependent tmux issue)
- Search and Replace in Project (Alt+A) — cross-file search
- Calibrate Keyboard wizard
- Block selection (Alt+Shift+Arrow) — tmux key sequence M-S-Down did NOT work in Run #12; need to find correct tmux key
- Flash: Jump plugin feature (visible in command palette — jumps to any visible match in any split)
- Package manager (seen in command palette: "Package: Packages", "Package: Install from URL")
- Dev Container features (seen in command palette)
- Live Diff plugin: vs Default Branch / vs Disk — test both modes
- Live Grep: Cycle Provider — test different backend providers

## Sprint 12: Bug Verification — COMPLETED (Run #13)
- [x] **TB01** — CONFIRMED: Keyboard Shortcuts buffer 'q' does not close
  - Same root cause as #2125 (Diagnostics panel). Comment added to #2125. Do NOT re-file.
- [x] **TB02** — CONFIRMED: Edit menu "Replace..." maps to Ctrl+Alt+R = Query Replace (interactive), not basic Replace (Ctrl+R)
  - Filed as new issue #2135. Do NOT re-file.
- [x] **TB03** — RESOLVED (NOT A BUG): Alt+W correctly closes tab in normal editing mode. Status shows "Tab closed".
  - Context-sensitive: Alt+W in search bar = toggle whole word; outside search = close tab. Expected behavior.
