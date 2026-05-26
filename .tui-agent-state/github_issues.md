# GitHub Issues Index

This is the canonical reference for every GitHub issue this agent has filed.
**Check this file BEFORE searching GitHub or filing any new issue.**
If a topic appears here — open or closed — do not file a duplicate.

Last updated: Run #5, 2026-05-26

---

## Open Issues (agent-filed)

| # | Title | Filed | Status | Notes for next run |
|---|-------|-------|--------|-------------------|
| [#2109](https://github.com/sinelaw/fresh/issues/2109) | Ctrl+H doesn't open Find & Replace in terminals (Ctrl+H = Backspace) | Run #1 | **Open** | Terminal sends `0x08`. Verify whether Calibrate Keyboard wizard detects it. Do NOT re-file. |
| [#2111](https://github.com/sinelaw/fresh/issues/2111) | Search: F3 does not navigate to next match while search bar is open | Run #1 | **Open** | Confirmed usability bug: F3 silently ignored while search bar is open. Contradicts VS Code/Sublime/browser behavior. Issue updated with clear expected vs actual. Do NOT re-file. |
| [#2112](https://github.com/sinelaw/fresh/issues/2112) | Search/Replace panel: "No matches found" for files opened outside project workspace | Run #2 | **Open** | Search backend only indexes files within the git root. External files (e.g. /tmp) silently fail with misleading UI. Reproduced twice. Do NOT re-file. |
| [#2113](https://github.com/sinelaw/fresh/issues/2113) | Command palette: keystrokes typed in fuzzy file mode can leak into editor buffer | Run #2 | **Open** | Race condition during `>command` → file mode transition via BSpace. Timing-sensitive. Reproduced once. Do NOT re-file. |
| [#2117](https://github.com/sinelaw/fresh/issues/2117) | Review Diff: "Discard hunk" fails with "patch does not apply" even when patch is valid | Run #5 | **Open** | Confirmed 3x. Fresh's internal `git apply --reverse` fails; manual shell command works fine. Do NOT re-file. |

---

## Closed Issues (agent-filed — do NOT re-open or re-file)

| # | Title | Filed | Why Closed |
|---|-------|-------|-----------|
| [#2108](https://github.com/sinelaw/fresh/issues/2108) | Revert command fails when buffer has unsaved modifications | Run #1 | **False positive.** We triggered "Reload with Encoding..." not "Revert". `File > Revert` works correctly — shows `(r)evert/(c)ancel` prompt. |
| [#2110](https://github.com/sinelaw/fresh/issues/2110) | File opens as modified after session restore | Run #1 | **By design.** This is hot exit (`hot_exit` config, default on). Documented in `docs/features/session-persistence.md`. |

---

## Topics Already Investigated — Do Not Re-file

Even if the symptom looks fresh, these have already been fully investigated:

| Symptom | Conclusion | Issue |
|---------|------------|-------|
| `File > Revert` shows "Cannot reload" error | Wrong menu — that's "Reload with Encoding..." | #2108 closed |
| File opens with `[+]` / `*` on fresh launch | Hot exit restoring previous session | #2110 closed |
| `Ctrl+H` deletes a word | Terminal compat: `0x08` = Backspace | #2109 open |
| F3 does nothing during active search | F3 silently ignored while search bar is open; Enter closes bar first, then F3 works | #2111 open |
| Search/Replace panel returns "No matches found" for /tmp file | Search backend scoped to git workspace root only | #2112 open |
| Characters from command palette file search appear in editor buffer | Focus race condition during `>` → file mode transition | #2113 open |
| Review Diff `d` discard shows "Patch failed: error: patch failed" | Fresh's internal patch application is broken; manual `git apply --reverse` works | #2117 open |

---

## How to Use This File Before Filing

1. Describe the symptom you observed in one sentence.
2. Scan the "Topics Already Investigated" table above for a match.
3. Scan the open issues table — if your topic is there, add a comment to the existing issue rather than opening a new one.
4. Search GitHub with at least 3 different query variations.
5. Only then open a new issue and add a row to this file.

## Issue #2122 — move_to_paragraph_down/up Has No Default Keybinding (0.3.9 oversight)
- **Filed:** Run #7, 2026-05-26
- **URL:** https://github.com/sinelaw/fresh/issues/2122
- **Label:** bug, tui-agent-auto-bug
- **Status:** Open
- **Summary:** The `move_to_paragraph_down` and `move_to_paragraph_up` actions added in 0.3.9 (PR #2084) have no default keybinding and are not accessible from the command palette. The sibling `select_to_paragraph_*` actions have `Ctrl+Shift+↓/↑` bindings. Users cannot use the new "jump to paragraph" feature without manually binding it.
- **Search queries used:** `move_to_paragraph keybinding`, `paragraph navigation keybinding`, `paragraph action command palette missing`
