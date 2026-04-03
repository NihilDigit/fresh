# Fresh Editor - Open Issues Review (Updated)

**Last Updated:** 2026-04-03
**Previous Review:** 2026-03-22

---

## Changes Since Last Review

### Issues Now CLOSED (18 total resolved since 2026-03-22)

| # | Title | Resolution |
|---|-------|-----------|
| #1434 | Crash saving file in non-existent directory | Fixed (bug) |
| #1362 | Panic at render.rs:622 on /mnt subdirs | Fixed with 11 regression tests |
| #1113 | CSI u sequences written as text in session mode | Fixed |
| #1263 | Comments displace inlay hints | Fixed |
| #1254 | Auto-Surround requires Auto-Close | Fixed (decoupled in 0.2.12) |
| #1073 | Highlight current cursor line | Implemented |
| #868 | Buffer-based autocompletion | Implemented |
| #1371 | Line wrap at ruler for markdown | Fixed |
| #1290 | Tilde characters on empty lines configurable | Fixed |
| #889 | Regex search/replace question | Addressed (regex + capture groups in 0.2.2) |
| #1054 | Can't type chars with Python syntax (Windows) | Fixed in 0.2.5 |
| #1255 | Hidden cursor in Zellij | Fixed |
| #431 | Auto-indent staircase on paste | Fixed (closed Mar 24) |
| #1312 | Keybinding editor + plugin actions | Fixed |
| #1305 | Find next/previous cursor position loss | Fixed |
| #1304 | Indent/unindent extra lines not in scope | Fixed |
| #1303 | Can't delete entry from keybinding editor | Fixed |
| #1317 | Basic text select problem | Fixed |

### NEW Issues Opened (Since Mar 22)

| # | Title | Type | Date |
|---|-------|------|------|
| #1445 | `fresh --cmd config paths` | Question | Apr 1 |
| #1438 | Language indicator broken for custom languages | Bug | Mar 31 |
| #1436 | Search editor like VSCode occur/wgrep | Feature | Mar 31 |
| #1434 | Crash saving to non-existent dir | Bug (CLOSED) | Mar 31 |
| #1433 | Chromebook keybind detection | Feature | Mar 31 |
| #1430 | Highlight current cursor column | Enhancement | Mar 31 |
| #1428 | Shift+Home/End selection unresponsive | Bug | Mar 30 |
| #1425 | Auto-indent is too diligent | Bug | Mar 30 |
| #1407 | Keybinding menu items don't describe changes | Bug | Mar 29 |
| #1406 | Compact/inline single child directory | Enhancement | Mar 29 |
| #1404 | Config for auto-reopening files | Enhancement | Mar 29 |
| #1403 | Auto-close single-click explorer files | Enhancement | Mar 29 |

---

## Current Top Priority Issues (Verified Open as of 2026-04-03)

### P0 — Hangs / Severe Bugs

| # | Title | Comments | Labels | Notes |
|---|-------|----------|--------|-------|
| **#1342** | Search/Replace locks up with binary files | 2 | — | Editor freezes completely. Must skip binary files. |
| **#1425** | Auto-indent is too diligent | 3 | — | Auto-indent fires incorrectly in various contexts. |
| **#1428** | Shift+Home/End selection unresponsive | 2 | — | Basic text selection broken. |

### P1 — Confirmed Bugs (Labeled)

| # | Title | Comments | Labels | Notes |
|---|-------|----------|--------|-------|
| **#1438** | Language indicator broken for custom languages | 0 | bug | NEW. 1 reaction. |
| **#1407** | Keybinding menu items don't describe what they change | 0 | bug | NEW. Settings UX. |
| **#1068** | Tab size always 8 | 3 | bug | Core bug fixed in 0.2.9, UX discoverability remains. |
| **#1012** | Scrollbar flashing | 5 | bug | Confirmed reproducible, no fix. |
| **#899** | JavaScript syntax highlighting bug | 1 | bug | Upstream syntect bug. 1 reaction. |
| **#1128** | Key binding issue with * | 2 | bug | Terminal limitation. |
| **#1118** | File Explorer width format mismatch | 0 | bug | Settings says 0.0-1.0 but value is integer. |
| **#1115** | Package manager navigation issues | 1 | bug | Can't navigate/install packages. |
| **#1112** | Settings UI mouse/keyboard editing issues | 0 | bug | Mouse offset wrong in scrolled lists. 1 reaction. |
| **#722** | LSP inlay hints sometimes rendered in wrong place | 0 | bug | May be partially fixed with #1263 closure. |
| **#780** | Copy/Paste broken on Putty (Windows) | 8 | bug | Clipboard config (0.2.4) may provide workaround. |

### P2 — Open Bugs (Unlabeled but Confirmed)

| # | Title | Comments | Notes |
|---|-------|----------|-------|
| **#1288** | Word select includes whitespace | 3 | Maintainer asked for details. |
| **#1281** | Custom theme doesn't work | 2 | Reporter says theme editor also fails. |
| **#1256** | Escaped chars in replace inserted raw | 0 | `\n` in replace string treated literally. |
| **#1363** | Word wrap wraps before whitespace not after | 0 | NEW. |
| **#1245** | Cannot select text in terminal | 1 | SSH terminal text selection. |
| **#1227** | Long Markdown links truncated on wrap | 2 | Ctrl-click broken on wrapped links. |
| **#1157** | Theme editor selection bg uses terminal bg | 2 | Wrong default color. |
| **#611** | File switch leaves buffer empty until scroll | 1 | Intermittent, could not reproduce. |
| **#677** | Sometimes can't scroll to end | 2 | Mouse-specific, could not reproduce. |

### P3 — High-Demand Enhancements

| # | Title | Reactions | Comments | Notes |
|---|-------|-----------|----------|-------|
| **#1036** | MacOS keybindings should use Cmd | 3 | 1 | Affects all Mac users |
| **#779** | Lines after EOF in different shade | 3 | 0 | Common editor feature |
| **#959** | Respect .editorconfig | — | — | Industry standard |
| **#867** | Keybinding editing UX | 2 | 1 | Power user need |
| **#878** | Move file functionality | 2 | 1 | Basic file management |
| **#971** | Multiple LSP servers per language | 1 | 3 | Python: ruff + pyright |
| **#229** | Diff view | — | — | Essential for code review |
| **#950** | File outline sidebar / markdown TOC | — | 3 | Navigation aid |
| **#1340** | Search/Replace improvements | 0 | 0 | NEW. UX improvements. |
| **#1341** | Project-wide search improvements | 0 | 0 | NEW. UX improvements. |

---

## Recommended Fix Order

**Sprint 1 — Stop the bleeding:**
1. **#1342** — Skip binary files in project search (prevents hangs)
2. **#1428** — Fix Shift+Home/End selection
3. **#1425** — Rein in auto-indent

**Sprint 2 — Bug cleanup:**
4. **#1438** — Language indicator for custom languages
5. **#1288** — Word select trailing whitespace
6. **#1256** — Support escape sequences in replace strings
7. **#1118** — Fix explorer width settings description
8. **#1115** — Package manager navigation

**Sprint 3 — Polish:**
9. **#1012** — Scrollbar flashing
10. **#1112** — Settings UI mouse offset
11. **#1281** — Custom theme loading
12. **#1068** — Tab size UX (show per-language override in settings)
