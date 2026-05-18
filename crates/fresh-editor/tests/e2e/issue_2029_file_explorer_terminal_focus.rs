//! Regression test for issue #2029 (sub-issue 1: file-explorer focus is
//! stolen back to the terminal).
//!
//! When the user is in an active terminal (`terminal_mode = true`) and
//! transfers focus to the file explorer — either by toggling it with
//! `Ctrl+B` or by clicking a file inside it — `terminal_mode` is left
//! stale. Because `dispatch_terminal_input` only checks the flag and
//! not `key_context`, the user's next keystroke is still forwarded to
//! the PTY even though the explorer is visually focused.
//!
//! Two reproductions covered:
//!
//! 1a. `Ctrl+B` while a terminal is active: the explorer opens, the
//!     "Explorer" menu becomes visible, status says "File explorer
//!     opened" — but Up/Down navigates bash history in the terminal
//!     instead of moving the file-explorer selection.
//!
//! 1b. Click on a file in the explorer while a terminal is active:
//!     per the docstring at `click_handlers.rs:554-557`, a single
//!     click should "Open the file but keep focus on file explorer".
//!     Today the click handler's `key_context = FileExplorer` write
//!     is undone by `set_active_buffer` (`active_focus.rs:103-107`),
//!     which resets `key_context = Normal` because we were leaving a
//!     terminal buffer.
//!
//! Observability: each test drives keyboard / mouse input and asserts
//! purely on the rendered screen, per CONTRIBUTING §Testing — file
//! contents only reach the screen when subsequent Down + Enter
//! presses actually drive the file explorer, so the screen check is
//! sufficient to verify focus transfer.

use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};
use portable_pty::{native_pty_system, PtySize};
use std::fs;

fn pty_available() -> bool {
    native_pty_system()
        .openpty(PtySize {
            rows: 1,
            cols: 1,
            pixel_width: 0,
            pixel_height: 0,
        })
        .is_ok()
}

fn explorer_row_for(harness: &EditorTestHarness, name: &str) -> u16 {
    let screen = harness.screen_to_string();
    const FIRST_EXPLORER_ROW: usize = 2;
    for (row, line) in screen.lines().enumerate().skip(FIRST_EXPLORER_ROW) {
        let prefix: String = line.chars().take(40).collect();
        if prefix.contains(name) {
            return row as u16;
        }
    }
    panic!("file {name} not found in file explorer;\nscreen:\n{screen}");
}

/// 1a — `Ctrl+B` from an active terminal must transfer focus to the
/// file explorer in a way that subsequent arrow keys reach the
/// explorer, not the underlying terminal PTY.
///
/// The assertion is purely on rendered output (per CONTRIBUTING
/// §Testing): after `Ctrl+B` + `Down` + `Enter`, the previewed file's
/// content must appear on screen. If keys were still being forwarded
/// to the PTY (the bug), Down would scroll bash history and Enter
/// would submit a (likely empty) command — `ALPHA_FILE_CONTENT` would
/// never appear.
#[test]
fn ctrl_b_from_terminal_transfers_keyboard_focus_to_file_explorer() {
    if !pty_available() {
        eprintln!("Skipping: PTY not available in this environment");
        return;
    }

    let mut harness = EditorTestHarness::with_temp_project(120, 40).unwrap();
    let project = harness.project_dir().unwrap();
    fs::write(project.join("alpha.txt"), "ALPHA_FILE_CONTENT\n").unwrap();

    // Open a terminal — this puts the editor in terminal mode.
    harness.editor_mut().open_terminal();
    harness.render().unwrap();
    assert!(
        harness.editor().is_terminal_mode(),
        "precondition: opening a terminal should enter terminal mode"
    );

    // Toggle the file explorer via the default `Ctrl+B` binding.
    // `send_key` is synchronous: by the time it returns, `take_focus`
    // has run, so the *immediate* post-condition is that we're no
    // longer in terminal mode. Asserting this here catches the bug
    // without waiting for async file-tree init (which could give
    // plugin background work a chance to mutate key_context further
    // and obscure whether the immediate effect of `Ctrl+B` was
    // correct).
    harness
        .send_key(KeyCode::Char('b'), KeyModifiers::CONTROL)
        .unwrap();
    assert!(
        !harness.editor().is_terminal_mode(),
        "Ctrl+B from terminal must clear terminal_mode so dispatch_terminal_input \
         stops swallowing keys destined for the file explorer (issue #2029)"
    );

    // End-to-end: with terminal_mode cleared, Down + Enter on the now
    // initialized explorer must select and open `alpha.txt`. If the
    // bug were re-introduced (Down going to the PTY), the file's
    // contents would never reach the screen and this wait would hang
    // until cargo nextest's external timeout fires.
    harness.wait_for_file_explorer().unwrap();
    harness.wait_for_file_explorer_item("alpha.txt").unwrap();
    harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness
        .wait_for_screen_contains("ALPHA_FILE_CONTENT")
        .unwrap();
}

/// 1b — single-clicking a file in the explorer while a terminal is the
/// active buffer must keep keyboard focus on the file explorer, so the
/// user can keep arrow-browsing previews. Today focus ends up on the
/// previewed editor buffer.
///
/// Observation: after click on `alpha.txt`, a `Down + Enter` should
/// open `beta.txt` (the next file in the tree). With focus stuck on
/// the editor instead of the explorer, Down moves the cursor inside
/// alpha.txt and `BETA_FILE_CONTENT` never appears.
#[test]
fn click_in_explorer_while_terminal_active_keeps_focus_on_explorer() {
    if !pty_available() {
        eprintln!("Skipping: PTY not available in this environment");
        return;
    }

    let mut harness = EditorTestHarness::with_temp_project(120, 40).unwrap();
    let project = harness.project_dir().unwrap();
    fs::write(project.join("alpha.txt"), "ALPHA_FILE_CONTENT\n").unwrap();
    fs::write(project.join("beta.txt"), "BETA_FILE_CONTENT\n").unwrap();

    // Open a terminal — terminal_mode = true.
    harness.editor_mut().open_terminal();
    harness.render().unwrap();
    assert!(harness.editor().is_terminal_mode());

    // Open the file explorer via `Ctrl+B`. The 1a fix clears
    // `terminal_mode` here; this 1b test stresses what happens *after*
    // — a single click on a file in the explorer must keep focus on
    // the explorer rather than handing it to the previewed buffer.
    harness
        .send_key(KeyCode::Char('b'), KeyModifiers::CONTROL)
        .unwrap();
    harness.wait_for_file_explorer().unwrap();
    harness.wait_for_file_explorer_item("alpha.txt").unwrap();
    harness.wait_for_file_explorer_item("beta.txt").unwrap();

    // Single-click `alpha.txt` in the explorer. The click is mouse-
    // synchronous: by the time `mouse_click` returns, the click
    // handler (and the preview open it triggers) have finished and
    // key_context's immediate value should be FileExplorer.
    let alpha_row = explorer_row_for(&harness, "alpha.txt");
    harness.mouse_click(10, alpha_row).unwrap();
    assert_eq!(
        harness.editor().get_key_context(),
        fresh::input::keybindings::KeyContext::FileExplorer,
        "single-click in explorer must keep focus on FileExplorer; \
         set_active_buffer must not steal it back to Normal when the \
         previous buffer was a terminal (issue #2029, click_handlers.rs:554-557)"
    );
    harness
        .wait_for_screen_contains("ALPHA_FILE_CONTENT")
        .unwrap();

    // End-to-end: with focus on the explorer, Down should advance the
    // *explorer* selection to `beta.txt` and Enter should preview it.
    // If focus leaked to the editor (the bug), Down would move the
    // cursor inside alpha.txt and the screen would never grow
    // `BETA_FILE_CONTENT` — the wait would hang until external timeout.
    harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness
        .wait_for_screen_contains("BETA_FILE_CONTENT")
        .unwrap();
}
