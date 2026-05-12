//! Per-window buffer storage with the underlying map encapsulated.
//!
//! `WindowBuffers` wraps the `HashMap<BufferId, EditorState>` that
//! used to be a `pub` field on `Window`. The inner map is private to
//! this module — meaning *no other code in the crate*, including
//! other `impl Window` blocks across `window/mod.rs`,
//! `window_actions.rs`, and the various `impl Editor` blocks that
//! reach in via `self.windows.get_mut(&id)`, can mutate the storage
//! except through the methods below. That funnels every add / remove
//! through one auditable surface, which is the prerequisite for
//! enforcing invariants like "every `BufferId` reachable from the
//! split tree is present in `WindowBuffers`" (issue #1939 root cause).
//!
//! This step is encapsulation only — the public API mirrors the
//! `HashMap` surface the call sites already use. A follow-up can
//! tighten `remove` to require a split-tree reconciliation handle, or
//! make `insert` produce a token only obtainable from the buffer
//! itself, without touching any call site again.

use fresh_core::BufferId;
use std::collections::HashMap;

use crate::state::EditorState;

/// Per-window storage of live `EditorState`s, keyed by `BufferId`.
///
/// Constructed empty via [`WindowBuffers::new`] and populated through
/// [`insert`](Self::insert). Removal goes through [`remove`](Self::remove).
/// Iteration and reads use the inherent methods that mirror the
/// `HashMap` surface plus `IntoIterator for &WindowBuffers` /
/// `&mut WindowBuffers`, so `for (id, state) in &window.buffers` and
/// `for state in window.buffers.values_mut()` keep working.
pub struct WindowBuffers {
    map: HashMap<BufferId, EditorState>,
}

impl WindowBuffers {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, id: &BufferId) -> Option<&EditorState> {
        self.map.get(id)
    }

    pub fn get_mut(&mut self, id: &BufferId) -> Option<&mut EditorState> {
        self.map.get_mut(id)
    }

    pub fn insert(&mut self, id: BufferId, state: EditorState) -> Option<EditorState> {
        self.map.insert(id, state)
    }

    pub fn remove(&mut self, id: &BufferId) -> Option<EditorState> {
        self.map.remove(id)
    }

    pub fn contains_key(&self, id: &BufferId) -> bool {
        self.map.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, BufferId, EditorState> {
        self.map.keys()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<'_, BufferId, EditorState> {
        self.map.values()
    }

    pub fn values_mut(
        &mut self,
    ) -> std::collections::hash_map::ValuesMut<'_, BufferId, EditorState> {
        self.map.values_mut()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, BufferId, EditorState> {
        self.map.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<'_, BufferId, EditorState> {
        self.map.iter_mut()
    }
}

impl Default for WindowBuffers {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a WindowBuffers {
    type Item = (&'a BufferId, &'a EditorState);
    type IntoIter = std::collections::hash_map::Iter<'a, BufferId, EditorState>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<'a> IntoIterator for &'a mut WindowBuffers {
    type Item = (&'a BufferId, &'a mut EditorState);
    type IntoIter = std::collections::hash_map::IterMut<'a, BufferId, EditorState>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}
