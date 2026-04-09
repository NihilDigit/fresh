# WIP: review-diff drill-down close should return to the buffer group

## Goal (LLM prompt)

You are picking up an in-progress fix in the `fresh` editor. Read this whole
file before touching code.

The user reported two related bugs in the `audit_mode` plugin
(`crates/fresh-editor/plugins/audit_mode.ts`), which renders a magit-style
"Review Diff" buffer group with a files panel + diff panel:

1. **Drill down + close lands on the wrong buffer.** From the review-diff
   group, pressing `Enter` on a file row triggers `review_drill_down`,
   which builds a side-by-side composite buffer (`*Diff: <file>*`) and
   `editor.showBuffer`s it. Pressing `q` in the composite (bound to the
   `close` editor action via the `diff-view` mode) closes the composite,
   but the active buffer becomes whatever was previously focused
   (typically `main.rs`) **instead of returning to the `*Review Diff*`
   buffer group tab the user came from**.

2. **With no other buffers open, closing the composite opens `[No Name]`.**
   Same root cause: the close-buffer replacement search ignores
   `TabTarget::Group(...)` entries in the active split's `open_buffers`
   list, so it falls through to "create a new empty buffer".

A failing e2e test reproducing bug 1 is in
`crates/fresh-editor/tests/e2e/plugins/audit_mode.rs` —
`test_review_diff_drill_down_close_returns_to_group` — currently marked
`#[ignore]` so it doesn't break CI. Run it with:

```
cargo nextest run -p fresh-editor --test e2e_tests \
    test_review_diff_drill_down_close_returns_to_group --run-ignored=only
```

## What we already tried

The current fix attempt is in
`crates/fresh-editor/src/app/buffer_management.rs` inside
`close_buffer_internal`:

```rust
let prefer_group_tab: Option<crate::model::event::LeafId> =
    self.split_view_states.get(&active_split).and_then(|vs| {
        use crate::view::split::TabTarget;
        let pos = vs.open_buffers.iter()
            .position(|t| matches!(t, TabTarget::Buffer(b) if *b == id))?;
        if pos > 0 {
            if let TabTarget::Group(leaf_id) = vs.open_buffers[pos - 1] {
                return Some(leaf_id);
            }
        }
        None
    });
// ... existing replacement-buffer logic runs ...
if let Some(group_leaf) = prefer_group_tab {
    self.activate_group_tab(group_leaf);
}
```

The intent: if the closing buffer is preceded in tab order by a Group tab
in the active split's `open_buffers`, activate that group tab after the
existing close cleanup. `activate_group_tab`
(`crates/fresh-editor/src/app/buffer_groups.rs:394`) sets
`vs.active_group_tab = Some(group_leaf)` and `vs.focused_group_leaf = Some(...)`,
which makes `active_target()` return `Group(...)` regardless of the
underlying `active_buffer`.

**It doesn't work.** Manual reproduction in tmux shows that after `q`, the
active buffer is still `main.rs` (the file the user opened before running
Review Diff). Debug tracing shows
`prefer_group_tab=None` — i.e. the closing buffer is NOT found in the
active split's `open_buffers` list, OR the previous `TabTarget` is not a
`Group`.

There's a `tracing::debug!` line in `close_buffer_internal` that dumps
`active_split`, `open_buffers`, and `closing` — but in the latest log
(`/home/noam/.local/state/fresh/logs/`) only the **stale-state cleanup**
calls were captured. The actual user-triggered `q` keypress on the
composite never reached `close_buffer_internal`. That's the next mystery
to investigate (see "Next steps" below).

## Suspicions / theories

1. **`q` may not actually be calling `close_buffer_internal` at all.** The
   diff-view mode binds `q` to the `close` action, which should map to
   `Action::Close → close_tab() → close_buffer() → close_buffer_internal()`.
   But the trace shows no call. Possible explanations:
   - The composite buffer's `close_tab` path takes a different branch
     (e.g. `close_active_split` or `close_buffer_group_by_leaf`) before
     reaching `close_buffer_internal`.
   - The `q` keypress is being routed to a different mode/context (e.g.
     the file explorer if it's still focused, or some popup).
   - The composite buffer has a custom close handler in the composite
     buffer machinery that bypasses the regular path.

2. **`set_active_buffer` clears `active_group_tab`** at
   `crates/fresh-editor/src/app/mod.rs:2477`. This is called when
   `editor.showBuffer(compositeBufferId)` runs from `review_drill_down`.
   So at the time `q` is pressed:
   - `vs.active_group_tab = None` (cleared by drill-down)
   - `vs.focused_group_leaf = None` (cleared by drill-down)
   - `vs.active_buffer = composite_id`
   - `vs.open_buffers` should still contain the `Group(review_diff_leaf)`
     entry from before drill-down + the new `Buffer(composite_id)` entry

   The `prefer_group_tab` check assumes the order is `[..., Group(review_diff_leaf), Buffer(composite_id)]`,
   but it's possible the composite is added in a different order or
   under a different split. **Verify this.**

3. **Active split may differ.** When `q` is pressed on the composite,
   `split_manager.active_split()` returns... what? If the composite is in
   the same split as the review-diff group's host (the main split tree
   leaf), then `active_split == main_leaf` and the open_buffers check
   should work. If the composite is in a separate split (which it shouldn't
   be, but verify), the check looks at the wrong split.

## Next steps for the next agent

Tackle these in order:

1. **Find out why `close_buffer_internal` isn't being called for the
   composite `q` press.** Add `tracing::debug!` to `close_tab()`
   (`crates/fresh-editor/src/app/buffer_management.rs:1896`) at every
   branch — the active_group_tab branch, the is_last_viewport branch, the
   non-last branch — so the next reproduction shows which path the close
   actually takes. If `close_tab` is called but `close_buffer_internal`
   isn't, the close goes through `close_active_split` or similar, and
   we need to handle the group preference there too.

2. **Reproduce manually in tmux against a clean repo.** The shared
   `/tmp/fresh-drill-test` repo accumulated stale state from earlier
   sessions and made the manual repro flaky. Make a fresh temp repo each
   time:
   ```bash
   D=$(mktemp -d) && cd "$D" && git init -q && mkdir src && \
       echo 'fn main() { println!("hello"); }' > src/main.rs && \
       git add . && git -c user.email=t -c user.name=t commit -q -m i && \
       echo 'fn main() { /* changed */ }' > src/main.rs && \
       /home/noam/repos/fresh/target/debug/fresh src/main.rs
   ```
   Then in the editor: `Ctrl+P`, type "Review Diff", `Enter`, `Enter` to
   drill down, `q` to close. Verify the active buffer in the status bar
   (no, **don't assert against status bar in tests** — see
   `feedback_test_no_status_bar.md` in your memory — but it's fine for
   manual debugging).

3. **Look at the latest fresh log under
   `/home/noam/.local/state/fresh/logs/fresh-PID.log`** to see what
   `close_buffer_internal` traces showed during the manual repro. Match
   PIDs carefully — the harness writes one log per process.

4. **Once `close_buffer_internal` IS being called and the open_buffers
   trace shows the expected `[..., Group, Buffer(composite)]` shape**,
   verify `prefer_group_tab` is set. If yes but the test still fails,
   look at what runs AFTER `activate_group_tab` — possibly something is
   undoing the group activation. Candidates: the existing replacement
   buffer switch (`set_active_buffer(replacement_buffer)`), the
   `set_split_buffer` calls, or the `remove_buffer(id)` loop. The
   `activate_group_tab` call needs to either run AFTER all of those, or
   they need to be skipped when `prefer_group_tab.is_some()`.

5. **If `prefer_group_tab` keeps coming back None**, instrument the
   composite buffer creation path (`composite_buffer_actions.rs:231`,
   `handle_show_buffer` in `plugin_commands.rs:981`) to confirm where
   exactly the composite gets added to `open_buffers`. The
   `set_active_buffer` flow at `crates/fresh-editor/src/app/mod.rs:2474`
   calls `view_state.add_buffer(buffer_id)` on the active split. Check
   `add_buffer`'s implementation to see whether it appends to the END of
   `open_buffers` (so position == len-1) or somewhere else.

6. **Bug variant 2 (no other buffers, opens `[No Name]`)** is the same
   root cause. Once variant 1 passes, write a second test that closes
   all other buffers before drilling down (this is the harder test
   setup; you may need to use the editor's `Close Buffer` action via
   the command palette, or programmatically via `harness.editor_mut()`
   if a clean key path doesn't exist).

## Test guideline reminder

Per `CONTRIBUTING.md` and the saved feedback in
`feedback_test_no_status_bar.md`:
- **Don't read the status bar** in test assertions or `wait_until`
  conditions. It's overwritten by many code paths and is flaky. Use
  buffer content (panel headers, file lists, tab bar text) instead.
- **Wait indefinitely.** No fixed timers, no in-test timeouts —
  `cargo nextest` provides an external timeout if the test hangs.
- **Reproduce before fixing.** The test must fail (or time out) BEFORE
  the fix and pass AFTER.

## Files touched in this WIP commit

- `crates/fresh-editor/src/app/buffer_management.rs` — `close_buffer_internal`
  has the (currently non-functional) `prefer_group_tab` detection +
  `activate_group_tab` call, plus debug `tracing::debug!` calls. Strip
  the tracing once the fix works.
- `crates/fresh-editor/tests/e2e/plugins/audit_mode.rs` — added
  `test_review_diff_drill_down_close_returns_to_group`, marked `#[ignore]`
  so CI doesn't run it until the fix lands. Remove the `#[ignore]` once
  the fix passes.

Earlier (already-committed) work on the same branch:
- `2a182fb4` — initial native scroll architecture (panel buffers as
  first-class for cursor motion).
- `162e48f8` — files panel plugin-managed selection + diff cache.
- `3362ef4a` — `skip_ensure_visible` clear-on-keypress fix
  (same `effective_active_split` shape applies to many places — that
  commit's pattern is a good model for finding the right "active leaf"
  for grouped panels).
- `79253a02` — `extend_to_line_end` overlay bleed fix (renderer
  `>=` → `>`). Same kind of off-by-one bug, different code path.
