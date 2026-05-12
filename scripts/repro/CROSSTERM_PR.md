# Upstream crossterm PR: drain tty fd to `EAGAIN` before returning from `try_read`

## Status

Patch lives at `scripts/repro/crossterm-drain-fix.patch`. A vendored copy
of the patched tree sits at `vendor/crossterm/` and is wired into the
workspace via `[patch.crates-io]` in the top-level `Cargo.toml`.
Once the PR lands upstream and crates.io has the release, delete the
vendor directory and the `[patch.crates-io]` block.

## Title

```
fix(event): drain tty fd to EAGAIN before returning from try_read
```

## Body

> mio registers the tty fd with `Interest::READABLE`, which on Linux maps
> to `EPOLLIN | EPOLLET` — edge-triggered. The previous read loop returned
> as soon as the parser produced its first event, leaving any remaining
> bytes in the kernel buffer. Because the edge has already been consumed,
> `epoll_wait` will not re-fire for those leftover bytes until *new* data
> arrives.
>
> In practice this surfaces as the well-known "a large paste blocks
> partway through and only resumes when I press a key" symptom — for
> example, pasting a few KB of text from tmux's `paste-buffer` (which
> does not use bracketed paste, so the bytes arrive as a flood of raw
> keystrokes). The first ~1 KiB is parsed, the parser returns the first
> event, the loop exits with bytes still in the kernel pty, and the next
> `epoll_wait` never fires. A real keypress later creates a new edge,
> the leftover bytes finally drain, and the paste appears to "continue".
>
> ## Fix
>
> Keep reading until the fd returns `WouldBlock` or a short read (which
> also indicates no more bytes are immediately available on a tty). All
> resulting events are queued in the parser. Only then do we return the
> first event to the caller; subsequent `try_read` calls drain the queue
> without any extra syscalls.
>
> No behavior change when each `try_read` call corresponds to a single
> small input (the typical interactive case): the read loop sees a
> short read and exits immediately, identical to the old fast path.
>
> ## Reproduction
>
> Inside a tmux session, run any TUI that uses crossterm in raw mode,
> then from another tmux pane:
>
> ```sh
> tmux load-buffer some-long-file.md
> tmux paste-buffer -t <target-pane>
> ```
>
> Without the fix, the paste stops partway through; pressing any key
> makes the rest appear. With the fix, the whole paste lands in one
> tick. Tested against tmux 3.4 on Linux 6.18.
>
> ## Notes
>
> * The `use-dev-tty` (`tty.rs`) path uses level-triggered `poll()`,
>   so it does not exhibit the stall — its early-return is benign
>   because subsequent polls re-report the fd as readable. The
>   equivalent change there would still be a small performance win
>   (fewer poll syscalls for large bursts) but is not required for
>   correctness; happy to fold it into this PR if reviewers prefer.
> * No new dependencies, no API changes, no test renames.

## How to submit

1. Push `vendor/crossterm/` content to a fork on GitHub
   (`gh repo fork crossterm-rs/crossterm --clone`).
2. `git am scripts/repro/crossterm-drain-fix.patch` on top of upstream
   `master` (or just commit the change directly — the patch is on top
   of upstream master as of 2026-05-12, commit `c02a080`).
3. Push the branch and open the PR against `crossterm-rs/crossterm`.
4. Once merged + released, switch `Cargo.toml`'s `[patch.crates-io]`
   block to a regular dependency bump and `rm -rf vendor/crossterm/`.
