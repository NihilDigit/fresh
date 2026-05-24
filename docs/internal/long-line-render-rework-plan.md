# Long-Line Render Rework — Plan (Option 3)

**Status:** Design, with the gating experiment resolved (see §0). No production
code has been written for this plan yet. It supersedes
the deferred Section 7 ("Render-consumer path") of
[`line-wrap-cache-plan.md`](./line-wrap-cache-plan.md) and is scoped to make the
*renderer itself* a cache consumer, fix the long-single-line scroll/navigation
cap, and remove the `apply_wrapping_transform` O(n²) hot path. It is written to
be reviewed before any implementation lands.

This document deliberately spends most of its length on **invariants** and
**cache-poisoning failure modes**, because every prior attempt in this area
either drifted between two layout implementations or short-circuited the
renderer incorrectly (see the deferred Section 7). Getting the contracts
written down first is the point.

---

## 0. Resolved findings (run before any Change-B code)

### 0.1 `apply_wrapping_transform` is already linear (Change A is done)

Reading `view/ui/split_rendering/transforms.rs:149` shows the O(n²) per-chunk
`split_word_bound_indices` rescan is **already fixed**: the word-boundary list
is computed once per Text token (`transforms.rs:326`) and walked as a monotonic
`wb_lo` cursor across chunks. Only one `split_word_bound_indices` call remains,
lifted out of the chunk loop. **Change A requires no work.** The residual
per-frame cost is therefore *not* algorithmic-per-token; it is the **whole
window being re-tokenised and re-wrapped every frame** (objective 1) and the
**build cap** (objective 2). This re-points the effort entirely at Change B.

### 0.2 Wrapping is NOT chunk-local — but IS row-boundary-local (§8.1 resolved)

Empirically (throwaway probe over `apply_wrapping_transform`, since removed):

| Build start byte | Result vs. canonical (from-line-start) wrap |
|---|---|
| Arbitrary mid-line byte (e.g. 1000, mid-row) | **Diverges.** Fresh rows start 1000/1080/1160…; canonical rows there are 1040/1120/1200…. |
| A byte that is itself a canonical **row-boundary** (e.g. 400) | **Matches exactly.** |

Mechanism: `apply_wrapping_transform` carries `current_line_width` /
`on_continuation` across the whole logical line (reset only on `Newline`,
`transforms.rs:224`). A fresh build at byte X starts with
`current_line_width = 0`, i.e. it *assumes X begins a row*. That assumption is
true iff X is a canonical row boundary.

**Design consequences:**

- The §6.3 "`MAX_LINE_BYTES`-chunk-aligned independent build" mitigation is
  **wrong**: 100 000-byte chunk boundaries are not row boundaries, so
  concatenating per-chunk builds would corrupt wrapping at every seam.
- The correct anchor is a **row-boundary byte**. Any build (and therefore any
  legal `top_byte`) must start on a canonical row boundary. This is precisely
  what PR #2085's `Viewport::top_visual_row_source_byte` /
  `wrap_segment_source_bytes` (`viewport.rs:424-575`) compute — *modulo* their
  own `MAX_LINE_BYTES` cap inside `line_iterator.next_line`, which must be
  lifted or checkpointed for positions past 100 KB (see §6.3-revised).
- This makes **wrap-state checkpointing** the spine of Change B: cache the
  sequence of row-boundary bytes for a logical line so the renderer can (a)
  set `top_byte` to the row boundary at the viewport top and build a correct,
  bounded window from there (fixing the cap), and (b) reuse the checkpoint list
  across frames (fixing the per-frame rebuild).

---

## 1. Objectives

In priority order:

1. **Eliminate the per-frame full rebuild of view data** during pure
   cursor/scroll movement. Today `Editor::render → render_content →
   compute_buffer_layout → build_view_data` rebuilds the whole visible window's
   layout every keypress
   (`view/ui/split_rendering/orchestration/render_buffer.rs:156`,
   `view/ui/split_rendering/view_data.rs:32`). The dominant cost (~47% of a
   render-bound profile) is `build_view_data`; ~25% is grapheme/width work and
   ~6% is allocation churn — all discarded after drawing ~50 rows.

2. **Fix the long-single-line scroll/navigation cap.** For a buffer that is one
   newline-free line, `Viewport::top_byte` is pinned at the line start and the
   scroll position lives in `top_view_line_offset`. `build_base_tokens`
   (`view/ui/split_rendering/base_tokens.rs:14`) always tokenizes *from
   `top_byte`* and stops after `max_lines ≈ visible_count + 4` segments —
   inserting a `MAX_SAFE_LINE_WIDTH` (10 000-char) break per segment, so it only
   ever covers the first `~visible_count × 10_000` bytes. The viewport — and any
   cursor — therefore can never advance past that window into a single line. The
   committed reproducer
   `crates/fresh-editor/tests/semantic/move_up_at_eof_single_long_line.rs`
   (`#[ignore]`d) pins the user-visible symptom: `MoveDocumentEnd` then `MoveUp`
   strands the caret at EOF.

3. **Make `apply_wrapping_transform` linear** in the bytes it processes. The
   existing line-wrap-cache plan's "Deeper issue not addressed" section records
   an O(n²) hot path: per-chunk `split_word_bound_indices` scans from byte 0 of
   the token. This is the first-hit cost the cache cannot hide.

Non-objectives (explicitly out of scope so the change stays bounded):

- The `render_view_lines`/`compute_char_style` per-cell overlay-stabbing hot
  path — covered separately by [`render-pipeline-perf-plan.md`](./render-pipeline-perf-plan.md).
- Changing the large-file (byte-offset) threshold or its semantics
  (`DEFAULT_LARGE_FILE_THRESHOLD = 100 MiB`,
  `model/buffer/mod.rs:86`). See §3.4 — long lines *under* the threshold are the
  primary target; files over it bypass wrap math entirely today and that stays
  true unless §8 decides otherwise.
- Plugin `view_transform` buffers — they keep their non-cached path.

### Success criteria

- A pure-scroll frame (cursor/scroll moved, no buffer/geometry/plugin-state
  change) does **zero** calls into `apply_wrapping_transform` /
  `ViewLineIterator` for rows it already laid out — verified by a counter
  assertion in a layout-scenario test, not just a wall-clock number.
- `MoveDocumentEnd` then repeated `MoveUp`/`MoveDown` on a ≥500 KB single line
  moves the caret one visual row per press and the viewport follows
  (the `#[ignore]` comes off `move_up_at_eof_single_long_line.rs`).
- Re-captured tmux latency on the 600 KB single-line fixture: Down ≈ baseline
  (tens of ms), not ~290 ms. 100 MB jump-to-end and per-row move drop from
  ~12–28 s to interactive.
- **No regression** in the wrap-navigation suite (`migrated_issue_1574_*`,
  `migrated_ctrl_end_wrapped`, `page_motion_single_long_line`,
  `scroll_wrapped_reach_last_line`, `line_wrap_cache_consistency`).

---

## 2. Current architecture (what we are building on)

### 2.1 The render pipeline (per frame)

```
compute_buffer_layout (render_buffer.rs:91)
  └─ build_view_data (view_data.rs:32)
       ├─ build_base_tokens(top_byte, …, max_lines)         base_tokens.rs:14
       │     └─ buffer.line_iterator(cursor, est_len)        primitives/line_iterator.rs
       ├─ apply_soft_breaks  (Compose, if any)               transforms.rs
       ├─ apply_conceal_ranges (Compose, if any)             transforms.rs
       ├─ apply_wrapping_transform(effective_width, …)       transforms.rs  ← O(n²)
       └─ ViewLineIterator::collect → Vec<ViewLine>          view_pipeline.rs
  └─ ensure_visible_in_layout (+ at most ONE rebuild)        viewport.rs:1133
```

`build_view_data` builds the **whole window from `top_byte`**, then
`top_view_line_offset` slices off the rows above the viewport. The render
machinery also publishes `LayoutCache.view_line_mappings`
(`app/types.rs:1269`, written in `app/render.rs:929` / `:3397`) — the per-row
`ViewLineMapping`s that `find_visual_row` / `byte_to_visual_column` /
`move_visual_line` read for cursor navigation (`app/types.rs:1274-1383`).

### 2.2 Two existing caches on `EditorState`

| Cache | Granularity | Key | Value | Used by | Used by renderer? |
|---|---|---|---|---|---|
| `LineWrapCache` (`view/line_wrap_cache.rs`) | one **logical line** | `LineWrapKey` = `(pipeline_inputs_version, view_mode, line_start, effective_width, gutter_width, wrap_column, hanging_indent, line_wrap_enabled)` | `Arc<Vec<ViewLine>>` | scroll math, cursor nav, scrollbar (via `layout_for_line` miss handler) | **No** — only *written* by the renderer (`view_data.rs:201-295`), never *read* by it |
| `VisualRowIndex` (`view/visual_row_index.rs`) | whole buffer | `VisualRowIndexKey` (geometry minus `line_start`) | prefix-sums of per-line row counts + line-start bytes | `total_rows`, `position_at_row`, `line_for_byte` (scrollbar/scroll math) | No |

Both are keyed on `pipeline_inputs_version` — a packed `u64` folding
`buffer.version()`, `soft_breaks.version()`, `conceals.version()`,
`virtual_texts.version()` (`line_wrap_cache.rs:102-113`). Both age out via FIFO
byte-budget eviction (8 MiB default). **This invalidation scheme is sound and
must be reused verbatim** — see §5.

### 2.3 Why the renderer is *not* a cache consumer today

From `line-wrap-cache-plan.md` Section 7 (deferred), confirmed by reading
`view_data.rs:201-295`:

- **Trailing end-of-buffer artefact.** `ViewLineIterator` emits a synthetic
  trailing empty row at buffer end (`at_buffer_end` rule). The per-logical-line
  cache entries don't model it, so a naive "all visible lines are cached → skip
  the build" returns fewer rows than the viewport expects and
  `ensure_visible_in_layout` mis-clamps.
- **Partial-window coverage.** The writeback only stores **complete logical-line
  groups** — a group must begin with `LineStart::Beginning` or
  `AfterSourceNewline` (`view_data.rs:247-258`). A window scrolled into the
  *middle* of a long wrapped line starts on an `AfterBreak` continuation row, so
  **nothing is stored** for that window. This is also why the long-line case
  currently gets *no* cache benefit.

### 2.4 The long-single-line cap (root of objective 2)

Confirmed empirically (instrumented build, tmux): a 30-row pane caps the
rendered window of one line at ~310 KB; `find_visual_row(EOF)` returns `None`
for a 400 KB line because the EOF cursor is beyond the tokenized window;
`move_visual_line`/`byte_to_visual_column` bail; the byte-based `MoveUp`
fallback's `LineIterator::prev` is `None` for a one-line buffer ⇒ no
`MoveCursor` event ⇒ Up is a no-op. 300 KB works (fits the window); 400 KB+
doesn't. The cap = `max_lines × MAX_SAFE_LINE_WIDTH` driven by
`base_tokens.rs:92` counting a break per 10 000 chars against
`max_lines = visible_count + 4`.

### 2.5 The `MAX_LINE_BYTES` chunking interaction (subtle, load-bearing)

`LineIterator::next_line` caps each emission at `MAX_LINE_BYTES = 100 000`
(`primitives/line_iterator.rs:28`) and `find_line_start_backward` scans back to
the physical line start — byte 0 for a newline-free line
(`line_iterator.rs:46`). So a single 600 KB line is, to the renderer, a
sequence of 100 KB pseudo-"lines" all sharing the same physical line start.
Any sub-line caching scheme **must** agree with these chunk boundaries or it
will disagree with `top_byte` (which can only legally sit at a chunk-aligned
position the renderer can build from). This is the single biggest source of
potential off-by-one / poisoning bugs in this work.

---

## 3. Root causes (precise)

1. **Build-from-line-start + capped window** (§2.4). The renderer can only see
   `[top_byte, top_byte + ~max_lines·10k)` and `top_byte` is pinned at the line
   start for a single line. Fix requires the build to start *near the scrolled
   position*, not the line start.

2. **Cache granularity mismatch.** `LineWrapCache`'s unit is "one logical line".
   For a 100 MB line that is one entry holding millions of `ViewLine`s — neither
   buildable nor cacheable. The long-line path needs a **sub-line** unit
   (per wrap-segment-run / per chunk), which today's key (`line_start`) cannot
   express.

3. **`apply_wrapping_transform` O(n²)** on long single tokens (per-chunk
   `split_word_bound_indices` from byte 0). Even with perfect caching, the first
   touch of a deep region pays this.

---

## 4. Invariants & properties to maintain (the contract)

These must hold **after** the rework. Violating any of them is a release
blocker. They are phrased so each can be turned into an assertion or property
test.

### 4.1 Single source of truth

- **P1 — Cache≡pipeline.** For any key, the cached value equals what the full
  pipeline (`build_base_tokens → apply_soft_breaks → apply_conceal_ranges →
  apply_wrapping_transform → ViewLineIterator`) produces for the same inputs.
  This is the existing `LineWrapCache` guarantee (`line_wrap_cache.rs:9-20`) and
  must extend to any new sub-line entries. *There must remain exactly one wrap
  implementation.* No re-introduction of a char-count shadow wrap.

- **P2 — Reader/renderer agreement.** The `ViewLine`s the renderer draws, the
  `ViewLineMapping`s in `LayoutCache.view_line_mappings`, and the cached
  `Vec<ViewLine>` for the same rows are byte-for-byte consistent
  (`source_start_byte`, `char_source_bytes`, `line_end_byte`, visual cols).
  Cursor nav (`find_visual_row`, `byte_to_visual_column`, `move_visual_line`)
  and rendering must never disagree about where byte X sits visually.

### 4.2 Coordinate-mapping totality

- **P3 — Every in-buffer byte resolves.** For any cursor position `p ∈
  [0, buffer_len]` that is *on screen after `ensure_visible`*,
  `find_visual_row(p)` is `Some`. The EOF position `p == buffer_len` must resolve
  (today it fails because the last wrapped row's `line_end_byte` is the last
  char, not one past — `app/types.rs:1141-1149`). The rework must guarantee an
  EOF-bearing row exists in the window whenever the caret is at EOF.

- **P4 — Vertical motion is total within a logical line.** On a wrapped line,
  `MoveUp` from any non-first visual row yields a strictly smaller byte;
  `MoveDown` from any non-last visual row yields a strictly larger byte; round
  trips are stable up to sticky-column semantics. This must hold even when the
  target row is off-screen (the bug in objective 2).

### 4.3 Scroll reachability

- **P5 — Last row reachable.** `max_scroll` must allow the final visual row of
  the buffer to reach the viewport (the original under-scroll bug,
  `scroll_wrapped_reach_last_line.rs`). For a single long line this means
  `top_byte` must be able to advance to the line's final chunk.

- **P6 — No over-scroll.** Scrolling never leaves the viewport showing rows
  past EOF beyond the single intentional trailing empty row.

### 4.4 Determinism & monotonicity (cheap to property-test)

- **P7 — Determinism.** Same inputs → same layout, always
  (`line_wrap_cache.rs:1083`).
- **P8 — Width monotonicity.** Widening `effective_width` never increases a
  line's row count (`line_wrap_cache.rs:1000`).
- **P9 — Prefix monotonicity.** A prefix of a line never has more rows than the
  line (`line_wrap_cache.rs:1062`).
- **P10 — ≥1 row.** Every logical line is at least one visual row.

### 4.5 Memory & liveness

- **P11 — Bounded memory.** Cache stays within its byte budget after every
  insert (`line_wrap_cache.rs:35-38`). The rework must not let a single 100 MB
  line's layout blow the budget (it would, if cached whole — see §6.4).
- **P12 — Progress.** A render must always make forward progress (produce a
  full viewport of rows or hit a genuine buffer boundary); no path may return a
  short window that causes `ensure_visible` to oscillate
  (the Section-7 mis-clamp).

---

## 5. Invalidation — reuse, do not reinvent

The existing version scheme is correct and **must be reused exactly**:

- `pipeline_inputs_version(buffer, soft_breaks, conceal, virtual_text)`
  (`line_wrap_cache.rs:102`) folds all mutable pipeline inputs into the key.
  Any new sub-line key type **must** carry it.
- Geometry (`effective_width`, `gutter_width`, `wrap_column`, `hanging_indent`,
  `line_wrap_enabled`, `view_mode`) are key fields. Resizing/wrap-toggling
  flips them; old entries become unreachable.
- There is **no active invalidate step** — staleness is impossible by
  construction because a changed input produces a different key, and old keys
  age out via FIFO. **Preserve this property.** Do not add an imperative
  "invalidate line N" API: it is the classic source of cache bugs (missed
  invalidation = poison). The whole-key-versioning approach is what makes this
  cache safe; the rework must extend it, not bypass it.

> **Hard rule:** if a new input begins to affect layout (e.g. a future
> per-line decoration that changes wrap), it must be folded into
> `pipeline_inputs_version` *in the same commit*. The
> `every_key_dimension_separates_entries` and
> `pipeline_inputs_version_changes_when_any_source_changes` tests
> (`line_wrap_cache.rs:918,1214`) are the guard; extend them when the key grows.

---

## 6. Cache-poisoning & caching-bug failure modes (the main event)

Each is a concrete way this rework could ship a correctness bug. For each:
*mechanism → symptom → mitigation → test.*

### 6.1 Partial-window write poisons a "full line" key

- **Mechanism.** The long-line fix will want to cache *sub-line* runs (a window
  scrolled into the middle of a giant line). If such a partial run is ever
  stored under a key a *whole-line* reader would query (the current
  `line_start`-keyed `LineWrapKey`), the reader gets a fragment and believes it
  is the entire line.
- **Symptom.** Row counts too low → under-scroll / `VisualRowIndex` totals wrong
  → scrollbar thumb wrong, last line unreachable (P5 violated).
- **Mitigation.** Sub-line entries get a **distinct key type** (e.g.
  `LineSegmentKey { line_start, segment_start_byte, … }` or a chunk index),
  never the whole-line `LineWrapKey`. The whole-line cache keeps its current
  invariant: an entry under `line_start` is the *complete* logical line or it is
  absent. The renderer's writeback already enforces "complete groups only"
  (`view_data.rs:276`); keep that gate and add the sub-line cache as a *separate*
  map.
- **Test.** Shadow-model: any `LineWrapKey` hit must equal the full
  `compute_line_layout` for that line; assert length == `VisualRowIndex`
  per-line count. A partial write under a whole-line key fails immediately.

### 6.2 The end-of-buffer trailing-row artefact (Section 7's killer)

- **Mechanism.** `ViewLineIterator` synthesises a trailing empty row at
  `at_buffer_end`; cache entries don't carry it. A render that assembles its
  window purely from cached rows omits it.
- **Symptom.** Window one row short at EOF → `ensure_visible_in_layout`
  mis-clamps (P12), cursor-at-EOF unresolvable (P3).
- **Mitigation.** Model the trailing row explicitly: either (a) the assembled
  window always appends the synthetic EOF row when `top_byte + Σ rows` reaches
  `buffer_len`, computed outside the cache; or (b) the cache entry for the *last*
  logical line/segment includes the trailing row and the key carries an
  `is_buffer_tail: bool` so a tail entry is never served as a non-tail.
  Option (a) is simpler and keeps the artefact out of the cache (recommended).
- **Test.** A scenario that scrolls to EOF on a wrapped buffer and asserts the
  EOF row exists and `find_visual_row(buffer_len)` is `Some`.

### 6.3 (revised) Build anchored off a non-row-boundary byte

> Revised after §0.2: the original "chunk-aligned" framing was wrong.

- **Mechanism.** A build (hence a legal `top_byte`) that starts on a byte that
  is **not** a canonical row boundary produces wrapping that diverges from the
  from-line-start canonical wrap (proven in §0.2). `MAX_LINE_BYTES` (100 KB)
  multiples are *not* row boundaries.
- **Symptom.** Duplicated/skipped rows; `char_source_bytes` off; the ~100 KB
  undershoot observed for the 400 KB EOF case; cursor lands on the wrong row.
- **Mitigation.** **Anchor every build and every `top_byte` to a canonical
  row-boundary byte.** Maintain a per-logical-line **checkpoint list** of
  row-boundary bytes (the cache value, or a sidecar). To render a window whose
  top is `top_view_line_offset` rows into the line, look up the row-boundary
  byte for that offset from the checkpoint list, set `top_byte` to it, reset
  `top_view_line_offset = 0`, and build forward — this build is correct because
  it starts on a row boundary. The checkpoint list itself is produced by a
  single canonical wrap from the line start; for lines longer than the build
  cap it must be produced incrementally and **persisted past `MAX_LINE_BYTES`**
  (today `line_iterator.next_line` caps at 100 KB and `build_base_tokens` caps
  at `max_lines` — both must be bypassed on the checkpoint-building path, which
  is the one place we accept an O(line length) pass, amortised by caching).
- **Open sub-question.** Building the checkpoint list for a 100 MB line is an
  O(100 MB) pass. Acceptable once (cached), but the byte budget can't hold the
  full `Vec<ViewLine>`; the checkpoint list (just `Vec<usize>` of row starts) is
  ~8 bytes/row ≈ a few MB for ~1M rows — bounded enough to keep even when the
  materialised `ViewLine`s are evicted. Treat the checkpoint list and the
  materialised rows as **two tiers**: cheap-and-kept row-boundary index vs.
  evictable full layout.
- **Test.** Property: for a single long line, rendering a window after setting
  `top_byte` to a checkpoint row-boundary equals a fresh from-line-start
  canonical wrap sliced at the same offset, byte-for-byte, swept across many
  offsets — including offsets past 100 KB and at EOF.

### 6.4 Unbounded entry size for a giant line

- **Mechanism.** Caching "one logical line" for a 100 MB line is one
  multi-gigabyte `Vec<ViewLine>`. Even byte-budget eviction can't help — a
  single entry dwarfs the 8 MiB budget, and `insert_fresh` accepts oversize
  entries (`line_wrap_cache.rs:246-258`).
- **Symptom.** OOM / pathological eviction churn (every insert evicts
  everything).
- **Mitigation.** The long-line path must **only ever cache bounded sub-line
  windows** (a few viewports' worth of rows), never whole giant lines. Add a
  guard: if a logical line exceeds N chunks, it is *only* eligible for the
  sub-line cache, never the whole-line cache. Document the per-entry size
  ceiling and assert it.
- **Test.** Insert a synthetic huge line; assert no single entry exceeds a
  ceiling and `current_bytes <= byte_budget`.

### 6.5 Version-key omission (the classic)

- **Mechanism.** A new layout input is added but not folded into
  `pipeline_inputs_version` / the key.
- **Symptom.** Stale layout served after the input changes — silent, intermittent,
  the worst kind.
- **Mitigation.** §5 hard rule. Additionally: keep the writeback's gate
  (`!has_view_transform && line_wrap_enabled && fold_skip.is_empty() &&
  virtual_texts.is_empty()`, `view_data.rs:201`) — these are inputs the per-line
  key does *not* fully capture (fold geometry, injected virtual rows), so the
  renderer refuses to cache when they're active. The sub-line cache must apply
  the **same gate**. Do not loosen it without folding the missing inputs into
  the key.
- **Test.** The existing `version_bump_makes_old_entry_unreachable` and
  `every_key_dimension_separates_entries`, extended to the new key type.

### 6.6 Arc aliasing / interior mutation

- **Mechanism.** Cache stores `Arc<Vec<ViewLine>>`; a consumer that mutates a
  `ViewLine` in place (e.g. fold placeholder, virtual-line injection,
  per-frame style overlay) through a cloned `Arc` would corrupt the shared
  cached value for every other reader.
- **Symptom.** Cross-frame bleed — a fold/overlay from one frame appears on an
  unrelated later frame.
- **Mitigation.** Cached `Vec<ViewLine>` is **immutable layout only**. All
  post-layout reshaping (`inject_virtual_lines`, `apply_folding`, style/overlay
  application — `view_data.rs:297-315`) must run on a *clone* or downstream of
  the cache, never mutate the cached Arc. This is why the writeback caches
  `source_lines` *before* `inject_virtual_lines`/`apply_folding`
  (`view_data.rs:168-176` then `:297`). Preserve that ordering: **cache the
  pre-fold, pre-virtual-text, pre-overlay layout.**
- **Test.** A scenario that renders a folded frame then an unfolded frame of the
  same line and asserts the unfolded frame has no placeholder rows.

### 6.7 `VisualRowIndex` / `LineWrapCache` divergence

- **Mechanism.** `VisualRowIndex` derives per-line counts; if the renderer
  starts writing sub-line entries, the index's "line i has K rows" can disagree
  with the sum of sub-line entries.
- **Symptom.** Scrollbar thumb size / `position_at_row` wrong (P2).
- **Mitigation.** `VisualRowIndex` must continue to be built from *whole-line*
  counts (`count_visual_rows_for_text*`), independent of the sub-line cache, OR
  be taught to sum sub-line entries — pick one and assert equality between the
  two in tests. Do not let them silently derive from different sources.
- **Test.** `VisualRowIndex.line_row_count(i)` equals
  `layout_for_line(line_i).len()` for all i, swept across widths.

---

## 7. Proposed approach + staff-level review

This is the recommended shape; alternatives and their trade-offs are in §7.4.

### 7.1 Two independent changes, sequenced

**Change A — Linearise `apply_wrapping_transform` (do this first, alone).**
It is a *pure-function* optimization with no caching semantics: walk
word-boundary indices once as a monotonic cursor instead of re-scanning each
chunk from byte 0. It is independently testable (P7–P10 already exist as
property tests), independently shippable, and de-risks everything after it (the
"first-hit" cost that the cache can't hide). **Recommend landing and measuring
this before touching any cache wiring.** It may, on its own, bring the 600 KB
case from ~290 ms to acceptable — in which case Change B's urgency drops.

**Change B — Renderer consumes a sub-line layout cache.** Introduce a sub-line
cache unit aligned to `build_base_tokens` chunk-anchored windows (§6.3) and
have `build_view_data` assemble its window from cached chunk entries on a
pure-scroll frame, building only the chunks not present. The long-line cap
(objective 2) falls out *if and only if* `top_byte` is allowed to advance to
deep chunk-aligned positions; that scroll change (`ensure_visible_in_layout`
/ `snap_to_logical_line_start`) is part of Change B.

### 7.2 Complications a reviewer should push on

1. **Does the renderer actually reset wrapping at chunk boundaries?** §6.3's
   mitigation assumes building from `top_byte = chunk_start` produces the same
   rows as the corresponding slice of a build from byte 0. Word-wrap is
   *cumulative within a build* (greedy from the start), so this is only true if
   each `build_base_tokens` invocation already restarts wrapping at its
   `top_byte`. **This must be verified empirically before committing to the
   chunk-aligned cache** — if wrapping is not chunk-local, the entire sub-line
   approach needs a different segmentation (e.g. re-wrap a lookback window).
   This is the highest-risk unknown in the plan.

2. **`top_byte` advancement changes scroll math.** Allowing `top_byte` to sit at
   deep chunk boundaries on a single line touches `snap_to_logical_line_start`
   (`viewport.rs:1327`), `calculate_view_anchor`, and the `top_view_line_offset`
   bookkeeping that issue #1574's tests guard. Expect to spend most of the
   review/test budget here, not on the cache primitive.

3. **Byte-offset (large-file) mode.** Files >100 MiB bypass wrap math; the 100 MB
   reproducer is in that mode. The plan's scroll fix must decide whether to (a)
   leave byte-offset mode as-is (the 100 MB Up-at-EOF stays broken, only <100 MB
   long lines are fixed) or (b) extend chunk-anchored building into byte-offset
   mode. Recommend (a) for this rework and a follow-up for (b) — scope control.

4. **`ensure_visible`'s single rebuild.** `compute_buffer_layout` does at most
   one scroll+rebuild per frame (`render_buffer.rs:225-248`). If the cache makes
   rebuilds cheap, consider whether a bounded *loop* (scroll → rebuild → re-check)
   is now affordable to converge the viewport on the caret in one frame instead
   of dribbling one chunk per keypress. Tempting, but it interacts with P12
   (oscillation) — gate behind a strict iteration cap and a "monotonic progress"
   assertion.

### 7.3 Simplification opportunities (reduce surface area)

- **Lean on the existing primitive.** `LineWrapCache` already has byte-budget
  eviction, Arc sharing, version keying, and a battle-tested test suite. The
  sub-line cache should be the *same* `LineWrapCache` machinery with a richer
  key, not a new structure — minimizing new eviction/aliasing bugs (§6.4, §6.6).
- **Keep the writeback gate.** Don't try to cache fold/virtual-text/transform
  frames. The gate at `view_data.rs:201` already encodes "only cache when the
  per-line key fully describes the output." Reuse it unchanged for sub-line
  writes.
- **Reuse `wrap_segment_source_bytes`.** PR #2085 already added
  `Viewport::top_visual_row_source_byte` and `wrap_segment_source_bytes`
  (`viewport.rs:424-575`) which map wrap-segment offsets ↔ bytes from the
  buffer. The off-screen vertical-move and the chunk-window assembly can build
  on these rather than inventing new wrap-segment math — *provided* §7.2.1 holds.
- **Consider doing only Change A.** If linearising `apply_wrapping_transform`
  plus the already-merged PR #2085 navigation fixes bring interactive latency to
  acceptable for realistic file sizes, Change B (the risky scroll/cache rework)
  may be deferrable. Measure after Change A before committing to B.

### 7.4 Alternatives considered

| Option | Pro | Con |
|---|---|---|
| **A only** (linearise wrap) | Smallest, safest, pure-function; big constant-factor win | Doesn't fix the *cap* (objective 2) — a line longer than the window is still unreachable, just faster to fail |
| **A + B sub-line cache** (recommended) | Fixes all three objectives | Highest risk in scroll math + the chunk-locality assumption (§7.2.1) |
| **Cache whole `ViewData` window** keyed by `(top_byte, top_view_line_offset, geometry, version)` | Conceptually simple; matches the "skip build on pure-scroll frame" idea directly | Key churns every scroll row (offset changes) → near-zero hit rate during scrolling, the exact workload we're optimizing; also doesn't fix the cap |
| **Re-tokenize from a lookback window near `top_byte`** instead of chunk-aligned | Avoids the chunk-locality assumption | Needs a correct lookback length to reproduce wrapping; reintroduces "build from somewhere" ambiguity |

---

## 8. Open questions (resolve before implementation)

1. ~~**Is `apply_wrapping_transform` wrapping chunk-local?**~~ **RESOLVED (§0.2):
   not chunk-local; row-boundary-local.** Builds must anchor on canonical
   row-boundary bytes; the segmentation strategy is now checkpoint-based
   (§6.3-revised), not `MAX_LINE_BYTES`-chunk-based.
2. **Byte-offset mode:** in-scope or follow-up? (§7.2.3) Recommend follow-up.
3. **Does `VisualRowIndex` stay whole-line-derived or sum sub-line entries?**
   (§6.7) Recommend whole-line-derived + an equality assertion.
4. **Single-rebuild vs bounded-loop `ensure_visible`?** (§7.2.4) Recommend
   keeping single rebuild for this rework; revisit if per-keystroke chunk
   dribble proves bad UX.

---

## 9. Testing strategy (gates per change)

Layer the existing `line_wrap_cache.rs` test taxonomy (Layers 1–7) and add:

- **Change A:** property tests P7–P10 already exist; add a direct
  byte-for-byte equivalence test (old vs new wrap on adversarial long tokens,
  multibyte, tabs, trailing spaces) + a complexity guard (a length-doubling
  input must not >2.5× the work — coarse, but catches accidental quadratic
  regressions).
- **Change B:**
  - **P2 render-vs-reader:** after a render, every assembled row equals a fresh
    mini-pipeline build for that row (extends `line_wrap_cache_consistency.rs`).
  - **P3/P4:** un-`#[ignore]` `move_up_at_eof_single_long_line.rs`; add
    `MoveDown` symmetry and a width/height sweep.
  - **§6.3 chunk-seam property:** assemble-from-cache == fresh-build swept over
    `top_byte`/`top_view_line_offset`.
  - **§6.2 EOF artefact:** scroll-to-EOF on a wrapped buffer; assert the
    trailing row and `find_visual_row(buffer_len) == Some`.
  - **§6.6 aliasing:** folded-then-unfolded frame of the same line shows no
    placeholder bleed.
  - **Pure-scroll zero-rebuild counter:** instrument the pipeline with a
    per-frame "wrap calls" counter; a pure-scroll frame over already-laid-out
    rows asserts zero.
  - **No-regression:** the full wrap-nav suite (§1 success criteria).

All tests follow `CONTRIBUTING.md`: observe rendered/`primary_caret` state, no
internal-state inspection beyond the existing test API, semantic waiting (no
timeouts), per-test isolation.

---

## 10. Implementation order (each a self-contained commit)

1. **This doc.**
2. ~~Answer §8.1~~ **DONE (§0).** Change A is already in tree; §8.1 resolved:
   anchor on row boundaries, not chunks.
3. **Change B-1 — checkpoint index.** Add a per-logical-line **row-boundary
   checkpoint list** (`Vec<usize>` of row-start bytes), built by a single
   canonical wrap from the line start with the build cap lifted *only on this
   path*. Two-tier with the existing `LineWrapCache` (cheap index kept; full
   `ViewLine`s evictable). Assert P1/P7-P10 + the §6.3 row-boundary equivalence
   property. No behaviour change yet (index built, not consumed).
4. **Change B-2 — renderer builds off a row-boundary `top_byte`.** For a single
   long line, resolve the viewport-top row offset to its checkpoint byte, set
   `top_byte` to it, reset `top_view_line_offset = 0`, build forward. Handle the
   EOF trailing artefact per §6.2. Zero-rebuild counter test for pure scroll.
5. **Change B-3 — scroll/`ensure_visible` reach the cap-free region.** Let the
   viewport advance to deep row boundaries; fix `snap_to_logical_line_start` /
   `calculate_view_anchor` interactions; un-`#[ignore]` the reproducer; full
   nav-suite gate (issue #1574 et al.).
6. **Cleanup:** delete any now-dead `wrap_line`/`compute_wrap_row_count_for_text`
   remnants flagged by the original line-wrap-cache plan.

> Byte-offset (>100 MiB) mode stays out of scope (§7.2.3): the 100 MB Up-at-EOF
> case remains a documented follow-up; this rework fixes long lines under the
> large-file threshold.
