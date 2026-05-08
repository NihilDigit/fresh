# Continue Conductor / Window architecture work

Branch `claude/plan-conductor-architecture-6YsJt` was merged in PR #1904.
Internal type is `Window` (Editor.windows, active_window); Conductor's
UX still says "session". Design: `docs/internal/conductor-sessions-design.md`.

## Top priority — Step 0 (Session-as-Window migration)

The merged branch shipped the *warm-swap interim*. Eliminate it by
making each `Window` self-contained (VS Code window model). See
`§ Migration sequence § Step 0` in the design doc.

Sub-steps, in order, each its own PR:

- **0a** Move `cached_layout` split/tab/file-explorer parts onto
  `Window`; chrome rects stay on `Editor`.
- **0b** Convert stashes (`splits_stash`, `file_explorer_stash`,
  `lsp_stash`, `panel_ids_stash`, `file_mod_times_stash`) to live
  fields on `Window`. Replace `self.<field>` with
  `self.active_window_mut().<field>` via accessors.
  `set_active_window` becomes a pointer write.
- **0c** Move `Editor.buffers` onto `Window`. `next_buffer_id` stays
  global.
- **0d** Move `terminal_manager`, `terminal_buffers`,
  `terminal_backing_files` onto `Window`. `closeWindow` joins PTY
  threads.
- **0e** Move `event_logs` onto `Window` (follows buffers).
- **0f** Move `position_history`, `bookmarks` onto `Window`.
- **0g** Audit commands: `self.buffers.iter()` →
  `self.active_window().buffers.iter()` everywhere. Save-all,
  quick-open, find-in-files all scope to the active window.
- **0h** Refactor render: `render_window(frame, area, &Window,
  &Editor)`. Preview becomes the same call with a sub-rect and a
  different `&Window`. Delete the transient-swap hack.
- **0i** Delete warm-swap helpers and stash fields.

## After Step 0 — resume MVP UX

- Single-key hotkeys `n`/`d`/`k` inside `Conductor: Open` (not just
  palette).
- `d` action: invoke existing review-diff feature.
- `m` action: merge READY window into base.
- AGENT, DIFF (+/-) columns; empty-state Screen 1 copy.
- Base window shows `RUN` when not active — should be `-`/`BASE`.
- conductor.ts wraps the agent in a long-lived shell so
  `terminal_exit` never fires; fix so READY/ERRORED states transition.

Do **not** add v1.1+ features (collision radar, KILLED tombstones,
multi-select, rename, native diff) until Step 0 is complete.

## Verify per sub-step

```sh
cargo fmt --all
cargo check --all-targets
cargo nextest run --locked --all-features --all-targets
```

Commit per sub-step. One PR per sub-step (or two combined). Don't
bundle 0a–0i into one giant PR.
