# Agent-Aware Terminal Resume on Session Restore

> **Status**: Design Document (proposal)
> **Date**: June 2026
> **Driving feature**: Restore terminals to a *useful* state on session
> restore — for known coding agents, re-launch the agent's own session
> rather than dropping the user into a dead/bare shell.
> **Prior art**: `herdr` (`herdr.dev`), `cmux` (`cmux.com`) both ship
> "resume the agent with its native resume command after a restart."
> **Composes with**: [Session Persistence](../features/session-persistence.md),
> [Orchestrator & Sessions](./orchestrator-sessions-design.md),
> [Authority](./AUTHORITY_DESIGN.md).

## Motivation

### What restore does today

Fresh persists each terminal in the workspace file as a
`SerializedTerminalWorkspace` (`crates/fresh-editor/src/workspace.rs`):

```rust
pub struct SerializedTerminalWorkspace {
    pub terminal_index: usize,
    pub cwd: Option<PathBuf>,
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
    pub log_path: PathBuf,      // raw ANSI log
    pub backing_path: PathBuf,  // last rendered screen + scrollback
}
```

On restore (`Editor::restore_terminal_from_workspace`,
`crates/fresh-editor/src/app/workspace.rs:691`) Fresh:

1. Spawns a **brand-new bare shell** through the active authority's
   `TerminalWrapper` (local `$SHELL`, or `ssh …` / `docker exec …` for
   remote/container authorities). The wrapper runs *only* the shell — there
   is no notion of an "initial command."
2. Loads the **backing file** (the last rendered screen) as a read-only
   buffer so the pane *looks* like it did at save time.

The net effect: the pane shows yesterday's output, but the live process
underneath is a fresh, empty shell. **Whatever was running in that pane —
`claude`, `codex`, `aider`, a dev server, a REPL — is gone.** For an AI
coding agent this is the worst case: the user had a long, expensive
conversation with full context, and after a restart they're staring at a
frozen screenshot of it with a `$` prompt waiting underneath.

### The opportunity

Every major coding agent now persists its own conversation server-side (or
on local disk) and exposes a **native resume command**:

| Agent | Resume by id | Resume latest in cwd | Notes |
|---|---|---|---|
| Agent | herdr's resume argv (verified, v0.6.7) | Session ref |
|---|---|---|
| Claude Code | `claude --resume <id>` | id |
| Codex (OpenAI) | `codex resume <id>` | id (subcommand, not a flag) |
| GitHub Copilot CLI | `copilot --resume=<id>` | id (equals form) |
| OpenCode | `opencode --session <id>` | id |
| pi | `pi --session <path>` | **session-file path** (falls back to id) |
| hermes | `hermes --resume <id>` | id |
| aider | — | no session id; aider auto-restores `.aider.chat.history.md` in-repo |

> These six argv forms are taken directly from herdr's `agent_resume::plan`
> (`src/agent_resume.rs`), so they are verified against a shipping tool, not
> guessed. The design still treats them as **data in a registry**, not
> hard-coded Rust, so a CLI flag rename is a config edit, not a release.
> Note pi is the odd one out: its session ref is an absolute path to a
> session `.jsonl`, not an opaque id.

So the feature is: **on restore, if a pane was running a known agent,
re-run that agent's resume command instead of a bare shell.** This is
exactly herdr's claim — "capture the session id through the integration
hook and, after restart, run the agent's native resume command in its
pane." It is, as they say, almost embarrassingly simple — *once you have
the session id and a place to run the command.* The design work is in
those two pieces and in the safety/fallback behaviour around them.

## Verified: how herdr actually does it (v0.6.7, read from source)

I cloned `ogulcancelik/herdr` and read the implementation rather than the
marketing. The mechanism, end to end:

**Capture (while the agent runs).**

1. herdr is a long-lived server that owns every pane's PTY. When it spawns a
   pane it injects three env vars (`src/pane.rs`): `HERDR_ENV=1`,
   `HERDR_SOCKET_PATH=<unix socket>`, `HERDR_PANE_ID=<pane id>`.
2. `herdr integration install claude` (and codex/opencode/copilot/pi/hermes)
   writes a small **hook script** into the agent's own config and registers
   it. For Claude Code it drops `~/.claude/hooks/herdr-agent-state.sh` and
   adds a single `SessionStart` hook (matcher `*`, the `session` action) to
   `~/.claude/settings.json`. The installer is idempotent and removes its own
   legacy hook entries first; the file is stamped `HERDR_INTEGRATION_VERSION`
   and marked "managed by herdr."
3. When the agent fires the hook, the script (bailing unless `HERDR_ENV=1`
   and the pane/socket vars are present) reads the agent's `session_id` from
   the hook's JSON payload, opens `HERDR_SOCKET_PATH`, and sends **one
   newline-delimited JSON line**:

   ```json
   {"id":"…","method":"pane.report_agent_session",
    "params":{"pane_id":"…","source":"herdr:claude","agent":"claude",
              "seq":<ns timestamp>,"agent_session_id":"<id>"}}
   ```

   Claude/Codex hooks are POSIX-sh wrappers around an inline `python3`
   one-liner; OpenCode is a JS plugin reporting on `session.created/updated/
   status`. So id-capture only needs `SessionStart`-type events; the
   *working/blocked/idle* state shown in the UI comes from separate **screen
   detection**, not from hooks (they deliberately removed the per-state
   command hooks).

4. The server validates and stores it (`handle_pane_report_agent_session` →
   `session_ref_from_report`). Crucially, only a **trusted `herdr:<agent>`
   source** is accepted for native resume; a `custom:` source is rejected.
   The id is length- and control-char-validated; pi additionally accepts an
   absolute *path* ref.

**Replay (on restore).**

5. The captured `(source, agent, session_ref)` is persisted in the session
   snapshot. On restore, `restore_plan_for_snapshot` — gated by
   `session.resume_agents_on_restore`, which **defaults to `true`** — calls
   `agent_resume::plan(...)` to reconstruct the argv from the trusted source
   (the table above). The id is **never** stored or replayed as a shell
   string; it is reconstructed into an `argv` from the hard-coded template,
   so a malicious id can at worst make resume fail, not inject a command.
6. Plans are **de-duplicated by session** across panes (`resumed_sessions`
   set keyed by `dedupe_key = source\0agent\0kind\0value`). If two panes
   reference the same agent session, only the first resumes — you can't run
   `claude --resume <same-id>` twice at once. The reservation is rolled back
   if the spawn fails.
7. **Launch is deferred until the host terminal theme is detected**
   (`src/app/agent_resume.rs`): a pending plan waits up to
   `PENDING_AGENT_RESUME_THEME_WAIT` so the resumed TUI inherits the right
   colors, then launches anyway on timeout. Hidden / inactive-tab /
   zoomed-out / background panes are resumed using the *current* terminal
   area without waiting for focus.
8. The replay itself is **not** an `exec` wrapper. herdr spawns an ordinary
   shell `TerminalRuntime` for the pane, shell-quotes the argv into a command
   string, and **types it into the shell followed by `\r`**
   (`try_send_bytes`). It then sets `respawn_shell_on_exit = false` so the
   pane doesn't bounce back to a shell loop when the agent quits.
9. When a native resume runs, the **old scrollback snapshot is not
   replayed** (`initial_history_ansi = None`); the resumed agent repaints the
   pane. Only *non-resumed* panes get their saved ANSI snapshot painted back
   as static history (herdr's equivalent of Fresh's backing-file restore).

In short, herdr's "embarrassingly simple" claim is real, but it rests on
four non-obvious pieces of engineering: a per-pane env token, a trusted-
source registry that turns ids into argv (never shell text), session-level
dedupe, and theme-gated deferred launch.

## Problem decomposition

The feature splits cleanly into two independent axes. Most of the design
space (and most of the tradeoffs) lives in axis A.

- **Axis A — Capture.** While the agent runs, learn (a) *which agent* is in
  this pane and (b) *which native session id* it is using, and persist that
  alongside the existing terminal record.
- **Axis B — Replay.** On restore, run the agent's resume command in the
  pane instead of (or layered over) the bare shell, on the correct
  authority, with graceful fallback.

They are orthogonal: any capture strategy feeds any replay strategy through
a single persisted field (`agent_resume` below).

## Proposed persisted shape

Add one optional field to `SerializedTerminalWorkspace`. Everything else in
the design serializes into it.

```rust
/// What was running in this terminal and how to bring it back.
/// `None` => plain shell, restore exactly as today.
#[serde(default, skip_serializing_if = "Option::is_none")]
pub agent_resume: Option<AgentResume>,

pub struct AgentResume {
    pub source: String,                // trusted key, e.g. "fresh:claude"
    pub agent: String,                 // registry key, e.g. "claude"
    pub session_ref: SessionRef,       // { kind: Id | Path, value: String }
    pub captured_via: CaptureSource,   // Hook | Store | Heuristic | Orchestrator
    pub captured_at: u64,              // unix secs; for staleness checks
}
```

This mirrors herdr's `PersistedAgentSession` (`source` + `agent` +
`session_ref{kind,value}`). The `kind: Id | Path` split matters: most agents
key on an opaque id, but **pi keys on an absolute session-file path**, so a
single `session_id: String` would have been wrong. `source` is the trust
anchor — only a built-in `fresh:<agent>` source is eligible for native
resume (a user/`custom:` source is captured but never auto-run), exactly as
herdr gates on `is_official_agent_source`.

`None` preserves today's behaviour bit-for-bit, so the change is additive
and old workspace files keep loading.

## Axis A — capturing agent + session id

### A1. Integration hooks (herdr's approach) — *recommended primary*

When Fresh spawns a terminal it already injects environment
(`crates/fresh-editor/src/services/terminal/manager.rs`: `TERM`,
`FRESH_SESSION`). Add a stable **per-terminal token**:

```
FRESH_TERMINAL_ID=<editor-session>/<terminal-index>
```

Then lean on each agent's own hook system to report its session id back,
keyed by that token. For Claude Code specifically this is a `SessionStart`
hook (and a `SessionEnd` hook to clear): the hook receives JSON on stdin
containing `session_id`, `cwd`, `transcript_path`. The hook body is a
one-liner that hands the pair back to Fresh over the **local control
socket** Fresh already runs (the same channel `FRESH_SESSION` advertises):

```bash
# installed by `fresh integration install claude`
fresh --cmd terminal report-agent \
    --terminal "$FRESH_TERMINAL_ID" \
    --agent claude \
    --session-id "$(jq -r .session_id)"
```

Fresh records the pair in memory against the live terminal and folds it
into the next workspace save. On restore it has an authoritative,
pane-accurate session id.

- **Pros.** Authoritative — it is the *exact* id the agent uses, for the
  *exact* pane (no "latest in cwd" guessing). Works even when the user
  launched the agent by hand inside a Fresh terminal, not via Orchestrator.
  Mirrors the approach two shipping tools already validate.
- **Cons.** Requires an `integration install` step per agent and a
  reporting verb on the control socket. Only covers agents that expose a
  hook/telemetry surface. Hook injection must be opt-in and idempotent
  (don't clobber a user's existing hooks; append/namespace them).
- **Effort.** Medium: a `report-agent` control verb, an env var, and a
  small per-agent installer that edits the agent's settings file.

### A2. Orchestrator-known command — *cheap win, partial coverage*

The Orchestrator already launches agents with a known command (the `AGENT`
column: `aider`, `claude -p`, `opencode`, …) and stores per-session plugin
state (`session_plugin_state` in `workspace.rs`). For Orchestrator-spawned
panes, Fresh *already knows the agent* without any hook — it can persist
`agent_resume { agent, session_id: None, captured_via: Orchestrator }` and
resume with the **`--continue`-style** command (latest session in cwd).

- **Pros.** Zero integration, works the day it ships, covers the headline
  Orchestrator use case (parallel agents in worktrees — one agent per cwd,
  so "latest in cwd" is unambiguous).
- **Cons.** `--continue` resumes "most recent session for this directory,"
  which is wrong if the user ran several sessions in one cwd. No id-level
  precision. Doesn't help hand-launched agents in plain terminals.
- **Effort.** Low. Recommended to ship *first*, behind the registry, as the
  zero-id fallback path that A1 later upgrades.

### A3. Read the agent's own session store — *no injection, brittle*

Each agent keeps its history on disk (e.g. Claude Code under
`~/.claude/projects/<cwd-hash>/*.jsonl`). On restore Fresh could look up the
newest session id for the pane's cwd directly.

- **Pros.** No hook install; recovers id-level resume even for hand-launched
  agents.
- **Cons.** Hard-codes per-agent storage layout and hashing scheme — a
  private contract that breaks without warning. Still only "latest in cwd,"
  not pane-accurate. Privacy-adjacent (reading another tool's data dir).
- **Verdict.** A fallback enrichment for A2, never the primary. Keep it out
  of MVP.

### A4. Process/output sniffing — *detection only*

Identify *that* a pane runs an agent by inspecting the child process argv
(`claude`, `codex`, …) or, like herdr's "screen detection," the terminal
output. This yields the **agent name** but not the **session id**; pair with
A2/A3 to get an id.

- **Pros.** Detects hand-launched agents with no integration, so Fresh at
  least knows to *offer* resume.
- **Cons.** No id by itself. argv sniffing is cheap and reliable; output
  scraping is fragile and locale-dependent. Composes, doesn't stand alone.

### Recommended capture stack

Layer them, best-available wins:

```
A1 hook id (authoritative)
  └─ else A2 orchestrator agent + --continue
       └─ else A4 argv-detected agent + A3 store id (best-effort)
            └─ else: plain shell (today's behaviour)
```

## Axis B — replaying the resume command

### B1. Type-and-send into a freshly spawned shell — *what herdr ships*

Spawn the pane's normal shell `TerminalRuntime` exactly as today, then
shell-quote the resume argv into a command string and write it to the PTY
followed by `\r` (herdr: `shell_command_from_argv` + `try_send_bytes`). Set
"don't respawn a shell when this process exits" so the pane doesn't bounce
back to a prompt loop when the agent quits.

- **Pros.** *Verified to work in a shipping tool.* No change to the spawn
  path or to `TerminalWrapper` — the agent is just the first command typed
  into the restored shell, so it inherits the user's full login env and rc.
  The agent runs as a child of that shell, which is what users expect (quit
  the agent → back at a shell). Composes with Fresh's authority wrappers for
  free, because the shell that receives the keystrokes is *already* the
  remote/container shell (`ssh …` / `docker exec …`).
- **Cons.** The command is briefly visible and, in principle, editable before
  it executes; it can race shell rc init. herdr mitigates the race by
  deferring launch until the terminal area + host theme are known (it spawns
  the shell and sends immediately, accepting that interactive shells are
  ready by the time bytes arrive). The shell-quoting must be correct
  (herdr's `shell_quote` allowlists safe bytes, single-quote-wraps the rest)
  — this is the load-bearing safety boundary, since the id originates
  off-process.
- **Effort.** Low. Add `pending_agent_resume` to the terminal state, send the
  quoted argv after `spawn`, and skip the backing-file load for resumed
  panes.

### B2. Resume command as the terminal's initial command (`exec`) — *cleaner, more plumbing*

`TerminalWrapper` today runs shell-only. Extend the spawn path with an
optional initial command so the pane comes up *as* the agent, `exec`-ed past
an idle shell:

```rust
TerminalWrapper { command, args, manages_cwd, /* new */ initial_command }
// → sh -lc 'exec claude --resume <id>'
```

- **Pros.** The agent is the pane's foreground process from the first frame
  (no typed command flashes on screen, no rc race). Argv stays argv until the
  remote `sh -lc`, so the shell-quoting surface is smaller.
- **Cons.** Threads `initial_command` through `TerminalWrapper`,
  `manager.spawn`, and every wrapper constructor (host/ssh/kube). `exec`
  means quitting the agent closes the pane rather than dropping to a shell —
  arguably worse UX than B1. **herdr considered this shape and did not take
  it**, which is a meaningful data point: the simpler type-and-send was good
  enough and avoided the plumbing.

**Recommendation:** ship **B1** to match the proven herdr behaviour and the
small diff; keep B2 noted as the option if a flashing command line or the
"agent is child of a shell" model ever becomes a problem.

### Display reconciliation

herdr's choice: when a pane is being natively resumed, it **does not** repaint
the saved scrollback (`initial_history_ansi = None`) — the resumed agent
repaints the pane itself (agents use the alternate screen). Only *non-resumed*
panes get their saved ANSI snapshot painted back as static history (herdr's
analogue of Fresh's backing-file restore at `app/workspace.rs`).

For Fresh this maps to: in `restore_terminal_from_workspace`, **skip
`load_terminal_backing_file_as_buffer` when an agent resume is planned**, and
let the live agent's redraw fill the pane. Two options if a blank pane during
agent boot feels bad:

- **(a) herdr parity** — blank until the agent paints (simplest; agents boot
  fast).
- **(b) screenshot-then-live** — load the backing file first as the "boot
  screenshot," then let the agent's alt-screen redraw replace it. One extra
  step; nicer for slow-starting agents. Defer unless users complain.

## The agent registry

The crux that keeps per-agent knowledge out of Rust and lets users add
agents. A config table (and a built-in default set), shape sketch:

```toml
[[terminal.agents]]
name        = "claude"
# Axis A4: recognise the agent from the child process.
match       = { program = "claude" }
# Axis A1: how its id is reported (the installer wires the hook).
capture     = "hook"            # hook | store | continue | none
# Axis A3 (optional): where to find the latest id for a cwd.
store_glob  = "~/.claude/projects/{cwd_hash}/*.jsonl"
# Axis B: resume templates.
resume      = "claude --resume {session_id}"
resume_cwd  = "claude --continue"   # used when session_id is absent
```

Built-in defaults ship for claude/codex/opencode/copilot/gemini/aider;
`config-schema.json` gains the table so the settings UI and validation pick
it up. Unknown agents → plain shell.

## Configuration surface

Mirror herdr's single switch, plus granularity:

```toml
[session]
# Master switch. herdr ships this as `true` by default (v0.6.7) after
# dogfooding. Recommend: default false while experimental in Fresh, flip to
# true once dogfooded — the exact path herdr took.
resume_agents_on_restore = false

# How to treat a resume that would re-enter a live agent conversation:
#   auto    – run it silently (herdr's behaviour: it just types & runs)
#   confirm – show the command, run on keypress
#   never   – capture ids but never auto-run (manual `r` in the pane)
# Note: herdr has no per-resume confirm; with the switch on it auto-runs.
# `confirm` is a Fresh-specific safety affordance, recommended as the
# default until the feature graduates from experimental.
resume_mode = "confirm"
```

- CLI parity with existing flags: `fresh --restore` honours the config;
  add `--resume-agents` / `--no-resume-agents` one-shot overrides next to
  the existing `--restore` / `--no-restore`.
- Per-agent opt-out via the registry (`capture = "none"`).

## Failure handling & fallback

Resume *will* sometimes fail (expired session, agent upgraded, network gone
for a remote authority, id rotated out of the store). The contract:

1. Attempt the resume command (B1).
2. If the process exits non-zero quickly (e.g. < 2 s) **or** the agent
   prints a recognisable "no such session" error, fall back to the **bare
   shell + backing file** path that exists today — never leave the user with
   a dead pane.
3. Surface a one-line status: *"Couldn't resume claude session; started a
   fresh shell."*

Because `agent_resume` is optional and the fallback is the current code
path, the worst case is exactly today's behaviour.

### Two behaviours Fresh must replicate (learned from herdr)

These are not optional polish — herdr needs both for correctness:

1. **Session-level dedupe.** Resume each captured session **at most once per
   restore pass**, keyed by `(source, agent, kind, value)`. Two panes that
   reference the same agent session must not both run `claude --resume
   <same-id>` — only the first wins; the rest fall back to a plain shell.
   Reserve the key before spawning and roll it back if spawn fails.
2. **Deferred launch until the render context is ready.** herdr holds the
   resume until the host terminal theme is detected (bounded by a timeout),
   so the agent's TUI inherits the right colors, and launches background /
   inactive-tab / zoomed-out panes using the *current* terminal area rather
   than waiting for focus. Fresh's equivalent: don't fire the resume until
   the window/pane has a non-zero render rect (and, if Fresh forwards a host
   palette, until that's known), with the same timeout escape hatch.

## Safety, trust, and provenance

Auto-running a captured command is a real side effect — it can spend API
tokens, hit rate limits, or (for a mis-tagged pane) run something
unexpected. Guardrails:

- **Opt-in master switch**, default off while experimental.
- **`confirm` as the default mode** — show the exact command before running.
- **Resume only from the registry, ids are data not shell text.** Fresh never
  persists or replays a free-form captured command line; it stores
  `(source, agent, session_ref)` and reconstructs the `argv` from the trusted
  template (herdr's `plan()` + `is_official_agent_source`). A compromised
  hook can supply a bogus *id* (annoying — resume fails, fallback fires) but
  not an arbitrary *command*. If replaying via type-and-send (B1), the
  argv→shell-string quoting (herdr's `shell_quote`) is the one remaining
  injection boundary and must be watertight; B2 (`exec` of an argv) avoids it
  entirely. herdr has an explicit test for this (`ids_are_data_not_shell_text`).
- **Authority scoping.** A resume is only attempted under the same authority
  kind that captured it; an id captured inside a devcontainer is never run
  against the local host.
- **Staleness.** `captured_at` lets the registry expire ids (e.g. don't
  auto-resume a week-old session without confirm).

## Interaction with existing persistence

- **Detach/reattach (server still alive):** unaffected — the agent process
  is still running; this design only touches the **cold restart** path where
  the process is gone.
- **Hot-exit / `restore_previous_session`:** `agent_resume` rides inside the
  per-dir workspace file, so it follows the same `restore_previous_session`
  / `--restore` / `--no-restore` gating already documented.
- **Orchestrator restore:** `orchestrator_persistence.rs` iterates windows;
  each window's terminals already serialize through `SerializedTerminalWorkspace`,
  so Orchestrator sessions inherit agent resume with no extra plumbing
  beyond A2 populating the field.

## Phasing

- **Phase 0 — plumbing (no behaviour change).** Add `agent_resume`
  (`source` + `agent` + `session_ref`) to `SerializedTerminalWorkspace`, the
  registry config + built-in defaults (the six verified argv templates), and
  a `pending_agent_resume` field on terminal state. Everything still resolves
  to "plain shell."
- **Phase 1 — replay via type-and-send (B1).** On restore, when a plan
  exists and `resume_agents_on_restore` is on: spawn the shell as today, skip
  the backing-file load, shell-quote the argv and send it + `\r`, set
  no-respawn-on-exit. Include **session dedupe** and **deferred launch** from
  day one (they're correctness, not polish). Source the plan from A2
  (Orchestrator-known agent) first — smallest end-to-end slice for the
  parallel-worktree use case.
- **Phase 2 — hook integrations (A1).** `fresh integration install claude`
  writes a `SessionStart` hook into `~/.claude/settings.json` that reports
  `(pane, agent, session_id)` to Fresh's control socket via a new
  `terminal report-agent` verb; inject `FRESH_TERMINAL_ID` on spawn. This
  gives id-accurate resume for hand-launched agents. Add
  codex/opencode/copilot/pi/hermes by porting herdr's hook assets.
- **Phase 3 — enrichment & polish.** Store-scraping fallback (A3), argv
  detection prompts (A4), staleness expiry, optional `confirm` UX, and flip
  the default to `true` once dogfooded — as herdr did.

## Open questions

1. **Hook install UX.** Auto-offer on first detected agent run, or strictly
   manual `fresh integration install`? (herdr makes it explicit; that's the
   safer default.)
2. **Multiple agents per pane over a session's life** (user quit claude, ran
   codex in the same pane). Capture should track the *last* agent at save
   time — `SessionEnd` hooks clearing the record handle this for A1; A4 reads
   the current child at save.
3. **Non-agent "smart restore" candidates.** The same `initial_command`
   machinery could re-launch dev servers / REPLs from a registry. Out of
   scope here, but the design deliberately generalises (the registry isn't
   agent-specific by construction).
4. **Confirm UX placement.** Inline in the pane (type-to-run) vs. a small
   prompt overlay. Inline is closer to "the command is right there in your
   terminal," which users already understand.

## Why this shape

- **Additive and reversible.** One optional field; absence == today. Old
  workspaces load; the fallback path *is* the current code.
- **Knowledge as data.** Per-agent specifics live in a registry, not Rust,
  so a CLI flag rename is a config edit. All six resume argv templates are
  verified against herdr v0.6.7 source rather than guessed.
- **Composition over special-casing.** Replay rides the existing shell spawn
  path (B1, type-and-send), so SSH/K8s/devcontainer resume falls out for free
  — the shell receiving the keystrokes is already the remote/container shell.
- **Verified against a shipping implementation.** Every load-bearing
  decision here was checked against herdr's actual code: the trusted-source
  registry (`agent_resume::plan`), type-and-send replay (`app/agent_resume`),
  session dedupe and theme-gated deferred launch (`persist/restore`), and
  hook-over-unix-socket capture (`integration/assets/*`). This adapts that
  proven shape to Fresh's authority + workspace model rather than inventing a
  new one.
