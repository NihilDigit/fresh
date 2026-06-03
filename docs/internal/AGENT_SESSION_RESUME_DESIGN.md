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
| Claude Code | `claude --resume <id>` (`-r`) | `claude --continue` (`-c`) | id also accepts a session *name*; bare `--resume` opens an interactive picker. **Verified.** |
| Codex (OpenAI) | `codex resume <id>` | `codex resume --last` | subcommand, not a flag |
| OpenCode | `opencode --session <id>` | `opencode --continue` | verify exact flag against installed version |
| GitHub Copilot CLI | `copilot --resume <id>` | `copilot --continue` | verify against installed version |
| Gemini CLI | `gemini --resume <id>` | — | verify |
| pi / hermes | (per integration) | — | verify |
| aider | — | (auto-restores `.aider.chat.history.md` in repo) | no session id; cwd is enough |

> The exact flag spelling drifts between CLI versions. The design below
> treats these invocations as **data in a registry**, not hard-coded Rust,
> precisely so a flag rename is a config edit, not a release. Only Claude
> Code's syntax is asserted as verified here; the rest must be confirmed
> against the installed CLI before shipping a built-in default.

So the feature is: **on restore, if a pane was running a known agent,
re-run that agent's resume command instead of a bare shell.** This is
exactly herdr's claim — "capture the session id through the integration
hook and, after restart, run the agent's native resume command in its
pane." It is, as they say, almost embarrassingly simple — *once you have
the session id and a place to run the command.* The design work is in
those two pieces and in the safety/fallback behaviour around them.

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
    pub agent: String,                 // registry key, e.g. "claude"
    pub session_id: Option<String>,    // native id, when captured
    pub captured_via: CaptureSource,   // Hook | Store | Heuristic | Orchestrator
    pub captured_at: u64,              // unix secs; for staleness checks
}
```

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

### B1. Resume command as the terminal's initial command — *recommended*

`TerminalWrapper` today runs shell-only. Extend the spawn path with an
optional initial command so the pane comes up *as* the agent:

```rust
TerminalWrapper { command, args, manages_cwd, /* new */ initial_command }
```

Realised as `exec`-into-agent so there's no idle shell underneath:

```
sh -lc 'exec claude --resume <id>'
```

This **composes with authorities for free** — the existing wrapper already
prefixes `ssh host …` / `docker exec -w … container …`, so the agent
resumes *on the host where it lived*, which is exactly right for
SSH/K8s/devcontainer sessions. The id and template come from the registry +
the persisted `agent_resume`.

- **Pros.** Clean: the agent is the pane's foreground process (matches
  "run the agent's native resume command in its pane"). Authority
  composition is automatic. No PTY-injection races.
- **Cons.** Needs threading `initial_command` through `TerminalWrapper`,
  `manager.spawn`, and every wrapper constructor (host/ssh/kube). The
  agent's full-screen (alt-screen) redraw replaces the restored backing-file
  view — see "Display reconciliation."

### B2. Type-and-send into a bare shell — *simplest, racy*

Spawn the normal shell, then write `claude --resume <id>\n` into the PTY.

- **Pros.** Tiny change; no wrapper plumbing.
- **Cons.** Races with shell rc init (command can land mid-prompt); the
  text is visible and user-editable before it runs; doesn't compose as
  cleanly with non-interactive wrappers. Fine as a stopgap, not the
  destination.

### B3. `exec` replacement at wrapper level — *B1 taken to its conclusion*

B1 already uses `exec`; the only variant is whether a shell wraps it at all.
For local panes `sh -lc 'exec …'` gives the agent the user's rc/env; for
authority wrappers the remote `sh -lc` does the same remotely. Keep the
shell wrapper (for env) but `exec` past it. This is the recommended form of
B1, called out separately only because "no shell at all" is tempting and
wrong (you lose login env).

### Display reconciliation

Today restore loads the backing file as the pane's content. With live
resume:

- Keep loading the backing file first (instant, looks identical to save
  time) — this is the "screenshot" while the agent boots.
- When the resumed agent starts painting (agents use the alternate screen),
  its redraw replaces the view. No special handling needed for full-screen
  TUIs.
- For agents/programs that *don't* take the alt screen, the resume output
  appends below the restored scrollback, which reads naturally as "picking
  up where we left off."

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
# Master switch. Experimental → default false initially, flip to true
# once dogfooded (herdr did exactly this).
resume_agents_on_restore = false

# How to treat a resume that would re-enter a live agent conversation:
#   auto    – run it silently
#   confirm – show the command, run on keypress (safest default)
#   never   – capture ids but never auto-run (manual `r` in the pane)
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

## Safety, trust, and provenance

Auto-running a captured command is a real side effect — it can spend API
tokens, hit rate limits, or (for a mis-tagged pane) run something
unexpected. Guardrails:

- **Opt-in master switch**, default off while experimental.
- **`confirm` as the default mode** — show the exact command before running.
- **Resume only from the registry.** Fresh never persists or replays a
  free-form captured command line; it stores `(agent, session_id)` and
  reconstructs the invocation from the trusted template. A compromised
  hook can supply a bogus *id* (annoying — resume fails, fallback fires) but
  not an arbitrary *command*.
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

- **Phase 0 — plumbing (no behaviour change).** Add `agent_resume` to the
  schema, the registry config + built-in defaults, and `initial_command` on
  `TerminalWrapper`/`spawn`. Everything still resolves to "plain shell."
- **Phase 1 — Orchestrator `--continue` (A2 + B1).** Smallest end-to-end
  slice that delivers the headline win for parallel-agent worktrees. Behind
  `resume_agents_on_restore`, `resume_mode = "confirm"`.
- **Phase 2 — hook integrations (A1).** `fresh integration install claude`
  + the `terminal report-agent` control verb → id-accurate resume for
  hand-launched agents. Add codex/opencode/copilot as their hook surfaces
  are confirmed.
- **Phase 3 — enrichment & polish.** Store-scraping fallback (A3), argv
  detection prompts (A4), staleness expiry, flip the default on once
  dogfooded.

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
  so a CLI flag rename is a config edit. Only Claude Code's syntax is
  asserted verified; the rest are data to confirm.
- **Composition over special-casing.** Replay rides the existing authority
  `TerminalWrapper`, so SSH/K8s/devcontainer resume falls out for free.
- **Two shipping precedents.** herdr and cmux both validate "capture id via
  integration hook, run native resume command in the pane." This adapts that
  proven shape to Fresh's authority + workspace model rather than inventing a
  new one.
