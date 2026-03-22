# Fresh Editor - Open Issues Review

**Date:** 2026-03-22
**Total Open Issues:** 127

## Categorized Issues Table

### Legend
- **Type:** Bug, Enhancement, UX, Question, Infra, Packaging
- **Severity:** Critical (crashes/data loss), High (broken core features), Medium (annoying quirks), Low (cosmetic/nice-to-have)
- **Difficulty:** S (small), M (medium), L (large), XL (very large)
- **Daily Driver Priority:** How much this impacts using Fresh as a daily driver (1=blocks daily use, 2=annoying daily, 3=occasional pain, 4=nice-to-have, 5=not relevant)

| # | Title | Type | Severity | Difficulty | Daily Driver |
|---|-------|------|----------|------------|--------------|
| **CRASHES / DATA LOSS / RELIABILITY** |
| 330 | Flaky tests | Bug/Infra | High | M | 2 |
| 184 | Fix ignored tests | Bug/Infra | Medium | M | 3 |
| **RENDERING / DISPLAY BUGS** |
| 1255 | Hidden cursor in Zellij terminal | Bug | Critical | M | 1 |
| 1263 | Comments displace inlay hints | Bug | High | M | 2 |
| 722 | LSP inlay hints sometimes rendered in wrong place | Bug | High | M | 2 |
| 653 | Line numbers out of sync after scrolling in large file mode | Bug | High | M | 1 |
| 677 | Sometimes can't scroll to the end | Bug | High | M | 1 |
| 611 | File switch leaves buffer empty until scroll | Bug | High | S | 1 |
| 1012 | Scrollbar flashing | Bug | Medium | S | 3 |
| 620 | Multi-select cursors inconsistent (block vs bar) | UX | Medium | S | 3 |
| 865 | Empty line wasted at bottom of editor | UX | Low | S | 4 |
| **CORE EDITING BUGS** |
| 1288 | Word select (Alt+Right) includes trailing whitespace | Bug | High | S | 1 |
| 1068 | Tab size always 8, ignoring settings | Bug | Critical | M | 1 |
| 431 | Auto-indent creates staircase on paste | Bug | High | M | 1 |
| 727 | Delete word / move cursor by word broken on macOS | Bug | High | S | 1 |
| 1054 | Can't write some chars with Python syntax on Windows 11 | Bug | Critical | M | 1 |
| 1256 | Escaped chars in replace string inserted raw (no \n support) | Bug | High | S | 2 |
| 1254 | Auto-surround doesn't work without auto-close | Bug | Medium | S | 2 |
| 1211 | Expand/Shrink selection doesn't work as intended | Bug | Medium | M | 2 |
| **SEARCH & REPLACE** |
| 699 | Shift-F3 (Find Previous) does not work | Bug | High | S | 1 |
| 702 | Search state is global, not buffer-specific | Enhancement | High | M | 2 |
| 889 | Regex search/replace capture groups unclear/missing | Enhancement | Medium | M | 2 |
| 1251 | Find Next should center match in viewport | Enhancement | Medium | S | 3 |
| 391 | Improve Live Grep (grep selection, paste from clipboard) | Enhancement | Medium | M | 3 |
| **LSP / LANGUAGE SUPPORT** |
| 625 | Rust LSP missing some hover support | Bug | Medium | M | 3 |
| 692 | Hover dismissed after mouse moves outside then clicks inside | Bug | Medium | S | 2 |
| 989 | LSP fails to autostart on Android (Termux) | Bug | Medium | M | 4 |
| 1150 | Nimlang LSP highlighting not working | Bug | Medium | M | 4 |
| 971 | Set multiple LSP servers per language (e.g. ruff + pyright) | Enhancement | High | L | 2 |
| 612 | Multiple language servers for single language | Enhancement | High | L | 2 |
| 1191 | Deno LSP support | Enhancement | Medium | M | 4 |
| 463 | Syntax highlighting: templ | Enhancement | Medium | M | 4 |
| 1026 | Support code lens | Enhancement | Medium | L | 3 |
| 603 | Make links in hover clickable | Enhancement | Low | M | 4 |
| 836 | Syntax highlighting in reference panel | Enhancement | Low | M | 4 |
| 686 | Support ghost text (Copilot) | Enhancement | Medium | XL | 3 |
| 243 | Missing LSP features (comprehensive list) | Enhancement | High | XL | 2 |
| **SYNTAX HIGHLIGHTING** |
| 1319 | C/C++ treats `{}` `;` as operators | Bug | Medium | M | 3 |
| 899 | JavaScript syntax highlighting bug (template literals) | Bug | Medium | M | 3 |
| 973 | Auto wrap line inconsistent across languages | Bug | Medium | S | 3 |
| 1090 | How to configure C++ tree-sitter highlighting | Question | Low | S | 4 |
| 966 | Svelte syntax highlighting | Enhancement | Low | M | 5 |
| 1031 | Nushell language support | Enhancement | Low | M | 5 |
| 1051 | Support rainbow brackets | Enhancement | Low | L | 4 |
| 1219 | Set default/fallback grammar for files | Enhancement | Medium | M | 3 |
| **FILE MANAGEMENT / EXPLORER** |
| 611 | File switch leaves buffer empty until scroll | Bug | High | S | 1 |
| 1212 | File Explorer settings allow invalid width values | Bug | Low | S | 3 |
| 1118 | File Explorer width description doesn't match format | Bug | Low | S | 4 |
| 1213 | Support absolute (fixed) width for file explorer | Enhancement | Low | S | 4 |
| 878 | Add Move file functionality | Enhancement | Medium | M | 3 |
| 490 | Move/copy file in explorer view | Enhancement | Medium | M | 3 |
| 554 | File tree icon support (nerd fonts) | Enhancement | Low | M | 5 |
| 926 | Recent files list | Enhancement | Medium | M | 3 |
| 950 | Sidebar with file outline / markdown TOC | Enhancement | Medium | L | 3 |
| **KEYBINDINGS / INPUT** |
| 1128 | Can't bind `*` key | Bug | Medium | S | 3 |
| 1113 | Ctrl+Enter writes `[13;5u]` when attaching to session | Bug | High | M | 1 |
| 1036 | Default macOS keybindings should use Cmd | Bug | High | M | 1 |
| 727 | Delete word / move by word broken on macOS | Bug | High | S | 1 |
| 699 | Shift-F3 Find Previous doesn't work | Bug | High | S | 1 |
| 1023 | Putty keybindings for delete char/word | Bug | Medium | S | 3 |
| 867 | Keybinding editing UX in settings | Enhancement | High | L | 2 |
| 1257 | Option to disable menu bar mnemonics (Alt-key) | Enhancement | Medium | S | 3 |
| 236 | Keybindings shown relative to cursor on bottom line | Enhancement | Medium | M | 4 |
| 1086 | Persistent Vi Mode toggle | Enhancement | Medium | S | 3 |
| 826 | Support helix mode | Enhancement | Low | XL | 5 |
| **TERMINAL / SHELL INTEGRATION** |
| 1245 | Cannot select text in terminal | Bug | High | M | 2 |
| 1316 | Suspend support (Ctrl+Z / fg) | Enhancement | Medium | M | 3 |
| 1053 | Opening remote terminal | Enhancement | Medium | M | 4 |
| 460 | Opening terminal triggers CrowdStrike alert | Enhancement | Low | M | 4 |
| **CLIPBOARD / COPY-PASTE** |
| 780 | Copy/Paste broken on Putty + Windows 11 (v0.1.77+) | Bug | High | M | 2 |
| 477 | Cannot copy to macOS clipboard over SSH | Bug | High | M | 2 |
| 586 | KDE middle-mouse paste doesn't work | Enhancement | Medium | M | 3 |
| 1057 | Paste in column mode | Enhancement | Medium | M | 3 |
| **UI / UX POLISH** |
| 1112 | Settings UI mouse offset / keyboard editing issues | Bug | Medium | M | 2 |
| 692 | Hover dismissed after mouse outside + click inside | Bug | Medium | S | 2 |
| 245 | Open file dialog doesn't scroll on small terminal | Bug | Medium | S | 3 |
| 1070 | Command to repaint whole console display | Enhancement | Low | S | 3 |
| 623 | Locale selector scroll / progress indicator | Enhancement | Low | S | 5 |
| 546 | Standard y/n keys for exit without saving | Enhancement | Medium | S | 3 |
| 1204 | Adjust gutter width relative to line numbers | Enhancement | Low | S | 4 |
| 315 | Right-click mouse context menu | Enhancement | Medium | M | 3 |
| 1253 | Auto-jump to line while typing line number | Enhancement | Low | S | 4 |
| **HOT EXIT / SESSION MANAGEMENT** |
| 1238 | Session handling given Hot Exit (wider feature) | Enhancement | High | L | 2 |
| 1236 | Unnamed sessions with Hot Exit | Enhancement | High | L | 2 |
| 1235 | Session Detach vs Quit with Hot Exit | Enhancement | Medium | M | 3 |
| **THEMES / APPEARANCE** |
| 1281 | Custom theme doesn't work ("Failed to load theme") | Bug | Medium | M | 2 |
| 1157 | Theme editor selection bg uses terminal bg instead of selection color | Bug | Medium | S | 3 |
| 1290 | Option to disable tilde ~ on empty lines | Enhancement | Low | S | 4 |
| 779 | Lines after EOF in different shade instead of ~ | Enhancement | Low | S | 4 |
| 1239 | Import VSCode themes | Enhancement | Low | L | 4 |
| 1073 | Highlight current cursor line and column | Enhancement | Low | S | 3 |
| **GIT INTEGRATION** |
| 909 | Magit-style git support (rebase, stage/unstage) | Enhancement | Medium | XL | 3 |
| 566 | Git log can't use j/k to navigate | Bug | Medium | S | 3 |
| 229 | Diff view (side-by-side/unified) | Enhancement | High | XL | 3 |
| 140 | Three-way merge like IntelliJ | Enhancement | Medium | XL | 4 |
| 432 | Diff plugin keyboard shortcuts | Enhancement | Medium | M | 4 |
| 197 | Line-diff highlighting | Enhancement | Low | M | 4 |
| **CONFIGURATION** |
| 351 | config.json hard to edit, consider YAML/HJSON | Enhancement | Low | L | 4 |
| 959 | Respect `.editorconfig` files | Enhancement | High | M | 2 |
| 1156 | Global toggle for "hide menu bar" (not per workspace) | Enhancement | Low | S | 3 |
| 1217 | Windows: uses roaming appdata instead of local | Enhancement | Low | S | 4 |
| **PACKAGING / INSTALLATION** |
| 833 | Suggested changes to AUR PKGBUILD | Enhancement | Low | S | 5 |
| 789 | Add flatpak to Flathub | Enhancement | Low | M | 5 |
| 465 | Add Winget release action | Enhancement | Low | M | 5 |
| 995 | Installation through conda | Enhancement | Low | M | 5 |
| 784 | Enable building for Windows on ARM | Enhancement | Medium | L | 4 |
| 1080 | Support for Gentoo | Enhancement | Low | M | 5 |
| 1038 | Add binary cache to flake nixConfig | Enhancement | Low | S | 5 |
| 528 | Upgrade to Rust edition 2024 | Infra | Low | M | 5 |
| **MARKDOWN / DOCUMENT EDITING** |
| 1227 | Long links in Markdown truncated when wrapping | Bug | Medium | M | 3 |
| 1206 | Keyboard shortcuts for Markdown formatting | Enhancement | Low | S | 4 |
| **LARGE FEATURES / VISIONARY** |
| 988 | Support DAP (Debug Adapter Protocol) | Enhancement | Medium | XL | 3 |
| 478 | Neovim plugin compatibility layer | Enhancement | Low | XL | 5 |
| 1111 | Support VSIX extensions (crazy idea) | Enhancement | Low | XL | 5 |
| 534 | WASM version | Enhancement | Low | XL | 5 |
| 381 | WakaTime support | Enhancement | Low | M | 5 |
| 570 | Taskfile support | Enhancement | Low | L | 5 |
| 394 | Shellcheck integration | Enhancement | Low | M | 4 |
| 875 | i18n menu mnemonics (e.g. German) | Enhancement | Low | M | 5 |
| 744 | i18n CLI help messages | Enhancement | Low | M | 5 |
| 186 | Rendering optimizations | Enhancement | Medium | L | 3 |
| 868 | Buffer-based autocompletion | Enhancement | Medium | M | 3 |
| 1066 | Auto-start preview for Tinymist LSP | Enhancement | Low | M | 5 |
| 348 | Use `ty` as Python LSP | Enhancement | Low | S | 5 |
| 473 | How to configure Python LSP (pyright) | Question | Low | S | 5 |
| 376 | Mouse support not working with SecureCRT | Bug | Medium | M | 4 |
| 1203 | IntelliJ-style git client idea | Enhancement | Low | XL | 5 |

---

## Summary Statistics

| Category | Count |
|----------|-------|
| **Bugs** | ~45 |
| **Enhancements** | ~70 |
| **Questions** | ~3 |
| **Infrastructure** | ~4 |
| **Packaging** | ~8 |

| Severity | Count |
|----------|-------|
| **Critical** | 3 |
| **High** | ~25 |
| **Medium** | ~50 |
| **Low** | ~49 |

---

## Top Priority: What to Fix First for a Reliable Daily Driver

These are ordered by impact on daily-driver reliability. Focus on eliminating crashes, data-loss risks, broken core editing, and annoying daily-use quirks.

### Tier 1: Blockers — Fix These First (Core editing broken / crashes)

| # | Issue | Why |
|---|-------|-----|
| **1068** | Tab size always 8, ignoring settings | Fundamental editing broken — every user hits this |
| **1054** | Can't write some chars with Python syntax (Windows) | Can't type code — editor unusable for Python on Windows |
| **1255** | Hidden cursor in Zellij | Cursor disappears — can't edit in a popular terminal multiplexer |
| **653** | Line numbers out of sync in large file mode | Core display corruption in large files |
| **677** | Sometimes can't scroll to the end | Can't reach end of file — basic navigation broken |
| **611** | File switch leaves buffer empty until scroll | Switching files shows blank screen |
| **431** | Auto-indent staircase on paste | Pasting code is broken — daily operation |
| **1113** | Ctrl+Enter writes `[13;5u]` in session attach | Garbage inserted into documents |

### Tier 2: High-Impact Daily Annoyances — Fix Next

| # | Issue | Why |
|---|-------|-----|
| **1288** | Word select includes trailing whitespace | Basic text selection wrong, hit constantly |
| **699** | Shift-F3 Find Previous doesn't work | Core search broken — used many times per session |
| **1036** | macOS default keybindings should use Cmd | All macOS users have wrong muscle memory |
| **727** | Delete word / move by word broken on macOS | Basic editing broken on macOS |
| **1256** | Replace doesn't interpret `\n` escape | Search & replace is crippled without regex escapes |
| **722** | Inlay hints rendered in wrong place | Visual noise — hints cover wrong code |
| **1263** | Comments displace inlay hints | Same category — inlay hint positioning |
| **780** | Copy/Paste broken on Putty (Windows) | Can't copy/paste in a common terminal |
| **1112** | Settings UI mouse/keyboard issues | Can't easily configure the editor |
| **1281** | Custom themes fail to load | Customization broken |
| **330** | Flaky tests | Undermines CI reliability, blocks confident releases |

### Tier 3: Important for Completeness — Fix Soon

| # | Issue | Why |
|---|-------|-----|
| **959** | Respect `.editorconfig` | Expected by nearly all projects |
| **971/612** | Multiple LSP servers per language | Python users need ruff + pyright together |
| **867** | Keybinding editing UX | Users can't easily customize keybindings |
| **702** | Search state global not buffer-specific | Confusing when editing multiple files |
| **1238/1236** | Hot Exit session handling | Data loss risk with multiple instances |
| **1245** | Can't select text in terminal | Terminal feature half-broken |
| **477** | Can't copy to macOS clipboard over SSH | Remote editing workflow broken |
| **229** | Diff view | Essential for code review workflows |
| **926** | Recent files | Basic productivity feature every editor has |
| **868** | Buffer-based autocompletion | Expected in any code editor |

### Recommended Approach

1. **Sprint 1:** Fix Tier 1 (8 issues) — eliminate all "editor is broken" moments
2. **Sprint 2:** Fix Tier 2 (11 issues) — eliminate daily annoyances
3. **Sprint 3:** Fix Tier 3 (10 issues) — reach feature parity for daily-driver status
4. **Ongoing:** Triage enhancements from the remaining ~100 issues based on user demand

The single most impactful area is **core editing correctness** (tab size, word selection, paste behavior, cursor visibility). Fixing these ~8 issues would dramatically improve the perception of Fresh as a reliable editor.
