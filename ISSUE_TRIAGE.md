# Fresh Editor - Open Issue Triage

**Date:** 2026-03-06 (updated from 2026-02-26, originally 2026-02-06)
**Total open issues (excluding PRs):** ~108
**Focus:** Low-complexity actionable items, duplicates, fixes vs. debatable changes

---

## Changes Since Last Triage (2026-02-26)

### Progress: 10 previously triaged issues now closed

**Closed from our Tier 1 bug list:**
- ~~#1114~~ Cursor bleeds through dropdowns — **FIXED** (was our #1 priority)
- ~~#1121~~ Can't scroll buffer with open-file panel — **FIXED** (was #2)
- ~~#1039~~ Comment delimiters wrong color — **FIXED** (was #4)
- ~~#1119~~ Settings descriptions truncated — **FIXED** (was #7)

**Closed from our Tier 2 bug list:**
- ~~#851~~ Blinking bar cursor only at EOL — **FIXED**

**Closed from our enhancement list:**
- ~~#1081~~ Support `path:line:column` in open file — **FIXED** (was #1 enhancement)

**Closed from other categories:**
- ~~#900~~ Code folding — **IMPLEMENTED**
- ~~#1122~~ Big files can't fold — **FIXED** (related to #900)
- ~~#1117~~ Markdown handling on language change — **FIXED**
- ~~#1116~~ Markdown de-indent bullet cycling — **FIXED**
- ~~#700~~ Testing macros — **FIXED**

**Correction from previous triage:** #716 (move line up/down) is still **OPEN**, not closed as previously stated.

**New issues filed since last triage:** 9 new issues (#1128-#1202)

---

## Duplicate / Overlapping Issues

| Group | Issues | Recommendation |
|-------|--------|----------------|
| Whitespace visibility | #893 + #664 | Merge. Both request configurable whitespace rendering. |
| WASM version | #534 + PR #596 | Close #534. Design doc exists in `docs/wasm.md`. |
| Diff view | #229 + #197 + #432 | #229 is umbrella. Link others as sub-tasks. |
| Mac keybindings | #1036 + #727 | Combine into a Mac keymap audit. #482 now closed. |
| Ignored/flaky tests | #184 + #330 | Keep separate, cross-reference. |
| PuTTY issues | #780 (copy/paste) + #1023 (delete char/word) | Both terminal escape sequence handling. May share root cause. |
| Column highlight / viewport | #1073 (highlight cursor column) + #779 (lines after EOF) | Both viewport rendering enhancements. |
| Deno / multi-LSP | #1191 (Deno LSP support) + #971 (multiple LSPs per language) | #1191 is a specific case of #971. Address together. |
| C# naming | #1158 (csharp/c_sharp/C# confusion) + language config consistency | Standalone naming/config cleanup. |
| Clipboard / OSC 52 | #1145 (OSC 52 non-functional) + #477 (macOS SSH clipboard) | Both clipboard issues. #1145 is about OSC 52 escape sequences not being sent. #477 is about Cmd+C over SSH. Related root cause. |

---

## Low Complexity - Recommended Fixes (Bugs)

### Tier 1: Highest confidence, smallest scope

| # | Title | Why it's low complexity | Status |
|---|-------|------------------------|--------|
| **#938** | Go to line shows wrong line numbers in large files | Indexing bug in `:` command palette prefix. Lines 1-2 correct, 3+ increasingly wrong. Reproducible in 100K+ line files. Likely related to line wrapping calculation. 8 comments = high engagement. | Open |
| **#899** | JavaScript syntax highlighting bug | TextMate grammar issue. Arrow function with template literal in class property breaks highlighting for rest of file. Fix is in `.tmLanguage` grammar. | Open |
| **#1120** | Keybinding style list doesn't show selected value | Missing visual indicator for active selection in Menu > View > Keybinding Style. Small rendering fix. | Open |
| **#1118** | Settings File Explorer Width: format mismatch | Mismatch between displayed format and actual value format. Documentation or rendering fix. | Open |
| **#566** | git log can't use j/k | UI hints say "j/k: navigate" but read-only buffer mode blocks input. Key events eaten before navigation handler. | Open |
| **#1113** | Ctrl+Enter in session writes `[13;5u]` as text | Terminal escape sequence for Ctrl+Enter not handled in session attach mode. Kitty keyboard protocol sequence leaking as literal text. Maintainer-filed. | Open |
| **#1128** | Key binding issue with `*` key | Cannot configure keybindings using asterisk (normal or numpad). Tried `*`, `asterisk`, `kp_multiply` — none work. Other keys work fine. Likely a key name parsing issue. | **NEW** |
| **#1158** | csharp / c_sharp / C# naming confusion | Three inconsistent names for C# across grammar, LSP, and manual language selection. LSP doesn't trigger for all variants. Small config/naming fix. | **NEW** |
| **#1157** | Theme editor: selection BG reset uses terminal BG | Default/Reset for selection background color uses terminal background instead of selection color. Theme system value resolution issue. | **NEW** |
| **#692** | Hover dismissed after mouse exit + re-entry click | Mouse event handling bug. If mouse never leaves hover, clicking inside works. If mouse exits then re-enters and clicks, hover is dismissed. Event state tracking issue. | Open |

### Tier 2: Well-defined but may require more investigation

| # | Title | Notes | Status |
|---|-------|-------|--------|
| **#1145** | OSC 52 text copy non-functional | Despite "Use OSC 52" setting being active, no escape sequences sent to terminal. OSC 52 works in same terminal outside Fresh. 4 comments. | **NEW** |
| **#1112** | Settings UI has mouse/keyboard editing UX issues | Two sub-bugs: (1) mouse click offset in scrolled LSP list, (2) Tab Width input doesn't respond to Enter/Tab/+/-. Maintainer-filed. | Open |
| **#1115** | Package manager navigation issues (macOS) | Can't navigate packages with arrows; Enter inserts newlines in virtual buffer instead of installing. Maintainer-filed. | Open |
| **#1012** | Scrollbar flashing / nearly invisible | Scrollbar handle changes color when cursor hovers over trough, becoming invisible. Chromebook/Debian terminal. | Open |
| **#1068** | Tab size always 8 | Per-language tab size overrides global setting. Go defaults to 8. UX problem: users can't easily change per-language defaults. Partially fixed but UX concern remains. | Open |
| **#722** | LSP inlay hints sometimes rendered in wrong place | Positioning calculation error. Maintainer-filed. | Open |
| **#431** | Auto-indent creates staircase code (Windows Terminal) | Indent detection fails for some code block patterns, causing cumulative indentation. | Open |
| **#865** | Empty line at bottom of editor wastes space | Status bar area takes a line even when no prompt is active. Could reclaim for content. | Open |
| **#699** | Find Previous with Shift-F3 does not work | Keybinding issue. F3 works, menu-based Find Previous works, but Shift-F3 doesn't trigger. | Open |
| **#653** | Line numbers out of sync in large file mode | Rendering desync after scrolling. Maintainer-filed. | Open |
| **#677** | Can't scroll to the end (split terminal) | Scroll bounds calculation may not account for split terminal dimensions correctly. | Open |

---

## Low Complexity - Recommended Improvements (Enhancements)

Clear improvements that are non-debatable and well-scoped.

| # | Title | Why it's clear-cut | Status |
|---|-------|--------------------|--------|
| **#716** | Move current line/selection up and down | Standard editor feature (VS Code Alt+Up/Down). Well-defined behavior, maintainer-filed. | Open |
| **#546** | Standard y/n keys for exit-without-saving prompt | Currently requires typing + Enter. Standard UX is single keypress y/n. Small prompt input change. | Open |
| **#1202** | Double click and drag selects whole words | Standard behavior on macOS and most editors. Double-click-drag should extend selection word-by-word. | **NEW** |
| **#744** | Add i18n in CLI help message | Project already has full i18n infrastructure (`locales/`). Wire up CLI help strings. | Open |
| **#619** | Add a `.desktop` file | Standard Linux packaging artifact. Just create the file. | Open |
| **#779** | Display lines after EOF | Show tilde lines or blank space below last line, like Vim/VS Code. | Open |
| **#833** | Suggested changes to PKGINFO | Packaging metadata fix. Small change. | Open |
| **#465** | Add Winget release action | CI/CD addition for Windows distribution. | Open |
| **#875** | Menu shortcuts should use i18n-dependent keys | German "Alt D" instead of "Alt F" based on localized menu labels. Already has i18n system. | Open |
| **#1073** | Highlight current cursor line and column | Standard editor feature. Line highlight likely exists; column highlight is the new part. | Open |
| **#1156** | Global toggle "hide menu bar" not per-workspace | Menu bar visibility toggling is per-workspace but should be global. Small settings scope fix. | **NEW** |

---

## Medium Complexity

Worth doing but require more design or broader changes.

| # | Title | Notes |
|---|-------|-------|
| #959 | Respect `.editorconfig` files | Well-defined spec, needs parsing library + integration with indent settings. |
| #926 | Recent files feature | Needs persistence (file history) + UI in file menu. |
| #1036 | Default macOS keybindings wrong (Cmd vs Ctrl) | Needs platform-aware modifier system (like VS Code's CtrlCmd). Broader than single keybinding. |
| #836 | Syntax highlighting in reference panel | Wire syntect into reference/hover panel renderer. |
| #867 | Keybinding editing UX in settings editor | No easy way to edit keybindings currently. 2 thumbs up. Maintainer-filed. |
| #878 | Add Move file functionality | File operations in explorer. |
| #868 | Buffer-based autocompletion | Complete from current buffer content when no LSP. |
| #611 | Buffer sometimes empty after switching file in explorer | Race condition between file loading and display. |
| #973 | Auto wrap line inconsistent across languages | Line wrapping behavior varies per language. |
| #1057 | Paste in column mode | Column/block paste like Notepad++. Needs multi-cursor paste logic. |
| #1148 | Autosave unnamed buffers | Preserve unsaved buffers on normal quit (like Sublime/Notepad++). Needs persistence strategy. |
| #1150 | Nimlang LSP highlighting not working | Language config issue. LSP works but syntax highlighting falls back to ASCII. May need .tmLanguage grammar. |
| #702 | Search state global not buffer specific | Architecture change: move SearchState from global to per-buffer. |
| #620 | Multi-select not consistent with cursor | Multi-cursor behavior inconsistencies. Needs careful UX decisions. |

---

## High Complexity / Large Features

| # | Title | Notes |
|---|-------|-------|
| #909 | magit-style git support | Large plugin feature. |
| #826 | Helix mode | Entire modal editing paradigm. |
| #1086 | Persistent Vi mode | Related to #826 but specifically Vi, not Helix. |
| #478 | Neovim plugin compatibility layer | Massive scope. |
| #140 | Three-way merge (IntelliJ-style) | Complex diff + merge UI. |
| #229 | Diff view (full) | Multi-phase implementation (docs exist). |
| #534 | WASM version | ~3-4 week effort per docs. Design exists. |
| #186 | Rendering optimizations | Broad performance work. |
| #160 | Plugin installation UX | Needs package registry/discovery design. |
| #988 | Support DAP (Debug Adapter Protocol) | Full debug integration. |
| #1026 | Support code lens | LSP code lens feature. |
| #1111 | Support vsix (VS Code extensions) | Massive compatibility layer. |

---

## Debatable / Needs Discussion

| # | Title | Why debatable |
|---|-------|---------------|
| #351 | Config format: JSON to YAML/HJSON | Breaking change. Could just add JSON5/comments support. |
| #348 | Use `ty` as Python LSP | Speculative, ty not stable. |
| #570 | Taskfile support | Niche. Questionable ROI. |
| #381 | WakaTime plugin support | Plugin system should handle this natively. |
| #460 | Terminal triggers CrowdStrike alert | Likely a CrowdStrike false positive, not a Fresh bug. May need documentation. |
| #528 | Rust edition 2024 | Mechanical but may surface issues. Needs testing. |
| #554 | Nerd font icons in file tree | Detection/fallback is tricky. |
| #236 | Cursor position keybinding in bottom line | UX preference. |
| #1066 | Auto start preview for Tinymist LSP | LSP-specific auto-preview. Niche. |
| #1053 | Opening remote terminal | Vague scope — SSH? Container? |
| #1051 | Rainbow brackets | Debatable visual feature. Popular but polarizing. PR #1088 exists. |

---

## Questions / Support Requests (Not Bugs)

| # | Title | Recommendation |
|---|-------|----------------|
| #473 | How to configure Python LSP (pyright) | Documentation gap. Add to docs/wiki then close. |
| #490 | How to move/copy files in Explorer view | Feature request disguised as question. Convert to enhancement. |
| #1090 | How to configure C++ syntax highlighting with tree-sitter | Documentation/support. Answer and close. |
| #889 | How does search and replace with regex work? | Documentation gap. |
| #1054 | Python chars on Windows 11 | Reporter confirmed fix in v0.2.5. **Should be closed.** |
| #1191 | Deno LSP support | Partially a question (how to configure), partially enhancement (#971). |

---

## Platform-Specific Issues

| # | Platform | Title |
|---|----------|-------|
| #780 | PuTTY/Windows→Oracle Linux | Copy/paste not working |
| #1023 | PuTTY | Delete char/word keybindings |
| #376 | SecureCRT | Mouse support not working |
| #477 | macOS SSH | Cannot copy to system clipboard |
| #1145 | Any (OSC 52) | OSC 52 clipboard copy non-functional |
| #586 | KDE | Middle-mouse paste not working |
| #784 | Windows ARM | Build support |
| #989 | Android/Termux | LSP fails to autostart |
| #1054 | Windows 11 | Python chars disappear (likely fixed in v0.2.5) |

PuTTY issues (#780, #1023) likely share a root cause (terminal escape sequence handling).
Clipboard issues (#477, #1145) may share root cause in OSC 52 implementation.

---

## Packaging / Distribution Requests

| # | Title | Complexity |
|---|-------|------------|
| #1080 | Gentoo support | Low — ebuild |
| #1038 | Add binary cache to flake nixConfig | Low — Nix config |
| #995 | Installation through conda | Low — packaging |
| #789 | Add flatpak to flathub | Low-Med — review process |
| #465 | Add Winget release action | Low — CI/CD |
| #833 | PKGINFO changes | Low — metadata fix |
| #784 | Windows ARM build | Medium — CI + cross-compile |

---

## Language / Syntax Support Requests

| # | Title | Complexity |
|---|-------|------------|
| #966 | Svelte syntax highlighting | Low — add .tmLanguage |
| #463 | templ syntax highlighting | Low — add .tmLanguage |
| #1031 | nushell language support | Low — syntax + LSP config |
| #1158 | C# naming inconsistency | Low — config fix |
| #1150 | Nim language highlighting | Low-Med — may need .tmLanguage |
| #1191 | Deno LSP support | Medium — relates to #971 |
| #971 | Multiple LSPs per language | Medium — config + orchestration |

---

## Recommended Priority Order for Low-Complexity Work

### Bugs (fix first)
1. **#938** - Go to line wrong numbers in large files (8 comments, high user impact, calculation fix)
2. **#899** - JS syntax highlighting broken (TextMate grammar, well-reproduced)
3. **#1128** - Keybinding with `*` key broken (key name parsing, NEW)
4. **#1158** - C# naming confusion (config cleanup, NEW, easy fix)
5. **#1120** - Keybinding list missing selection indicator (small rendering fix)
6. **#1113** - Ctrl+Enter writes escape sequence in sessions (input handling)
7. **#566** - git log j/k navigation broken (read-only buffer input)
8. **#1118** - Settings width format mismatch (display fix)
9. **#1157** - Theme editor selection BG reset wrong (theme value resolution, NEW)
10. **#692** - Hover dismiss on re-entry click (mouse state tracking)

### Enhancements (then improve)
1. **#716** - Move line up/down (standard editor feature, maintainer-filed)
2. **#1202** - Double click and drag word selection (standard behavior, NEW)
3. **#546** - y/n exit confirmation (small UX win)
4. **#744** - i18n CLI help messages (infrastructure exists)
5. **#619** - Add .desktop file (packaging, standalone)
6. **#875** - i18n-dependent menu shortcuts (infrastructure exists)
7. **#779** - Lines after EOF display (viewport rendering)
8. **#1156** - Global menu bar toggle (settings scope fix, NEW)

---

## Velocity & Trends

| Metric | Feb 6 | Feb 26 | Mar 6 |
|--------|-------|--------|-------|
| Total open issues | ~72 | ~97 | ~108 |
| Net new (period) | — | +25 | +11 |
| Closed from triage list | — | 5 | 10 |
| Top bug priority resolved | — | 0/10 | 4/10 |
| Top enhancement resolved | — | 0/5 | 1/5 |

**Trend:** Issue volume growing (~11/week), but maintainer is actively closing triaged items. 4 of our top 10 bug priorities were resolved in 8 days — indicating the triage list is being used effectively. The backlog is growing faster than closures, suggesting triage-driven prioritization is valuable.
