# Design: Customizable Status Bar

## Current State

The status bar is rendered by `StatusBarRenderer::render_status` in
`crates/fresh-editor/src/view/ui/status_bar.rs`. It is a single monolithic
function (~600 lines) that:

1. Computes every element inline (filename, cursor position, diagnostics,
   cursor count, messages, chord display, line ending, encoding, language,
   LSP status, warnings, update indicator, command palette hint).
2. Concatenates left-side elements into one `String`, builds right-side
   `Span`s one by one, computes padding, and writes to a single ratatui
   `Line`.
3. Tracks click-target regions via `StatusBarLayout` for mouse interaction.
4. Is always exactly 1 terminal row.

**Configuration system**: `Config` / `EditorConfig` / `PartialEditorConfig`
with serde + JSON Schema, layered merging (System -> User -> Project ->
Session), and a settings UI driven by the schema (`view/settings/`).

**UI controls**: A reusable control library lives in `view/controls/`
(Toggle, Dropdown, NumberInput, TextInput, TextList, MapInput, Button,
KeybindingList). There is no DualList control today.

---

## Design Alternatives

### Alternative A: Element-Trait with Registry (recommended)

**Core idea**: Define a `StatusBarElement` trait. Each element (filename,
cursor, diagnostics, ...) is a struct implementing the trait. A registry maps
string tags (`"{filename}"`, `"{cursor}"`, ...) to constructor functions.
At render time, iterate the configured left/right lists, look up each tag,
call `render()`, and collect `Span`s.

```rust
/// One rendered chunk ready for display.
pub struct RenderedElement {
    pub spans: Vec<Span<'static>>,
    /// Total visual width of spans.
    pub width: usize,
    /// Optional click-target identifier for StatusBarLayout.
    pub click_id: Option<StatusBarClickTarget>,
}

pub trait StatusBarElement {
    /// Render the element given current editor state.
    /// Return None to hide (e.g., cursor_count == 1).
    fn render(&self, ctx: &StatusBarContext) -> Option<RenderedElement>;
}
```

A `StatusBarContext` bundles the ~15 parameters currently passed to
`render_status`:

```rust
pub struct StatusBarContext<'a> {
    pub state: &'a EditorState,
    pub cursors: &'a Cursors,
    pub status_message: Option<&'a str>,
    pub plugin_status_message: Option<&'a str>,
    pub lsp_status: &'a str,
    pub theme: &'a Theme,
    pub display_name: &'a str,
    pub keybindings: &'a KeybindingResolver,
    pub chord_state: &'a [(KeyCode, KeyModifiers)],
    pub update_available: Option<&'a str>,
    pub warning_level: WarningLevel,
    pub general_warning_count: usize,
    pub hover: StatusBarHover,
    pub remote_connection: Option<&'a str>,
    pub session_name: Option<&'a str>,
    pub read_only: bool,
}
```

**Config representation**:

```rust
// In config.rs, inside EditorConfig
pub struct StatusBarConfig {
    /// Number of terminal rows (1 or 2; 2 allows keybind_hints row).
    pub lines: u8,
    /// Element tags for the left side, rendered in order.
    pub left: Vec<String>,
    /// Element tags for the right side, rendered in order.
    pub right: Vec<String>,
    /// Whether to show nano-style keybind hints (occupies line 2).
    pub show_keybind_hints: bool,
}
```

The default value reproduces today's hardcoded layout exactly:
```rust
impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            lines: 1,
            left: vec![
                "{filename}".into(), "{cursor}".into(),
                "{diagnostics}".into(), "{cursor_count}".into(),
                "{messages}".into(),
            ],
            right: vec![
                "{line_ending}".into(), "{encoding}".into(),
                "{language}".into(), "{lsp}".into(),
                "{warnings}".into(), "{update}".into(),
                "{palette}".into(),
            ],
            show_keybind_hints: false,
        }
    }
}
```

**Render pipeline** (replaces the monolith):

```
1. Parse left/right tag lists (cached on config change).
2. For each tag, look up the Element in the registry.
3. Call element.render(ctx) -> Option<RenderedElement>.
4. Collect left spans, compute total left width.
5. Collect right spans, compute total right width.
6. Insert padding between left and right.
7. If total exceeds terminal width, truncate left side with "...".
8. Build StatusBarLayout from click_ids and accumulated column offsets.
9. Emit Line to ratatui Frame.
```

**Click target handling**: Each `RenderedElement` optionally carries a
`StatusBarClickTarget` enum variant. After layout, the positions are
recorded into `StatusBarLayout` exactly as today — the mouse input code
doesn't change.

```rust
pub enum StatusBarClickTarget {
    LineEnding,
    Encoding,
    Language,
    Lsp,
    Warnings,
    Message,
}
```

**Variant support** (`{cursor}` vs `{cursor:compact}`): The tag parser
splits on `:` to extract the variant string, which is passed to
`Element::render()` as an optional parameter or stored in the element
struct at construction time.

#### Pros
- Clean separation: each element is independently testable.
- Extensible: plugins can register new elements via the plugin API.
- Click-target tracking is unified and automatic.
- The context struct eliminates the 18-parameter function signature.
- Incremental migration: elements can be extracted one at a time from the
  monolith while keeping the existing rendering working.

#### Cons
- Moderate refactor scope: ~15 element structs + trait + registry + new
  render loop.
- Dynamic dispatch adds a thin layer of indirection (negligible cost for
  a status bar rendered at ~60 Hz).
- Variant parsing needs a small parser (split on `:`).

#### Estimated scope
- New files: `status_bar/element.rs` (trait + context), `status_bar/elements/*.rs` (one per element), `status_bar/registry.rs`.
- Modified: `config.rs`, `partial_config.rs`, `app/render.rs`, `app/mouse_input.rs`, `view/ui/status_bar.rs`.
- Lines: ~800 new, ~500 deleted from monolith.

---

### Alternative B: Template-String Approach

**Core idea**: The left and right config values are format strings rather
than tag arrays. The renderer does a single `str::replace` pass over the
template, substituting each `{tag}` with its rendered text.

```json
{
  "editor": {
    "status_bar": {
      "left": "{filename} | Ln {line}, Col {col}{diagnostics}{cursor_count}{messages}",
      "right": "{line_ending} {encoding} {language} {lsp}{warnings}{update} {palette}"
    }
  }
}
```

**Implementation**: A function `expand_template(template, values: &HashMap<&str, String>) -> String`
performs substitution. Styling is applied post-hoc by scanning the
expanded string for known substrings (e.g., the LSP status text) and
wrapping them in styled `Span`s.

#### Pros
- Very flexible: users can embed literal separators (`" | "`), change
  wording, or add text between elements.
- Simple to implement: just string substitution.

#### Cons
- **Styling is extremely difficult.** The current status bar applies
  different foreground/background colors to each element (LSP gets
  error/warning colors, palette gets `help_indicator_*`, etc.). After
  template expansion, it's hard to know where each element's text begins
  and ends, especially with variable-length content. This would likely
  require a two-pass approach: first compute styled spans per element,
  then assemble them — at which point you've essentially reinvented
  Alternative A but with a more complex user-facing format.
- **Click-target tracking breaks.** Mouse hit testing needs exact column
  ranges for each clickable element. With free-form templates, the
  positions depend on all preceding text and user-added literals.
  Calculating them requires running the template through a layout engine.
- **Validation is harder.** Typos in tags silently produce `{typo}` in
  the status bar. With an array of known tags, unknown tags can be
  rejected at config load time.
- **Multi-line and ordering are awkward.** Keybinding hints spanning a
  second row don't fit the template model. Left/right alignment requires
  separate templates anyway, reducing the flexibility advantage.

#### Estimated scope
- Smaller initial implementation (~400 new lines) but styling/click
  tracking adds significant complexity on top.

---

### Alternative C: Ordered Struct with Visibility Flags

**Core idea**: Keep the current monolithic renderer but add per-element
visibility booleans and a sort-order integer to the config. No element
trait, no registry.

```rust
pub struct StatusBarConfig {
    pub show_filename: bool,        // default true
    pub show_cursor: bool,          // default true
    pub show_diagnostics: bool,     // default true
    pub show_cursor_count: bool,    // default true
    pub show_messages: bool,        // default true
    pub show_line_ending: bool,     // default true
    pub show_encoding: bool,        // default true
    pub show_language: bool,        // default true
    pub show_lsp: bool,             // default true
    pub show_warnings: bool,        // default true
    pub show_update: bool,          // default true
    pub show_palette: bool,         // default true
    pub show_clock: bool,           // default false
    pub show_keybind_hints: bool,   // default false
    pub cursor_format: String,      // "normal" | "compact"
}
```

The existing render function wraps each section in `if config.show_X { ... }`.

#### Pros
- **Minimal refactor**: ~50 lines of config + ~30 `if` guards in the
  existing function.
- **No new abstractions**: no trait, no registry, no new files.
- **Easy to review and merge**.

#### Cons
- **No reordering**: elements are rendered in hardcoded order. Moving
  `{language}` to the left side requires a code change.
- **No left/right reassignment**: elements are stuck on whichever side
  they're currently hardcoded to.
- **Doesn't satisfy the spec**: the spec explicitly requires user-defined
  ordering and left/right placement.
- **Doesn't scale**: every new element requires a new bool field in config,
  partial config, merge logic, schema, and settings UI.
- **No DualList UI**: the settings UI would just be a wall of toggles,
  not the DualList picker described in the spec.

#### Estimated scope
- ~150 new lines across config.rs, partial_config.rs, status_bar.rs.

---

### Alternative D: Data-Driven Element Descriptors (no trait objects)

**Core idea**: Instead of a trait with dynamic dispatch, use an enum that
lists every known element. A single `render_element(kind, ctx)` match
function dispatches to per-element rendering code. The config stores
`Vec<StatusBarElementKind>` for left and right.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusBarElementKind {
    Filename,
    Cursor,
    CursorCompact,
    Diagnostics,
    CursorCount,
    Messages,
    Chord,
    LineEnding,
    Encoding,
    Language,
    Lsp,
    Warnings,
    Update,
    Palette,
    KeybindHints,
    Clock,
}
```

Config stores them as strings that (de)serialize to the enum:

```json
{ "left": ["filename", "cursor", "diagnostics"] }
```

A single function:

```rust
fn render_element(
    kind: &StatusBarElementKind,
    ctx: &StatusBarContext,
) -> Option<RenderedElement> {
    match kind {
        StatusBarElementKind::Filename => { /* ... */ }
        StatusBarElementKind::Cursor => { /* ... */ }
        // ...
    }
}
```

#### Pros
- **No dynamic dispatch overhead** (though negligible either way).
- **Exhaustive matching**: the compiler ensures every element is handled.
- **Simpler than trait objects**: one file, one match, no registry.
- **Serde-friendly**: enum variants serialize to clean strings.
- **Still satisfies the spec**: full reordering, left/right reassignment.

#### Cons
- **Not plugin-extensible**: adding a new element requires modifying the
  enum and the match. Plugins cannot register custom elements without
  adding a `Custom(String)` escape hatch.
- **Large match arm**: the match function will have ~16 arms, each with
  10-30 lines of rendering logic (~300-400 lines). Still better than the
  current monolith but less modular than separate files.
- **Variants** (`{cursor:compact}`) require either separate enum variants
  (`Cursor` vs `CursorCompact`) or an associated data field.

#### Estimated scope
- New files: `status_bar/elements.rs` (enum + context + match function).
- Modified: `config.rs`, `partial_config.rs`, `app/render.rs`, `status_bar.rs`.
- Lines: ~600 new, ~500 deleted from monolith.

---

## Cross-Cutting Concerns

### 1. Config & Partial Config

All alternatives (except C) add a `StatusBarConfig` sub-struct to
`EditorConfig` and a corresponding `PartialStatusBarConfig` to
`PartialEditorConfig`. The merge semantics for lists should be **replace,
not merge** — if the user specifies `left`, the entire left list is
replaced, not appended to defaults.

```rust
// In PartialEditorConfig:
pub status_bar: Option<PartialStatusBarConfig>,

pub struct PartialStatusBarConfig {
    pub lines: Option<u8>,
    pub left: Option<Vec<String>>,    // or Vec<StatusBarElementKind>
    pub right: Option<Vec<String>>,
    pub show_keybind_hints: Option<bool>,
}
```

### 2. JSON Schema & Settings UI

The `config-schema.json` (auto-generated by `schemars`) must include the
new `status_bar` sub-object. For Alternative A/D, the `left`/`right`
arrays need `enum` constraints listing valid element tags so the settings
UI can offer dropdowns, and so invalid tags are rejected.

### 3. DualList Settings Control

The spec calls for a new `DualList` picker in the settings panel. This is
a new control for `view/controls/`:

```
┌─ Available ──────────┐  ┌─ Left Side ───────────┐
│ {chord}              │  │ {filename}             │
│ {clock}              │  │ {cursor}               │
│ {keybind_hints}      │  │ {diagnostics}          │
│                      │  │ {cursor_count}         │
│                      │  │ {messages}             │
└──────────────────────┘  └────────────────────────┘
   Tab: switch column   Ctrl+↑/↓: reorder   Enter: move
```

**Complexity**: This is the most complex new UI control. It needs:
- Two scrollable columns with independent selection cursors.
- Move-between-columns (Enter/Tab).
- Reorder within column (Ctrl+Arrow).
- Mouse drag support (optional, can defer).
- Separate instances for left and right configuration.

**Alternative UI approach**: Instead of a full DualList, the existing
`TextList` control could be extended with drag-reorder support, plus a
dropdown to add elements from the available pool. This is simpler to
implement but less discoverable.

### 4. Multi-Line Status Bar (`lines: 2`)

When `lines: 2` and `show_keybind_hints: true`, the status bar occupies
2 rows. The layout in `app/render.rs` currently hardcodes
`Constraint::Length(1)` for the status bar. This must become dynamic:

```rust
Constraint::Length(if self.status_bar_visible {
    config.editor.status_bar.lines.max(1) as u16
} else {
    0
})
```

The keybind hints row is a separate render pass below the main status
line. This is straightforward regardless of which alternative is chosen.

### 5. The `{clock}` Element

A blinking-colon clock (`HH:MM`) requires periodic redraws. The editor
already has a tick-based event loop (`editor_tick` in `main.rs`). The
clock element would:
- Check `Instant::now()` on each render (already called at ~60 Hz when
  the terminal has focus).
- Toggle colon visibility every 500ms based on elapsed time.
- No additional timer needed.

### 6. Separator Handling

The current left side uses `" | "` as separators between elements. With
configurable ordering, separators should be:
- **Option 1**: Automatically inserted between consecutive visible
  elements (current behavior, just generalized). A separator element
  is *not* user-configurable.
- **Option 2**: An explicit `{separator}` element that users can place.
  More flexible but more verbose config.

**Recommendation**: Option 1 (auto-separators) for simplicity, with the
separator character configurable as a single string
(`separator: " | "`).

### 7. Backward Compatibility

All alternatives must produce identical output when using default config.
The `Default` impl for `StatusBarConfig` must match the current hardcoded
layout exactly. Existing configs without `status_bar` will get the
defaults via `#[serde(default)]`.

### 8. Mouse Hit Testing

The existing `StatusBarLayout` struct and `mouse_input.rs` click handling
assume specific elements exist at specific positions. With configurable
layout, click targets must be dynamically tracked. Alternatives A and D
handle this naturally (each element declares its click target).
Alternative B makes this harder (post-hoc positional scanning).

---

## Recommendation

**Alternative D (Data-Driven Enum)** offers the best balance for this
codebase:

1. **Matches the spec fully** — reordering, left/right assignment, multi-line.
2. **Simpler than A** — no trait objects, no registry, one match function.
   The fresh codebase doesn't have a plugin API that would need runtime
   element registration, so the extensibility of trait objects isn't needed
   yet.
3. **Compiler-enforced exhaustiveness** — adding a new element and
   forgetting to handle it is a compile error.
4. **Clean serde** — the enum serializes directly to/from JSON strings.
5. **Moderate scope** — less code than A, much more capable than C.

If plugin-extensible elements become a requirement later, the enum's match
arms can be extracted into trait implementations with minimal rework (the
`StatusBarContext` and `RenderedElement` types carry over unchanged).

### Suggested implementation order

1. Add `StatusBarConfig` to config + partial config + schema.
2. Extract `StatusBarContext` from the existing render function parameters.
3. Create `StatusBarElementKind` enum with serde.
4. Write `render_element()` by moving code out of the monolith one element
   at a time.
5. Rewrite `render_status()` as a loop over `config.left` / `config.right`.
6. Update `StatusBarLayout` tracking to work with dynamic positions.
7. Add `{clock}` and `{keybind_hints}` elements.
8. Add multi-line support (`lines: 2`) to the layout constraints.
9. Build the DualList control in `view/controls/`.
10. Wire the DualList into the settings UI for `status_bar.left` /
    `status_bar.right`.

Steps 1-6 can be done incrementally with each step producing a working
editor. Steps 7-8 add new functionality. Steps 9-10 are the settings UI,
which can ship separately.
