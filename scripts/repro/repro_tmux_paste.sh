#!/usr/bin/env bash
#
# Reproduces the tmux :paste indent bug in Fresh.
#
# Scenario: a long markdown document is loaded into tmux's paste buffer
# and then pasted into a brand-new empty Fresh buffer using tmux's own
# `paste-buffer` command (equivalent to `: paste` in the tmux command
# prompt).
#
# Observed bug:
#   * Each successive line is indented further than the previous one
#     (the "staircase" effect from auto-indent firing on every newline).
#   * The paste stalls partway through and only resumes after a keypress.
#
# This script captures the full pane content after the paste and writes
# it to scripts/repro/out.txt so the staircase indentation is visible.
#
# Requirements: tmux, a built `fresh` binary on PATH or at
# target/{release,debug}/fresh.

set -u

cd "$(dirname "$0")/../.."

# Locate the fresh binary --------------------------------------------------
FRESH_BIN=""
for candidate in \
    "$PWD/target/release/fresh" \
    "$PWD/target/debug/fresh" \
    "$(command -v fresh || true)"; do
    if [ -n "$candidate" ] && [ -x "$candidate" ]; then
        FRESH_BIN="$candidate"
        break
    fi
done
if [ -z "$FRESH_BIN" ]; then
    echo "error: could not find a fresh binary; run \`cargo build\` first" >&2
    exit 1
fi
echo "using fresh: $FRESH_BIN"

SAMPLE="$PWD/scripts/repro/sample.md"
OUT="$PWD/scripts/repro/out.txt"
SESSION="fresh_paste_repro_$$"

# Make sure we leave no orphan tmux sessions even on error.
cleanup() {
    tmux kill-session -t "$SESSION" 2>/dev/null || true
}
trap cleanup EXIT

# Start tmux with fresh running in an empty scratch buffer ----------------
# `--no-restore --no-plugins` keeps the run hermetic and avoids interference
# from a saved workspace; no FILES argument means a new untitled buffer.
tmux new-session -d -s "$SESSION" -x 120 -y 40 \
    "$FRESH_BIN --no-restore --no-plugins"

# Give fresh a moment to enable bracketed paste / mouse / etc.
sleep 1.5

# Load the markdown sample into tmux's paste buffer and paste it ----------
tmux load-buffer -b paste_repro "$SAMPLE"
echo "loaded $(wc -l <"$SAMPLE") lines / $(wc -c <"$SAMPLE") bytes into tmux buffer"

# Equivalent to typing `:paste` (or `:paste-buffer`) at tmux's command
# prompt — this is exactly what the user described.
tmux paste-buffer -b paste_repro -t "$SESSION"

# Give the paste 2s to do its (mis)handling. The bug typically stalls the
# paste before this completes, which is itself part of the reproduction.
sleep 2

# Capture the pane after the (buggy) paste --------------------------------
tmux capture-pane -t "$SESSION" -p -S - -E - > "$OUT"
echo "captured pane to $OUT"

echo "----- last 25 lines of pane (note the staircase indent) -----"
tail -n 25 "$OUT"
echo "-------------------------------------------------------------"

# Heuristic check: count lines that start with >= 4 spaces. If the paste
# was clean, only the deliberately-indented examples in sample.md should
# match. If the bug is present, almost every line is shifted.
indented=$(grep -c '^[[:space:]]\{4,\}' "$OUT" || true)
echo "lines starting with >=4 leading spaces in pane capture: $indented"
echo "(sample.md has $(grep -c '^[[:space:]]\{4,\}' "$SAMPLE") such lines by design)"
