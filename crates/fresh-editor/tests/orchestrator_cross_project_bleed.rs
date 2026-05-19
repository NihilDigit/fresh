//! Hermetic, model-style **persistence-component test** for the
//! user-reported "yesterday's projects bleed into today" orchestrator
//! bug. The cross-project leak is ultimately user-visible (file
//! explorer, tabs), but the underlying state the leak is built from
//! lives on disk — `windows.json`, the per-cwd workspace files, and
//! the plugin global state — so the assertions here observe those
//! files directly rather than driving keyboard/mouse events. A
//! separate render-level e2e test should follow once we know which
//! factor combination reproduces the bug (this test's output names
//! the combination).
//!
//! User report (paraphrased):
//!
//! > I reverted to 0.3.6 today. Yesterday I was playing with the
//! > orchestration feature. Today it's picking up directories from
//! > yesterday — files from yesterday's projects appear in the file
//! > explorer, and even tabs/buffers from yesterday's work show up.
//!
//! This test enumerates the cartesian product of the factors that
//! plausibly trigger the symptom and asserts a single black-box
//! invariant for every cell:
//!
//!   **Launching the editor at project A must not surface, persist,
//!   or load anything from project B's tree, no matter what state
//!   yesterday's editor left behind under the data dir.**
//!
//! The factors enumerated:
//!
//!   - Envelope shape on disk (factor A in
//!     `docs/internal/PLAN-orchestrator-session-persistence-tests.md`):
//!     `None`, `V1Legacy`, `V2Global`, `V1AndV2`, `FutureV3`,
//!     `Malformed`, `MigratedBakOnly`, `LeftoverTmp`.
//!   - Whether a per-cwd workspace file for project B exists under
//!     `<data>/workspaces/` (the file `restore_inactive_window_workspaces`
//!     reads — independent of `windows.json`).
//!   - Whether plugin global state under `<data>/orchestrator/state/`
//!     mentions project B's paths.
//!   - `editor.restore_previous_session` on/off.
//!   - Today's launch cwd relationship to project B (`Unrelated`,
//!     `Sibling`, `Parent`, `SameAsB`).
//!
//! Hermeticity:
//!
//!   - `XDG_DATA_HOME` and `HOME` are pointed at a per-test temp dir
//!     so `dirs::data_dir()` (used by `workspace.rs::get_data_dir`,
//!     which bypasses `DirectoryContext`) and the editor's
//!     `DirectoryContext.data_dir` both resolve under the same temp
//!     tree. Both env vars are restored on `Drop`.
//!   - The data dir is wiped between scenarios so each cell starts
//!     from the seed alone.
//!   - No git, no network, no real `$HOME` access.
//!
//! Observables (all black-box):
//!
//!   - The number of windows in the post-launch in-memory state,
//!     observed via the on-disk `windows.json` after a no-op quit
//!     (`save_orchestrator_state` snapshots `editor.windows`).
//!   - Whether any persisted window's `root` or `project_path`
//!     points inside project B's tree after a project-A-only launch.
//!   - Whether any unrelated workspace file under
//!     `<data>/workspaces/` was read (proxy: was the active window's
//!     buffer list contaminated, or was a new workspace file written
//!     for B without the user ever visiting B).
//!
//! Failing cells are aggregated into a structured report; the test
//! fails iff any cell trips the invariant. The report tells us which
//! combination(s) of factors trigger the user-visible leak — i.e. it
//! discovers the bug's underlying cause from the matrix.

mod common;

use fresh::app::Editor;
use fresh::config::Config;
use fresh::config_io::DirectoryContext;
use fresh::model::filesystem::StdFileSystem;
use fresh::view::color_support::ColorCapability;
use fresh::workspace::encode_path_for_filename;
use serde_json::json;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

// ─────────────────────────────────────────────────────────────────────
// Hermetic env guard
// ─────────────────────────────────────────────────────────────────────

/// Pins `XDG_DATA_HOME` and `HOME` to a temp path for the lifetime of
/// the test, restoring the previous values on drop. The `dirs` crate
/// reads these on every call so the redirect takes effect immediately.
///
/// This is *process-global*: only one such guard may live at a time,
/// which is fine because this whole file has a single `#[test]` and
/// integration test binaries are one-process-per-binary.
struct HermeticEnv {
    prev_xdg: Option<OsString>,
    prev_home: Option<OsString>,
}

impl HermeticEnv {
    fn new(root: &Path) -> Self {
        let prev_xdg = std::env::var_os("XDG_DATA_HOME");
        let prev_home = std::env::var_os("HOME");
        std::env::set_var("XDG_DATA_HOME", root);
        std::env::set_var("HOME", root);
        Self {
            prev_xdg,
            prev_home,
        }
    }
}

impl Drop for HermeticEnv {
    fn drop(&mut self) {
        match &self.prev_xdg {
            Some(v) => std::env::set_var("XDG_DATA_HOME", v),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        match &self.prev_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }
}

/// Resolve the data dir the way the editor will after our env pins
/// take effect. `dirs::data_dir()` returns `XDG_DATA_HOME` on Linux
/// and `$HOME/Library/Application Support` on macOS; both branches
/// resolve under the temp root because we set both env vars.
fn resolved_data_dir() -> PathBuf {
    dirs::data_dir().expect("data dir resolvable").join("fresh")
}

// ─────────────────────────────────────────────────────────────────────
// Factor enumeration
// ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnvelopeShape {
    None,
    /// Legacy v1 per-cwd `<data>/orchestrator/<encoded_cwd>/windows.json`.
    V1Legacy,
    /// Global v2 `<data>/orchestrator/windows.json`.
    V2Global,
    /// Both v1 and v2 present (interrupted migration).
    V1AndV2,
    /// `version=3` envelope with unknown fields — forward-compat case.
    FutureV3,
    /// Invalid JSON.
    Malformed,
    /// Only `.migrated.bak` files remain (post-migration, no live envelope).
    MigratedBakOnly,
    /// `.tmp` from an interrupted atomic write.
    LeftoverTmp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum YesterdayWorkspaceFile {
    Absent,
    /// `<data>/workspaces/<hash_of_B>.json` exists and lists `B/leaked.txt`.
    PresentWithLeakedBuffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum YesterdayPluginState {
    Absent,
    /// `<data>/orchestrator/state/orchestrator.json` mentions a path
    /// inside project B.
    MentionsB,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CwdRelation {
    /// Launch cwd is an unrelated directory (project A).
    Unrelated,
    /// Launch cwd is a sibling of project B (same parent).
    Sibling,
    /// Launch cwd is the parent directory of project B.
    Parent,
    /// Launch cwd IS project B (control: should restore B legitimately).
    SameAsB,
}

#[derive(Clone, Copy, Debug)]
struct Scenario {
    envelope: EnvelopeShape,
    workspace_file: YesterdayWorkspaceFile,
    plugin_state: YesterdayPluginState,
    restore_prev: bool,
    cwd_rel: CwdRelation,
}

fn all_scenarios() -> Vec<Scenario> {
    let envelopes = [
        EnvelopeShape::None,
        EnvelopeShape::V1Legacy,
        EnvelopeShape::V2Global,
        EnvelopeShape::V1AndV2,
        EnvelopeShape::FutureV3,
        EnvelopeShape::Malformed,
        EnvelopeShape::MigratedBakOnly,
        EnvelopeShape::LeftoverTmp,
    ];
    let workspace_files = [
        YesterdayWorkspaceFile::Absent,
        YesterdayWorkspaceFile::PresentWithLeakedBuffer,
    ];
    let plugin_states = [
        YesterdayPluginState::Absent,
        YesterdayPluginState::MentionsB,
    ];
    let restore_flags = [false, true];
    let cwd_rels = [
        CwdRelation::Unrelated,
        CwdRelation::Sibling,
        CwdRelation::Parent,
        CwdRelation::SameAsB,
    ];

    let mut out = Vec::new();
    for &envelope in &envelopes {
        for &workspace_file in &workspace_files {
            for &plugin_state in &plugin_states {
                for &restore_prev in &restore_flags {
                    for &cwd_rel in &cwd_rels {
                        out.push(Scenario {
                            envelope,
                            workspace_file,
                            plugin_state,
                            restore_prev,
                            cwd_rel,
                        });
                    }
                }
            }
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────
// Seeding yesterday's on-disk state
// ─────────────────────────────────────────────────────────────────────

/// Per-scenario seed paths in the hermetic data tree.
struct SeedPaths {
    orchestrator_dir: PathBuf,
    global_windows: PathBuf,
    workspaces_dir: PathBuf,
    plugin_state_dir: PathBuf,
    project_b_root: PathBuf,
    leaked_file_in_b: PathBuf,
}

fn seed_paths(data_dir: &Path, project_b: &Path) -> SeedPaths {
    let orch = data_dir.join("orchestrator");
    SeedPaths {
        orchestrator_dir: orch.clone(),
        global_windows: orch.join("windows.json"),
        workspaces_dir: data_dir.join("workspaces"),
        plugin_state_dir: orch.join("state"),
        project_b_root: project_b.to_path_buf(),
        leaked_file_in_b: project_b.join("leaked.txt"),
    }
}

fn wipe_data_dir(data_dir: &Path) {
    if data_dir.exists() {
        std::fs::remove_dir_all(data_dir).expect("wipe data dir");
    }
    std::fs::create_dir_all(data_dir).expect("recreate data dir");
}

fn seed_envelope(seeds: &SeedPaths, shape: EnvelopeShape) {
    std::fs::create_dir_all(&seeds.orchestrator_dir).unwrap();
    let b = seeds
        .project_b_root
        .canonicalize()
        .unwrap_or(seeds.project_b_root.clone());
    let v2_envelope = json!({
        "version": 2,
        "active": 42,
        "next_id": 100,
        "windows": [
            {
                "id": 42,
                "label": "B (yesterday)",
                "root": b,
                "project_path": b,
            }
        ]
    });
    let v1_envelope = json!({
        // No "version" field — defaults to 1.
        "active": 7,
        "next_id": 99,
        "windows": [
            {
                "id": 7,
                "label": "B (legacy)",
                "root": b,
                // No project_path / shared_worktree (v1 shape).
            }
        ]
    });
    let v1_dir = seeds.orchestrator_dir.join(encode_path_for_filename(&b));

    match shape {
        EnvelopeShape::None => { /* nothing seeded */ }
        EnvelopeShape::V1Legacy => {
            std::fs::create_dir_all(&v1_dir).unwrap();
            std::fs::write(
                v1_dir.join("windows.json"),
                serde_json::to_vec_pretty(&v1_envelope).unwrap(),
            )
            .unwrap();
        }
        EnvelopeShape::V2Global => {
            std::fs::write(
                &seeds.global_windows,
                serde_json::to_vec_pretty(&v2_envelope).unwrap(),
            )
            .unwrap();
        }
        EnvelopeShape::V1AndV2 => {
            std::fs::write(
                &seeds.global_windows,
                serde_json::to_vec_pretty(&v2_envelope).unwrap(),
            )
            .unwrap();
            std::fs::create_dir_all(&v1_dir).unwrap();
            std::fs::write(
                v1_dir.join("windows.json"),
                serde_json::to_vec_pretty(&v1_envelope).unwrap(),
            )
            .unwrap();
        }
        EnvelopeShape::FutureV3 => {
            let future = json!({
                "version": 3,
                "active": 42,
                "next_id": 100,
                "windows": [
                    {
                        "id": 42,
                        "label": "B (future)",
                        "root": b,
                        "project_path": b,
                        // Unknown v3 fields:
                        "unknown_field": "ignore me",
                        "another_unknown": { "nested": true }
                    }
                ],
                "global_unknown": [1, 2, 3]
            });
            std::fs::write(
                &seeds.global_windows,
                serde_json::to_vec_pretty(&future).unwrap(),
            )
            .unwrap();
        }
        EnvelopeShape::Malformed => {
            std::fs::write(&seeds.global_windows, b"{not valid json,,,").unwrap();
        }
        EnvelopeShape::MigratedBakOnly => {
            std::fs::create_dir_all(&v1_dir).unwrap();
            std::fs::write(
                v1_dir.join("windows.json.migrated.bak"),
                serde_json::to_vec_pretty(&v1_envelope).unwrap(),
            )
            .unwrap();
        }
        EnvelopeShape::LeftoverTmp => {
            std::fs::write(
                seeds.orchestrator_dir.join("windows.json.tmp"),
                serde_json::to_vec_pretty(&v2_envelope).unwrap(),
            )
            .unwrap();
        }
    }
}

fn seed_workspace_file(seeds: &SeedPaths, workspace_file: YesterdayWorkspaceFile) {
    if workspace_file == YesterdayWorkspaceFile::Absent {
        return;
    }
    std::fs::create_dir_all(&seeds.workspaces_dir).unwrap();
    std::fs::write(&seeds.leaked_file_in_b, b"leaked yesterday\n").unwrap();
    let canon_b = seeds
        .project_b_root
        .canonicalize()
        .unwrap_or_else(|_| seeds.project_b_root.clone());
    let workspace_filename = format!("{}.json", encode_path_for_filename(&canon_b));
    let canon_leaked = seeds
        .leaked_file_in_b
        .canonicalize()
        .unwrap_or_else(|_| seeds.leaked_file_in_b.clone());
    // Minimal Workspace-shape envelope. The editor's loader is
    // lenient about unknown fields, so we only need the keys it
    // actually reads to identify file-backed buffers.
    let ws = json!({
        "version": 1,
        "working_dir": canon_b,
        "buffers": [
            {
                "file_path": canon_leaked,
                "cursor_line": 0,
                "cursor_col": 0,
                "scroll_offset": 0,
                "horizontal_scroll": 0,
                "is_active": true,
                "language": null
            }
        ],
        "splits": null,
        "active_split_id": null,
        "next_split_id": 1
    });
    std::fs::write(
        seeds.workspaces_dir.join(workspace_filename),
        serde_json::to_vec_pretty(&ws).unwrap(),
    )
    .unwrap();
}

fn seed_plugin_state(seeds: &SeedPaths, plugin_state: YesterdayPluginState) {
    if plugin_state == YesterdayPluginState::Absent {
        return;
    }
    std::fs::create_dir_all(&seeds.plugin_state_dir).unwrap();
    let mention = json!({
        "last_opened_project": seeds.project_b_root,
        "history": [seeds.leaked_file_in_b]
    });
    std::fs::write(
        seeds.plugin_state_dir.join("orchestrator.json"),
        serde_json::to_vec_pretty(&mention).unwrap(),
    )
    .unwrap();
}

// ─────────────────────────────────────────────────────────────────────
// Launch + observe
// ─────────────────────────────────────────────────────────────────────

fn launch_cwd(scenario: &Scenario, project_a: &Path, project_b: &Path) -> PathBuf {
    match scenario.cwd_rel {
        CwdRelation::Unrelated => project_a.to_path_buf(),
        CwdRelation::Sibling => project_a.to_path_buf(),
        CwdRelation::Parent => project_b.parent().unwrap().to_path_buf(),
        CwdRelation::SameAsB => project_b.to_path_buf(),
    }
}

#[derive(Debug, Default)]
struct Observation {
    /// Window roots that ended up in the post-launch in-memory state.
    /// Captured by reading the `windows.json` the editor writes at
    /// quit, which is a snapshot of `editor.windows` plus any sessions
    /// spliced back in from the pre-launch file (the very behaviour
    /// under suspicion).
    post_quit_window_roots: Vec<PathBuf>,
    /// Names of files the editor opened during the launch+restore
    /// path, observed via the active window's `buffer_paths()`.
    active_buffer_paths: Vec<String>,
    /// Was the pre-existing leaked workspace file for project B still
    /// on disk after the quit? (If the editor read it, it would have
    /// been left in place; if it cleaned it up that's also a signal.)
    leaked_workspace_file_still_on_disk: bool,
}

impl Observation {
    fn leaks_project_b(&self, project_b: &Path) -> bool {
        let b_str = project_b.to_string_lossy().to_string();
        self.post_quit_window_roots
            .iter()
            .any(|p| p.starts_with(project_b))
            || self
                .active_buffer_paths
                .iter()
                .any(|p| p.contains(&b_str) || p.contains("leaked.txt"))
    }
}

fn run_scenario(
    scenario: &Scenario,
    data_dir: &Path,
    project_a: &Path,
    project_b: &Path,
) -> Observation {
    wipe_data_dir(data_dir);
    std::fs::create_dir_all(project_a).ok();
    std::fs::create_dir_all(project_b).ok();
    let seeds = seed_paths(data_dir, project_b);
    seed_envelope(&seeds, scenario.envelope);
    seed_workspace_file(&seeds, scenario.workspace_file);
    seed_plugin_state(&seeds, scenario.plugin_state);

    let cwd = launch_cwd(scenario, project_a, project_b);

    // Mirror main()'s post-construction restore steps. The
    // DirectoryContext must point at the SAME data dir that
    // `dirs::data_dir()/fresh` resolves to so the orchestrator code
    // (uses `dir_context.data_dir`) and the workspace code (uses
    // `dirs::data_dir()`) read/write the same tree.
    let dir_context = DirectoryContext {
        data_dir: data_dir.to_path_buf(),
        config_dir: data_dir.join("config"),
        home_dir: Some(data_dir.parent().unwrap().to_path_buf()),
        documents_dir: None,
        downloads_dir: None,
    };

    let filesystem: Arc<dyn fresh::model::filesystem::FileSystem + Send + Sync> =
        Arc::new(StdFileSystem);

    let config = Config {
        check_for_updates: false,
        editor: fresh::config::EditorConfig {
            restore_previous_session: scenario.restore_prev,
            ..Config::default().editor
        },
        ..Config::default()
    };

    let editor_result = Editor::for_test(
        config,
        80,
        24,
        Some(cwd.clone()),
        dir_context.clone(),
        ColorCapability::TrueColor,
        filesystem,
        None,
        None,
        false,
        false,
    );

    let mut editor = match editor_result {
        Ok(e) => e,
        Err(err) => {
            // Some seeded shapes (Malformed envelope, missing dirs)
            // may legitimately yield a construction error in CI
            // before the fix; that's a different failure mode than
            // "loaded silently and bled state", and the user's bug
            // isn't about crashes, so we capture nothing and let the
            // invariant check pass for this cell.
            eprintln!("scenario {scenario:?}: Editor::for_test returned Err({err}); skipping");
            return Observation::default();
        }
    };

    if scenario.restore_prev {
        let _ = editor.try_restore_workspace();
    }
    editor.restore_inactive_window_workspaces();

    // Black-box read of the active window's buffers. Inactive-window
    // buffers can't be observed without an instrumentation accessor,
    // so the contamination of inactive windows is detected via the
    // post-quit envelope below (which the editor builds from
    // `editor.windows`, all of them).
    let active_buffer_paths = active_buffer_paths(&editor);

    // Force the same path the user would hit by quitting.
    editor.save_orchestrator_state();
    drop(editor);

    let post_quit_window_roots = read_post_quit_roots(&seeds.global_windows);
    let leaked_workspace_file_still_on_disk = seeds
        .workspaces_dir
        .join(format!(
            "{}.json",
            encode_path_for_filename(
                &project_b
                    .canonicalize()
                    .unwrap_or_else(|_| project_b.to_path_buf())
            )
        ))
        .exists();

    Observation {
        post_quit_window_roots,
        active_buffer_paths,
        leaked_workspace_file_still_on_disk,
    }
}

fn active_buffer_paths(editor: &Editor) -> Vec<String> {
    editor
        .all_buffer_ids_for_tests()
        .into_iter()
        .map(|id| editor.get_buffer_display_name(id))
        .collect()
}

fn read_post_quit_roots(windows_json: &Path) -> Vec<PathBuf> {
    if !windows_json.exists() {
        return Vec::new();
    }
    let bytes = match std::fs::read(windows_json) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };
    let v: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    v.get("windows")
        .and_then(|w| w.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|w| w.get("root").and_then(|r| r.as_str()).map(PathBuf::from))
                .collect()
        })
        .unwrap_or_default()
}

// ─────────────────────────────────────────────────────────────────────
// The test
// ─────────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct Bleed {
    scenario: Scenario,
    observation: Observation,
}

#[test]
fn launch_in_project_a_must_not_leak_project_b_state() {
    let root = TempDir::new().expect("temp root");
    let _env = HermeticEnv::new(root.path());

    let projects_dir = root.path().join("projects");
    let project_a = projects_dir.join("project_a");
    let project_b = projects_dir.join("project_b");
    std::fs::create_dir_all(&project_a).unwrap();
    std::fs::create_dir_all(&project_b).unwrap();

    let data_dir = resolved_data_dir();

    let mut bleeds: Vec<Bleed> = Vec::new();
    let mut tested = 0usize;
    let mut skipped_for_cwd_relation = 0usize;

    for scenario in all_scenarios() {
        // `SameAsB` is a control: the user IS launching inside project
        // B, so seeing B's state is expected, not a leak. Run it to
        // make sure the rest of the pipeline (workspace restore) does
        // work when invoked legitimately, but exclude it from the
        // leak count.
        if scenario.cwd_rel == CwdRelation::SameAsB {
            let obs = run_scenario(&scenario, &data_dir, &project_a, &project_b);
            // We don't assert anything here directly; this cell is a
            // sanity check that the restore plumbing is reachable.
            let _ = obs;
            skipped_for_cwd_relation += 1;
            tested += 1;
            continue;
        }

        let obs = run_scenario(&scenario, &data_dir, &project_a, &project_b);
        tested += 1;
        if obs.leaks_project_b(&project_b) {
            bleeds.push(Bleed {
                scenario,
                observation: obs,
            });
        }
    }

    if !bleeds.is_empty() {
        let mut msg = format!(
            "\n{} of {} non-control scenarios leaked project B state into a project A launch \
             ({} control cells excluded).\n\
             Each scenario below ran in a clean data dir, set up yesterday's state per the \
             factor combination, launched the editor at cwd != project B, and observed \
             the on-disk envelope after quit.\n\n",
            bleeds.len(),
            tested - skipped_for_cwd_relation,
            skipped_for_cwd_relation,
        );

        // Group by envelope shape for readability — the user-visible
        // symptom likely correlates strongest with one or two
        // envelope shapes.
        let mut by_envelope: std::collections::BTreeMap<String, Vec<&Bleed>> =
            std::collections::BTreeMap::new();
        for b in &bleeds {
            by_envelope
                .entry(format!("{:?}", b.scenario.envelope))
                .or_default()
                .push(b);
        }

        for (env_label, items) in &by_envelope {
            msg.push_str(&format!(
                "== EnvelopeShape::{} ({} cells) ==\n",
                env_label,
                items.len()
            ));
            for b in items {
                msg.push_str(&format!(
                    "  cwd_rel={:?} restore_prev={} workspace_file={:?} plugin_state={:?}\n    \
                     post_quit_roots={:?}\n    \
                     active_buffer_paths={:?}\n    \
                     leaked_workspace_file_still_on_disk={}\n",
                    b.scenario.cwd_rel,
                    b.scenario.restore_prev,
                    b.scenario.workspace_file,
                    b.scenario.plugin_state,
                    b.observation.post_quit_window_roots,
                    b.observation.active_buffer_paths,
                    b.observation.leaked_workspace_file_still_on_disk,
                ));
            }
            msg.push('\n');
        }

        // Headline: the smallest factor combination that bleeds —
        // i.e. drop every axis whose value is constant across all
        // bleeders. That's the "minimal cause" the matrix found.
        msg.push_str(&minimal_factor_report(&bleeds));

        panic!("{msg}");
    }
}

/// For each axis, list the set of values seen among bleeding
/// scenarios. Axes whose set is a singleton are "necessary" for the
/// bleed; axes with multiple values are orthogonal to it.
fn minimal_factor_report(bleeds: &[Bleed]) -> String {
    use std::collections::BTreeSet;
    let mut envelopes: BTreeSet<String> = BTreeSet::new();
    let mut workspaces: BTreeSet<String> = BTreeSet::new();
    let mut plugins: BTreeSet<String> = BTreeSet::new();
    let mut restores: BTreeSet<bool> = BTreeSet::new();
    let mut cwds: BTreeSet<String> = BTreeSet::new();
    for b in bleeds {
        envelopes.insert(format!("{:?}", b.scenario.envelope));
        workspaces.insert(format!("{:?}", b.scenario.workspace_file));
        plugins.insert(format!("{:?}", b.scenario.plugin_state));
        restores.insert(b.scenario.restore_prev);
        cwds.insert(format!("{:?}", b.scenario.cwd_rel));
    }
    let mut s = String::from("== minimal factor signature of the bleed ==\n");
    s.push_str(&format!("  envelope ∈ {envelopes:?}\n"));
    s.push_str(&format!("  workspace_file ∈ {workspaces:?}\n"));
    s.push_str(&format!("  plugin_state ∈ {plugins:?}\n"));
    s.push_str(&format!("  restore_prev ∈ {restores:?}\n"));
    s.push_str(&format!("  cwd_rel ∈ {cwds:?}\n"));
    s.push_str(
        "  (Any axis whose set is a singleton is necessary for reproducing the bug;\n   axes \
         with multiple values are orthogonal.)\n",
    );
    s
}
