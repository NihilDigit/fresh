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

## The landscape: five ways to restore a terminal

Surveying the field (multiplexers, terminal emulators, IDEs, process-
checkpoint tools, and the new wave of agent orchestrators), every approach
is one of five archetypes — or a layering of them. Each was checked against a
real implementation.

**1. Keep the process alive (daemon / server + reattach).** Never kill the
program; a resident background process owns the PTY and clients reattach.
- *Examples:* tmux/screen, `abduco`/`dtach`, iTerm2 Session Restoration
  (jobs run in long-lived servers), VS Code "process reconnect" (pty-host
  survives window reload), **claude-squad** (tmux-backed), **Superset**
  (its `pty-daemon` even hands off live PTY master fds to a successor daemon
  on upgrade — verified from source — so agents survive app *and* daemon
  restarts), and **Fresh's own detach/reattach** (background server).
- *Trade-off:* perfect fidelity (the live process, all in-memory state). But
  only survives *soft* restarts — app reload, disconnect, daemon upgrade —
  **not** host reboot, daemon crash, or OS kill. Costs a resident process +
  memory.

**2. Re-launch from saved metadata (replay the command).** Process dies; on
restore re-run a command.
- *2a — bare re-run:* tmux-resurrect "restore programs" (conservative
  whitelist + `~prog->prog *` rules), zellij resurrection (re-runs command
  panes behind a "Press ENTER to run" banner; `--force-run-commands` to
  skip).
- *2b — native agent resume:* **herdr**, **cmux**, **Conductor** (re-drives
  Claude Code via its own resume/chat-history), VS Code "revive process".
  **This is the archetype this doc proposes for Fresh.**
- *Trade-off:* survives full reboots, cheap, no resident process. But loses
  live in-memory state (only what the program persisted comes back) and needs
  the program to support resume for any fidelity. Re-running is a side effect,
  hence the confirm banners.

**3. Snapshot the rendered screen only (cosmetic).** Save/restore the painted
text; the process is gone.
- *Examples:* tmux-resurrect `capture-pane-contents`, zellij
  `pane_viewport_serialization`, iTerm2 OS window restoration (content-only
  → reverse-video "Session Restored" banner), VS Code scrollback restore, and
  **Fresh's current backing-file restore**.
- *Trade-off:* instant, looks identical, zero risk — but it's a screenshot,
  not interactive. Almost always layered under #1 or #2.

**4. Checkpoint/restore the process image (freeze/thaw).** CRIU, DMTCP.
- *Trade-off:* highest fidelity for *arbitrary* processes with no resume
  support needed. But heavy and fragile: needs intact parent-child trees and
  the *same PID*, special handling for PTYs/fds/IPC namespaces, usually root,
  and it cannot save live network sockets or GPU state — fatal for an agent
  mid-conversation with a remote API. Impractical here.

**5. App-level reconstruction from the agent's transcript.** Don't restore a
terminal at all; a GUI owns the conversation and re-drives the agent.
- *Examples:* **Conductor**, **Crystal**, **Vibe Kanban**, agent-sessions.
- *Trade-off:* richest UX (searchable history, kanban, checkpoints, diff
  review) but a bespoke per-agent integration, not generic terminal restore;
  the "terminal" is an implementation detail they often hide.

### Where Fresh sits, and why 2b is the gap to fill

Fresh already implements **#1** (detach/reattach to the background server)
and **#3** (backing-file screenshot on cold restore). The hole is the cold-
restart path: today it's #3 + a bare shell. The agent orchestrators converge
on two answers for that hole — **keep agents alive in a daemon (#1, Superset/
claude-squad)** or **resume them natively (#2b, herdr/cmux/Conductor)**:

| If Fresh wanted… | Pick | Cost | Notes |
|---|---|---|---|
| Survive app restart / detach with *zero* state loss | **#1** (already have it) | resident server | dies on reboot/crash |
| Survive *reboot/crash* for agent panes | **#2b** (this doc) | per-agent resume registry | only as good as the agent's own resume |
| Make a dead pane *look* restored instantly | **#3** (already have it) | screenshot | layer under #2b during agent boot |
| Restore *any* process losslessly | #4 | very high / fragile | rejected |
| Own the whole conversation UX | #5 | bespoke GUI per agent | that's the Orchestrator's job, not terminal restore |

The recommendation stands: **layer #2b on top of the existing #1 and #3.**
#1 handles soft restarts; #2b extends coverage to hard restarts for known
agents; #3 is the instant "boot screenshot" while the resumed agent repaints.
The four hard parts are the same ones herdr already solved (trusted-source
registry, replay, dedupe, deferred launch) — see the verified section above.

## Adopting 2b in Fresh — the concrete plan

> **Decision:** adopt archetype **2b** (native agent resume), layered on the
> existing #1 (detach server) and #3 (backing-file screenshot). Two hard
> requirements shape the implementation and push it *away* from a literal
> port of herdr:
>
> 1. It must work across **every authority** — local, SSH, K8s, devcontainer.
> 2. Per-agent logic (detection + resume command) must **not be hardcoded in
>    Rust core**; it must be configurable and plugin-extensible (TOML config
>    and an `init.ts` API), the way LSP servers and grammars already are.
>
> These two requirements are why the Fresh design differs from herdr in its
> two load-bearing mechanisms (capture transport and registry location),
> while keeping herdr's proven *shape* (capture id → reconstruct argv from a
> trusted template → type-and-send → dedupe → deferred launch).

### Why herdr's transport doesn't port directly

herdr's capture is a **unix-socket callback**: the agent hook connects to
`HERDR_SOCKET_PATH` and sends `pane.report_agent_session`. That works because
**every herdr pane is local**. Fresh panes are not: a terminal opened under
the SSH / K8s / devcontainer authority runs the agent *on a remote host*
(`manager.spawn` runs the authority's `TerminalWrapper` — `ssh …` /
`docker exec …` / `kubectl exec …`). On those hosts:

- Fresh's local control socket (`FRESH_SESSION`, `local_control.rs`) is **not
  reachable**, and
- env vars set on the local `CommandBuilder` (`manager.rs:332`) **don't cross
  the wrapper** to the remote process.

So a socket/env callback is a *local-only fast path*, not the primary
mechanism. The authority-robust mechanism has to ride a channel that already
spans the authority boundary — and there is exactly one: the **PTY data
stream** Fresh is already reading.

### Capture (authority-robust): in-band PTY marker — *primary*

The agent's hook emits a **sentinel escape sequence** to stdout carrying its
identity; Fresh scans it out of the PTY byte stream, captures it, and strips
it from the display. Concretely:

- Hook prints e.g. an OSC: `\033]5379;agent=claude;id=<session-id>\007`
  (5379 = "FRESH"; bikeshed later). For Claude Code this is a one-line
  `SessionStart` hook — the same install point herdr uses, different payload
  (print, not socket-connect).
- Fresh already funnels every terminal's output through
  `TerminalState::process_output(&mut self, data: &[u8])`
  (`services/terminal/term.rs:178`) before handing bytes to the
  `alacritty_terminal` VTE `Processor`. The marker is recognised there
  (a cheap byte-scan for the `\033]5379;` prefix), captured against **this
  terminal** (attribution is automatic — it's *this* PTY), and elided from
  the bytes fed to the emulator so it never renders.

Why this is the right primary:

- **Authority-agnostic by construction.** It is pure in-band PTY data, so it
  behaves identically whether the agent runs locally, over SSH, in a pod, or
  in a container. No socket reachability, no env propagation, no per-authority
  code.
- **No per-terminal token needed.** Because Fresh reads the marker off a
  specific terminal's stream, it knows the owning `TerminalId`/`WindowId`
  without the agent echoing back an id (herdr needs `HERDR_PANE_ID` precisely
  because the socket callback is out-of-band).
- **Harmless when unobserved.** An agent that prints the marker into a plain
  terminal (no Fresh) just emits an ignored OSC. So hooks can emit
  unconditionally; no env-gating required for remote.

Alternatives kept as secondary:

| Capture transport | Authorities | Notes |
|---|---|---|
| **In-band PTY marker** | **all** | primary; rides the data channel |
| Unix-socket / `fresh --cmd terminal report-agent` | local only | fast path; reuses `local_control.rs` + a new `ClientControl::ReportAgent` variant (insertion points already scoped: `main.rs` cmd parse, `protocol.rs` enum, `editor_server.rs` dispatch) |
| Marker file via `authority.filesystem` | all (needs FS) | hook writes id to a file; Fresh reads it through the same remote-FS abstraction used in `restore_terminal_from_workspace`. Slower, polled; fallback when a host strips OSC |
| Agent store scrape / argv detection | local mostly | last-resort enrichment (A3/A4 below) |

### Replay (authority-robust): type-and-send into the wrapper's own shell

Use **B1 (type-and-send)** exactly as herdr does, and it composes with
authorities *for free* — for a subtle but decisive reason: on restore Fresh
spawns the pane's shell through the **same authority `TerminalWrapper`**, so
the shell that receives the typed keystrokes **is already the remote/container
shell** (`ssh …`, `docker exec …`). Writing `claude --resume <id>\r` via
`TerminalHandle::write(&[u8])` (`manager.rs:76`) therefore runs the resume
command *on the host where the agent lived*, with no authority-specific logic.
The `exec`/`initial_command` variant (B2) would instead require threading the
command through every wrapper constructor — more work for worse UX. B1 wins
twice under the authority requirement.

### Registry-as-data: config + `init.ts` API + a bundled default plugin

No agent names, match rules, or resume flags live in Rust. The core holds a
**generic registry** populated from three layers (later overrides earlier):

1. **Built-in defaults** shipped as a **bundled TS plugin**
   (`crates/fresh-editor/plugins/agent-resume.ts`, like the existing
   `k8s-workspace.ts`) that registers claude/codex/opencode/copilot/pi/hermes
   /aider. This keeps even the defaults out of Rust and makes them
   user-overridable.
2. **User config** — TOML `[[terminal.agents]]` entries.
3. **Plugins** — `init.ts` calls, following the established
   `registerLspServer` / `registerGrammar` pattern
   (`quickjs_backend.rs:2930` / `:2860`), with the same collision handling.

The registry entry shape (config and API mirror each other):

```ts
fresh.terminal.registerAgent({
  name: "claude",
  // detection (for argv-sniff capture + UI labelling)
  match: { program: "claude" },
  // capture transport this agent uses
  capture: "marker",                  // "marker" | "socket" | "file" | "none"
  // resume invocation — argv ARRAY with a placeholder element, never a
  // concatenated string, so the captured id can't break out into shell words
  resume:     ["claude", "--resume", "{session_id}"],
  resumeCwd:  ["claude", "--continue"],   // when no id was captured
  sessionRefKind: "id",               // "id" | "path"  (pi uses "path")
});
```

The Orchestrator plugin registers nothing extra — it already knows the agent
it launched, so it can pre-seed `agent_resume` for its panes (covers the
parallel-worktree case with zero hooks).

### Persist the resolved template, not a registry pointer

To keep **restore independent of plugin load order** (plugins init at
startup; terminal restore is early), persist the *resolved* resume template
with the captured ref, not just `(agent)`:

```rust
// added to SerializedTerminalWorkspace (workspace.rs:409), Option => today's behaviour
pub agent_resume: Option<AgentResume>,

pub struct AgentResume {
    pub agent: String,               // for UI + re-resolve
    pub session_ref: SessionRef,     // { kind: Id|Path, value: String }
    pub resume_argv: Vec<String>,    // resolved template w/ "{session_id}" slot
    pub captured_via: String,        // "marker" | "socket" | "orchestrator" | …
    pub captured_at: u64,
}
```

On restore the core expands `resume_argv` with `session_ref.value` (pure
string substitution into the placeholder *element*, then shell-quote each
element for type-and-send) — **no registry lookup required**. If the agent is
still registered, optionally re-resolve to pick up a corrected template; else
use the stored one. The captured **id is always data**: it only ever lands in
the placeholder argv element and is shell-quoted, so a hostile id can make
resume fail but cannot inject a command — even though the template now comes
from config/plugins rather than hardcoded Rust.

### Code-level change map

| Concern | Where | Change |
|---|---|---|
| Registry store | new `services/terminal/agent_registry.rs` | generic `AgentDef` list; resolve `(argv, kind)` from a detected/known agent |
| Plugin API | `quickjs_backend.rs` (by `registerLspServer`) | `registerAgent(def)` + collision handling; expose on `fresh.terminal` |
| Config | `config.rs` `TerminalConfig` (:1861) + `config-schema.json` | `agents: Vec<AgentDef>` |
| Built-in defaults | `crates/fresh-editor/plugins/agent-resume.ts` (+ hook assets) | bundled plugin registering the 7 agents; ports herdr's hook scripts to emit the marker |
| Capture (marker) | `services/terminal/term.rs:178` `process_output` | scan/strip marker; surface `(TerminalId, agent, session_ref)` |
| Capture (socket, local) | `main.rs` cmd-parse + `protocol.rs` `ClientControl` + `editor_server.rs` dispatch | optional `terminal report-agent` fast path |
| Per-terminal capture store | `Window`/`Editor` | `HashMap<TerminalId, AgentResume>`; populated by capture, read at save |
| Persist | `workspace.rs:409` + capture block at `app/workspace.rs:2020` | add `agent_resume` field; write it in `capture_workspace` |
| Replay | `app/workspace.rs:691` `restore_terminal_from_workspace` | when `agent_resume` present + enabled: skip backing-file load, `TerminalHandle::write` the quoted argv + `\r`, mark no-respawn |
| Dedupe + deferred launch | restore path + a per-frame check (cf. herdr `app/agent_resume.rs`) | resume each `session_ref` once; defer until pane has a render rect |
| Config switch | `config.rs` `[session]` | `resume_agents_on_restore` (+ `resume_mode`) |

This satisfies both requirements: **authority-robustness** comes from the
in-band marker (capture) and type-and-send into the wrapper's own shell
(replay), neither of which has per-authority code; **no-hardcoding** comes
from the generic registry fed by config + `registerAgent` + a bundled
defaults plugin, with the resolved template persisted so the Rust core never
needs to know what "claude" is.

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

> **Superseded by the concrete plan above.** The axes below are the original
> analysis. Under Fresh's authority + no-hardcoding requirements, the chosen
> capture transport is the **in-band PTY marker** (not the unix-socket
> callback A1 describes), and the registry lives in config/plugins (not Rust).
> A1's *hook install point* still applies — the hook just **prints a marker**
> instead of connecting to a socket. Read this section for the trade-offs;
> read "Adopting 2b in Fresh" for what we build.

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

- **Phase 0 — generic plumbing (no behaviour change).** Add the generic
  agent **registry** (`agent_registry.rs`), the `registerAgent` plugin API +
  `[[terminal.agents]]` config, the `agent_resume` field on
  `SerializedTerminalWorkspace` (with the **resolved `resume_argv`**), and the
  per-terminal capture store. No agent names in Rust. Everything still
  resolves to "plain shell."
- **Phase 1 — Orchestrator pre-seed + replay (B1), authority-robust.** The
  Orchestrator knows the agent it launched, so it pre-seeds `agent_resume`
  (resume-by-`--continue`). On restore, when present and
  `resume_agents_on_restore` is on: skip the backing-file load, type the
  shell-quoted argv + `\r` via `TerminalHandle::write`, mark no-respawn.
  **Session dedupe** and **deferred launch** ship here (correctness, not
  polish). Works on every authority because the keystrokes go to the
  wrapper's own (possibly remote) shell. Ship the bundled `agent-resume.ts`
  defaults plugin.
- **Phase 2 — in-band marker capture (id-accurate, all authorities).** Scan
  the marker in `process_output`; port herdr's hook scripts to **print the
  marker** (Claude `SessionStart`, etc.) via `fresh integration install …`.
  This upgrades `--continue` to `--resume <id>` and covers hand-launched
  agents under SSH/K8s/devcontainer. Add the **local unix-socket fast path**
  (`terminal report-agent`) opportunistically.
- **Phase 3 — enrichment & polish.** Marker-file fallback via
  `authority.filesystem` for hosts that strip OSC; store-scrape / argv
  detection (A3/A4); staleness expiry; optional `confirm` UX; flip the
  default to `true` once dogfooded — as herdr did.

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
