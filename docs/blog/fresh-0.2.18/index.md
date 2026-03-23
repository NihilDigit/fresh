---
title: "What's New in Fresh (0.2.18)"
outline: false
---

# What's New in Fresh (0.2.18)

*March 23, 2026*

A roundup of everything that landed since the [0.2.9 release](../fresh-0.2.9/) — eight point releases spanning project-wide search & replace, inline diagnostics, 30 new syntax grammars, broad LSP coverage, and a long tail of editing refinements and bug fixes.

## Project-Wide Search & Replace

Search and replace across your entire project. Type a query, tab to the replacement field, and press **Alt+Enter** to replace all matches. Handles unsaved buffers, large files, and up to 10,000 results.

<div class="showcase-demo">
  <img src="./project-search-replace/showcase.gif" alt="Project-Wide Search & Replace demo" />
</div>

## Inline Diagnostics

Diagnostic messages now appear right-aligned at the end of the offending line. Errors and warnings are visible without opening a panel or hovering — just glance at the code. Staleness detection dims stale diagnostics after edits. Disabled by default; enable "diagnostics inline text" in the Settings UI.

<div class="showcase-demo">
  <img src="./inline-diagnostics/showcase.gif" alt="Inline Diagnostics demo" />
</div>

## Surround Selection

Select text and type an opening delimiter — `(`, `[`, `{`, `"`, or backtick — to wrap the selection instead of replacing it. For example, selecting `hello` and typing `(` produces `(hello)`. Controlled by the `auto_surround` config option with per-language overrides.

<div class="showcase-demo">
  <img src="./surround-selection/showcase.gif" alt="Surround Selection demo" />
</div>

## 30 New Syntax Grammars

Dockerfile, CMake, INI, SCSS, LESS, PowerShell, Kotlin, Swift, Dart, Elixir, F#, Nix, Terraform/HCL, Protobuf, GraphQL, Julia, Nim, Gleam, V, Solidity, KDL, Nushell, Starlark, Justfile, Earthfile, Go Module, Vue, Svelte, Astro, and Hyprlang. These grammars are preliminary — please report highlighting issues for your language so we can improve them.

<div class="showcase-demo">
  <img src="./syntax-grammars/showcase.gif" alt="Syntax Grammars demo" />
</div>

## Whitespace Indicators

Granular control over whitespace visibility. Configure space (·) and tab (→) indicators independently for leading, inner, and trailing positions. A new `whitespace_indicator_fg` theme color lets you tune the indicator brightness. Per-language overrides are supported.

<div class="showcase-demo">
  <img src="./whitespace-indicators/showcase.gif" alt="Whitespace Indicators demo" />
</div>

## Theme Editor Redesign

The theme editor now uses virtual scrolling and mouse support for smooth navigation through large theme files. A new "Inspect Theme at Cursor" command and **Ctrl+Right-Click** popup show exactly which scope and color applies at any position in your code.

<div class="showcase-demo">
  <img src="./theme-editor/showcase.gif" alt="Theme Editor Redesign demo" />
</div>

## Also New

### Editing

- **Hanging line wrap** — wrapped continuation lines preserve the indentation of their parent line.
- **Smart quote suppression** — quotes typed inside an existing string no longer auto-close, preventing doubled quotes.
- **Separate auto-close config** — `auto_close` toggle to independently control bracket/quote auto-close, skip-over, and pair deletion. Per-language overrides via `languages.<lang>.auto_close`.
- **Read-only mode** — files without write permission and library/toolchain paths automatically open as read-only with a `[RO]` indicator. Override with "Toggle Read Only".
- **Open File jump syntax** — `path:line[:col]` in Open File and Quick Open prompts to jump directly.
- **Hot exit** — all buffers including unnamed scratch buffers persist across sessions automatically.

### Broad LSP Support

Added LSP configs and helper plugins (with install instructions) for **30+ languages**: Nix, Kotlin, Swift, Scala, Elixir, Erlang, Haskell, OCaml, Clojure, R, Julia, Perl, Nim, Gleam, F#, Dart, Nushell, Solidity, Vue, Svelte, Astro, Tailwind CSS, Terraform/HCL, CMake, Protobuf, GraphQL, SQL, Bash, Lua, Ruby, PHP, YAML, TOML, and Typst. Deno projects are auto-detected.

### Platform

- **macOS native GUI** (early work-in-progress) — exploring a concept of Fresh shipping without a terminal while remaining fully compatible with terminal mode. Currently includes a native menu bar with dynamic conditions, Cmd keybindings, app icon, and `.app` bundle.
- **Linux desktop integration** — icons and `.desktop` files for deb, rpm, Flatpak, and AppImage packages.
- **Windows input overhaul** — bracketed paste on Windows Terminal, new `fresh-winterm` crate replacing crossterm's input handling, with corrupt mouse sequence detection and UTF-16 surrogate support.

### Performance

- Bracket matching caps scanning at 1MB with 16KB bulk reads, preventing hangs on large files.
- Chunked incremental search with viewport-only overlays and 100K match cap for multi-GB files.
- Non-blocking grammar builds on a background thread.
- Native Rust diff indicators replacing JS plugin round-trips.
- Log volume reduced ~90% by default (from ~266MB to ~5-10MB).

### Plugin API

- `registerHandler()` API replacing `globalThis` pattern.
- `restartLspForLanguage`, process limits for `registerLspServer`, async `reloadGrammars()`.
- "Load Plugin from Buffer" — run and hot-reload plugins directly from an open buffer.
- Strict TypeScript across all built-in plugins.

### Quality of Life

- **Workspace storage** — session state always restored on startup, even when opening specific files from CLI.
- **`--wait` flag** — blocks the CLI until the buffer is closed, enabling use as `git core.editor`.
- **Indent-based code folding** — folding works without LSP, using indentation analysis as a fallback.
- **Status bar toggle** — show/hide via command palette.
- **Tab naming** — duplicate tab names disambiguated with appended numbers.
- **File deletion uses trash** — `removePath` uses the system trash instead of permanent deletion.

## Related

- [Full changelog](https://github.com/sinelaw/fresh/blob/master/CHANGELOG.md)
- [All features](/features/)
- [Getting started](/getting-started/)
