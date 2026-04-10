# Review Diff Mode - UX Audit & Bug Report

**Auditor:** Automated TUI audit  
**Date:** 2026-04-10  
**Editor version:** Fresh 0.2.22 (debug build)  
**Test environment:** tmux 3.4, 160x45 terminal, Linux 4.4.0  
**Test repository:** Custom repo with staged, unstaged, untracked, deleted, renamed files, 500-char lines, and 200-line massive diffs.

---

## Executive Summary

The Review Diff mode is functionally usable for basic file navigation, staging/unstaging, and side-by-side diff viewing. However, **five bugs** were discovered, two of which are **critical** — they render entire feature surfaces non-functional. The root causes trace to focus management between the File Explorer sidebar and the buffer group, and to the `CompositeInputRouter` being implemented but never wired into the keyboard dispatch pipeline.

---

## Bug #1: File Explorer Sidebar Steals Focus from Review Diff Panels

**Severity:** High  
**Impact:** All review-mode keybindings silently fail when the File Explorer sidebar is visible and has focus (the default state on launch).

### User Goal
Navigate the review diff file list using `j`/`k` keys after opening Review Diff.

### Reproduction Steps
1. Open the editor with a File Explorer sidebar visible (default).
2. Open Command Palette (`Ctrl+P`), type "Review Diff", press Enter.
3. Wait for review diff to load (status bar shows "Review Diff: N hunks").
4. Press `j` to navigate down in the file list.

### Expected Behavior
The `j` key triggers `review_nav_down`, moving the file selection down in the review diff file list panel.

### Actual Behavior
The `j` key is captured by the File Explorer sidebar, triggering its quick-search feature. The File Explorer title bar changes to `/j`, and the review diff file list does not respond.

### Evidence (Before)
```
┌ File Explorer ─────────────────────────────×─┐ [No Name] ×   *Review Diff* ×
│▌ diff-test-repo                            ● │ s Stage  u Unstage  d Discard │ ...
│  > plugins                                 U │───────────────────────────────
│    ...                                       │ GIT STATUS
│    ...                                       │▸ Staged
│    ...                                       │>M  massive_diff.py              │-Original line 0: ...
*files* [RO] | Review Diff: 11 hunks
```

### Evidence (After pressing `j`)
```
┌ /j ────────────────────────────────────────×─┐ [No Name] ×   *Review Diff* ×
│▼ diff-test-repo                            ● │ s Stage  u Unstage  ...
```
Note the title changed to `/j` — the File Explorer intercepted the keystroke.

### Workaround
Press `Ctrl+Shift+E` to toggle focus from File Explorer to the editor area. After this, review-mode keys work correctly. Alternatively, close the File Explorer before opening Review Diff.

### Root Cause
`get_key_context()` in `app/input.rs:8-24` returns `self.key_context` which is `FileExplorer` when the sidebar has focus. Mode bindings are only checked when context is `KeyContext::Normal` (line 121-122). The review diff plugin's `start_review_diff()` does not explicitly move focus away from the File Explorer after creating the buffer group.

---

## Bug #2: `Escape` Key Does Not Exit File Explorer Focus to Review Diff

**Severity:** Medium  
**Impact:** Users trapped in File Explorer focus cannot escape back to the review diff using the intuitive `Escape` key.

### Reproduction Steps
1. With Review Diff open and File Explorer sidebar focused.
2. Press `Escape`.

### Expected Behavior
`Escape` should move focus from the File Explorer back to the review diff panel.

### Actual Behavior
`Escape` has no effect on focus state. The status bar continues to show "File explorer focused". Repeated `Escape` presses remain ineffective.

### Evidence
```
*files* [RO] | File explorer focused     (before Escape)
*files* [RO] | File explorer focused     (after Escape)
```

### Workaround
Use `Ctrl+Shift+E` instead.

---

## Bug #3: Composite Buffer (Side-by-Side Diff) Keyboard Navigation Non-Functional

**Severity:** Critical  
**Impact:** In the side-by-side diff view opened via `Enter` drill-down, **all vim-style navigation keys are broken**. Users cannot scroll with `j`/`k`, switch panes with `Tab`/`h`/`l`, navigate hunks with `n`/`p`, or close with `Escape`. Only arrow keys and `q` work.

### Reproduction Steps
1. Open Review Diff, select a file, press `Enter` to drill down.
2. Side-by-side diff view opens with OLD/NEW panes.
3. Press `j` to scroll down.

### Expected Behavior
The `diff-view` mode defines `j` → scroll down, `k` → scroll up, `Tab` → switch pane, `n`/`p` → hunk navigation, `Escape` → close.

### Actual Behavior
- `j` → "Editing disabled in this buffer" (treated as text insertion)
- `k` → "Editing disabled in this buffer"
- `Tab` → "Editing disabled in this buffer"
- `n` → "Editing disabled in this buffer"  
- `Escape` → No effect (view does not close)
- `g` → "Editing disabled in this buffer"
- Only `q` closes the view (via the `diff-view` mode's `["q", "close"]` binding)
- Arrow keys work (via standard editor navigation, not mode bindings)

### Evidence
```
*Diff: multi_hunk.txt* [RO] | Ln 1, Col 1 | Side-by-side diff: +4 -4 ~4 | 'q' to return

(After pressing j:)
*Diff: multi_hunk.txt* [RO] | Ln 1, Col 1 | Editing disabled in this buffer

(After pressing Escape:)
*Diff: multi_hunk.txt* [RO] | Ln 1, Col 1 | Editing disabled in this buffer
(View remains open — Escape did not close it)
```

### Root Cause
See Bug #4 below.

---

## Bug #4: `CompositeInputRouter` Is Dead Code — Never Wired Into Key Dispatch

**Severity:** Critical (Architectural)  
**Impact:** The entire `CompositeInputRouter` system (`crates/fresh-editor/src/input/composite_router.rs`) — which handles vim-style scrolling, pane switching, hunk navigation, visual selection, and yank for composite buffers — is **never called from the application's key dispatch pipeline**.

### Evidence

Searching `crates/fresh-editor/src/app/` for any call to `CompositeInputRouter::route_key_event` or `CompositeInputRouter` yields **zero results**. The router is only referenced in its own module and its unit tests.

The composite buffer action methods (`composite_scroll`, `composite_focus_next`, `composite_next_hunk`, etc.) in `composite_buffer_actions.rs` are only called from **scrollbar mouse handlers**, never from keyboard input processing.

### What Exists But Is Unused
- `CompositeInputRouter::route_key_event()` — routes `j`/`k` to scroll, `Tab` to pane switch, `n`/`p` to hunk nav, `v`/`V` to visual selection, `y` to yank, `q`/`Escape` to close
- `CompositeInputRouter::navigate_to_hunk()` — jumps viewport to next/prev hunk
- `CompositeInputRouter::display_to_source()` — coordinate translation
- `CompositeInputRouter::click_to_pane()` — mouse click pane detection
- All associated action types: `ScrollAction`, `CursorAction`, `SelectionAction`, `BufferAction`
- Full unit test suite (4 tests, all passing in isolation)

### What Should Happen
In `app/input.rs`, after mode binding resolution (around line 164), there should be a check: if the active buffer is a composite buffer (`is_composite_buffer(buffer_id)`), route the key event through `CompositeInputRouter::route_key_event()` and dispatch the resulting `RoutedEvent` to the appropriate `composite_*` methods.

---

## Bug #5: Hunk Navigation (`n`/`p`) Non-Functional in Review Diff's Diff Panel

**Severity:** High  
**Impact:** The `n` (next hunk) and `p` (previous hunk) keys in the review diff's diff panel do not move the cursor to hunk headers, despite the handler being invoked.

### Reproduction Steps
1. Open Review Diff, select a multi-hunk file (e.g., `multi_hunk.txt` with 4 hunks).
2. Press `Tab` to switch focus to the diff panel.
3. Verify toolbar shows `n Next  p Prev`.
4. Press `n` to jump to the next hunk.

### Expected Behavior
Cursor jumps from current position to the next `@@` hunk header line.

### Actual Behavior
Cursor stays at its current position. No error message. No visible change.

### Evidence
```
(Before pressing n — cursor at Ln 1:)
*diff* [RO] | Ln 1, Col 1 | Review Diff: 11 hunks

(After pressing n — cursor still at Ln 1:)
*diff* [RO] | Ln 1, Col 1 | Review Diff: 11 hunks

(After moving to Ln 14 with j, then pressing n — cursor still at Ln 14:)
*diff* [RO] | Ln 14, Col 1 | ...

(After pressing End to go to Ln 38, then pressing p — cursor still at Ln 38:)
*diff* [RO] | Ln 38, Col 1 | ...
```

### Confirmed Working
Other review-mode keys (`c` for comment, `s` for stage, `u` for unstage, `d` for discard) work correctly from the same panel, proving the mode bindings ARE resolving. The issue is isolated to the `review_next_hunk`/`review_prev_hunk` handler logic.

### Probable Root Cause
The `review_next_hunk()` handler at `audit_mode.ts:1818-1828` iterates `state.hunkHeaderRows` looking for `row > state.diffCursorRow`. Either:
1. `state.hunkHeaderRows` is empty (not populated for the current file), or
2. `state.diffCursorRow` is not being updated by the `cursor_moved` event handler (the event may not fire when cursor is moved via `editor.executeAction("move_down")`), or
3. The byte offsets in `state.diffLineByteOffsets` are stale after panel content changes, causing `jumpDiffCursorToRow()` to silently fail at the bounds check (line 1811: `if (idx < 0 || idx >= state.diffLineByteOffsets.length) return`).

---

## Features Verified Working

| Feature | Status | Notes |
|---------|--------|-------|
| File list navigation (`j`/`k`/`Up`/`Down`) | **Working** | After ensuring editor has focus (not File Explorer) |
| File list boundary clamping | **Working** | Pressing `k` past top or `j` past bottom is safely clamped |
| `Home`/`End` in file list | **Working** | Jumps to first/last file correctly |
| `PageUp`/`PageDown` in file list | **Working** | Steps by viewport height |
| `Tab` focus toggle (files ↔ diff) | **Working** | Toolbar updates to show panel-appropriate hints |
| `s` (stage file/hunk) | **Working** | Successfully stages files in git |
| `u` (unstage file/hunk) | **Working** | Successfully unstages files in git |
| `d` (discard) confirmation prompt | **Working** | Shows "Discard changes? This cannot be undone." with Discard/Cancel |
| `r` (refresh) | **Working** | Re-reads git status and rebuilds file list |
| `c` (comment) prompt | **Working** | Opens "Comment on hunk:" input prompt |
| `Enter` (drill-down to side-by-side) | **Working** | Opens composite side-by-side diff view |
| Side-by-side diff layout | **Working** | OLD/NEW panes with proper line alignment, filler lines |
| Side-by-side horizontal scrolling | **Working** | Arrow keys scroll both panes synchronously |
| Long line handling | **Working** | Lines truncated at panel edge; horizontal scroll reveals content |
| `q` (close review diff / side-by-side) | **Working** | Closes tab and returns to previous view |
| Rapid Tab toggling | **Working** | 10 rapid Tab presses don't crash or trap |
| Rapid `j`/`k` at boundaries | **Working** | No crash, selection properly clamped |

---

## Recommendations

1. **Bug #1 (High Priority):** In `start_review_diff()`, after creating the buffer group, explicitly focus the files panel and ensure `key_context` is set to `Normal`. Consider calling `editor.focusBufferGroupPanel(state.groupId, 'files')` followed by a mechanism to dismiss File Explorer focus.

2. **Bug #4 (Critical Priority):** Wire `CompositeInputRouter::route_key_event()` into the key dispatch pipeline in `app/input.rs`. When the active buffer is a composite buffer, intercept key events before they reach the standard text editing path. This will fix Bug #3 and enable all the already-implemented composite view features (vim scrolling, pane switching, hunk nav, visual selection, yank).

3. **Bug #5 (High Priority):** Add debug logging to `review_next_hunk()` to trace `state.hunkHeaderRows` and `state.diffCursorRow` values. Most likely, `hunkHeaderRows` is empty because `buildDiffPanelEntries()` is returning cached data that doesn't include hunk header rows for the unstaged diff path, or the `cursor_moved` event isn't firing for plugin-driven cursor movements.
