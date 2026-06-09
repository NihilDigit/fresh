# Confirmed Bugs Registry

## Format
Each bug entry:
- **ID:** BUG-NNN
- **Title:** Short description
- **Severity:** Critical / High / Medium / Low
- **Status:** Open / Fixed / Closed
- **GitHub Issue:** #NNN (if filed)
- **Reproduction Steps:** (tmux send-keys sequence)
- **Expected:** What should happen
- **Actual:** What happened (from tmux capture-pane)
- **First Seen:** Date of first occurrence

---

## BUG-007: Workspace Trust Confirm Restarts Editor, Discarding Open File + Unsaved Edits (--no-restore)
- **ID:** BUG-007
- **Title:** "Trust folder & Allow Tooling" → full editor restart → CLI file and unsaved edits silently lost when launched with `--no-restore`
- **Severity:** High (silent data loss; no prompt, no recovery offer)
- **Status:** Open — GitHub issue #2291 filed (Run #22)
- **GitHub Issue:** [#2291](https://github.com/sinelaw/fresh/issues/2291)
- **Reproduction:**
  1. Folder with `compile_commands.json` (trust trigger); ensure no trust.json recorded
  2. `fresh --no-restore main.cpp` → SECURITY WARNING dialog
  3. (Variant) Keep Restricted → type into buffer (modified) → palette "Workspace Trust…"
  4. Select "Trust folder & Allow Tooling (T)", press Enter
- **Expected:** Open editors and unsaved content preserved (VS Code behavior)
- **Actual:** Editor restarts; main.cpp tab replaced by empty [No Name] + File Explorer; unsaved edits destroyed with no prompt. Recovery chunk written but never offered on reopen.
- **Log:** `INFO fresh::app::lifecycle: Restart requested with new working directory: <same cwd>`
- **Notes:** Default mode (session restore) rebuilds buffers incl. unsaved edits — bug is --no-restore specific. "Keep Restricted" does NOT restart.
- **First Seen:** Run #22, 2026-06-09 (v0.3.12); 3/3 reproducible

---

## BUG-006: SSH URL-style URI (`ssh://host/path`) Treated as Local File Path
- **ID:** BUG-006
- **Title:** `ssh://host/path` CLI argument silently opens empty local file instead of SSH connection
- **Severity:** High (documented feature not working; no error shown to user)
- **Status:** Open — GitHub issue #2221 filed (Run #21)
- **GitHub Issue:** [#2221](https://github.com/sinelaw/fresh/issues/2221) — filed Run #21 (2026-06-03)
- **Reproduction:**
  1. Launch Fresh with URL-style SSH URI: `fresh --no-restore "ssh://localhost/etc/hosts"`
  2. Observe: Tab opens titled "hosts", status bar shows "Local | ssh://localhost/etc/hosts", buffer is empty
  3. Check logs: `path="/home/user/fresh/ssh://localhost/etc/hosts"` — treated as relative local path
- **Expected:** Fresh connects via SSH per docs/features/ssh.md; status bar shows `[SSH:localhost]`
- **Actual:** Fresh treats URI as local relative path (CWD + URI). No connection, no error, empty file opened.
- **Contrast:** scp-style form (`user@host:/path`) correctly detects SSH and shows "Connecting via SSH to..."
- **First Seen:** Run #21, 2026-06-03

---

## BUG-001 (FIXED): *Keyboard Shortcuts* Buffer 'q' Does Not Close
- **ID:** BUG-001
- **Title:** `*Keyboard Shortcuts*` buffer 'q' does not close despite in-buffer documentation
- **Severity:** Low (Documentation/UX)
- **Status:** **FIXED** in v0.3.12 — confirmed via UI Run #22 ("Tab closed"); #2165 closed by maintainer 2026-06-07
- **GitHub Issue:** [#2165](https://github.com/sinelaw/fresh/issues/2165) — filed Run #16 (2026-05-31)
- **Reproduction:**
  1. Launch Fresh with `--no-restore`
  2. Press `Shift+F1` — `*Keyboard Shortcuts*` buffer opens
  3. Line 4 reads: "Press 'q' to close this buffer."
  4. Press `q`
  5. `tmux capture-pane -t SESSION -p | tail -3`
- **Expected:** Buffer closes
- **Actual:** Status bar shows "Editing disabled in this buffer"; buffer stays open
- **Workaround:** Use `Alt+W` to close
- **First Seen:** Run #12, 2026-05-27
- **Confirmed:** Run #14 (0.3.9), Run #15 (0.3.9), Run #16 (0.3.10)

## BUG-003 (FIXED): Review Diff "Discard hunk" Fails with "patch does not apply"
- **ID:** BUG-003
- **Title:** Review Diff "Discard hunk" fails with "Patch failed: error: patch does not apply"
- **Severity:** High (feature broken)
- **Status:** **FIXED** in 0.3.10 (Run #16, 2026-05-31)
- **GitHub Issue:** [#2117](https://github.com/sinelaw/fresh/issues/2117) — closed by maintainer
- **First Seen:** Run #5
- **Confirmed Fixed:** Run #16 — review_diff_test16.txt +4 lines, discard → "Review Diff: 0 hunks", file reverted to original state

---

## BUG-002: Edit Menu "Replace..." Label Maps to Query Replace (Ctrl+Alt+R), Not Basic Replace (Ctrl+R)
- **ID:** BUG-002
- **Title:** Edit menu mislabels "Query Replace" as "Replace..."
- **Severity:** Low (Documentation/UX)
- **Status:** Open
- **GitHub Issue:** [#2135](https://github.com/sinelaw/fresh/issues/2135) — filed in Run #13
- **Reproduction:**
  1. Launch Fresh: `fresh /tmp/any-file.txt`
  2. Press `F10` → navigate Right to Edit menu
  3. Find "Replace..." item — note shortcut: `Ctrl+Alt+R`
  4. Press Escape, open Command Palette (`Ctrl+P`), search "replace"
  5. Observe: "Replace" = `Ctrl+R` (basic); "Query Replace" = `Ctrl+Alt+R` (interactive)
- **Expected:** Edit menu "Replace..." should use `Ctrl+R` OR be labeled "Query Replace..."
- **Actual:** "Replace..." in Edit menu maps to `Ctrl+Alt+R` which is Query Replace (interactive). Basic Replace (`Ctrl+R`) has no Edit menu entry.
- **First Seen:** Run #12, 2026-05-27
- **Confirmed:** Run #13, 2026-05-27

---

## BUG-005 (FIXED): LSP Code Actions (Alt+.) Always Report "No Code Actions Available" for Diagnostic-Based Fixes
- **ID:** BUG-005
- **Title:** Alt+. code actions silently fail for clangd-reported "fix available" diagnostics due to empty `context.diagnostics`
- **Severity:** High (feature non-functional for all diagnostic-based fixes)
- **Status:** **FIXED** in v0.3.12 — confirmed via UI Run #22 (fix popup appears and applies); #2212 closed by maintainer 2026-06-08
- **GitHub Issue:** [#2212](https://github.com/sinelaw/fresh/issues/2212) — filed Run #19 (2026-06-03)
- **Reproduction:**
  1. Install clangd; configure `{"lsp": {"cpp": {"command": "clangd", "enabled": true}}}`
  2. Create `main.cpp` with `#include <string>` (unused) and `int z; return z;` (uninit)
  3. Launch Fresh, start clangd via LSP Status menu
  4. Wait for "LSP (cpp) ready"; open Diagnostics panel
  5. Observe `[W] 2:1 Included header string is not used directly **(fixes available)**`
  6. Navigate cursor to line 2, col 1; press `Alt+.`
  7. Status bar shows: **"No code actions available"**
- **Expected:** Code action popup with "Remove unused include" fix
- **Actual:** "No code actions available" — clangd returns empty `[]` because Fresh sends `"context":{"diagnostics":[]}` (empty) in every codeAction request
- **Evidence from LSP log:**
  - Fresh RECEIVED: `publishDiagnostics` with 7 diagnostics including "(fix available)" markers
  - Fresh SENT: `codeAction` with `"context":{"diagnostics":[]}` (always empty)
  - clangd replied: `"result":[]`
- **Root cause:** `context.diagnostics` in `textDocument/codeAction` is always empty — the "TODO: Implement diagnostic retrieval when needed" from source comment is not yet implemented
- **Workaround:** None — Alt+. does not provide diagnostic-based fixes
- **First Seen:** Run #18 (inconclusive), Run #19 (confirmed)
- **Confirmed:** Run #19, 2026-06-03

## BUG-004: Pyright LSP — All Request-Based Features Timeout After 30s
- **ID:** BUG-004
- **Title:** Pyright LSP: hover, definition, completions, signatureHelp all timeout; diagnostics not published
- **Severity:** High (major feature non-functional with real LSP)
- **Status:** Open
- **GitHub Issue:** [#2197](https://github.com/sinelaw/fresh/issues/2197) — filed in Run #17
- **Reproduction:**
  1. Install pyright: `pip install pyright`
  2. Config: `{"lsp": {"python": {"command": "pyright-langserver", "args": ["--stdio"], "enabled": true}}}`
  3. Create small Python project in /tmp with main.py
  4. Launch Fresh from that directory: `fresh --no-restore main.py`
  5. Wait for "LSP (python) ready" in status bar
  6. Try F12 (definition), Alt+K (hover), Ctrl+Space (completion) — all timeout after 30s
- **Expected:** Standard LSP features work (definition, hover, completion, diagnostics)
- **Actual:** Initialize succeeds ("Async LSP server initialized successfully") but ALL subsequent requests timeout. Diagnostics panel shows 0 items despite `[⚠ N]` counter (which counts timeout warnings, not code diagnostics).
- **Hint:** Log shows `LSP initialize result: position_encoding=None` — possible UTF-16 encoding mismatch causing pyright to discard all requests silently.
- **First Seen:** Run #17, 2026-06-02
- **Confirmed:** Run #17, 2026-06-02 (10/10 requests timed out across hover, definition, completion, signatureHelp)
