//! Repro for: user-configured `formatter.args` for the C language don't seem
//! to be honored when invoking the formatter (Format Buffer).
//!
//! User config (from bug report):
//!     "languages": {
//!       "c": {
//!         "formatter": {
//!           "command": "clang-format",
//!           "args": [
//!             "--style=file",
//!             "--fallback-style=none",
//!             "--assume-filename=%path%"
//!           ]
//!         }
//!       }
//!     }
//!
//! We replace `clang-format` with a shim that captures its argv to a sentinel
//! file, then run "Format Buffer" on a `.c` file. Two facts get exercised:
//!
//! 1. Are the user-supplied args reaching the formatter at all?
//! 2. Is the `%path%` token in `--assume-filename=%path%` substituted?
//!    (The editor only substitutes `$FILE` — see
//!    `crates/fresh-editor/src/app/on_save_actions.rs` `run_formatter`.)

#![cfg(unix)]

use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};
use fresh::config::{Config, FormatterConfig};
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn write_argv_capture_shim(path: &std::path::Path, sentinel: &std::path::Path) {
    // Writes "$#\n$1\n$2\n..." to the sentinel, drains stdin, emits a fixed
    // string on stdout so the editor has *something* to replace the buffer
    // with (so the format action visibly "succeeds").
    let script = format!(
        "#!/bin/sh\n\
         {{\n\
           printf '%s\\n' \"$#\"\n\
           for a in \"$@\"; do printf '%s\\n' \"$a\"; done\n\
         }} > {sentinel}\n\
         cat > /dev/null\n\
         printf 'FORMATTED\\n'\n",
        sentinel = sentinel.display(),
    );
    fs::write(path, script).unwrap();
    let mut perm = fs::metadata(path).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(path, perm).unwrap();
}

fn config_with_c_clang_format(shim: &std::path::Path) -> Config {
    let mut config = Config::default();
    let entry = config.languages.get_mut("c").expect("default c lang config");
    entry.format_on_save = false;
    entry.formatter = Some(FormatterConfig {
        command: shim.display().to_string(),
        args: vec![
            "--style=file".to_string(),
            "--fallback-style=none".to_string(),
            "--assume-filename=%path%".to_string(),
        ],
        stdin: true,
        timeout_ms: 10_000,
    });
    config
}

fn run_format_buffer_via_palette(harness: &mut EditorTestHarness) {
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.wait_for_prompt().unwrap();
    harness.type_text("Format Buffer").unwrap();
    harness.wait_for_screen_contains("Format Buffer").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.wait_for_prompt_closed().unwrap();
}

#[test]
fn clang_format_user_args_are_forwarded_with_placeholder_substituted() {
    let tmp = tempfile::TempDir::new().unwrap();
    let shim = tmp.path().join("clang-format");
    let sentinel = tmp.path().join("argv.txt");
    write_argv_capture_shim(&shim, &sentinel);

    let c_path = tmp.path().join("sample.c");
    fs::write(&c_path, "int main(void){return 0;}\n").unwrap();

    let config = config_with_c_clang_format(&shim);
    let mut harness = EditorTestHarness::with_config(80, 24, config).unwrap();
    harness.open_file(&c_path).unwrap();
    harness.render().unwrap();

    run_format_buffer_via_palette(&mut harness);

    // Give the shim a moment to land its sentinel.
    harness
        .wait_until(|_| sentinel.exists())
        .expect("formatter shim never wrote argv sentinel");

    let captured = fs::read_to_string(&sentinel).unwrap();
    eprintln!("=== captured clang-format argv ===\n{captured}=== end ===");
    let mut lines = captured.lines();
    let argc: usize = lines.next().unwrap().parse().unwrap();
    let argv: Vec<&str> = lines.collect();
    assert_eq!(argc, argv.len(), "argc/argv mismatch");

    // (1) The user's args must be passed through — not stripped.
    assert!(
        argv.iter().any(|a| *a == "--style=file"),
        "expected --style=file in argv, got {argv:?}"
    );
    assert!(
        argv.iter().any(|a| *a == "--fallback-style=none"),
        "expected --fallback-style=none in argv, got {argv:?}"
    );

    // (2) `%path%` must be replaced with the buffer's path. If this fails,
    // the editor is passing the literal token to clang-format, which then
    // can't resolve `.clang-format` relative to the real file — matching the
    // user's "args appear not to be applied" symptom.
    let assume = argv
        .iter()
        .find(|a| a.starts_with("--assume-filename="))
        .copied()
        .expect("missing --assume-filename in argv");
    assert!(
        !assume.contains("%path%"),
        "placeholder `%path%` was passed literally to clang-format \
         (arg was {assume:?}); editor only substitutes `$FILE`."
    );
    let expected_suffix = c_path.display().to_string();
    assert_eq!(
        assume,
        format!("--assume-filename={expected_suffix}"),
        "expected --assume-filename to be substituted with the file path"
    );
}
