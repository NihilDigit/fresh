//! Test that syntax highlighting works for embedded languages (CSS inside HTML)
//! even when the viewport is far from the embedding tag.
//!
//! The fixture `embedded_css_long.html` has ~400 CSS rules inside a `<style>` block
//! (21KB), with `.target-rule` CSS at line 405. The `<style>` tag is at byte ~60.
//! The default `context_bytes` is 10KB, so jumping to line 405 requires parse state
//! checkpoints to preserve the embedded CSS context.

use crate::common::harness::{EditorTestHarness, HarnessOptions};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Color;
use std::io::Write;
use std::path::PathBuf;

fn fixture_path(filename: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("tests/fixtures/syntax_highlighting")
        .join(filename)
}

/// Collect distinct non-default foreground colors from the content area of the screen.
fn collect_highlight_colors(harness: &EditorTestHarness, row_start: u16, row_end: u16) -> usize {
    let mut colors = std::collections::HashSet::new();
    for y in row_start..row_end {
        for x in 8..100 {
            if let Some(style) = harness.get_cell_style(x, y) {
                if let Some(fg) = style.fg {
                    match fg {
                        Color::Indexed(15) => {}  // default white text
                        Color::White => {}        // default white text (alternate repr)
                        Color::Indexed(244) => {} // line numbers
                        Color::Indexed(237) => {} // tilde empty lines
                        Color::Indexed(0) => {}   // black
                        Color::Indexed(236) => {} // dark gray UI
                        Color::Rgb(140, 140, 140) => {} // line numbers (RGB)
                        Color::Reset => {}
                        _ => {
                            colors.insert(format!("{:?}", fg));
                        }
                    }
                }
            }
        }
    }
    colors.len()
}

fn create_harness() -> EditorTestHarness {
    EditorTestHarness::create(
        120,
        40,
        HarnessOptions::new()
            .with_project_root()
            .with_full_grammar_registry(),
    )
    .unwrap()
}

fn goto_line(harness: &mut EditorTestHarness, line: usize) {
    harness
        .send_key(KeyCode::Char('g'), KeyModifiers::CONTROL)
        .unwrap();
    harness.type_text(&line.to_string()).unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();
}

/// Jump directly to line 405 (>10KB from `<style>` tag). Checkpoints must be
/// built from byte 0 to preserve embedded CSS context.
#[test]
fn test_embedded_css_highlighting_at_large_offset() {
    let path = fixture_path("embedded_css_long.html");
    assert!(path.exists(), "Fixture not found: {}", path.display());

    let mut harness = create_harness();
    harness.open_file(&path).unwrap();
    harness.render().unwrap();

    // Sanity check: highlighting works at top
    let top_colors = collect_highlight_colors(&harness, 2, 20);
    assert!(
        top_colors >= 2,
        "Sanity check: expected highlighting at top of file, got {} colors",
        top_colors
    );

    // Jump to the target CSS past the 10KB boundary
    goto_line(&mut harness, 405);

    harness.assert_screen_contains("display");
    harness.assert_screen_contains("background");

    let offset_colors = collect_highlight_colors(&harness, 2, 20);
    assert!(
        offset_colors >= 2,
        "CSS inside <style> at large offset (line 405, >10KB from <style> tag) \
         should have syntax highlighting, but got only {} distinct highlight colors. \
         This indicates the TextMate parser lost embedded language context.",
        offset_colors
    );
}

/// Scroll gradually to line 405 via PageDown. Checkpoints are built incrementally
/// as the viewport advances.
#[test]
fn test_embedded_css_highlighting_via_scrolling() {
    let path = fixture_path("embedded_css_long.html");
    let mut harness = create_harness();
    harness.open_file(&path).unwrap();
    harness.render().unwrap();

    // Scroll down with PageDown until we pass line 400
    // The terminal is 40 lines tall, ~36 content lines per page.
    // 405 / 36 ≈ 12 PageDowns to reach the target area.
    for _ in 0..13 {
        harness
            .send_key(KeyCode::PageDown, KeyModifiers::NONE)
            .unwrap();
    }
    harness.render().unwrap();

    // Should now show CSS content near line 400+
    let colors = collect_highlight_colors(&harness, 2, 20);
    assert!(
        colors >= 2,
        "CSS highlighting should work after gradual scrolling, got {} colors",
        colors
    );
}

/// Edit CSS content at line 405, verify highlighting survives cache invalidation.
#[test]
fn test_embedded_css_highlighting_after_edit() {
    let path = fixture_path("embedded_css_long.html");
    let mut harness = create_harness();
    harness.open_file(&path).unwrap();
    harness.render().unwrap();

    // Jump to the CSS target area
    goto_line(&mut harness, 405);

    let colors_before = collect_highlight_colors(&harness, 2, 20);
    assert!(
        colors_before >= 2,
        "Pre-edit: expected CSS highlighting, got {} colors",
        colors_before
    );

    // Type some CSS text (this triggers invalidate_range on the buffer)
    harness
        .send_key(KeyCode::End, KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.type_text("            color: green;").unwrap();
    harness.render().unwrap();

    // Highlighting should still work after the edit
    let colors_after = collect_highlight_colors(&harness, 2, 20);
    assert!(
        colors_after >= 2,
        "Post-edit: CSS highlighting should survive cache invalidation, got {} colors",
        colors_after
    );
}

/// Edit HTML before the `<style>` tag, then return to the CSS area.
/// This tests that checkpoint invalidation (all checkpoints discarded because
/// the edit is before them) correctly rebuilds parse state.
#[test]
fn test_embedded_css_highlighting_after_edit_before_style() {
    let path = fixture_path("embedded_css_long.html");
    let mut harness = create_harness();
    harness.open_file(&path).unwrap();
    harness.render().unwrap();

    // First, jump to line 405 to build checkpoints
    goto_line(&mut harness, 405);
    let colors_initial = collect_highlight_colors(&harness, 2, 20);
    assert!(
        colors_initial >= 2,
        "Initial: expected CSS highlighting, got {} colors",
        colors_initial
    );

    // Go to line 1 (before <style> tag) and insert a line.
    // This invalidates ALL checkpoints since the edit is at byte ~0.
    goto_line(&mut harness, 1);
    harness
        .send_key(KeyCode::End, KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.type_text("<!-- inserted -->").unwrap();
    harness.render().unwrap();

    // Return to the CSS area (now line 406 due to insertion)
    goto_line(&mut harness, 406);

    let colors_after = collect_highlight_colors(&harness, 2, 20);
    assert!(
        colors_after >= 2,
        "After editing before <style> tag, CSS highlighting should still work \
         (checkpoints rebuilt from byte 0), got {} colors",
        colors_after
    );
}

/// Delete a line of CSS content where checkpoints exist.
/// This tests that marker deletion/collapse doesn't cause panics (orphan markers)
/// when checkpoint markers exist in the deleted range.
#[test]
fn test_embedded_css_highlighting_after_delete() {
    let path = fixture_path("embedded_css_long.html");
    let mut harness = create_harness();
    harness.open_file(&path).unwrap();
    harness.render().unwrap();

    // Jump to CSS area to build checkpoints
    goto_line(&mut harness, 200);
    let colors_before = collect_highlight_colors(&harness, 2, 20);
    assert!(
        colors_before >= 2,
        "Pre-delete: expected CSS highlighting, got {} colors",
        colors_before
    );

    // Select and delete multiple lines (Shift+Down then Backspace)
    // This deletes content where checkpoint markers exist
    harness
        .send_key(KeyCode::Home, KeyModifiers::NONE)
        .unwrap();
    for _ in 0..5 {
        harness
            .send_key(KeyCode::Down, KeyModifiers::SHIFT)
            .unwrap();
    }
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Should not panic and highlighting should still work
    let colors_after = collect_highlight_colors(&harness, 2, 20);
    assert!(
        colors_after >= 2,
        "Post-delete: CSS highlighting should survive, got {} colors",
        colors_after
    );

    // Type some text to trigger another convergence walk
    harness.type_text("        .new-rule { color: red; }").unwrap();
    harness.render().unwrap();

    let colors_final = collect_highlight_colors(&harness, 2, 20);
    assert!(
        colors_final >= 2,
        "Post-delete+insert: highlighting should work, got {} colors",
        colors_final
    );
}

/// Rapid typing at a deep offset in a large Rust file — reproduces a panic
/// where `checkpoint_states[&id]` failed because a marker existed in the
/// MarkerList but had no corresponding state entry.
#[test]
fn test_no_panic_on_rapid_typing_in_large_rust_file() {
    // Use the editor's own render.rs as a large Rust file (~210KB, ~4700 lines)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = std::path::PathBuf::from(manifest_dir).join("src/app/render.rs");
    if !path.exists() {
        // Skip if file doesn't exist (e.g. in CI with different layout)
        return;
    }

    let mut harness = create_harness();
    harness.open_file(&path).unwrap();
    harness.render().unwrap();

    // Jump to line 4079 (deep into the file, ~171KB offset)
    goto_line(&mut harness, 4079);

    // Rapidly type characters — each triggers notify_insert + invalidate_range + render
    for ch in "// test comment".chars() {
        harness
            .send_key(KeyCode::Char(ch), KeyModifiers::NONE)
            .unwrap();
    }

    // Delete some characters
    for _ in 0..5 {
        harness
            .send_key(KeyCode::Backspace, KeyModifiers::NONE)
            .unwrap();
    }

    // Type more
    for ch in "edit".chars() {
        harness
            .send_key(KeyCode::Char(ch), KeyModifiers::NONE)
            .unwrap();
    }

    // Should not panic
    harness.render().unwrap();
    let colors = collect_highlight_colors(&harness, 2, 20);
    assert!(
        colors >= 1,
        "After rapid typing in large Rust file, should not panic, got {} colors",
        colors
    );
}

/// Verify highlighting at the top of the file still works (regression guard).
#[test]
fn test_highlighting_near_top_still_works() {
    let path = fixture_path("embedded_css_long.html");
    let mut harness = create_harness();
    harness.open_file(&path).unwrap();
    harness.render().unwrap();

    // The top of the file has HTML + the opening of the <style> block with CSS
    let colors = collect_highlight_colors(&harness, 2, 20);
    assert!(
        colors >= 2,
        "Highlighting at top of file should work, got {} colors",
        colors
    );
}

// ============================================================
// Performance counter tests
// ============================================================

/// After the initial parse, subsequent renders without edits should be cache hits
/// (zero bytes re-parsed).
#[test]
fn test_perf_cache_hit_no_reparse() {
    let path = fixture_path("embedded_css_long.html");
    let mut harness = create_harness();
    harness.open_file(&path).unwrap();
    harness.render().unwrap();

    // Initial render parses the viewport
    goto_line(&mut harness, 200);

    // Reset stats after initial parse
    harness.reset_highlight_stats();

    // Render again without any edits — should be a pure cache hit
    harness.render().unwrap();

    let stats = harness.highlight_stats().expect("should have TextMate stats");
    assert!(
        stats.cache_hits >= 1,
        "Second render without edits should be a cache hit, got {} hits",
        stats.cache_hits
    );
    assert_eq!(
        stats.bytes_parsed, 0,
        "No bytes should be re-parsed on cache hit, got {}",
        stats.bytes_parsed
    );
}

/// Typing a normal character should only re-parse the viewport region once
/// (single pass, not double).
#[test]
fn test_perf_single_char_edit_single_parse() {
    let path = fixture_path("embedded_css_long.html");
    let mut harness = create_harness();
    harness.open_file(&path).unwrap();
    harness.render().unwrap();

    goto_line(&mut harness, 200);
    harness.reset_highlight_stats();

    // Type one character
    harness
        .send_key(KeyCode::Char('x'), KeyModifiers::NONE)
        .unwrap();

    let stats = harness.highlight_stats().expect("should have TextMate stats");
    assert_eq!(
        stats.cache_misses, 1,
        "Single char edit should cause exactly 1 cache miss, got {}",
        stats.cache_misses
    );
    // bytes_parsed should be roughly the viewport + context, not double that
    // Viewport is ~40 lines * ~55 bytes = ~2200 bytes, context = 10KB each side
    // So total parse should be under ~25KB, definitely not over 50KB (which would
    // indicate a double-parse bug).
    assert!(
        stats.bytes_parsed < 50_000,
        "Single char edit should parse < 50KB (single pass), got {} bytes",
        stats.bytes_parsed
    );
}

/// After typing an opening quote (state-changing edit), the first subsequent
/// keystroke re-parses the viewport (state diverged). But the second keystroke
/// and beyond should converge almost immediately — the "inside string" state
/// matches the checkpoints updated by the first keystroke.
#[test]
fn test_perf_convergence_after_state_change() {
    let path = fixture_path("embedded_css_long.html");
    let mut harness = create_harness();
    harness.open_file(&path).unwrap();
    harness.render().unwrap();

    goto_line(&mut harness, 200);

    // Type an opening quote — diverges state for everything after it.
    // This keystroke + the first char after it will do a full viewport parse.
    harness
        .send_key(KeyCode::Char('"'), KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Char('a'), KeyModifiers::NONE)
        .unwrap();

    // Now reset stats. Subsequent keystrokes should converge quickly because
    // the checkpoints already have the "inside string" state from the parse above.
    harness.reset_highlight_stats();

    // Type 5 more characters inside the string
    for ch in "hello".chars() {
        harness
            .send_key(KeyCode::Char(ch), KeyModifiers::NONE)
            .unwrap();
    }

    let stats = harness.highlight_stats().expect("should have TextMate stats");

    // With convergence, each keystroke should parse only from the checkpoint
    // before the edit to the first converging checkpoint (~256-512 bytes).
    // 5 keystrokes * ~500 bytes = ~2500 bytes. Definitely under 10KB.
    // Without convergence (the old bug), it would be ~5 * 22KB = ~110KB.
    assert!(
        stats.bytes_parsed < 10_000,
        "5 keystrokes after state stabilization should parse < 10KB total \
         (convergence after ~256 bytes each), got {} bytes (avg {} per keystroke)",
        stats.bytes_parsed,
        stats.bytes_parsed / 5
    );

    // Each keystroke should trigger at least one convergence detection
    assert!(
        stats.convergences >= 5,
        "5 keystrokes should each converge at least once, got {} convergences",
        stats.convergences
    );

    // Checkpoints_updated should be low — mostly convergence, few updates
    assert!(
        stats.checkpoints_updated <= stats.convergences,
        "Should update fewer checkpoints than convergences, got {} updates vs {} convergences",
        stats.checkpoints_updated,
        stats.convergences
    );
}

/// After typing multiple characters, the highlighting on lines AFTER the edit
/// must remain stable AND convergence must actually kick in (not fall back to
/// full re-parse). Verifies that span cache offset adjustment works correctly.
#[test]
fn test_perf_no_highlight_drift_after_typing() {
    let path = fixture_path("embedded_css_long.html");
    let mut harness = create_harness();
    harness.open_file(&path).unwrap();
    harness.render().unwrap();

    // Jump to a CSS rule and type initial chars to warm up checkpoints
    goto_line(&mut harness, 200);
    harness
        .send_key(KeyCode::End, KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Char('x'), KeyModifiers::NONE)
        .unwrap();

    // Capture reference colors on a line below the edit
    let colors_before: Vec<_> = (8..60)
        .filter_map(|x| {
            harness
                .get_cell_style(x, 15)
                .and_then(|s| s.fg)
                .map(|fg| (x, format!("{:?}", fg)))
        })
        .collect();

    // Reset stats, then type more characters
    harness.reset_highlight_stats();
    for ch in "0123456789".chars() {
        harness
            .send_key(KeyCode::Char(ch), KeyModifiers::NONE)
            .unwrap();
    }
    harness.render().unwrap();

    // Check that convergence actually happened (not just full re-parses)
    let stats = harness.highlight_stats().expect("should have TextMate stats");
    assert!(
        stats.convergences >= 1,
        "Expected convergence to kick in during typing, got {} convergences. \
         Without convergence the span offset adjustment isn't exercised.",
        stats.convergences
    );

    // Check colors didn't drift
    let colors_after: Vec<_> = (8..60)
        .filter_map(|x| {
            harness
                .get_cell_style(x, 15)
                .and_then(|s| s.fg)
                .map(|fg| (x, format!("{:?}", fg)))
        })
        .collect();

    assert_eq!(
        colors_before, colors_after,
        "Highlight colors on lines after the edit should not drift after typing. \
         This indicates cached span byte offsets are not being adjusted for inserts."
    );
}

// ============================================================
// Large file tests (> 1MB, fallback path)
// ============================================================

/// Generate a large Rust file (~11MB) for testing.
/// Uses the same `let var_N = "...";` pattern that reproduced the bug in tmux.
fn create_large_rust_file() -> tempfile::NamedTempFile {
    let mut f = tempfile::Builder::new()
        .suffix(".rs")
        .tempfile()
        .expect("create temp file");
    writeln!(f, "// Large test file").unwrap();
    writeln!(f, "fn main() {{").unwrap();
    for i in 0..270_000 {
        writeln!(f, "    let var_{} = \"hello world {}\";", i, i).unwrap();
    }
    writeln!(f, "    println!(\"done\");").unwrap();
    writeln!(f, "}}").unwrap();
    f.flush().unwrap();
    let size = f.as_file().metadata().unwrap().len();
    assert!(size > 10_000_000, "Test file should be > 10MB, got {} bytes", size);
    f
}

/// In a large file (>10MB): jump to end, jump to beginning, jump back to end.
/// The highlighting on the second visit to the end must match the first visit.
///
/// Reproduces a bug where the large-file fallback path either:
/// (a) didn't create checkpoints, so the second visit had no state to resume from, or
/// (b) found a distant checkpoint from the beginning and tried to parse the entire file.
///
/// Both cases result in highlighting loss or hang on the second visit.
#[test]
fn test_large_file_highlighting_survives_navigation() {
    // Use /tmp/large_test.rs — the same file used for tmux reproduction
    let path = std::path::Path::new("/tmp/large_test.rs");
    assert!(path.exists(), "Run: python3 -c \"...\" to generate /tmp/large_test.rs first");

    let file_size = std::fs::metadata(path).unwrap().len();

    let mut harness = create_harness();
    harness.open_file(path).unwrap();
    harness.render().unwrap();

    // Helper to dump rows with text and color info
    let dump_screen = |harness: &EditorTestHarness, label: &str| {
        eprintln!("=== {} === cursor={}", label, harness.cursor_position());
        eprintln!("  has_highlighter={}", harness.has_highlighter());
        for row in 2..8u16 {
            // Dump actual text
            let mut text = String::new();
            for col in 0..100u16 {
                text.push_str(&harness.get_cell(col, row).unwrap_or(" ".to_string()));
            }
            eprintln!("  text r{:02}: {}", row, text.trim_end());
            // Dump unique fg colors on this row (content area only)
            let mut row_colors = std::collections::HashSet::new();
            for col in 12..80u16 {
                if let Some(fg) = harness.get_cell_style(col, row).and_then(|s| s.fg) {
                    row_colors.insert(format!("{:?}", fg));
                }
            }
            eprintln!("  fgs  r{:02}: {:?}", row, row_colors);
        }
        if let Some(stats) = harness.highlight_stats() {
            eprintln!("  stats: bytes_parsed={} cache_hits={} cache_misses={} convergences={} checkpoints_updated={}",
                stats.bytes_parsed, stats.cache_hits, stats.cache_misses, stats.convergences, stats.checkpoints_updated);
        }
        let colors = collect_highlight_colors(harness, 2, 20);
        eprintln!("  highlight_colors={}", colors);
        colors
    };

    // Jump to end — extra tick_and_render to ensure async chunk loading completes
    harness.reset_highlight_stats();
    harness
        .send_key(KeyCode::End, KeyModifiers::CONTROL)
        .unwrap();
    for _ in 0..5 {
        harness.tick_and_render().unwrap();
    }
    let colors_end_1 = dump_screen(&harness, "FIRST CTRL+END");
    assert!(
        harness.cursor_position() > file_size as usize / 2,
        "Ctrl+End should reach end, cursor={} file={}",
        harness.cursor_position(), file_size
    );
    // First visit to end should have syntax highlighting (strings are green).
    // Before the fix, find_parse_resume_point would find a distant checkpoint
    // and slice_bytes would return empty for the unloaded range, giving 0 colors.
    assert!(
        colors_end_1 >= 1,
        "First visit to end of large file must have highlighting, got {} colors. \
         This indicates slice_bytes returned empty for the viewport range \
         (buffer chunks not loaded, or find_parse_resume_point chose a distant checkpoint).",
        colors_end_1
    );

    // Jump to beginning
    harness
        .send_key(KeyCode::Home, KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    let colors_home = dump_screen(&harness, "CTRL+HOME");
    assert!(
        colors_home >= 2,
        "Top of file must have highlighting (sanity check), got {} colors",
        colors_home
    );

    // Jump back to end — with the fix, this should use checkpoints from the
    // first visit and parse only the viewport region (~22KB), not the entire
    // file from a distant checkpoint (11MB).
    harness.reset_highlight_stats();
    harness
        .send_key(KeyCode::End, KeyModifiers::CONTROL)
        .unwrap();
    for _ in 0..3 {
        harness.tick_and_render().unwrap();
    }
    let colors_end_2 = dump_screen(&harness, "SECOND CTRL+END");

    // Second visit should also have highlighting (the original bug: 0 spans here)
    assert!(
        colors_end_2 >= 1,
        "Second visit to end of large file must have highlighting, got {} colors",
        colors_end_2
    );

    let stats = harness.highlight_stats().expect("should have TextMate stats");
    // With the fix: second visit should use nearby checkpoints, parsing ~22KB.
    // Without the fix: finds distant checkpoint from HOME and parses ~11MB.
    // Use 1MB as the threshold — anything over that indicates the distant
    // checkpoint bug.
    assert!(
        stats.bytes_parsed < 1_000_000,
        "Second visit to end of large file should not re-parse the entire file. \
         Parsed {} bytes (expected < 1MB). This indicates the distant checkpoint bug: \
         find_parse_resume_point found a checkpoint at the beginning of the file \
         and tried to parse from there.",
        stats.bytes_parsed
    );
}
