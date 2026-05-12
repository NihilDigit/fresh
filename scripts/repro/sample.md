# Fresh Editor - Comprehensive Markdown Reference

This document is a long markdown sample used to reproduce the tmux paste
indent bug. It deliberately mixes paragraphs, headings, bullets, numbered
lists, code blocks, and quotes so we can see exactly how the indentation
gets corrupted when pasting via `tmux :paste`.

## Introduction

Fresh is a modern terminal text editor with zero configuration. It supports
mouse interaction, IDE-style keybindings, multi-cursor editing, and a
language server protocol integration. Pasting text into Fresh should be
safe even for very large clipboard payloads.

When the clipboard text is many lines long, the editor must:

1. Insert the text verbatim, preserving leading whitespace.
2. Not apply auto-indent on every embedded newline (otherwise each line
   ends up cumulatively shifted to the right).
3. Not stall halfway through the paste because of pty back-pressure.
4. Not require an extra keypress to "wake up" and finish applying the
   pending bytes.

## Why this matters

A user copying a stack trace, a markdown document, or a code snippet from
tmux scrollback expects the paste to look exactly like the source. Any
extra indentation is a correctness bug, and a stalled paste is a UX bug.

### Symptoms in the wild

- Each successive line is shifted further to the right than the previous
  one (a "staircase" effect).
- After ~one screenful of text the paste appears to hang.
- Pressing any key (Down, Right, Space, ...) causes the remaining text
  to suddenly appear.
- The resulting buffer needs to be discarded; users have to undo many
  times or close the file without saving.

## Reproduction recipe

1. Open a fresh tmux session.
2. Inside tmux, run `fresh` and create a new empty scratch buffer.
3. Load a long markdown text into tmux's paste buffer via
   `tmux load-buffer`.
4. Trigger the paste from tmux itself with `tmux paste-buffer` (or by
   pressing the tmux paste binding).
5. Observe: each line ends up indented further than the last, and the
   paste blocks until a key is pressed.

## Bullet list examples

- First top-level bullet with some descriptive text that wraps onto a
  second line so we can see how continuation lines are handled.
- Second top-level bullet.
  - Nested bullet A.
  - Nested bullet B with a longer trailing description that should not
    accidentally pick up additional indentation when pasted.
    - Deeply nested bullet.
    - Another deeply nested bullet.
- Third top-level bullet.

## Numbered list examples

1. Step one: prepare the environment.
2. Step two: gather data.
   1. Sub-step two-a.
   2. Sub-step two-b.
3. Step three: analyze the results.
4. Step four: report findings.

## Paragraph block

Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod
tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim
veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea
commodo consequat.

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum
dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non
proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

## Code-ish block (still plain markdown)

    fn main() {
        let greeting = "Hello, paste!";
        for line in greeting.lines() {
            println!("{}", line);
        }
    }

## Block quote

> "Pasting should be invisible plumbing: the bytes you copied are the
> bytes you get back. Anything else — auto-indent, smart formatting,
> bracket completion — belongs to typing, not to pasting."

## Mixed final section

Below is a mix of bullets, paragraphs, and a final numbered list to make
sure we exercise every path through the input parser one more time:

- Alpha.
- Beta with some additional text that continues for a while so that we
  cross the typical pty chunk boundary somewhere mid-line.
- Gamma.

Final paragraph: if you can read this paragraph at column 1 (no leading
whitespace) and the previous bullets are also at column 1, the bug is
fixed. If this paragraph starts halfway across the screen, the bug is
still present.

1. End of document.
2. Thanks for reading!
3. End.
