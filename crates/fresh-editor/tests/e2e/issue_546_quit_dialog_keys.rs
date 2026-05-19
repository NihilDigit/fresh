//! Regression tests for issue #546 — the unsaved-buffer quit dialog must
//! accept y/n keys (in addition to the localized save/discard letters)
//! and must submit on the first keypress without requiring Enter.
//!
//! Observability: each test drives a key event and inspects the editor's
//! user-visible signals (`should_quit()`, on-screen prompt text, file
//! contents on disk). No private model state is inspected.

use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};
use fresh::config::Config;

/// Pressing `n` (a locale-independent alias for discard) on the
/// unsaved-buffer prompt must discard and quit — without requiring
/// Enter.
#[test]
fn n_discards_and_quits_without_enter() {
    let mut config = Config::default();
    config.editor.hot_exit = false;
    let mut harness = EditorTestHarness::with_temp_project_and_config(120, 24, config).unwrap();

    harness.type_text("scratch").unwrap();
    harness.render().unwrap();

    harness
        .send_key(KeyCode::Char('q'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Sanity: prompt is up, we have NOT quit yet.
    assert!(!harness.editor().should_quit());
    let screen = harness.screen_to_string();
    assert!(
        screen.contains("unsaved changes"),
        "expected the unsaved-changes prompt on screen, got:\n{screen}"
    );

    // Single keypress, no Enter.
    harness
        .send_key(KeyCode::Char('n'), KeyModifiers::NONE)
        .unwrap();

    assert!(
        harness.editor().should_quit(),
        "'n' on the unsaved-buffer prompt must discard-and-quit without requiring Enter"
    );
}

/// Pressing `d` (the existing localized discard key) must also submit
/// without Enter — the modal change must not regress the existing keys.
#[test]
fn d_discards_and_quits_without_enter() {
    let mut config = Config::default();
    config.editor.hot_exit = false;
    let mut harness = EditorTestHarness::with_temp_project_and_config(120, 24, config).unwrap();

    harness.type_text("scratch").unwrap();
    harness.render().unwrap();

    harness
        .send_key(KeyCode::Char('q'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    assert!(!harness.editor().should_quit());

    harness
        .send_key(KeyCode::Char('d'), KeyModifiers::NONE)
        .unwrap();

    assert!(
        harness.editor().should_quit(),
        "'d' on the unsaved-buffer prompt must discard-and-quit without requiring Enter"
    );
}

/// Pressing `y` (locale-independent alias for save) on the
/// unsaved-buffer prompt with a *named* dirty buffer must save the file
/// to disk and quit — all triggered by a single keypress.
#[test]
fn y_saves_and_quits_named_buffer_without_enter() {
    let mut config = Config::default();
    config.editor.hot_exit = false;
    let mut harness = EditorTestHarness::with_temp_project_and_config(120, 24, config).unwrap();
    let project_dir = harness.project_dir().unwrap();

    let file_path = project_dir.join("notes.txt");
    std::fs::write(&file_path, "initial\n").unwrap();
    harness.open_file(&file_path).unwrap();

    harness.type_text("new ").unwrap();
    harness.render().unwrap();

    harness
        .send_key(KeyCode::Char('q'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    assert!(!harness.editor().should_quit());

    // Single 'y' — no Enter.
    harness
        .send_key(KeyCode::Char('y'), KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    assert!(
        harness.editor().should_quit(),
        "'y' on the unsaved-buffer prompt must save-and-quit without requiring Enter"
    );
    let on_disk = std::fs::read_to_string(&file_path).unwrap();
    assert!(
        on_disk.starts_with("new "),
        "save-and-quit via 'y' must persist the buffer to disk; on-disk contents were: {on_disk:?}"
    );
}

/// `Esc` on the unsaved-buffer prompt must cancel — not quit, and not
/// leave a leaked prompt on screen.
#[test]
fn esc_cancels_quit_prompt() {
    let mut config = Config::default();
    config.editor.hot_exit = false;
    let mut harness = EditorTestHarness::with_temp_project_and_config(120, 24, config).unwrap();

    harness.type_text("keep me").unwrap();
    harness.render().unwrap();

    harness
        .send_key(KeyCode::Char('q'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    assert!(!harness.editor().should_quit());

    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    assert!(
        !harness.editor().should_quit(),
        "Esc on the quit prompt must not quit the editor"
    );
    let screen = harness.screen_to_string();
    assert!(
        !screen.contains("unsaved changes"),
        "Esc must dismiss the quit prompt; screen still showed it:\n{screen}"
    );
}

/// The clean-session confirm-quit prompt (opt-in via `editor.confirm_quit`)
/// must also submit on a single `y` without requiring Enter.
#[test]
fn clean_session_y_quits_without_enter() {
    let mut config = Config::default();
    config.editor.confirm_quit = true;
    let mut harness = EditorTestHarness::with_temp_project_and_config(120, 40, config).unwrap();

    harness
        .send_key(KeyCode::Char('q'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    assert!(!harness.editor().should_quit());

    harness
        .send_key(KeyCode::Char('y'), KeyModifiers::NONE)
        .unwrap();

    assert!(
        harness.editor().should_quit(),
        "'y' on the clean-session confirm-quit prompt must quit without requiring Enter"
    );
}
