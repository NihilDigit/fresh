# Dev Container Integration Plan

## Context

Two devcontainer implementations currently exist in the tree:

- **Plugin** (`crates/fresh-editor/plugins/devcontainer.ts`, on master). Parses `devcontainer.json`, shows info panels (image / features / ports / env), runs lifecycle commands, offers `Rebuild` and `Open Terminal` commands. Assumes the container is reached explicitly via CLI commands; does not auto-attach. Detailed in `DEVCONTAINER_PLUGIN_DESIGN.md`.
- **Core changes from PR #1609** (`services/devcontainer/`, `Editor.container_id/user/workspace`, `main.rs` auto-connect, terminal routing, status bar). Runs `devcontainer up` on startup, stores the container identity on `Editor`, rewrites every terminal spawn to `docker exec -it -u <user> -w <workspace> <id> bash -l`, shows `[Container:<id>]` in the status bar.

They overlap (double detection, double rebuild paths, double activation UX) and conflict (plugin's `Open Terminal` would now double-wrap `docker exec ... devcontainer exec ...`; plugin's `Rebuild` desynchronizes the core's cached `container_id`).

This document supersedes the earlier "trim the plugin + add small accessors" sketch. The plan below is the target architecture; the PR as-merged is an acceptable interim state.

## Target architecture: Authority Provider

Core already exposes two traits used by SSH remote editing:

- `FileSystem` — abstracts file reads/writes.
- `ProcessSpawner` — abstracts process execution (used by plugins via `editor.spawnProcess`).

Today the concrete choice between local and SSH is made once in `initialize_app` and threaded through the `Editor`. We generalize this into a single **active authority** slot that any provider can own.

### Responsibilities

**Core owns:**

1. **The authority slot.** An `ActiveAuthority` on `Editor` carrying `{ filesystem, process_spawner, display_string, ready }`. Exactly one authority is active per workspace. `Local`, `Ssh`, and `Plugin(provider_id)` are the variants.
2. **Terminal routing.** `TerminalManager::spawn_terminal` asks the active authority for its spawn wrapper. Every spawn site — user `:term`, plugin-created terminals, future features — goes through this, no devcontainer-specific branches.
3. **Lifecycle ordering.** LSPs, file watchers, indexers, autosave, and other "auto-spawn on buffer open" services wait on `authority.ready` before starting. Prevents the "LSP flaps because container isn't up yet" failure mode the Gemini landscape report calls out.
4. **Status-bar slot.** The existing `connection_display_string` path renders whatever the authority provides. One formatter, one slot, SSH and containers both flow through it.
5. **Credential surface** (future, not for first cut). SSH agent / git keys exposed to authority providers so each one doesn't reinvent.

**Plugins own (as authority providers):**

1. **Detection.** "Is this workspace mine?" A provider plugin inspects the workspace and decides whether to claim the authority. Core asks registered providers in priority order at startup; first claimant wins. User can override via config.
2. **Handshake.** The provider brings the environment up (`devcontainer up`, `nix develop`, `toolbox enter`, etc.), then returns a `ProcessSpawner` + `FileSystem` + display string to core and signals `ready`.
3. **Specifics of the tool.** Spec parsing (`devcontainer.json`, `Vagrantfile`, `shell.nix`), orchestration commands, tool-specific UI (info panels, feature lists, rebuild dialogs), lifecycle-command runners.
4. **Rebuild / teardown.** Provider-defined. Provider is responsible for invalidating its own state and signalling a ready-cycle to core.

## What each concrete deliverable looks like

### Core extension points (new)

```rust
// crates/fresh-editor/src/services/authority/mod.rs
pub trait AuthorityProvider: Send + Sync {
    fn id(&self) -> &str;
    fn detect(&self, workspace: &Path) -> Option<DetectedAuthority>;
    fn connect(&self, detected: DetectedAuthority) -> BoxFuture<'static, Result<ActiveAuthority>>;
}

pub struct ActiveAuthority {
    pub id: String,                              // "local" | "ssh" | "devcontainer" | ...
    pub filesystem: Arc<dyn FileSystem>,
    pub process_spawner: Arc<dyn ProcessSpawner>,
    pub display_string: Option<String>,          // shown in status bar
    pub ready: tokio::sync::watch::Receiver<bool>,
}
```

Registered via an `AuthorityRegistry` that plugins populate through a new plugin op (`editor.registerAuthorityProvider(id, handlers)`). Core owns provider ordering and conflict resolution.

### Plugin-side API (new)

```ts
editor.registerAuthorityProvider("devcontainer", {
  detect: (workspacePath) => { ... return { configPath } | null },
  connect: async (detected) => {
    // run `devcontainer up`, parse output
    return {
      displayString: `Container:${shortId}`,
      processWrapper: (cmd, args, cwd) => ({
        cmd: "docker",
        args: ["exec", "-it", "-u", user, "-w", workspace, id, cmd, ...args],
      }),
      // filesystem: undefined -> local (bind-mount)
      ready: true,
    };
  },
});
```

### LSP / indexer gating

Every core subsystem that auto-spawns on workspace open takes a `ready` signal. For now: LSP manager, file watcher, indexer. In practice this is an `await ready.wait_for(|r| *r)` in the service's startup path.

## What the current PR becomes under this model

The PR's Rust code is 80% of the generic authority machinery already — it just hardcodes `devcontainer` as the only case:

- `services::devcontainer::{detect,cli}` → move to `plugins/devcontainer.ts` once the plugin runtime has the equivalent primitives (`editor.spawnProcess`, `editor.readFile`, JSONC parsing — all already present on master).
- `Editor.container_id/user/workspace` → collapse into `Editor.active_authority.display_string` and a provider-opaque process wrapper. No `container_*` fields at the `Editor` level.
- `TerminalManager` branch on `container_id` → replace with `authority.process_wrapper(cmd)`. Same effect, zero Docker awareness in core.
- `main.rs` `connect_devcontainer` → replace with generic `registry.connect_first_matching(workspace)` that invokes whichever provider claims the workspace.
- `DevcontainerConfig { auto_detect, cli_path }` → generic `authority.enabled_providers` and per-provider settings under `plugins.<id>.settings`.
- Status bar `connection_display_string` stays; just reads from `active_authority.display_string`.

## Migration path (incremental, each step ships independently)

1. **Land PR #1609 as-is** (with the schema revert already on this branch). Gets users the attach-mode UX immediately. Technical debt is acknowledged and bounded.
2. **Fix the plugin/core conflicts.** In the plugin: drop the activation popup (core auto-connects), drop `Dev Container: Open Terminal` (every terminal is already inside), route the plugin's `Rebuild` through a new core op so `container_id` stays in sync.
3. **Introduce `AuthorityProvider` and `ActiveAuthority` in core.** Migrate the existing local/SSH path onto it first — no new behavior, just refactor. Prove the abstraction holds.
4. **Add `editor.registerAuthorityProvider` plugin op.** Port the SSH path as a reference provider (or leave it in core; either is fine).
5. **Move devcontainer into a provider in the plugin.** The plugin becomes the authority provider; core's `services/devcontainer/` disappears. `Editor.container_*` fields disappear.
6. **Add LSP / indexer / file-watcher gating on `authority.ready`.** Closes the "container not up yet" race. Required once the provider can be async and slow (cold builds).
7. **Reconsider startup UX.** With lifecycle moved into the plugin, the editor can render immediately and the provider can stream progress — no more frozen startup on cold builds.

Steps 1–2 address the immediate integration. Steps 3–7 are the architectural target and can land over time without breaking users.

## Open questions

- **One authority per workspace, or multiple?** The Gemini landscape report flags monorepo workspaces with per-directory authorities as common. First cut: one per workspace. Multi-authority is a follow-up once the primitive exists.
- **Where does `editor.spawnProcess` (plugin API) route through?** Today it's host-local. Under the authority model, plugin `spawnProcess` calls should go through the active authority by default, with an opt-out for plugins that genuinely need the host (e.g. the devcontainer provider itself, when running `devcontainer up` from the host). Needs an explicit `target: "host" | "authority"` parameter.
- **UID/GID syncing.** Handled today only by `-u <remoteUser>` from devcontainer.json. Not in scope for the first cut but worth a follow-up.
- **Clone-sources mode.** The devcontainer spec supports git-clone-inside-container (no bind mount) for performance. Not addressed; bind-mount only for now.

## Non-goals

- Reaching feature parity with VS Code Dev Containers. No container-side extension host, no port-forwarding UI, no dotfiles sync, no GUI forwarding.
- Managing Docker directly. Everything goes through the `devcontainer` CLI, which itself shells out to Docker / Compose / Podman.
- Cross-provider composition (e.g. "SSH into host, then devcontainer on that host"). Interesting, out of scope.
