# Fresh Editor - Open Issues Review (Updated)

**Last Updated:** 2026-04-08
**Total Open Issues:** 137

---

## Changes Since Last Review (2026-04-03)

### Newly CLOSED
| # | Title |
|---|-------|
| #1438 | Language indicator broken for custom languages |
| #1454 | Fresh OOM (memory issue was in terminal emulators, not fresh) |
| #1490 | Memory leak (duplicate of #1454) |
| #1497 | Memory spike on folder open (duplicate of #1454) |
| #1434 | Crash saving file in non-existent directory |

### New Issues Opened (Apr 3-8)
| # | Title | Type | Labels |
|---|-------|------|--------|
| #1504 | Devcontainer support | Feature | — |
| #1503 | Review diff comments degradation in 0.2.22 | Bug | bug |
| #1502 | Word wrap squished/not right | Bug | bug |
| #1501 | Escape key to exit when no changes | Feature | — |
| #1468 | Move sidebar to right side | Feature | — |
| #1458 | Add Adwaita theme | Enhancement | enhancement |
| #1457 | Inherit theme colors from terminal | Feature | — |
| #1453 | Windows-1251 encoding support | Feature | — |
| #1447 | Open linked/imported file | Enhancement | enhancement |

---

## Current Top Priority Issues (Verified 2026-04-08)

### P0 — Regressions / Confirmed Bugs Blocking Daily Use

| # | Title | Labels | Comments | Notes |
|---|-------|--------|----------|-------|
| **#1503** | Review diff comments degradation in 0.2.22 | bug | 0 | **NEW regression** — diff workflow broken since latest release |
| **#1502** | Word wrap squished/not right | bug | 0 | **NEW** — rendering bug with screenshots |
| **#1342** | Search/Replace locks up with binary files | — | 2 | Editor hangs. Reporter has >10GB tar.gz files in project. |
| **#1425** | Auto-indent is too diligent | — | 3 | Auto-indent misbehaves after unindented lines |
| **#1428** | Shift+Home/End selection unresponsive | — | 2 | Basic text selection broken |

### P1 — Confirmed Bugs (Labeled or With Repro)

| # | Title | Labels | Comments | Notes |
|---|-------|--------|----------|-------|
| **#1407** | Keybinding menu items all have same description | bug | 0 | UX bug in keybinding editor |
| **#1388** | Hidden files overrides gitignore hiding | bug | 0 | Explorer shows gitignored files when hidden files shown |
| **#1386** | Mac version can't see plugins | bug | 6 | macOS-specific |
| **#1363** | Word wrap wraps before whitespace | — | 0 | Related to #1502 |
| **#1227** | Long Markdown links truncated on wrap | bug | 2 | Also wrap-related |
| **#1068** | Tab size always 8 | bug | 3 | Core fix landed, UX discoverability remains |
| **#899** | JavaScript syntax highlighting bug | bug | 1 | Upstream syntect issue |
| **#1115** | Package manager navigation issues | bug | 1 | Can't navigate/install packages |
| **#1118** | File Explorer width format mismatch | bug | 0 | Settings description wrong |
| **#699** | Find Previous with Shift-F3 doesn't work | bug | 6 | Basic search operation |
| **#780** | Copy/Paste broken on Putty | bug | 8 | Clipboard config may help |
| **#653** | Line numbers out of sync in large files | bug | 2 | Byte offsets shown as line numbers |
| **#692** | Hover dismissed on click inside | bug | 0 | |
| **#677** | Sometimes can't scroll to end | bug | 2 | Mouse-specific |

### P2 — Unlabeled Bugs / UX Issues

| # | Title | Comments | Notes |
|---|-------|----------|-------|
| **#1288** | Word select includes whitespace | 3 | Maintainer asked for details |
| **#1281** | Custom theme doesn't work | 2 | Theme editor also fails for reporter |
| **#1256** | Escaped chars in replace inserted raw | 0 | \n treated literally |
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
| **#959** | Respect .editorconfig | — | 0 |

### P4 — Feature Requests / Nice-to-Have

Remaining ~60 issues are feature requests, language support, packaging, and long-term roadmap items. Not listed individually.

---

## Word Wrap Bug Cluster

Three related issues around word wrapping:
- **#1502** — Word wrap squished/not right (new, bug label)
- **#1363** — Wrap should occur after whitespace, not before
- **#1227** — Long Markdown links truncated on wrap

These likely share a root cause and could be fixed together.

---

## Recommended Fix Order

**Sprint 1 — Regressions (fix immediately):**
1. **#1503** — Diff comments regression in 0.2.22 (regression = top priority)
2. **#1502** — Word wrap rendering bug
3. **#1342** — Project search hangs on binary files

**Sprint 2 — Core editing bugs:**
4. **#1425** — Auto-indent too diligent
5. **#1428** — Shift+Home/End selection
6. **#699** — Shift-F3 Find Previous
7. **#1288** — Word select includes whitespace
8. **#1256** — Replace string escape sequences

**Sprint 3 — Word wrap cluster:**
9. **#1363** — Wrap after whitespace
10. **#1227** — Markdown link truncation on wrap

**Sprint 4 — Polish:**
11. **#1388** — Hidden files vs gitignore precedence
12. **#1407** — Keybinding descriptions
13. **#1068** — Tab size UX
14. **#1115** — Package manager navigation
