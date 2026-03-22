# Tier 1 Issues - Reproduction Investigation

**Date:** 2026-03-22
**Build:** debug (no --release)
**Environment:** Linux x86_64, tmux 3.4

## Summary Table

| Issue | Title | Reproduced? | Verdict |
|-------|-------|-------------|---------|
| #1068 | Tab size always 8 | **YES** | Per-language Go default overrides global setting; global `tab_size` has no effect on Go files |
| #1054 | Can't type chars with Python syntax (Windows) | **SKIPPED** | Reporter confirmed fixed in 0.2.5 |
| #1255 | Hidden cursor in Zellij | **SKIPPED** | Zellij-specific; works fine in tmux per reporter |
| #653 | Line numbers out of sync in large files | **YES** | Gutter shows byte offsets instead of line numbers in large file mode |
| #677 | Sometimes can't scroll to end | **NOT REPRODUCED** | Keyboard scrolling works; issue requires mouse scroll in specific narrow terminal split |
| #611 | File switch leaves buffer empty | **NOT REPRODUCED** | Intermittent race condition; tab switching and explorer switching both worked correctly in testing |
| #431 | Auto-indent staircase on paste | **YES** | Non-bracketed paste (line-by-line Enter) triggers auto-indent creating staircase; bracketed paste works correctly |
| #1113 | Ctrl+Enter writes [13;5u] | **YES** | CSI u key sequences are written as literal text in session attach mode; works correctly without session |

## Detailed Findings

---

### #1068 - Tab size always 8 (Go files)

**Status: REPRODUCED - but nuanced**

**Root cause:** Go language has a built-in default `tab_size` of 8, which overrides the global `editor.tab_size` setting. This is technically by design (Go convention is 8-wide tabs), but the UX is confusing because:

1. The global Settings > Editor > Tab Size shows "4" but Go files render tabs as 8-wide
2. Users don't know they need to set `languages.go.tab_size` in config.json
3. The per-language settings are not easily discoverable through the Settings UI

**Reproduction steps:**
1. Open a Go file with tab characters (e.g., standard Go code from `gofmt`)
2. Observe tabs render as 8 spaces wide
3. Set global `editor.tab_size` to 2 in config.json
4. Restart - Go tabs still render as 8 wide
5. Set `languages.go.tab_size` to 4 in config.json
6. Restart - Go tabs now render as 4 wide (workaround works)

**Evidence:** Captured tmux panes show line 6 `\tfmt.Println` rendering as 8 spaces with default config, and 4 spaces only after per-language override.

---

### #653 - Line numbers out of sync in large files

**Status: REPRODUCED**

**Root cause:** In large file mode (triggered for files > some threshold), the gutter displays **byte offsets** instead of line numbers, but they look identical to line numbers. For example, line 668 shows as `39302` in the gutter.

**Reproduction steps:**
1. Create a 300MB file: `for i in {1..5000000}; do echo "Line $i: content"; done > largefile.txt`
2. Open in fresh - notice first line shows `0` instead of `1`
3. Scroll down - gutter shows values like `39302`, `39361`, etc. instead of actual line numbers
4. Status bar shows correct cursor position, but gutter is misleading

**The maintainer acknowledged this as a UX issue** (not a bug per se): "In large file mode, line numbers are estimated by byte offset." Plans to add config for line number vs byte offset mode and use different formatting for byte offsets.

---

### #677 - Sometimes can't scroll to end

**Status: NOT REPRODUCED**

Tested with 200-line file in narrow (20-column) and normal terminal widths. Keyboard navigation (Ctrl+End, Page Down, Down arrow) all reached the end of file correctly. The scrollbar appeared accurate.

The issue may be specific to:
- Mouse wheel scrolling (could not simulate in tmux)
- Specific terminal splitting configuration
- Possibly already fixed in current codebase

---

### #611 - File switch leaves buffer empty until scroll

**Status: NOT REPRODUCED**

Tested multiple scenarios:
1. Opened large file, scrolled to line ~1100, switched to small file via Ctrl+Tab - content displayed correctly
2. Opened files via file explorer sidebar - content displayed correctly
3. Rapid Ctrl+Tab switching between files of different sizes - all displayed correctly

The comment suggests it happens when "open file X, go to line 1000, then open file Y which is smaller - editor displays file Y at line 1000 position." This scenario may be intermittent or depend on specific explorer interaction timing. Could also be fixed already.

---

### #431 - Auto-indent staircase on paste

**Status: REPRODUCED**

**Root cause:** When text is pasted without bracketed paste mode (i.e., the terminal sends characters line-by-line with Enter keys), fresh's auto-indent fires on each newline. Since the pasted text already contains indentation, each line gets BOTH its original indentation AND the auto-indent's additional indentation, creating a staircase effect.

**Reproduction steps:**
1. Open a Java file in fresh
2. Select all (Ctrl+A), cut (Ctrl+X) to clear and copy to clipboard
3. Send the Java code line-by-line with Enter between lines (simulating non-bracketed paste)
4. Result: escalating indentation on each line

**Example output:**
```
public class Test {                          <- correct
        public static void main(...) {       <- 8 spaces instead of 4 (staircase!)
                    System.out.println(...)  <- 20 spaces instead of 8
}                                            <- garbled
```

**Contrast with bracketed paste:** When the same content is pasted with bracketed paste markers (`\e[200~...\e[201~`), indentation is preserved correctly.

**Impact:** This affects users on Windows Terminal (which intercepts Ctrl+V and sends text without bracketed paste), and any terminal that doesn't support bracketed paste. The workaround is to use Edit > Paste from the menu, or remove the Ctrl+V binding from Windows Terminal's settings.

---

### #1113 - Ctrl+Enter writes [13;5u] in session attach mode

**Status: REPRODUCED**

**Root cause:** When fresh runs in session attach mode (`-a`), the session relay does not properly parse CSI u (Kitty keyboard protocol) escape sequences. The ESC byte is consumed but the remaining bytes `[13;5u` are passed through as literal text and inserted into the document.

**Reproduction steps:**
1. Start fresh in session mode: `fresh -a test_session`
2. Send CSI u sequence for Ctrl+Enter: `\x1b[13;5u`
3. Observe `[13;5u` is inserted as text at cursor position

**Control test:** Opening the same file WITHOUT session (`fresh test.java`) and sending the same sequence does NOT insert text - the key is properly handled or ignored.

**The reporter notes** this affects ALL custom CSI u keybindings in session mode, not just Ctrl+Enter. Any key encoded in the Kitty keyboard protocol format will be written as text when attached to a session.

---

## Priority Reassessment After Investigation

| Issue | Original Priority | Revised Priority | Notes |
|-------|------------------|-----------------|-------|
| #1068 | Tier 1 | **Tier 2** | It's a UX/discoverability issue, not completely broken. Per-language override works. |
| #1054 | Tier 1 | **CLOSED** | Fixed in 0.2.5, reporter confirmed. |
| #1255 | Tier 1 | **Tier 2** | Zellij-only, tmux works fine. May be a Zellij bug. |
| #653 | Tier 1 | **Tier 2** | Acknowledged UX issue, not data corruption. Byte offsets are confusing but functional. |
| #677 | Tier 1 | **Tier 3** | Could not reproduce. Possibly already fixed or very edge-case. |
| #611 | Tier 1 | **Tier 3** | Could not reproduce. Possibly already fixed or intermittent. |
| #431 | Tier 1 | **Tier 1** | Confirmed broken for non-bracketed paste. Affects Windows Terminal users daily. |
| #1113 | Tier 1 | **Tier 1** | Confirmed. Session mode is fundamentally broken for Kitty keyboard protocol. |

**Revised top priorities:**
1. **#431** - Fix non-bracketed paste auto-indent (detect paste heuristically or disable auto-indent for rapid input)
2. **#1113** - Fix CSI u parsing in session attach mode
3. **#1068** - Improve tab size UX (show per-language override in settings UI, or make global override languages)
4. **#653** - Differentiate byte offset display from line numbers in large file mode
