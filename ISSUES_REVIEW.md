# Fresh Editor - Open Issues Review (Updated)

**Last Updated:** 2026-04-09
**Previous Review:** 2026-04-08

---

## Changes Since Last Review (24 hours)

### Newly CLOSED

| # | Title | Notes |
|---|-------|-------|
| **#1502** | Word wrap squished | Fixed same day — fast turnaround |
| **#1012** | Scrollbar flashing | Finally fixed after months open |
| #1501 | Escape key to exit | Closed (likely rejected/wontfix) |
| #1469 | Editing disabled in buffer | Fixed |
| #1466 | Cursor position feedback during Find Next | Fixed |
| #1452 | Review diff not showing added file | Fixed |
| #1459 | AutoHotkey | Closed |
| #1219 | Default/fallback grammar | Fixed |

### Newly Opened

| # | Title | Labels |
|---|-------|--------|
| **#1511** | Customizable status bar | — |
| **#1509** | Allow double-escape as bindable key | enhancement |

---

## Current Top Priority Issues

### P0 — Regressions / Critical Bugs

| # | Title | Labels | Comments | Notes |
|---|-------|--------|----------|-------|
| **#1503** | Review diff comments degradation in 0.2.22 | bug | 2 | **Only remaining regression** from v0.2.22 |
| **#1342** | Search/Replace locks up with binary files | — | 2 | Editor hangs on >10GB binary files in project |
| **#1386** | Mac version can't see plugins | bug | 6 | macOS-specific, 6 comments |
| **#1425** | Auto-indent is too diligent | — | 3 | Core editing annoyance |
| **#1428** | Shift+Home/End selection unresponsive | — | 2 | Basic text selection broken |

### P1 — Confirmed Bugs (Labeled)

| # | Title | Labels | Comments | Notes |
|---|-------|--------|----------|-------|
| **#699** | Find Previous with Shift-F3 doesn't work | bug | 6 | Basic search operation |
| **#780** | Copy/Paste broken on Putty | bug | 8 | Clipboard config workaround |
| **#1068** | Tab size always 8 | bug | 3 | Core fix landed, UX discoverability remains |
| **#899** | JavaScript syntax highlighting bug | bug | 1 | Upstream syntect issue |
| **#1115** | Package manager navigation issues | bug | 1 | Can't navigate/install packages |
| **#1407** | Keybinding menu items same description | bug | 0 | UX bug |
| **#1388** | Hidden files overrides gitignore | bug | 0 | Explorer filter bug |
| **#1227** | Long Markdown links truncated on wrap | bug | 2 | Wrap-related |
| **#1118** | File Explorer width format mismatch | bug | 0 | Settings description wrong |
| **#1112** | Settings UI mouse/keyboard editing issues | bug | 0 | 1 reaction |
| **#653** | Line numbers out of sync in large files | bug | 2 | Byte offsets shown as line numbers |
| **#677** | Sometimes can't scroll to end | bug | 2 | Mouse-specific |
| **#722** | LSP inlay hints wrong place | bug | 0 | May be partially fixed |
| **#727** | Delete/move word broken | bug | 2 | |
| **#692** | Hover dismissed on click inside | bug | 0 | |

### P2 — Unlabeled Bugs / UX Issues

| # | Title | Comments | Notes |
|---|-------|----------|-------|
| **#1363** | Word wrap wraps before whitespace | 0 | Related to closed #1502 |
| **#1288** | Word select includes whitespace | 3 | Maintainer asked for details |
| **#1281** | Custom theme doesn't work | 2 | Theme editor also fails |
| **#1256** | Escaped chars in replace inserted raw | 0 | `\n` treated literally |
| **#1245** | Cannot select text in terminal | 1 | SSH terminal |
| **#1157** | Theme editor selection bg wrong color | 2 | |
| **#611** | File switch leaves buffer empty | 1 | Intermittent |

### P3 — High-Demand Enhancements (by reactions)

| # | Title | Reactions | Comments |
|---|-------|-----------|----------|
| **#465** | Add Winget release action | 5 | 3 |
| **#1036** | MacOS keybindings should use Cmd | 4 | 1 |
| **#1051** | Rainbow brackets | 3 | 1 |
| **#966** | Svelte syntax highlighting | 3 | 1 |
| **#779** | Lines after EOF in different shade | 3 | 0 |
| **#478** | Neovim plugin compatibility layer | 3 | 0 |
| **#229** | Diff view (side-by-side/unified) | 2 | 4 |
| **#878** | Move file functionality | 2 | 1 |
| **#867** | Keybinding editing UX | 2 | 1 |
| **#716** | Move line up/down | 2 | 1 |
| **#686** | Ghost text (Copilot) support | 2 | 1 |
| **#1211** | Expand/shrink selection | 2 | 0 |
| **#1111** | Support VSIX | 2 | 0 |
| **#1086** | Persistent Vi Mode | 2 | 0 |

---

## Recommended Fix Order

**Sprint 1 — Regressions (fix immediately):**
1. **#1503** — Diff comments regression in 0.2.22 (only remaining regression)
2. **#1342** — Project search hangs on binary files

**Sprint 2 — Core editing bugs:**
3. **#1425** — Auto-indent too diligent
4. **#1428** — Shift+Home/End selection
5. **#699** — Shift-F3 Find Previous (6 comments, long-standing)
6. **#1288** — Word select includes whitespace
7. **#1256** — Replace string escape sequences

**Sprint 3 — Polish:**
8. **#1386** — Mac plugins visibility
9. **#1388** — Hidden files vs gitignore precedence
10. **#1407** — Keybinding descriptions
11. **#1068** — Tab size UX
12. **#1115** — Package manager navigation

---

## Notable Trend

The maintainer has been very active — **8+ issues closed in the past 24 hours**, including long-standing ones like #1012 (scrollbar flashing, months old) and same-day fix for #1502 (word wrap squished). The editor is rapidly improving toward daily-driver reliability.
