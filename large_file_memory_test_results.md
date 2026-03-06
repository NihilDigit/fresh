# Large File Memory Usage Test Results

## Test Setup
- **Build**: Debug mode (`cargo build`, no `--release`)
- **File**: 209MB text file (~928K lines, ~236 bytes/line)
- **Platform**: Linux 4.4.0, 21.5GB RAM
- **Method**: Ran editor in tmux, measured via `/proc/<pid>/status`

## Memory Measurements

| Scenario | VmRSS (KB) | VmSize (KB) | VmData (KB) |
|---|---|---|---|
| File opened, idle (t=3s) | 123,416 | 649,892 | 44,992 |
| After Ctrl+End (jump to EOF) | 126,056 | 649,892 | 44,992 |
| After search ("quick brown fox", 100K+ matches) | 133,548 | 656,008 | 51,108 |
| After scrolling 50 pages | 133,996 | 656,456 | 51,556 |

## Key Findings

1. **RSS ~131MB for a 209MB file** — the editor uses roughly 63% of the file
   size in resident memory. This is good — it doesn't duplicate the entire file
   in memory.

2. **VmData stays at ~50MB** — heap allocations are well-contained, suggesting
   the piece-tree data structure is efficient. The gap between VmData and VmRSS
   is likely memory-mapped file data and shared libraries.

3. **Jumping to EOF adds only ~2.6MB RSS** — no large spike from seeking to the
   end of a 200MB file.

4. **Search with 100K+ matches adds ~10MB** — searching the full file for a
   common pattern is relatively cheap.

5. **Scrolling adds negligible memory** — 50 PageDown operations added only
   ~450KB.

6. **Only 4 threads used** — lean thread usage.

## Summary

The editor handles a 209MB file well in debug mode, using ~131MB RSS. Memory
stays stable across navigation and search operations with no runaway growth.
