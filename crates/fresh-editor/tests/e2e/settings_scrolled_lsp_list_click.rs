//! E2E test for clicking on items in a scrolled LSP list.
//!
//! Regression test for #1824: when the settings panel is scrolled and the LSP
//! map's top is clipped above the viewport, clicking on a visible LSP entry
//! should select that entry — not an entry offset by the scroll amount.

use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_settings_scrolled_lsp_list_mouse_click() {
    let mut harness = EditorTestHarness::new(100, 24).unwrap();

    harness.open_settings().unwrap();

    // Search and jump to the LSP map (General > /lsp).
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    harness.type_text("Lsp").unwrap();
    harness.render().unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("Lsp:");

    // Scroll the settings panel so the "Lsp:" label is clipped above the
    // viewport but later entries remain visible.
    let scroll_col = 60u16;
    let scroll_row = 15u16;
    for _ in 0..15 {
        harness.mouse_scroll_down(scroll_col, scroll_row).unwrap();
    }
    harness.render().unwrap();

    if harness.screen_to_string().contains("Lsp:") {
        for _ in 0..10 {
            harness.mouse_scroll_down(scroll_col, scroll_row).unwrap();
        }
        harness.render().unwrap();
    }

    let target = find_clickable_lsp_entry(&harness);
    let screen = harness.screen_to_string();
    assert!(
        target.is_some(),
        "Expected to find an LSP entry on screen after scrolling.\nScreen:\n{}",
        screen
    );
    let (target_name, click_col, click_row) = target.unwrap();

    harness.mouse_click(click_col, click_row).unwrap();
    harness.render().unwrap();

    let screen_after = harness.screen_to_string();
    let mut found = false;
    for line in screen_after.lines() {
        if line.contains("[Enter to edit]") {
            assert!(
                line.contains(&target_name),
                "After clicking on '{}' (at row {}), expected it to become focused, \
                 but '[Enter to edit]' appeared on a different entry.\n\
                 Focused line: {}\nFull screen:\n{}",
                target_name,
                click_row,
                line.trim(),
                screen_after
            );
            found = true;
            break;
        }
    }
    assert!(
        found,
        "No '[Enter to edit]' found on screen after clicking on '{}'.\nScreen:\n{}",
        target_name, screen_after
    );
}

/// Find a visible LSP entry on screen. Returns (language, col, row).
fn find_clickable_lsp_entry(harness: &EditorTestHarness) -> Option<(String, u16, u16)> {
    // Languages known to have an LSP entry in the default config. Picked from
    // the schema's default `lsp` map.
    let known_langs = [
        "astro",
        "bash",
        "c",
        "clojure",
        "cmake",
        "cpp",
        "csharp",
        "css",
        "dart",
        "dockerfile",
        "elixir",
        "erlang",
        "fsharp",
        "gleam",
        "go",
        "graphql",
        "haskell",
        "html",
        "java",
        "javascript",
        "json",
        "julia",
        "kotlin",
        "lua",
        "markdown",
        "nix",
        "ocaml",
        "perl",
        "php",
        "python",
        "r",
        "racket",
        "ruby",
        "rust",
        "scala",
        "solidity",
        "sql",
        "svelte",
        "swift",
        "tailwindcss",
        "templ",
        "terraform",
        "toml",
        "typescript",
        "typst",
        "vue",
        "yaml",
        "zig",
    ];

    let buf = harness.buffer();
    for row in 0..buf.area.height {
        let row_text = harness.screen_row_text(row);
        if row_text.contains("[Enter to edit]")
            || row_text.contains("Lsp:")
            || row_text.contains("Universal Lsp")
            || row_text.contains("Name")
            || row_text.contains("[+]")
            || row_text.contains("──")
        {
            continue;
        }
        for lang in &known_langs {
            // Map entries look like: "      langname              command-name..."
            if row_text.contains(&format!("  {}  ", lang)) {
                let col = 40u16;
                return Some((lang.to_string(), col, row));
            }
        }
    }
    None
}
