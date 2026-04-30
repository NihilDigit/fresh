use crate::common::harness::{copy_plugin, copy_plugin_lib, EditorTestHarness, HarnessOptions};
use crossterm::event::{KeyCode, KeyModifiers};
use fresh::config::Config;
use std::path::PathBuf;

#[test]
fn test_repro_popup_layout_with_autohide_prompt() {
    // 1. Set up project with the markdown_compose plugin
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_root = temp_dir.path().join("project");
    std::fs::create_dir(&project_root).unwrap();

    let plugins_dir = project_root.join("plugins");
    std::fs::create_dir(&plugins_dir).unwrap();
    copy_plugin(&plugins_dir, "markdown_compose");
    copy_plugin_lib(&plugins_dir);

    let md_path = project_root.join("test.md");
    std::fs::write(&md_path, "# Test File\n").unwrap();

    // 2. Create harness with show_prompt_line = false (auto-hide)
    // Note: EditorTestHarness::create forces show_prompt_line = true, 
    // so we need to toggle it off after creation.
    let mut harness =
        EditorTestHarness::create(80, 24, HarnessOptions::new()
            .with_working_dir(project_root)
            .without_empty_plugins_dir())
            .unwrap();

    // Toggle prompt line OFF (to auto-hide mode)
    harness.editor_mut().toggle_prompt_line();
    harness.render().unwrap();
    
    assert!(!harness.editor().prompt_line_visible(), "Prompt line should be in auto-hide mode");

    // Open a file
    harness.open_file(&md_path).unwrap();
    harness.render().unwrap();

    // 3. Trigger "Set Compose Width"
    // Open command palette
    harness.send_key(KeyCode::Char('p'), KeyModifiers::CONTROL).unwrap();
    harness.render().unwrap();
    
    // Type command
    harness.type_text("Set Compose Width").unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("Set Compose Width");

    // Execute command
    harness.send_key(KeyCode::Enter, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Now we should be in the "Compose Width" prompt.
    // The prompt line should have appeared.
    // There should be suggestions (None, 120).
    
    let screen = harness.screen_to_string();
    println!("--- Screen with prompt active (with suggestions) ---");
    println!("{}", screen);

    // 4. Verify initial state with suggestions
    harness.assert_screen_contains("Compose width"); // Fixed casing
    harness.assert_screen_contains("None");
    harness.assert_screen_contains("120");

    // In this state (with suggestions), the status bar is hidden by the suggestions.
    // Row 23: Prompt Line
    // Row 22: Popup bottom border
    // Row 21: Suggestion "120"
    // Row 20: Suggestion "None"
    // Row 19: Popup top border
    let row_23 = harness.get_row_text(23);
    let row_22 = harness.get_row_text(22);
    let row_21 = harness.get_row_text(21);
    println!("Row 23: {:?}", row_23);
    println!("Row 22: {:?}", row_22);
    println!("Row 21: {:?}", row_21);
    
    assert!(row_23.contains("Compose width"), "Prompt should be on row 23");
    assert!(row_22.contains("┘"), "Row 22 should be the popup bottom border");
    assert!(row_21.contains("120"), "Suggestion '120' should be on row 21");

    // 5. Type something that doesn't match any suggestions
    harness.type_text("999").unwrap();
    harness.render().unwrap();

    let screen_no_suggestions = harness.screen_to_string();
    println!("--- Screen with prompt active (NO suggestions) ---");
    println!("{}", screen_no_suggestions);

    // Row 23: Prompt Line
    // Row 22: Status Bar (should be visible now!)
    let row_23_no_sug = harness.get_row_text(23);
    let row_22_no_sug = harness.get_row_text(22);
    println!("Row 23 (No Sug): {:?}", row_23_no_sug);
    println!("Row 22 (No Sug): {:?}", row_22_no_sug);

    assert!(row_23_no_sug.contains("Compose width"), "Prompt should still be on row 23");
    // Verify status bar content (typically contains file name, line/col, etc.)
    // In our test, it should have "test.md"
    assert!(row_22_no_sug.contains("test.md"), "Status bar should be visible on row 22 when no suggestions");

    // 6. Backspace to make suggestions reappear
    harness.send_key(KeyCode::Backspace, KeyModifiers::NONE).unwrap();
    harness.send_key(KeyCode::Backspace, KeyModifiers::NONE).unwrap();
    harness.send_key(KeyCode::Backspace, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    let screen_backspaced = harness.screen_to_string();
    println!("--- Screen after backspacing (suggestions back) ---");
    println!("{}", screen_backspaced);

    harness.assert_screen_contains("None");
    harness.assert_screen_contains("120");
    
    let row_22_back = harness.get_row_text(22);
    let row_21_back = harness.get_row_text(21);
    assert!(row_22_back.contains("┘"), "Row 22 should be the popup bottom border again");
    assert!(row_21_back.contains("120"), "Suggestion '120' should return to row 21");
    assert!(!row_22_back.contains("test.md"), "Status bar should be hidden again by suggestions");
}
