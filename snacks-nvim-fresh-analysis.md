# Snacks.nvim Feature Analysis & Fresh Plugin API Comparison

## 1. Snacks.nvim Complete Feature List

Snacks.nvim (by folke) is a **collection of 30+ small QoL plugins** for Neovim, bundled under one umbrella. Each module is independently opt-in:

| Module | Description |
|--------|-------------|
| **animate** | Efficient animation library with 45+ easing functions; used internally by scroll/indent |
| **bigfile** | Automatically disables expensive features (treesitter, LSP, etc.) for large files |
| **bufdelete** | Delete buffers without disrupting window layout |
| **dashboard** | Beautiful declarative startup screen with sections, multi-pane layout, shortcut keys |
| **debug** | Pretty-print inspect and backtraces for debugging Lua code |
| **dim** | Focus on active scope by dimming surrounding code |
| **explorer** | File explorer (built on top of the picker) |
| **gh** | GitHub CLI integration |
| **git** | Git utilities (get root, blame, etc.) |
| **gitbrowse** | Open current file/branch/commit in browser (GitHub, GitLab, Bitbucket) |
| **image** | Image viewer using Kitty Graphics Protocol |
| **indent** | Animated indent guides and scope highlighting |
| **input** | Enhanced `vim.ui.input` replacement |
| **keymap** | Extended keymap system with filetype and LSP client support |
| **layout** | Window layout management |
| **lazygit** | Float-based lazygit integration with colorscheme sync |
| **notifier** | Beautiful notification system replacing `vim.notify` |
| **notify** | Utility functions for working with `vim.notify` |
| **picker** | Universal fuzzy picker (files, grep, buffers, etc.) |
| **profiler** | Neovim Lua profiler |
| **quickfile** | Fast initial file render before plugins load |
| **rename** | LSP-integrated file renaming with plugin support |
| **scope** | Scope detection, text objects, and jumping via treesitter/indent |
| **scratch** | Persistent scratch buffers |
| **scroll** | Smooth animated scrolling |
| **statuscolumn** | Customizable status column (line numbers, signs, folds) |
| **terminal** | Floating/split terminal creation and toggling |
| **toggle** | Toggle keymaps with which-key integration |
| **util** | Shared utility functions (library) |
| **win** | Floating window and split creation/management |
| **words** | Auto-show LSP references and navigate between them |
| **zen** | Zen/distraction-free coding mode |

## 2. Dashboard Deep Dive

The dashboard is snacks.nvim's flagship UI feature — a declarative startup screen.

### Architecture
- Fully **declarative** — defined as a tree of `Section` items
- Each section can be: a static item, a generator function, or a nested array
- Sections resolve lazily at render time

### Built-in Sections
- `header` — ASCII art or custom header text (centered)
- `keys` — Shortcut keys (Find File, New File, Find Text, Recent Files, Config, Restore Session, Quit)
- `recent_files` — MRU files, filterable by cwd
- `projects` — Recent git project roots with session restore
- `session` — Auto-detects session plugins (persistence.nvim, persisted.nvim, etc.)
- `terminal` — Colored terminal output with automatic caching (TTL-based)
- `startup` — Shows plugin load count and startup time

### Layout System
- Configurable `width` (default 60 chars), `row`/`col` positioning (default: centered)
- **Multi-pane** support — items can specify `pane = 2` to go in a second vertical column
- `pane_gap` for spacing between panes
- Items support `indent`, `align` (left/center/right), `gap`, `padding`

### Interactivity
- Items can have `key` shortcuts and `action` (command string, keymap string, or function)
- `autokey` auto-assigns numerical/alphabetical keys
- Actions execute on keypress: `:command`, keymap feed, or function call

### Rendering
- Renders into a Neovim buffer with extmarks for highlighting
- Terminal sections use floating windows with cached output
- Custom highlight groups (SnacksDashboardHeader, SnacksDashboardKey, etc.)
- Responsive — re-renders on window resize

## 3. Fresh Plugin API Capabilities

Fresh is a **terminal text editor** (Rust-based, VS Code-like UX) with a TypeScript plugin system running in sandboxed QuickJS.

### Key API Surface (from fresh.d.ts)

**Core Editor Operations:**
- Buffer CRUD: create, open, close, save, read text, insert, delete
- Cursor management: get/set positions, selections, multi-cursor
- Viewport: scroll, get viewport info, scroll sync groups

**Visual/Decoration System:**
- `addOverlay()` — styled byte-range decorations (fg, bg, bold, italic, underline, strikethrough, URLs)
- `addVirtualText()` — inline text not in buffer
- `addVirtualLine()` — full lines above/below positions
- `addConceal()` — hide/replace byte ranges
- `addSoftBreak()` — marker-based line wrapping
- `submitViewTransform()` — full token-level rendering pipeline
- `setLineIndicator()` — gutter symbols
- `setFileExplorerDecorations()` — file tree badges

**Virtual Buffers (key for dashboard-like UIs):**
- `createVirtualBuffer()` — in current split, with entries, modes, hide from tabs
- `createVirtualBufferInSplit()` — in a new split with configurable direction/ratio
- `createVirtualBufferInExistingSplit()` — into specific split
- `setVirtualBufferContent()` — update entries (text + properties + styling)
- `TextPropertyEntry` — rich text entries with per-entry styling and inline overlays
- `getTextPropertiesAtCursor()` — read metadata at cursor (for click handling)

**UI Interaction:**
- `prompt()` / `startPrompt()` — input prompts with suggestions
- `showActionPopup()` — modal dialog with buttons
- `registerCommand()` — command palette entries
- `defineMode()` — custom keybinding modes
- `setContext()` — conditional command visibility

**Split/Layout Management:**
- `closeSplit()`, `focusSplit()`, `setSplitBuffer()`, `setSplitRatio()`, `setSplitLabel()`
- `createCompositeBuffer()` — side-by-side/stacked multi-source buffers (for diffs)
- `distributeSplitsEvenly()`

**System/IO:**
- `spawnProcess()` — async subprocess with stdout/stderr
- `spawnBackgroundProcess()` — long-running processes
- `createTerminal()` — integrated terminal in splits
- File system: read/write/exists/readDir/createDir/rename/copy/remove
- Path utilities, environment variables, working directory

**Plugin Ecosystem:**
- `loadPlugin()`, `unloadPlugin()`, `reloadPlugin()`, `listPlugins()`
- `registerGrammar()`, `registerLanguageConfig()`, `registerLspServer()`
- Theme management: apply, save, reload, get schema
- i18n: `t()`, `pluginTranslate()`
- State persistence: `setGlobalState()`, `setViewState()`

## 4. Feasibility Matrix: Snacks Features in Fresh

| Snacks Feature | Feasibility | Implementation Strategy |
|---------------|-------------|------------------------|
| **dashboard** | **YES** | `createVirtualBuffer()` + `TextPropertyEntry[]` + `defineMode()` for keys |
| **terminal sections** | **YES** | `spawnProcess()` to run commands, render output into virtual buffer |
| **scratch** | **YES** | `createVirtualBuffer()` + `writeFile()`/`readFile()` for persistence |
| **notifier** | **PARTIAL** | `setStatus()` for simple messages; no floating toast system |
| **gitbrowse** | **YES** | `spawnProcess()` for git remote URL, construct browser URL |
| **git utilities** | **YES** | `spawnProcess("git", [...])` — full CLI access |
| **rename** | **PARTIAL** | `renamePath()` exists; LSP rename via `sendLspRequest()` |
| **toggle** | **YES** | `registerCommand()` + `setContext()` + state tracking |
| **words** | **YES** | `sendLspRequest()` for references + `addOverlay()` for highlights |
| **indent** | **YES** | `addOverlay()` or `addVirtualText()` for guide characters |
| **dim** | **YES** | `addOverlay()` with dimmed styling on out-of-scope lines |
| **statuscolumn** | **PARTIAL** | `setLineIndicator()` for gutter symbols; limited API |
| **bigfile** | **PARTIAL** | Can detect via `getBufferLength()`, can't disable built-in features |
| **picker** | **YES** | Already implemented as the `Finder` library |
| **explorer** | **PARTIAL** | Built-in explorer exists; extending limited to decorations |
| **zen** | **NO** | No API to hide editor chrome (tabs, status bar, menus) |
| **scroll** (smooth) | **NO** | No frame-by-frame scroll animation API |
| **animate** | **NO** | No animation primitives |
| **image** | **NO** | No image rendering protocol support |
| **profiler** | **NO** | No access to editor internals for profiling |
| **layout** (complex) | **PARTIAL** | Split creation/ratio exists, no layout DSL |

## 5. Dashboard Implementation Blueprint for Fresh

A snacks.nvim-style dashboard for Fresh could work as follows:

### Step 1: Detect Startup
Register a `buffer_opened` event handler to detect when the editor opens with no file argument.

### Step 2: Create the Dashboard Buffer
```typescript
const result = await editor.createVirtualBuffer({
  name: "Dashboard",
  hiddenFromTabs: true,
  showLineNumbers: false,
  showCursors: false,
  editingDisabled: true,
  mode: "dashboard",
  entries: buildDashboardEntries(),
});
```

### Step 3: Build Section Entries
Build `TextPropertyEntry[]` with:
- **Header**: ASCII art with `style: { fg: "syntax.keyword", bold: true }`
- **Key shortcuts**: `"  [f]  Find File"` with `inlineOverlays` for key highlighting
- **Recent files**: From `readDir()` or state persistence
- **Footer**: Editor version info

### Step 4: Define Keybindings
```typescript
editor.defineMode("dashboard", [
  ["f", "dashboard_find_file"],
  ["r", "dashboard_recent"],
  ["n", "dashboard_new_file"],
  ["q", "dashboard_quit"],
]);
```

### Step 5: Handle Actions
Use `getTextPropertiesAtCursor()` to detect selected item, then `openFile()` or execute the appropriate action.

### Step 6: Multi-pane (Optional)
Use `createVirtualBufferInSplit()` for a second pane showing recent files or project info.

## 6. Key Gaps for Full Parity

1. **No floating windows** — Fresh plugins can only work within the split system
2. **No animation/transition API** — Can't do smooth scrolling, fading, or animated guides
3. **No chrome control** — Can't hide tabs, status bar, or menus for zen mode
4. **No cached terminal rendering** — Can run commands but can't embed colored terminal output in virtual buffers the way snacks does
5. **No image support** — No Kitty Graphics Protocol or equivalent
6. **No startup timing** — No API to get plugin load time or startup metrics

## 7. Conclusion

**Fresh's plugin API is surprisingly capable** for building many snacks.nvim-like features. The `createVirtualBuffer` + `TextPropertyEntry` + `defineMode` + `addOverlay` combination provides enough primitives for rich, interactive UI panels — including a dashboard.

The **strongest matches** are: dashboard, scratch buffers, git utilities, toggle commands, code highlighting overlays (dim, words, indent), and anything using the Finder/picker pattern.

The **biggest gaps** are architectural: no floating windows (everything is split-based), no animation primitives, no ability to hide editor chrome, and no image rendering.

**A "Fresh Snacks" plugin pack is feasible** and could realistically provide: a startup dashboard, persistent scratch buffers, git browse, distraction-dimming, word highlighting, and enhanced indent guides — all within the existing API.
