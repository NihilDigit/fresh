//! Reproducer for a residual caret bug on a buffer that is a SINGLE very
//! long line with NO trailing newline (e.g. a minified JS bundle saved
//! without a final EOL).
//!
//! ## The bug
//!
//! With the caret at the very end of the document (`MoveDocumentEnd`, i.e.
//! Ctrl+End), pressing Up does **nothing** — the caret stays at EOF instead
//! of moving up one visual row. Repeating Up never moves, so "jump to end,
//! then scroll up one row at a time" is impossible on such a file.
//!
//! ## Root cause
//!
//! For a single logical line the viewport keeps `top_byte` pinned at the
//! line start (0) and encodes the scroll position in `top_view_line_offset`.
//! `build_base_tokens` always tokenizes *from `top_byte`* and stops after
//! roughly `visible_count + 4` segments — and because it emits a
//! `MAX_SAFE_LINE_WIDTH` (10 000-char) break per segment, it only ever
//! covers the first ~`visible_count * 10_000` bytes of the line. The
//! viewport — and therefore any caret — can never advance past that window
//! into a single newline-free line.
//!
//! `MoveDocumentEnd` still parks the caret at the true EOF (well beyond the
//! tokenized window), so:
//!   * the wrap-aware Up/Down intercept can't find the off-screen caret's
//!     visual row in the layout cache (`find_visual_row` → None) and bails;
//!   * the byte-based `MoveUp` fallback walks logical lines via
//!     `LineIterator::prev`, which returns None for a one-line buffer.
//! With neither path producing a `MoveCursor` event, Up is a no-op.
//!
//! This is the same build-from-line-start, capped-window architecture that
//! makes vertical scrolling on long wrapped lines slow; the fix is therefore
//! deferred (it overlaps that render rework) and this test is `#[ignore]`d so
//! it documents the bug without failing CI. Remove the `#[ignore]` once the
//! renderer can build/scroll a window near the caret's position.
//!
//! ## Why a >~310 KB line and a real EOF (no trailing newline)
//!
//! The window cap is `~visible_count * 10_000` bytes. A ~36 KB line (the
//! sibling `single_long_wrapped_line.txt` fixture) fits entirely inside the
//! window at any tested height, so its EOF caret is always in the layout
//! cache and Up works. The bug only appears once the line exceeds the cap,
//! so this fixture is ~500 KB. A trailing newline would land
//! `MoveDocumentEnd` on the empty final line (a separate logical line whose
//! start is in-window), masking the bug — hence the fixture has no EOL.

use crate::common::scenario::layout_scenario::{
    assert_layout_scenario, LayoutScenario, ScenarioConfigOverrides, StepAssertion,
};
use crate::common::scenario::render_snapshot::RenderSnapshotExpect;
use fresh::test_api::Action;
use std::path::PathBuf;

/// The fixture is one ~500 KB line with no trailing newline. Its EOF byte is
/// the file length; `MoveDocumentEnd` lands the caret there.
const FIXTURE_BYTES: usize = 500_005;

fn wrap_overrides() -> ScenarioConfigOverrides {
    ScenarioConfigOverrides {
        line_wrap: Some(true),
        ..Default::default()
    }
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("single_long_line_no_eol.txt")
}

/// After `MoveDocumentEnd`, pressing `MoveUp` must move the caret up one
/// visual row — it must NOT leave the caret stranded at EOF.
///
/// A render is forced after `MoveDocumentEnd` (via `step_assertions`) so the
/// wrap-aware Up handler sees a fresh layout cache — the exact precondition
/// under which the bug manifests. The step assertion also pins that the
/// setup actually reached EOF, so the final assertion's failure can only mean
/// "Up didn't move", not "we never got to the end".
///
/// `cursor_byte_in: (1, FIXTURE_BYTES - 1)` enforces both halves: the caret
/// moved off EOF (<= FIXTURE_BYTES - 1) and didn't teleport to the document
/// start (>= 1). Without the fix the caret stays at `FIXTURE_BYTES` and the
/// upper bound fails.
#[test]
#[ignore = "known bug: Up at EOF of a single >~310KB newline-free line is a no-op; \
            fix overlaps the long-line render rework (see module docs)"]
fn up_at_eof_single_long_line_moves_up_one_row() {
    let widths: [u16; 2] = [60, 80];
    let heights: [u16; 2] = [20, 30];
    for &height in &heights {
        for &width in &widths {
            assert_layout_scenario(LayoutScenario {
                description: format!(
                    "Up after MoveDocumentEnd on a single long newline-free line \
                     moves up one visual row (width={width}, height={height})"
                ),
                initial_file: Some(fixture_path()),
                width,
                height,
                actions: vec![Action::MoveDocumentEnd, Action::MoveUp],
                config_overrides: wrap_overrides(),
                // Force a render after MoveDocumentEnd and pin that the caret
                // actually reached EOF before the Up under test.
                step_assertions: vec![StepAssertion {
                    after_action_index: 0,
                    expect: RenderSnapshotExpect {
                        cursor_byte: Some(FIXTURE_BYTES),
                        ..Default::default()
                    },
                }],
                expected_snapshot: RenderSnapshotExpect {
                    cursor_byte_in: Some((1, FIXTURE_BYTES - 1)),
                    ..Default::default()
                },
                ..Default::default()
            });
        }
    }
}
