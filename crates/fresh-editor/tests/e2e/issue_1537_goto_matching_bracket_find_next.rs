//! E2E tests reproducing issue #1537:
//! "go to matching bracket messes up find next/previous"
//!
//! <https://github.com/sinelaw/fresh/issues/1537>
//!
//! The reporter describes the following flow:
//!   1. Ctrl+F, search for a term with multiple matches. Find next/previous
//!      behaves correctly.
//!   2. Place the cursor on a bracket and invoke Go to Matching Bracket
//!      (Ctrl+]).
//!   3. Invoke find next / find previous. The cursor lands on the bracket
//!      instead of on the next / previous search match:
//!
//!       > once you do a go to matching bracket, it seems to insert the
//!       > matching bracket into the find next /. previous.
//!
//! Root cause (reproduced here): `goto_matching_bracket` moves the cursor
//! off the active search match and onto a bracket character. The next
//! invocation of the "quick find" actions — `find_selection_next` /
//! `find_selection_previous`, bound by default to Ctrl+F3 / Ctrl+Shift+F3
//! (with Alt+N / Alt+P as terminal-friendly alternatives) — detects that
//! the cursor is no longer on a match, throws away the active NEEDLE
//! search state, and starts a brand new search using whatever
//! `get_selection_or_word_for_search_with_pos` returns for the bracket
//! position. That "new search" is effectively the bracket / bracket
//! context, which is exactly what the reporter describes.
//!
//! Each test:
//!   * sets up a file with 3 NEEDLE matches and a pair of matching braces,
//!   * performs Ctrl+F "NEEDLE" Enter,
//!   * positions the cursor on a bracket,
//!   * invokes Go to Matching Bracket (Ctrl+]),
//!   * invokes a find-next / find-previous quick-find action,
//!   * asserts the cursor landed on the expected NEEDLE match and *not*
//!     on a bracket character.

use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};
use tempfile::TempDir;

/// Test file with three NEEDLE matches and a pair of matching braces.
///
///   line 0: "line 0 has a NEEDLE here\n"
///   line 1: "line 1 filler text for padding\n"
///   line 2: "line 2 brace { start\n"              ← '{'
///   line 3: "line 3 content inside\n"
///   line 4: "line 4 NEEDLE inside the braces\n"
///   line 5: "line 5 continues here\n"
///   line 6: "line 6 brace } end\n"                ← '}'
///   line 7: "line 7 NEEDLE after the braces\n"
///   line 8: "line 8 more filler text\n"
///   line 9: "line 9 last line\n"
const CONTENT: &str = "\
line 0 has a NEEDLE here
line 1 filler text for padding
line 2 brace { start
line 3 content inside
line 4 NEEDLE inside the braces
line 5 continues here
line 6 brace } end
line 7 NEEDLE after the braces
line 8 more filler text
line 9 last line
";

fn setup() -> (EditorTestHarness, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, CONTENT).unwrap();

    let mut harness = EditorTestHarness::new(140, 30).unwrap();
    harness.open_file(&file_path).unwrap();
    harness.render().unwrap();
    (harness, temp_dir)
}

/// Byte offsets of the three NEEDLE matches.
fn match_positions() -> (usize, usize, usize) {
    let m1 = CONTENT.find("NEEDLE").unwrap();
    let m2 = CONTENT[m1 + 1..].find("NEEDLE").unwrap() + m1 + 1;
    let m3 = CONTENT[m2 + 1..].find("NEEDLE").unwrap() + m2 + 1;
    (m1, m2, m3)
}

fn open_bracket_pos() -> usize {
    CONTENT.find('{').unwrap()
}

fn close_bracket_pos() -> usize {
    CONTENT.find('}').unwrap()
}

/// Perform Ctrl+F "NEEDLE" Enter and verify the cursor landed on the first
/// match.
fn start_search(harness: &mut EditorTestHarness, m1: usize) {
    harness
        .send_key(KeyCode::Char('f'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("NEEDLE").unwrap();
    harness.render().unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.process_async_and_render().unwrap();

    assert_eq!(
        harness.cursor_position(),
        m1,
        "Initial search should land on the first NEEDLE match"
    );
    harness.assert_screen_contains("Found 3 matches");
}

/// Move the cursor to an absolute byte offset.
fn move_cursor_to(harness: &mut EditorTestHarness, pos: usize) {
    harness
        .editor_mut()
        .active_cursors_mut()
        .primary_mut()
        .position = pos;
    harness.render().unwrap();
    assert_eq!(harness.cursor_position(), pos);
}

fn assert_not_on_bracket(harness: &EditorTestHarness, action: &str) {
    let pos = harness.cursor_position();
    let byte = CONTENT.as_bytes()[pos] as char;
    assert!(
        !"(){}[]<>".contains(byte),
        "Cursor after {} must not be on a bracket character. Got {:?} at pos {}",
        action,
        byte,
        pos
    );
}

/// Issue #1537 — find_selection_next (Ctrl+F3 / Alt+N) following Go to
/// Matching Bracket must not throw away the active NEEDLE search and land
/// on a bracket / bracket context.
#[test]
fn test_find_selection_next_after_goto_matching_bracket_lands_on_needle() {
    let (mut harness, _tmp) = setup();
    let (m1, _m2, m3) = match_positions();
    let open_pos = open_bracket_pos();
    let close_pos = close_bracket_pos();

    start_search(&mut harness, m1);

    // Place cursor on '{' and jump to its matching '}'.
    move_cursor_to(&mut harness, open_pos);
    harness
        .send_key(KeyCode::Char(']'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    assert_eq!(
        harness.cursor_position(),
        close_pos,
        "Go to Matching Bracket should move cursor to the matching '}}'"
    );

    // Ctrl+F3 — find_selection_next (see default keymap).
    harness
        .send_key(KeyCode::F(3), KeyModifiers::CONTROL)
        .unwrap();
    harness.process_async_and_render().unwrap();

    assert_not_on_bracket(&harness, "Ctrl+F3 after Go to Matching Bracket");
    assert_eq!(
        harness.cursor_position(),
        m3,
        "After Ctrl+F3 following Go to Matching Bracket, cursor should be on \
         the next NEEDLE match (line 7 at pos {}), but was at pos {}",
        m3,
        harness.cursor_position()
    );
}

/// Issue #1537 — find_selection_previous (Ctrl+Shift+F3 / Alt+P) following
/// Go to Matching Bracket must not throw away the active NEEDLE search and
/// land on a bracket / bracket context.
#[test]
fn test_find_selection_previous_after_goto_matching_bracket_lands_on_needle() {
    let (mut harness, _tmp) = setup();
    let (m1, _m2, _m3) = match_positions();
    let open_pos = open_bracket_pos();
    let close_pos = close_bracket_pos();

    start_search(&mut harness, m1);

    // Place cursor on '}' and jump to its matching '{'.
    move_cursor_to(&mut harness, close_pos);
    harness
        .send_key(KeyCode::Char(']'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    assert_eq!(
        harness.cursor_position(),
        open_pos,
        "Go to Matching Bracket should move cursor to the matching '{{'"
    );

    // Ctrl+Shift+F3 — find_selection_previous (see default keymap).
    harness
        .send_key(KeyCode::F(3), KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        .unwrap();
    harness.process_async_and_render().unwrap();

    assert_not_on_bracket(&harness, "Ctrl+Shift+F3 after Go to Matching Bracket");
    assert_eq!(
        harness.cursor_position(),
        m1,
        "After Ctrl+Shift+F3 following Go to Matching Bracket, cursor should \
         be on the previous NEEDLE match (line 0 at pos {}), but was at pos {}",
        m1,
        harness.cursor_position()
    );
}
