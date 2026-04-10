# Review Diff Mode — Bug Report

**Date**: 2026-04-10
**Editor**: Fresh Editor (debug build, commit 9ab13b3)
**Test Environment**: 120x35 terminal via tmux, test git repo with staged, unstaged, deleted, and untracked files

---

## Test Setup

A test repository was created at `/tmp/test-repo` with the following git status:

| File | Status |
|------|--------|
| `config.toml` | Staged (modified) |
| `main.py` | Staged (modified) |
| `long_lines.txt` | Unstaged (modified, with very long lines) |
| `to_delete.txt` | Unstaged (deleted) |
| `utils.py` | Unstaged (modified) |
| `new_module.py` | Untracked (new file) |

---

## Discovered Keybinding Map (review-mode)

### Files Panel
| Key | Action | Status |
|-----|--------|--------|
| `Up`/`k` | Previous file | ✅ Works |
| `Down`/`j` | Next file | ✅ Works |
| `PageUp` | Page up in file list | ✅ Works |
| `PageDown` | Page down in file list | ✅ Works |
| `Home` | First file | ✅ Works |
| `End` | Last file | ✅ Works |
| `Tab` | Switch to diff panel | ✅ Works |
| `Enter` | Drill down to side-by-side | ✅ Works (with caveats, see Bug #3) |
| `s` | Stage file | ✅ Works |
| `u` | Unstage file | ✅ Works |
| `d` | Discard file | ✅ Works (confirmation dialog) |
| `c` | Add comment | ⚠️ Partial (see Bug #5) |
| `N` | Add/edit note | ✅ Works |
| `x` | Delete comment | ✅ Works |
| `r` | Refresh | ✅ Works |
| `e` | Export to markdown | ✅ Works |
| `q` | Close review diff | ✅ Works |

### Diff Panel
| Key | Action | Status |
|-----|--------|--------|
| `Up`/`Down` | Move cursor within diff | ✅ Works |
| `PageUp`/`PageDown` | Page scroll in diff | ✅ Works |
| `Home`/`End` | Top/bottom of diff | ✅ Works |
| `Tab` | Switch to files panel | ✅ Works |
| `n` | Next hunk | ⚠️ Partial (see Bug #4) |
| `p` | Previous hunk | ⚠️ Partial (see Bug #4) |
| `Enter` | Drill down to side-by-side | ✅ Works |
| `c` | Add line comment | ⚠️ Partial (see Bug #5) |

### Side-by-Side Diff View (diff-view mode)
| Key | Action | Status |
|-----|--------|--------|
| `Up`/`Down` | Move cursor | ⚠️ Status bar updates, viewport doesn't scroll (see Bug #6) |
| `n`/`]` | Next hunk | ✅ Works |
| `p`/`[` | Previous hunk | ✅ Works |
| `q` | Close and return | ✅ Works |

---

## Bug Reports

### Bug #1: Review Diff Does Not Auto-Focus When File Explorer Is Open (LOW SEVERITY — UX Discoverability)

**Description**: When the File Explorer sidebar is open (visible) and has focus, opening Review Diff does NOT automatically move focus to the Review Diff panels. The user must press `Ctrl+E` to switch focus from the File Explorer to the editor area (which contains the Review Diff). Without knowing about `Ctrl+E`, it appears that keyboard shortcuts are broken.

**Mitigating Factors**:
- `Ctrl+E` correctly switches focus between the File Explorer and the Review Diff panels. Once pressed, all review-mode keybindings work properly.
- There ARE visual indicators that the File Explorer has focus: the explorer border is bright white when focused vs. gray/dimmed when unfocused, a `▌` cursor is shown, and the header changes to `File Explorer (Ctrl+E)` when unfocused (hinting at how to re-focus it).

**The UX Issue**: The problem is discoverability. When a user opens Review Diff, they expect it to receive input immediately. Instead, keystrokes silently go to the File Explorer. The `(Ctrl+E)` hint only appears in the explorer header AFTER focus has been manually switched away — i.e., only after the user already knows the shortcut. There is no onscreen hint visible while the explorer is focused that tells the user how to switch away.

**Steps to Reproduce**:
1. Open the editor with the File Explorer sidebar visible and focused (default state)
2. Open Review Diff via `Ctrl+P` → "Review Diff"
3. Press `Down` arrow key — it moves the explorer cursor, not the review diff selection
4. Press `Ctrl+E` — focus shifts to Review Diff; now `Down` works correctly

**Suggestion**: Either (a) automatically switch focus to the Review Diff panels when the mode is activated, or (b) show the `(Ctrl+E)` hint in the explorer header at all times (not only when unfocused), so users can discover the shortcut.

---

### Bug #2: Terminal Resize Destroys Review Diff Layout (HIGH SEVERITY)

**Description**: Resizing the terminal window while in Review Diff mode causes catastrophic rendering corruption. The toolbar, header, separator, and most content disappear. Content spills past the status bar. The layout does not recover even after resizing back to the original size. Neither `r` (refresh) nor navigation keys restore the layout.

**Steps to Reproduce**:
1. Open Review Diff (with File Explorer closed for clean state)
2. Resize the tmux window: `tmux resize-window -t test -x 80 -y 24`
3. Observe the display
4. Resize back: `tmux resize-window -t test -x 120 -y 35`

**Expected**: The layout re-renders correctly at the new size (as per the design doc: "Listen to `resize` event to update viewportWidth/viewportHeight and re-render")  
**Actual**: 
- After resize down: Top portion of UI (menu bar, tabs, toolbar, header row) disappears. Only bottom portion of file list is visible.
- After resize back up: Layout remains broken. Content overlaps and spills past the status bar line.
- Pressing `r` (refresh) does NOT fix the layout.
- Pressing `Home` causes diff content to render BELOW the status bar.

**Evidence** (after resize back to 120x35):
```
 M  utils.py                       │
▸ Untracked                        │
>A  .review/                       │
 A  new_module.py                  │
                                   │
                                   │
[...22 empty lines...]
*files* [RO] | Discard cancelled
```
Menu bar, toolbar, header, horizontal separator, and diff content are all missing.

**Evidence** (after pressing Home — content below status bar):
```
*files* [RO] | Discard cancelled ...
                                     [logging]
                                    -level = "INFO"
                                    +level = "WARNING"
```

**Impact**: Users who resize their terminal while reviewing diffs lose the entire UI and must close and reopen the review diff.

---

### Bug #3: Side-by-Side Drill-Down Fails for Deleted Files (MEDIUM SEVERITY)

**Description**: When drilling down (`Enter`) into a deleted file (`to_delete.txt` with status `D`), the side-by-side view never opens. The status bar shows "Loading side-by-side diff..." indefinitely.

**Steps to Reproduce**:
1. Open Review Diff
2. Navigate to a deleted file (e.g., `to_delete.txt` with `D` status)
3. Press `Enter` to drill down

**Expected**: A side-by-side view opens showing the OLD content on the left and empty content on the right  
**Actual**: The status bar shows "Loading side-by-side diff..." and the view never opens. The warning counter increases (indicating a suppressed error).

**Evidence**:
```
>D  to_delete.txt                  │
...
*files* [RO] | Loading side-by-side diff...      LF UTF-8 text [⚠ 2]
```
Warning count increased from 1 to 2.

**Root Cause**: In `review_drill_down()` (line ~1666), the code calls `editor.readFile(absoluteFilePath)` for the new version. For a deleted file, the file doesn't exist on disk, so `readFile` returns `null`. The function exits early with `editor.setStatus(editor.t("status.failed_new_version"))`, but the status message may not be visible (it might be immediately overwritten or the loading message persists).

---

### Bug #4: Hunk Navigation (n/p) — Inconsistent Cursor Tracking (LOW-MEDIUM SEVERITY)

**Description**: When using `n`/`p` to jump between hunks in the diff panel, `jumpDiffCursorToRow()` calls `editor.setBufferCursor()` and `editor.scrollBufferToLine()`, but the `on_review_cursor_moved` callback may not fire consistently, causing `state.diffCursorRow` to become out of sync with the actual cursor position. The status bar sometimes shows the same line number after consecutive `n` presses.

**Steps to Reproduce**:
1. Switch to diff panel (`Tab`) on a file with multiple hunks (e.g., `main.py`)
2. Press `Home` to go to line 1
3. Press `n` — cursor jumps to a later hunk, view scrolls, status bar updates
4. Press `n` again — status bar shows the SAME line number as before, no visible movement

**Evidence**:
```
After first n: *diff* [RO] | Ln 5, Col 1
After second n: *diff* [RO] | Ln 5, Col 1  <-- unchanged
```

**Root Cause Hypothesis**: After `jumpDiffCursorToRow()` sets the cursor via `editor.setBufferCursor()`, the `on_review_cursor_moved` event may not fire if the buffer cursor position didn't actually change (byte offset might be the same due to the virtual buffer layout). Alternatively, `state.diffCursorRow` is being updated in `jumpDiffCursorToRow()` but the native cursor reports a different line.

---

### Bug #5: Comments Added from Files Panel (or Hunk Headers) Never Display Inline (MEDIUM SEVERITY)

**Description**: Comments added while the files panel is focused, or when the cursor is on a hunk header line in the diff panel, are stored with no `lineType`/`oldLine`/`newLine` (hunk-level comments). The `pushLineComments()` function (line 487) only matches comments that have specific `line_type` AND matching `old_line`/`new_line`, so hunk-level comments are never rendered inline in the diff view.

**Steps to Reproduce**:
1. With the files panel focused, press `c` to add a comment
2. The prompt shows "Comment on hunk:" (no line reference)
3. Type a comment and press Enter
4. Status bar says "Comment added to hunk"
5. Switch to diff panel — no inline comment is visible anywhere

Also reproducible from the diff panel: even when the cursor is on a specific added/removed line, the prompt still shows "Comment on hunk:" instead of "Comment on +N:" or "Comment on -N:", indicating `getCurrentLineInfo()` is falling back to the hunk-level path.

**Evidence**:
```
Comment on hunk:   <-- No line reference shown
...
*diff* [RO] | Ln 8, Col 1 | Comment added to hunk   <-- Added but invisible
```

The comments ARE stored (verified via export to `.review/session.md`) but never rendered inline.

**Root Cause**: `getCurrentLineInfo()` at line 1898 calls `readPropsAtCursor('diff')` which reads text properties from the diff buffer's native cursor position. When the files panel is focused, the diff cursor may not be on a line with `hunkId`/`lineType`/`oldLine`/`newLine` properties, so it falls back to `{ hunkId: hunk.id, file: hunk.file }` with no line-level info. Even when the diff panel is focused, the native cursor position may not map to a text property with the expected keys.

---

### Bug #6: Side-by-Side View — Down Arrow Doesn't Scroll Viewport (LOW SEVERITY)

**Description**: In the side-by-side diff view, pressing `Down` moves the native cursor (status bar updates from Ln 1 to Ln 10) but the viewport does not scroll to follow the cursor. The view stays frozen showing the same lines.

**Steps to Reproduce**:
1. Drill down into a file's side-by-side view (press `Enter`)
2. Press `Down` 10 times

**Expected**: The viewport scrolls to keep the cursor visible  
**Actual**: The status bar updates (Ln 1 → Ln 10) but the viewport stays at the top showing the same lines

**Evidence**:
```
Before: Ln 1, Col 1   | View shows lines 1-29
After 10 Down presses: Ln 10, Col 1  | View STILL shows lines 1-29
```

**Note**: The hunk navigation keys (`n`/`p`) DO scroll the viewport correctly in side-by-side view.

---

### Bug #7: Escape Key Not Mapped to Close Review Diff (LOW SEVERITY — UX Gap)

**Description**: The `Escape` key does nothing in review-mode. The design docs (review-diff-feature-restoration-plan.md, line 95) specify that both `q` and `Esc` should close the review diff, but only `q` is bound.

**Steps to Reproduce**:
1. Open Review Diff
2. Press `Escape`

**Expected**: Review diff closes (matching magit convention and the design spec)  
**Actual**: Nothing happens

**Design Reference**: `docs/internal/review-diff-feature-restoration-plan.md` line 95: `| q/Esc | Close review diff | Close review diff |`

**Code Evidence**: In `audit_mode.ts` line 2675: `["q", "close"]` — no `["Escape", "close"]` binding exists.

---

### Bug #8: Toolbar "Export" Label Truncated (COSMETIC)

**Description**: With the File Explorer sidebar open (narrower viewport for the review diff), the toolbar's last hint `e Export` is truncated to just `e E`. The space calculation doesn't properly account for the full label width.

**Evidence** (with File Explorer open):
```
s Stage  u Unstage  d Discard │ c Comment  N Note  x Del │ ↵ Open  Tab Switch  e
```
The `Export` label after `e` is cut off to just the first character or missing entirely.

**Evidence** (with File Explorer closed — toolbar shows correctly):
```
s Stage  u Unstage  d Discard │ c Comment  N Note  x Del │ ↵ Open  Tab Switch  e Export  r Refresh  q Close
```

---

## Summary

| # | Bug | Severity | Category |
|---|-----|----------|----------|
| 1 | Review Diff doesn't auto-focus when File Explorer is open | **LOW** | UX discoverability |
| 2 | Terminal resize destroys Review Diff layout | **HIGH** | Rendering/resize |
| 3 | Side-by-side drill-down fails for deleted files | **MEDIUM** | Edge case |
| 4 | Hunk navigation (n/p) — inconsistent cursor tracking | **LOW-MEDIUM** | Cursor sync |
| 5 | Comments from files panel never display inline | **MEDIUM** | Comment system |
| 6 | Side-by-side view — Down arrow doesn't scroll viewport | **LOW** | Scroll sync |
| 7 | Escape key not mapped to close | **LOW** | UX gap |
| 8 | Toolbar "Export" label truncated | **COSMETIC** | Rendering |

### Features That Work Well
- File list navigation (Up/Down/j/k/Home/End/PageUp/PageDown) — all correct with boundary clamping
- Tab switching between files and diff panels — robust even under rapid toggling
- Diff panel scrolling and cursor movement (Up/Down/PageUp/PageDown/Home/End)
- Side-by-side drill-down for normal files (modified, untracked) — displays correctly
- Closing side-by-side with `q` returns to review diff cleanly
- Stage (`s`), Unstage (`u`), Discard (`d`) operations — work correctly with proper confirmation dialogs
- Refresh (`r`) — re-queries git and updates display
- Export (`e`) — generates markdown review file
- Note (`N`) — adds a note section to the file list
- Diff coloring — added lines (green bg), removed lines (red bg), context (default), word-level highlighting on adjacent +/- pairs
- Section headers (▸ Staged, ▸ Changes, ▸ Untracked) properly organize files
