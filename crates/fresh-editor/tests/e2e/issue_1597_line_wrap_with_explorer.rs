//! Regression guard for issue #1597: "Line wrap doesn't work on quoted strings".
//!
//! The bug as described — a long quoted string on line 67 rendered flush
//! against the right edge of a narrowed editor pane (file explorer open) —
//! could not be reproduced from the information in the issue.  The test
//! below sets up the suspected-cause scenario (file explorer open, long
//! quoted string in the buffer) and asserts that every character of the
//! string is visible on screen (i.e. the line wraps inside the reduced
//! editor pane).  If a regression ever breaks wrapping in this layout the
//! test will fail, giving a concrete repro to work from.

use crate::common::harness::EditorTestHarness;
use fresh::config::Config;

/// With file explorer open and line wrap on, a long line should still wrap
/// within the (reduced) editor pane, not be truncated.
#[test]
fn test_issue_1597_long_line_wraps_with_file_explorer_open() {
    let config = Config {
        editor: fresh::config::EditorConfig {
            line_wrap: true,
            wrap_indent: false,
            ..Default::default()
        },
        ..Default::default()
    };

    // 120-column terminal, tall enough for several wrapped rows.
    // Use a temp project directory so the file explorer can open.
    let mut harness =
        EditorTestHarness::with_temp_project_and_config(120, 40, config).unwrap();

    // Open the file explorer FIRST (so the editor pane is already narrowed
    // when the buffer is rendered). Default explorer width is 30%.
    harness.editor_mut().toggle_file_explorer();
    harness.wait_for_file_explorer().unwrap();
    assert!(harness.editor().file_explorer_visible());

    // Load a Kotlin-like file with a long quoted string constant.
    // The long string has no spaces so it can only wrap at the width
    // boundary (no word-boundary shortcuts).
    let long_string_body = "FOLDER_CHOICE_".repeat(20);
    let file_contents = format!(
        "package com.example\n\
         \n\
         object Paths {{\n\
             val FOLDER_CHOICES = \"{}\"\n\
         }}\n",
        long_string_body,
    );
    let _fixture = harness.load_buffer_from_text(&file_contents).unwrap();

    // Refocus the editor so everything renders the buffer.
    harness.editor_mut().focus_editor();
    harness.render().unwrap();
    assert!(
        harness.editor().file_explorer_visible(),
        "File explorer should still be visible after loading the buffer"
    );

    let screen = harness.screen_to_string();

    // Count every 'C' character from the long string.  Each
    // "FOLDER_CHOICE_" contributes two C's (both in CHOICE), so 20 repeats
    // => 40 C's.  If the line wraps correctly every C appears somewhere on
    // screen; if wrapping fails (bug condition) the rightmost tail is
    // clipped and the C count drops below the expected value.
    let expected_cs_in_body = long_string_body.chars().filter(|c| *c == 'C').count();
    let visible_cs = screen.chars().filter(|c| *c == 'C').count();

    assert!(
        visible_cs >= expected_cs_in_body,
        "Long quoted string should wrap within the editor pane when file \
         explorer is open, but only {} 'C' characters were visible on \
         screen (expected at least {} from the string body alone).\n\
         Screen:\n{}",
        visible_cs,
        expected_cs_in_body,
        screen
    );
}
