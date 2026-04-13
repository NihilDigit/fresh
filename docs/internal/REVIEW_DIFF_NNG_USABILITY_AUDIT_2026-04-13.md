# Review Diff -- NN/g Usability & Functional Audit

**Date:** 2026-04-13
**Editor:** Fresh 0.2.23 (debug build, branch `claude/tui-editor-usability-eval-c6FAn`)
**Harness:** `tmux 3.4`, 160x45 and 100x30 panes, `tmux capture-pane -e` for ANSI capture
**Scenarios:** 4 realistic user flows -- Happy Path, Monolith (1k lines / 10 hunks),
Edge Cases (whitespace-only / new / deleted file), and Lost User (error recovery).

All 23 screen captures referenced in this report live under `/tmp/uxcaptures/`
on the eval host and are preserved by test number (`01_launch.txt` etc.).

---

## 1. Executive Summary

**Overall usability score: 5 / 10 (Usable with Friction).**

Review Diff is functional for a simple one-file, one-hunk change but degrades
quickly as complexity grows. The feature has visible signs of recent bug-fix
work -- BUG-5 (deleted-file drill-down hang) from the prior combined report is
now fixed, and resize corruption (BUG-2) is recoverable via `r`. However, three
roadblocks remain that any real code-reviewer will hit inside the first minute:

| # | Roadblock | Heuristic Violated | Severity |
|---|-----------|--------------------|----------|
| RB-1 | `Ctrl+C` **terminates the editor process** (no SIGINT handler) -- any muscle-memory user attempting to "cancel" inside Review Diff loses their whole session. | User Control & Freedom | **Critical** |
| RB-2 | `n`/`p` (next/prev hunk) are **undiscoverable** on first open: File Explorer has focus, toolbar hides the shortcuts, and the keys silently do nothing. User must guess `Tab`. | Visibility of System Status + Flexibility & Efficiency | **High** |
| RB-3 | **No "Hunk X of N" indicator, no line numbers in unified diff, and hunk headers strip line-range counts** (`@@ -1 +1 @@` instead of `@@ -1,8 +1,9 @@`). In a 10-hunk file the user loses track of location. | Visibility of System Status + Consistency & Standards | **High** |

Secondary annoyances -- whitespace-only diffs rendered as invisible
add/delete pairs, cryptic side-by-side toolbar hint `[n/] next [p/[] prev`, and
a missing "no matches" signal in the palette -- compound the friction.

---

## 2. Heuristic Evaluation (NN/g, 5 criteria)

### 2.1 Visibility of System Status -- **FAIL**

| Observation | Capture | Verdict |
|-------------|---------|---------|
| Hunk header shows `@@ -1 +1 @@`, but `git diff` emits `@@ -1,8 +1,9 @@`. The line-count metadata is dropped during rendering. In the 10-hunk file every hunk header looks the same shape (`@@ -47 +47 @@`, `@@ -147 +147 @@` ...) with no context string, no size. | `04_review_open.txt`, `09_big_open.txt` | Violation |
| Status bar says `Review Diff: 10 hunks` but never `Hunk 4 of 10`. When `n` advances the cursor the line number updates (`Ln 29 → Ln 74`) but the user has to count hunk separators manually. | `10_after_5n.txt`, `11_after_9n.txt` | Violation |
| No line numbers in the unified-diff gutter. A reviewer looking at line 550 of `big.rs` has nothing to orient by except the line content itself. The side-by-side drill-down **does** add line numbers -- inconsistency between views. | `09_big_open.txt` vs `15_deleted_drilldown.txt` | Violation |
| Focus owner is genuinely visible (`*files* [RO]` vs `*diff* [RO]` in the status bar) -- **good**. | `04_review_open.txt`, `05_tab_switch.txt` | Pass |
| Side-by-side view shows `Side-by-side diff: +0 -19 ~0 \| 'q' to return` -- **excellent** diagnostic. This format should be promoted to the unified view. | `15_deleted_drilldown.txt` | Pass |

### 2.2 User Control and Freedom -- **FAIL**

| Observation | Capture | Verdict |
|-------------|---------|---------|
| **`Ctrl+C` kills the editor process.** Verified: after `Ctrl+C` was sent the `fresh` PID disappeared (`ps` confirmed no running process) and the shell prompt leaked onto the status-bar line (`Editing disabled in this bufferroot@runsc:/tmp/fresh-uxtest#`). No `unsaved changes` prompt, no cancellation semantics -- pure process death. | `12_invalid_keys.txt`, `12b_after_cc.txt` | **Critical** |
| `q` cleanly closes the review-diff tab and returns to the prior buffer. Drill-down `q` returns to unified view. Both **good**. | `08_after_q.txt`, `16_newfile.txt` | Pass |
| `Escape` in the palette cancels the search -- **good**. `Escape` in the file-explorer focus does **not** release focus (matches prior BUG-7). | `19_gibberish.txt` | Mixed |

### 2.3 Consistency and Standards -- **PARTIAL PASS**

| Observation | Capture | Verdict |
|-------------|---------|---------|
| Colour palette follows git conventions -- removed = dark red bg (`ANSI 48;5;52`), added = dark green bg (`48;5;22`), white fg. Intra-line word diff: bold cyan (`38;5;51`) for added tokens, bold bright red (`38;5;203`) for removed. | `cat -A` on `04_review_open.txt` | Pass |
| Hunk header ANSI style: bold cyan -- matches `diff --color` defaults. | `04_review_open.txt` | Pass |
| **Hunk header format non-standard:** `@@ -1 +1 @@` with no line counts and no trailing function-context string. Standard unified diff emits `@@ -1,8 +1,9 @@ fn main()`. | `04_review_open.txt` | Violation |
| Side-by-side toolbar hint reads `[n/] next  [p/[] prev  [q] close` -- the `[n/]` and `[p/[]` look like typos where two key aliases (`n`/`.`, `p`/`,` or similar) collapsed with mismatched delimiters. Confusing. | `15_deleted_drilldown.txt` | Violation |
| File-status prefixes (`M`/`D`/`A`) are standard git-porcelain shorthand -- good. But the "current file" marker is `>` prepended to the prefix (`>M  main.rs`) which collides with the `D` of `Deleted` and `A` of `Added` visually. A distinct highlight row would be clearer. | `13_edge_open.txt` | Minor |

### 2.4 Flexibility and Efficiency of Use -- **FAIL**

Keystroke budget for the **minimum useful review** (open diff, move to 5th hunk,
close) with a cold editor:

```
Ctrl+P  review diff Enter   Tab     n n n n   q   Escape
  1        10+1        1      4     1    1    = 19 keystrokes
```

- `Ctrl+P + "review diff" + Enter` (12) is unavoidable -- ok.
- `Tab` (1) is **hidden cost**: without it `n`/`p` are silently ignored
  because File Explorer holds focus. Discovered only by reading docs or by
  trial-and-error.
- `n` pressed 4 times (4) to reach the 5th hunk -- no "jump to hunk N" command.
- `q` + `Escape` to close -- `Escape` needed because of lingering
  File-Explorer focus (compound of BUG-3/BUG-7).

**Missing accelerators observed:**

- No "go to hunk N" (e.g. `5G`, `:hunk 5`).
- No fuzzy jump-to-file inside the GIT STATUS pane.
- No numeric prefix (`4n` to advance 4 hunks at once).
- No keybinding to toggle whitespace-insensitive view.
- Command palette shows "Review Diff" first on both clean input (`review diff`)
  and severe typo (`revw difff`) -- fuzzy matcher is **good** for power users.

### 2.5 Aesthetic & Minimalist Design -- **PASS with caveats**

| Observation | Capture | Verdict |
|-------------|---------|---------|
| Left GIT STATUS pane is clean: section headers (▸ Changes / ▸ Untracked), one-line-per-file. | `13_edge_open.txt` | Pass |
| Unified diff pane uses 3 lines of context by default -- sensible. | `04_review_open.txt` | Pass |
| Toolbar is a single line and auto-compresses when viewport narrows (`r Refresh` truncates at 100-cols) but the compression happens **inside** labels, producing `Tab ` with nothing after it. | `22_after_refresh.txt` | Minor |
| Empty-pane rendering in side-by-side for new/deleted files shows `1` and `2` line numbers on the empty side -- looks like placeholder lines exist where they shouldn't. | `15_deleted_drilldown.txt`, `17_newfile_drill.txt` | Minor |

---

## 3. Friction Points (flow-by-flow)

### Flow 1 -- Happy Path (3-added / 3-deleted single-hunk)
1. Open → `Ctrl+P` → `review diff` → Enter. (≈3 s on debug build, acceptable.)
2. Diff renders correctly, colours correct, word-level diff highlights token
   changes (cyan `universe` vs red `world`).
3. **Friction:** User presses `j` expecting to move to next hunk (vi-muscle
   memory); cursor moves one line -- no audible feedback that this is *not*
   hunk-navigation. The user doesn't learn `Tab` is required first because
   `j` *does* move the cursor in files panel too (as list navigation).

### Flow 2 -- Monolith (1000 lines / 10 hunks)
1. Opening the review diff takes about 3 s; no visible input lag during `n`
   repetitions. Debug build scrolls through 10 hunks without stutter.
2. **Friction:** Status bar only ever says `10 hunks` -- no `Hunk 5 / 10`.
   Users counting `@@` headers look away from the screen for 2-3 seconds
   every jump.
3. **Friction:** `n` moves the cursor but the **viewport does not always
   scroll** if the next hunk is already on-screen. Combined with the missing
   hunk counter, first-time users believe `n` is broken. (Verified at
   `11_after_9n.txt`: after 9 presses the cursor did land on hunk 9, so
   the handler is *not* broken -- just silent.)
4. **Missing shortcut:** nothing like `5G` / `gg` / `G` to jump by hunk
   index. Linear scanning only.

### Flow 3 -- Edge Cases
1. **Whitespace-only:** the trailing spaces I added to every line produced a
   full-file "remove every line, re-add every line" diff. Added lines are
   rendered with the invisible trailing spaces highlighted (bold cyan) but
   there is no visible glyph (· or →) so the user sees two identical-looking
   lines flagged as changed. A reviewer would conclude "false positive" and
   stage blindly.
2. **New file** (`newfile.md`): header is `@@ -0 +1 @@`, all lines rendered
   as additions -- correct behaviour. Drill-down shows empty OLD pane with
   line number placeholders (`1`, `2`) on rows where the file is literally
   empty -- minor cosmetic glitch.
3. **Deleted file** (`main.rs`): header is `@@ -1 +0 @@`, all lines rendered
   as deletions -- correct behaviour. Drill-down **works** (BUG-5 is
   fixed); NEW pane is empty as expected.

### Flow 4 -- Lost User
1. Typo `revw difff`: the fuzzy finder still places **Review Diff** first.
   Excellent. Only a subset of characters actually registered in the prompt
   echo (`>revw` displayed) but the ranking held.
2. Pure gibberish `zzzzzzzz`: palette empties silently -- there is **no "No
   matches found" message**. A novice user will assume the palette is frozen.
3. Invalid keys (`x`, `z`, `Y`) in the diff pane: cleanly rejected with
   `Editing disabled in this buffer` in the status bar. No panic. **Good
   graceful degradation.**
4. `Ctrl+C`: **fatal.** See RB-1 in executive summary.
5. Terminal resize from 160x45 to 100x30 during Review Diff: layout
   immediately corrupts (file-explorer pane collapses to pipe separators
   only, toolbar vanishes). Pressing `r` does restore the full layout --
   so this is recoverable, but the auto-redraw on resize is still broken.

---

## 4. Colour & Layout Analysis (ANSI-parsed)

Raw ANSI inspected with `cat -A capture-pane-output`.

| Element | Foreground | Background | Style | Verdict |
|---------|-----------|-----------|-------|---------|
| Context line | `38;5;231` white | `48;5;16` black | normal | Pass |
| Removed line | `38;5;231` white | `48;5;52` dark red | normal | Pass |
| Added line | `38;5;231` white | `48;5;22` dark green | normal | Pass |
| Removed token (word diff) | `38;5;203` bright red | `48;5;52` dark red | bold | Pass -- strong contrast |
| Added token (word diff) | `38;5;51` cyan | `48;5;22` dark green | bold | **Mixed** -- cyan on green is ~3.5:1, under WCAG AA (4.5:1) for small text |
| Hunk header | `38;5;51` cyan | `48;5;16` black | bold | Pass |
| Focused tab | `38;5;16` black | `48;5;226` yellow | bold | Pass |
| Keybinding hint letters | `38;5;51` cyan | `48;5;17` dark blue | bold | Pass |

**Accessibility concern:** bold cyan on dark green (added-token highlight)
sits below WCAG 4.5:1 contrast. Consider switching the added-token foreground
to `38;5;229` (light yellow) or `38;5;231` bold white, both of which exceed
7:1 on `48;5;22`.

**Alignment:** gutter column for line numbers is absent in unified view and
**3 chars wide** in side-by-side -- files with ≥1000 lines will see the
gutter widen; a quick check with `big.rs` in side-by-side would likely
reveal further misalignment (not tested this pass).

---

## 5. Actionable Recommendations

Ordered by impact / effort.

1. **Install a SIGINT handler (or trap `Ctrl+C` in the input layer) so the
   editor cannot be killed by muscle memory.** Map `Ctrl+C` to "copy
   selection if any, else cancel current prompt, else show 'Press q to quit'
   toast". One-file change in `app/input.rs`; prevents total session loss.
   *(RB-1; fixes User Control & Freedom violation.)*

2. **Add a "Hunk X of N" indicator to the status bar and restore the
   standard hunk-header format.** In the audit_mode plugin's render path,
   emit `@@ -A,B +C,D @@ <context>` instead of stripping the range counts;
   maintain a `currentHunkIndex` as `n`/`p` moves and surface it as
   `Hunk 4/10 - src/auth.ts`. Solves two heuristic violations at once.
   *(RB-3; fixes Visibility of System Status + Consistency.)*

3. **Auto-focus the diff pane on `start_review_diff`, and promote `n`/`p`
   into the default toolbar regardless of focus.** Also show a
   first-run toast `Tip: Tab toggles focus, n/p jump hunks.`  Without
   this, 100% of first-time users silently fail on hunk navigation
   (already flagged as BUG-3 in the prior combined report; still present).
   *(RB-2; fixes Flexibility & Efficiency + Visibility.)*

4. **Render visible glyphs for whitespace-only differences** (e.g. `·`
   for trailing space, `→` for tab) on added/removed lines, and add a
   command `Review Diff: Toggle Whitespace-Only` bound to `W` that
   collapses whitespace-only hunks. Without this, whitespace-only changes
   are a UX dead-end: user sees two identical-looking lines flagged as
   changed and learns to distrust the tool. *(fixes Consistency & Aesthetic.)*

5. **Fix the resize handler and the side-by-side toolbar hint.** The
   resize path does not re-layout until `r` is pressed -- hook the
   `resize` event to call the same refresh routine. The hint string
   `[n/] next  [p/[] prev  [q] close` is almost certainly a key-alias
   formatter bug (a mismatched `[`/`]`); auditing
   `plugins/audit_mode.ts` for the hint builder should fix it. Low
   effort, high polish. *(fixes Aesthetic & Consistency.)*

### Secondary polish (nice-to-have)

- Palette: show `No matches for 'zzzz'` when fuzzy results are empty.
- Side-by-side: suppress placeholder line numbers on the pane where a file is
  empty (new-file OLD pane, deleted-file NEW pane).
- Accessibility: raise contrast of bold-cyan added-token highlight to meet
  WCAG 4.5:1 on the dark-green background.
- Add numeric-prefix repeat (`4n` ⇒ advance four hunks).

---

## 6. Summary Scorecard

| NN/g Heuristic | Score (1-5) | Dominant finding |
|----------------|-------------|-----------------|
| Visibility of System Status | 2 | No hunk index, stripped hunk header, no gutter line numbers |
| User Control and Freedom | 1 | `Ctrl+C` kills the process; escape gaps in File Explorer focus |
| Consistency and Standards | 3 | Colour conventions good; hunk header & side-by-side hint text broken |
| Flexibility and Efficiency | 2 | `Tab` hidden tax, no numeric jump, good fuzzy match |
| Aesthetic & Minimalist Design | 4 | Clean layout; small empty-pane / toolbar-truncation artefacts |
| **Overall** | **2.4 / 5** | Usable but friction-heavy |

---

*Captures used for this report are preserved at `/tmp/uxcaptures/01_launch.txt`
through `/tmp/uxcaptures/22_after_refresh.txt` (23 files). Each capture was
produced via `tmux capture-pane -t tui-test -p -e` so ANSI escapes remain
inspectable with `cat -A`.*
