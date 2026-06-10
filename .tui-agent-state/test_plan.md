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

## RUN #23+ PRIORITY ORDER (coverage-first; work top-down)

> **Run #17 directive (permanent):** Focus on NEW coverage. Increase % of product tested. Avoid re-testing old passing features unless a bug was just fixed.
> **Build directive (Run #22):** Always build from **origin/master** — the state branch is frozen at v0.3.8.

1. ~~**Workspace Trust deep-dive**~~ — **DONE (Run #23).** Full 3-state matrix mapped: Restricted (git+ripgrep allowed, LSP gated off) / Block All (all processes denied: "workspace trust is set to Blocked — no processes may run"; git blame + live grep both blocked) / Trusted (clangd-lsp plugin ungated; actual start still governed by `auto_start`). Block All also restarts editor but preserves file. Only one palette command ("Workspace Trust…"); no direct block/restrict commands. #2291 CONFIRMED FIXED (file survives restart). UX note → IMP-015.
2. ~~**Orchestrator Dock (0.3.12)**~~ — **DONE (Run #24, PASS).** Alt+O dock; arrow live-switch (no restart, bidirectional); view card↔compact; project dropdown; `/` filter; Manage→full dialog; right-click menu (Visit/Archive/Delete) + Archive confirmation; New Session dialog ALL 4 types (Local/SSH/k8s/Devcontainer) with auto-detect + per-type reflow; Local worktree session created. No bugs. Avoided a false positive on keyboard Create-Session (works; was measurement error). Details → learning_db "Orchestrator Dock (Run #24)".
3. **Go to LSP Symbol (0.3.12)** — symbol finder with live preview (use clangd; `apt-get install clangd` works after `apt-get update`)
4. **#2197 pyright recheck** — still open; labeled in-progress. Re-test when a fix lands (pip install pyright)
5. **Rainbow bracket colorization (0.3.12)** — verify with ANSI capture on nested brackets
6. **Open file from diff (0.3.12)** — Enter in side-by-side/review-diff views opens working-tree/HEAD file
7. **Terminal tab auto-naming (0.3.12)** — `editor.terminal_auto_title`
8. **Keybinding editor count anomaly** — re-observe: total bindings differed between opens (866 vs 548) in Run #22; if reproducible with steps, file
9. ~~SSH scp-style~~ DONE (Run #22, PASS). ~~#2113~~ CLOSED not_planned. ~~Keybinding Delete/Record~~ DONE (Run #22, PASS). ~~Tour~~ DONE (Run #6).

Note: Run #24 COMPLETE. Orchestrator Dock (priority #2) COMPREHENSIVE PASS — Alt+O dock, arrow live-switch (no restart), view toggle, project dropdown, `/` filter, Manage→full dialog, right-click menu + Archive confirm, New Session dialog all 4 types, Local worktree session created. NO bugs filed; avoided a false positive on keyboard Create-Session button (verified it works before filing). tmux gotcha logged: use `BTab` not `S-Tab`. NEXT: priority #3 Go to LSP Symbol (clangd 18; `apt-get install clangd` then start via LSP menu, symbol finder w/ live preview), then #5 Rainbow brackets (ANSI capture on nested brackets), then #6 Open file from diff.

Note: Run #23 COMPLETE. Workspace Trust DEEP-DIVE done — 3-state enforcement matrix (Restricted/Block All/Trusted) fully mapped; enforcement works correctly per the dialog's own documented contract. #2291 CONFIRMED FIXED via UI (file survives trust restart; prior interrupted Run #23 already commented). Dialog now has explicit [OK]/[Quit] buttons + per-option enforcement descriptions. No new bug. IMP-015 logged (Blocked-mode tools fail with generic messages, not "blocked by trust"). NEXT: priority #2 Orchestrator Dock (Alt+O), then #3 Go to LSP Symbol (clangd 18 now installed), then #5 Rainbow brackets.

Note: Run #22 COMPLETE. #2165 + #2212 CONFIRMED FIXED in v0.3.12 (comments added). NEW BUG #2291 filed (Workspace Trust restart discards open file + unsaved edits with --no-restore). SSH scp-style END-TO-END PASS (real sshd). #2221 still broken. Keybinding Delete + Record Key Search PASS. #2122 still open. #2113 closed not_planned (retired).

Note: Run #21 COMPLETE. SSH URL-style BUG FOUND (#2221 filed): `ssh://` URI treated as local path; scp-style works correctly. Keybinding editor PASS (add/edit/save/verify all work). Search in selection: NOT IMPLEMENTED (no such toggle in search bar). Multi-root workspaces PASS (workspace scoping correct; cross-workspace files included in search when open). #2165 CONFIRMED STILL OPEN. #2113 NOT REPRODUCED (8 more attempts).

Note: Run #20 COMPLETE. text-actions Decode ALL PASS (Base64/URI/JSON/Hex; NEW decode commands discovered). Bookmarks ALL PASS (Alt+0-9). Keyboard macros PASS (5-action complex macro). Markdown compose PASS (bold/italic ANSI, code blocks with syntax-hl, editing inside blocks). #2212 CONFIRMED STILL OPEN in v0.3.10 (comment added).

Note: Run #19 COMPLETE. Code Actions BUG CONFIRMED + FILED (#2212): empty context.diagnostics always sent. Encoding handling TESTED (PASS — detect/reload/set/save all work). Themes TESTED (all 8 including new 'nord' PASS). Clangd auto-start RESOLVED: `auto_start` setting exists (default false); IMP-013 updated.

Note: Run #18 COMPLETE. clangd LSP TESTED (hover/def/complete/refs/rename all PASS; Code Actions INCONCLUSIVE). text-actions plugin TESTED (install + Base64 encoding PASS). Git Blame multi-commit TESTED (PASS — 'b' depth tracking confirmed). #2122 CONFIRMED still no keybinding. #2165 CONFIRMED still broken.

Note: Run #17 COMPLETE. File Explorer TESTED (PASS), Settings TextList [x] CONFIRMED mouse-only (Delete key = keyboard equivalent), pyright LSP BROKEN (#2197).
Note: Sprint 10 COMPLETE, Sprint 11 COMPLETE, Sprint 12 COMPLETE (TB01/TB02/TB03), Alt+A TESTED (PASS), Calibrate Keyboard TESTED, Block Selection TESTED (PASS Run #15), Flash:Jump TESTED (PASS Run #15), Package Manager TESTED (PASS Run #15+16), Live Diff TESTED (PASS Run #15+16), Live Grep Cycle Provider TESTED (PASS Run #15), Dev Container TESTED (PASS Run #15+16), Git Blame TESTED (PASS Run #16), Orchestrator TESTED (PASS Run #16), Color Highlighter TESTED (PASS Run #16), Review Diff CONFIRMED FIXED (Run #16, 0.3.10).

Note: Sprint 10 COMPLETE (T45/T46/T47/T48 all PASS), Sprint 11 COMPLETE (T28/T30/T37 all PASS), Sprint 12 COMPLETE (TB01/TB02/TB03), Alt+A TESTED (PASS), Calibrate Keyboard wizard TESTED (24 steps/5 groups; does NOT test Ctrl+H).

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
- [x] **T47** — Rapid keystrokes; verify no dropped input
  - PASS (Run #14): 50 rapid chars → all intact; 20 rapid Ctrl+Z → all undone correctly. No dropped input, no corruption.
- [x] **T48** — Resize tmux window while editing; verify layout reflow
  - PASS (Run #14): Resize 220×50 → 80×24 → 180×40 all reflow correctly. Status bar truncates gracefully at narrow width. Editor remains responsive. Resize mid-typing (100×30 → 160×45 during text entry) produced no corruption.

---

## Sprint 15: Plugin Features + Bug Rechecks — COMPLETED (Run #15)
- [x] ***Keyboard Shortcuts* 'q' recheck** — STILL BROKEN. "Editing disabled in this buffer" (#2125 related, still open)
- [x] **#2117 Review Diff discard hunk recheck** — STILL BROKEN. "Patch failed" error persists; manual git apply --reverse works fine (Fresh's internal apply is broken)
- [x] **Flash: Jump plugin** — PASS: Hint overlay activates; pressing hint character jumps cursor to target position
- [x] **Package Manager: Packages** — PASS: 13 packages listed, search filter works, category tabs work, detail panel shows install button
- [x] **Package Manager: Install from URL** — PASS: "Git URL or local path:" prompt appears
- [x] **Live Diff: vs HEAD** — PASS: Green `│` gutter markers on added lines, status bar confirms mode
- [x] **Live Diff: vs Disk** — PASS: `+` marker on unsaved buffer content, status bar confirms mode
- [x] **Live Diff: vs Branch...** — PASS: Branch prompt with "main" pre-filled, status "comparing against main"
- [x] **Live Grep: Cycle Provider** — PASS: Alt+P cycles git-grep → rg → grep; all providers work
- [x] **Block selection (Alt+Shift+Arrow)** — PASS: M-S-Down/M-S-Right work in this build (rectangular selection confirmed by typing test)
- [x] **Dev Container: Create Config** — PASS: Creates .devcontainer/devcontainer.json with Ubuntu template
- [x] **Dev Container: Show Info** — PASS: Shows config with action buttons; q closes correctly
- [x] **Dev Container: Show Features** — PASS: "No features configured" message
- [x] **Dev Container: Show Forwarded Ports** — PASS: Panel with "No configured or runtime ports" + q close

## Backlog (Future Runs)
- LSP features — tested in Run #11 with fake-pylsp; could test more LSP commands
- Plugin system testing (TypeScript plugins)
- Git integration features — tested partially; more edge cases possible
- Markdown preview — tested in Run #10; verify behavior with embedded code blocks
- Keyboard macros — tested in Run #10; verify complex multi-step macros
- Bookmarks — tested in Run #11; verify all bookmark slots (Alt+0-9)
- F10 reliability: Sometimes inserts `[21~]` escape sequence instead of opening menu (timing-dependent tmux issue)
- Block selection: CONFIRMED WORKING (Run #15) — M-S-Down/M-S-Right work
- Flash: Jump plugin: CONFIRMED WORKING (Run #15)
- Package manager: CONFIRMED WORKING (Run #15)
- Dev Container features: CONFIRMED WORKING (Run #15)
- Live Diff plugin: CONFIRMED WORKING (Run #15) — all modes tested
- Live Grep: Cycle Provider: CONFIRMED WORKING (Run #15)
- Git Blame plugin — "Show git blame for current file (magit-style)" — not yet tested
- Package: actually install a package and verify it becomes active
- Dev Container: Attach — test error handling when no docker binary
- text-actions plugin — install and test via Package Manager

## Sprint 13: Alt+A + Calibrate Keyboard + Bug Rechecks — COMPLETED (Run #14)
- [x] **T47** — PASS: Rapid keystrokes, no dropped input (50 chars burst, 20 rapid undos)
- [x] **T48** — PASS: Resize reflow works (220×50 ↔ 80×24 ↔ 180×40; mid-type resize safe)
- [x] **Alt+A** — PASS: Project-wide Search & Replace works: 9 matches/4 files, file scoping via Space, Replace All with confirmation dialog, status "Replaced 3 occurrences in 1 files"
- [x] **Calibrate Keyboard wizard** — TESTED: 24 steps/5 groups (Basic Editing, Line Navigation, Word Navigation, Document Navigation, Emacs-Style). Does NOT test Ctrl+H. s/b/g/a controls all work.
- [x] **#2125 partial** — Diagnostics panel 'q' CONFIRMED FIXED (commit 89caf72). *Keyboard Shortcuts* 'q' STILL BROKEN.
- [x] **#2112** — CONFIRMED FIXED (commit b7e7e64). Search/Replace panel now finds matches in /tmp files outside workspace root.

## Sprint 12: Bug Verification — COMPLETED (Run #13)
- [x] **TB01** — CONFIRMED: Keyboard Shortcuts buffer 'q' does not close
  - Same root cause as #2125 (Diagnostics panel). Comment added to #2125. Do NOT re-file.
- [x] **TB02** — CONFIRMED: Edit menu "Replace..." maps to Ctrl+Alt+R = Query Replace (interactive), not basic Replace (Ctrl+R)
  - Filed as new issue #2135. Do NOT re-file.
- [x] **TB03** — RESOLVED (NOT A BUG): Alt+W correctly closes tab in normal editing mode. Status shows "Tab closed".
  - Context-sensitive: Alt+W in search bar = toggle whole word; outside search = close tab. Expected behavior.
