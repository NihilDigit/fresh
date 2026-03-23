# Investigation: E2E Test Does Not Reproduce Large File Highlighting Bug

## Goal

Make the e2e test in `crates/fresh-editor/tests/e2e/syntax_highlighting_embedded_offset.rs`
(`test_large_file_highlighting_survives_navigation`) reproduce the exact same
user-visible behavior as the real editor running in tmux.

## The Bug (tmux reproduction — works)

Using commit `c606febc` on branch `syntax-highlight-markers`, with the debug
binary and `/tmp/large_test.rs` (11MB Rust file, 270K lines of
`let var_N = "hello world N";`):

1. Open file, Ctrl+End → **210 highlight spans**, green strings visible
2. Ctrl+Home → correct highlighting at top
3. Ctrl+End again → **0 highlight spans**, all white text — **BUG**

The bug is caused by `find_parse_resume_point` in
`crates/fresh-editor/src/primitives/highlight_engine.rs` doing an unbounded
`query_range(0, desired_start+1)` which finds a distant checkpoint at byte ~12KB
(created during the HOME visit) and tries to parse from there to byte 11MB. The
resulting 11MB parse produces wrong/no highlights.

### Tmux reproduction command

```bash
cargo build  # debug build at commit c606febc
tmux new-session -d -s test -x 120 -y 40
tmux send-keys -t test "/path/to/target/debug/fresh /tmp/large_test.rs 2>/tmp/log" Enter
sleep 5
tmux send-keys -t test C-End    # step 1: first visit to end
sleep 5
tmux capture-pane -t test -e -p | sed -n '4p'   # shows [38;5;2m green colors
tmux send-keys -t test C-Home   # step 2: go home
sleep 3
tmux send-keys -t test C-End    # step 3: second visit to end
sleep 15
tmux capture-pane -t test -e -p | sed -n '4p'   # shows [38;5;15m all white — BUG
```

## The Problem (e2e test — doesn't match)

The e2e test opens the SAME file (`/tmp/large_test.rs`), does the same
Ctrl+End / Ctrl+Home / Ctrl+End sequence, but:

- **Step 1 (first Ctrl+End): 0 spans, 0 highlight colors** — tmux shows 210 spans
- Step 2 (Ctrl+Home): 5 highlight colors — matches tmux
- Step 3 (second Ctrl+End): 0 spans — matches tmux but trivially (both are 0)

The test can't detect the regression because step 1 already has no colors.

### Key diagnostic data

Both tmux and the test harness call `full_parse` with nearly identical viewport
ranges for the END visit:

| | tmux | test harness |
|---|---|---|
| viewport | `11384863..11387835` | `11384949..11387835` |
| desired_parse | `11374863..11387835` | `11374949..11387835` |
| `has_highlighter` | true | true |
| spans returned | **210** | **0** |

Same binary. Same file. Same `highlight_viewport` code. Same viewport byte
ranges (within ~100 bytes). Yet tmux gets 210 spans and the test gets 0.

### What we added to investigate

An `eprintln!` at the end of `full_parse` that prints the span count:
```rust
eprintln!("full_parse: returning {} spans for vp={}..{}", result.len(), viewport_start, viewport_end);
```

## Test file

Generate `/tmp/large_test.rs` (must exist before running the test):
```bash
python3 -c "
with open('/tmp/large_test.rs', 'w') as f:
    f.write('// Large test file\n')
    f.write('fn main() {\n')
    for i in range(270000):
        f.write(f'    let var_{i} = \"hello world {i}\";\n')
    f.write('    println!(\"done\");\n')
    f.write('}\n')
"
```

## Where to look next

1. **Buffer content at the viewport**: Does `buffer.slice_bytes(actual_start..parse_end)`
   return the same content in both cases? The test harness may load the large file
   differently (chunked/lazy loading might not have the END chunks loaded).

2. **Theme**: The test harness and real editor might use different themes. Check if
   `highlight_color(category, theme)` returns different colors.

3. **`scope_to_category` mapping**: Even if syntect produces the same scope stack,
   the category mapping might differ if the scope strings are different.

4. **`actual_start` in full_parse**: Both should start from `desired_parse_start`
   (the large file fallback). If the test's `actual_start` differs from tmux's,
   that explains the span count difference. The debug log shows this.

5. **Large file mode**: Check if `Buffer::load_from_file` with
   `large_file_threshold_bytes` results in a different buffer state than what the
   real editor uses after the user confirms the "large file" dialog.

## Files involved

- `crates/fresh-editor/src/primitives/highlight_engine.rs` — `TextMateEngine::full_parse`, `find_parse_resume_point`
- `crates/fresh-editor/tests/e2e/syntax_highlighting_embedded_offset.rs` — test `test_large_file_highlighting_survives_navigation`
- `crates/fresh-editor/tests/common/harness.rs` — `EditorTestHarness`, `open_file`, `send_key`, `render`
- `crates/fresh-editor/src/app/buffer_management.rs` — file loading with `large_file_threshold_bytes`
- `crates/fresh-editor/src/model/buffer.rs` — `Buffer::load_from_file`, `slice_bytes`
