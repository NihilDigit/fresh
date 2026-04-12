#!/usr/bin/env python3
"""
Record a scripted Fresh demo as an asciinema v2 cast.

Drives a fresh binary inside a fork()'d pty, sends a canned key timeline,
and writes the output to an asciicast file that can be played back with
asciinema-player on the website.

Usage:
    scripts/record-asciinema/record.py [output.cast] [--fresh PATH] [--demo DIR]

If no paths are given, defaults to:
    output   : homepage/public/fresh-demo.cast
    fresh    : ./target/release/fresh  (or  ./target/debug/fresh)
    demo dir : /tmp/fresh-demo-workspace  (created by setup-demo.sh)
"""
import argparse
import fcntl
import json
import os
import pty
import select
import struct
import sys
import termios
import time

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
DEFAULT_OUT = os.path.join(REPO_ROOT, "homepage", "public", "fresh-demo.cast")
DEFAULT_DEMO = "/tmp/fresh-demo-workspace"

COLS = 110
ROWS = 30

# --- Key constants ------------------------------------------------------------
ESC = b"\x1b"
CR = b"\r"
BS = b"\x7f"
TAB = b"\t"
def CTRL(c): return bytes([ord(c.upper()) - 64])
DOWN = ESC + b"[B"
UP = ESC + b"[A"
CS_RIGHT = ESC + b"[1;6C"          # Ctrl+Shift+Right (select word)
HOME = ESC + b"[H"
PGDN = ESC + b"[6~"
PGUP = ESC + b"[5~"

def type_text(s, per_char=0.06):
    return [(per_char, ch.encode("utf-8")) for ch in s]

# --- Demo timeline ------------------------------------------------------------
# Each tuple is (delay_before_event_seconds, bytes_to_write).
# An empty payload (b"") is a pure pause.
#
# Keep this script in sync with what the UI actually does. Verified against
# Fresh v0.2.23 by interacting manually via tmux before recording.
TIMELINE = []

# 0. Grace period so Fresh's first paint settles.
TIMELINE += [(2.2, b"")]

# 1. Scroll the file a little so the audience sees more than the opening frame.
for _ in range(2):
    TIMELINE += [(0.55, PGDN)]
TIMELINE += [(0.6, PGUP), (0.6, PGUP)]

# 2. Open command palette, skim through commands so the viewer sees how
#    much ships in the palette.
TIMELINE += [(0.7, CTRL("P")), (1.0, b"")]
TIMELINE += [(0.14, DOWN)] * 6
TIMELINE += [(0.7, b"")]
TIMELINE += [(0.14, UP)] * 4
TIMELINE += [(0.6, b"")]

# 3. Live-preview theme picker — one of the more visually striking features.
TIMELINE += type_text("select theme")
TIMELINE += [(0.6, b""), (0.5, CR), (1.0, b"")]
# Each Down cycles to a theme and applies it immediately.
TIMELINE += [(0.55, DOWN)] * 5
TIMELINE += [(0.8, b"")]
TIMELINE += [(0.3, UP)] * 3
TIMELINE += [(0.6, b""), (0.4, CR), (1.0, b"")]

# 4. Fuzzy file finder: backspace past the default `>` prefix, type, open.
TIMELINE += [(0.6, CTRL("P")), (0.4, b""), (0.15, BS)]
TIMELINE += type_text("not")
TIMELINE += [(0.7, b""), (0.3, CR), (1.0, b"")]

# 5. Buffer switcher (# prefix) back to main.rs.
TIMELINE += [(0.5, CTRL("P")), (0.3, b""), (0.15, BS)]
TIMELINE += type_text("#main")
TIMELINE += [(0.5, b""), (0.3, CR), (0.9, b"")]

# 6. Multi-cursor demo: jump to line, select a word, Ctrl+D a few times.
TIMELINE += [(0.4, CTRL("G")), (0.3, b"")]
TIMELINE += type_text("7")
TIMELINE += [(0.2, CR), (0.5, b""), (0.3, HOME), (0.3, CS_RIGHT), (0.6, b"")]
TIMELINE += [(0.55, CTRL("D"))] * 3
TIMELINE += [(1.0, b""), (0.5, ESC), (0.5, b"")]

# 7. Live Grep — split preview is eye-catching.
TIMELINE += [(0.5, CTRL("P")), (0.3, b"")]
TIMELINE += type_text("live grep")
TIMELINE += [(0.6, b""), (0.4, CR), (1.0, b"")]
TIMELINE += type_text("User", per_char=0.1)
TIMELINE += [(1.8, b"")]
TIMELINE += [(0.35, DOWN)] * 4
TIMELINE += [(0.8, b""), (0.4, ESC), (0.5, b"")]

# 8. Magit-style Review Diff.
TIMELINE += [(0.5, CTRL("P")), (0.3, b"")]
TIMELINE += type_text("review diff")
TIMELINE += [(0.6, b""), (0.4, CR), (2.0, b"")]
TIMELINE += [(0.5, TAB), (0.6, b"")]              # switch to diff panel
TIMELINE += [(0.6, b"n"), (0.6, b"n"), (0.8, b"p"), (0.8, b"")]
TIMELINE += [(0.7, b"s"), (1.4, b"")]              # stage a hunk
TIMELINE += [(0.6, b"q"), (0.8, b"")]              # close review

# 9. Calm final beat.
TIMELINE += [(1.2, b"")]


def find_fresh_binary():
    """Prefer $FRESH, then release, then debug target builds."""
    env = os.environ.get("FRESH")
    if env and os.path.isfile(env) and os.access(env, os.X_OK):
        return env
    for rel in ("target/release/fresh", "target/debug/fresh"):
        candidate = os.path.join(REPO_ROOT, rel)
        if os.path.isfile(candidate) and os.access(candidate, os.X_OK):
            return candidate
    # Last resort: PATH.
    for p in os.environ.get("PATH", "").split(os.pathsep):
        candidate = os.path.join(p, "fresh")
        if os.path.isfile(candidate) and os.access(candidate, os.X_OK):
            return candidate
    raise SystemExit(
        "Could not find 'fresh' binary. Build with `cargo build --bin fresh` "
        "(or `cargo build --release --bin fresh`) or set FRESH=/path/to/fresh."
    )


def parse_args():
    p = argparse.ArgumentParser(description=__doc__.splitlines()[1])
    p.add_argument("output", nargs="?", default=DEFAULT_OUT,
                   help=f"output .cast path (default: {DEFAULT_OUT})")
    p.add_argument("--fresh", default=None, help="path to fresh binary")
    p.add_argument("--demo", default=DEFAULT_DEMO,
                   help=f"demo workspace directory (default: {DEFAULT_DEMO})")
    return p.parse_args()


def record(out_path, fresh, demo):
    pid, master_fd = pty.fork()
    if pid == 0:
        # Child: exec Fresh inside the pty.
        os.chdir(demo)
        env = os.environ.copy()
        env["TERM"] = "xterm-256color"
        env["COLUMNS"] = str(COLS)
        env["LINES"] = str(ROWS)
        os.execvpe(
            fresh,
            [fresh, "--no-upgrade-check", "--no-restore", "main.rs"],
            env,
        )
        os._exit(1)

    # Parent: size the pty window and record.
    fcntl.ioctl(
        master_fd,
        termios.TIOCSWINSZ,
        struct.pack("HHHH", ROWS, COLS, 0, 0),
    )

    os.makedirs(os.path.dirname(os.path.abspath(out_path)) or ".", exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        header = {
            "version": 2,
            "width": COLS,
            "height": ROWS,
            "timestamp": int(time.time()),
            "env": {"SHELL": "/bin/sh", "TERM": "xterm-256color"},
            "title": "Fresh Editor",
        }
        f.write(json.dumps(header) + "\n")

        start = time.monotonic()

        # Turn the relative timeline into absolute send times.
        schedule = []
        t = 0.0
        for delay, payload in TIMELINE:
            t += delay
            if payload:
                schedule.append((t, payload))
        total = t + 1.0

        idx = 0
        while True:
            now = time.monotonic()
            elapsed = now - start

            while idx < len(schedule) and schedule[idx][0] <= elapsed:
                try:
                    os.write(master_fd, schedule[idx][1])
                except OSError:
                    pass
                idx += 1

            next_deadline = schedule[idx][0] if idx < len(schedule) else total
            timeout = max(0.005, min(0.04, next_deadline - elapsed))

            try:
                r, _, _ = select.select([master_fd], [], [], timeout)
            except (OSError, ValueError):
                break

            if master_fd in r:
                try:
                    chunk = os.read(master_fd, 4096)
                except OSError:
                    chunk = b""
                if not chunk:
                    break
                ts = time.monotonic() - start
                f.write(json.dumps([ts, "o", chunk.decode("utf-8", errors="replace")]) + "\n")

            if elapsed >= total:
                break
            try:
                wpid, _ = os.waitpid(pid, os.WNOHANG)
            except ChildProcessError:
                break
            if wpid != 0:
                break

    try:
        os.kill(pid, 9)
    except ProcessLookupError:
        pass
    try:
        os.close(master_fd)
    except OSError:
        pass


def main():
    args = parse_args()
    fresh = args.fresh or find_fresh_binary()
    if not os.path.isdir(args.demo):
        sys.stderr.write(
            f"error: demo workspace not found at {args.demo}\n"
            f"  run: scripts/record-asciinema/setup-demo.sh {args.demo}\n"
        )
        sys.exit(2)

    print(f"fresh:  {fresh}")
    print(f"demo:   {args.demo}")
    print(f"output: {args.output}")

    t0 = time.monotonic()
    record(args.output, fresh, args.demo)
    dt = time.monotonic() - t0
    sz = os.path.getsize(args.output)
    print(f"wrote {args.output} ({dt:.1f}s elapsed, {sz} bytes)")


if __name__ == "__main__":
    main()
