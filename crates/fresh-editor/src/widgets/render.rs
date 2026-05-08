//! Render a `WidgetSpec` tree into `Vec<TextPropertyEntry>`.
//!
//! This is the path from declarative spec to the bytes the existing
//! virtual-buffer pipeline already knows how to display. By going
//! through `TextPropertyEntry`, widgets paint via exactly the same
//! renderer that today's `setVirtualBufferContent` uses — no parallel
//! render path. This is what makes the new widget API additive: the
//! buffer mid-bytes are indistinguishable from hand-rolled output.
//!
//! v1 dispatches on four kinds:
//!   * `Row` — children laid out left-to-right within a single line
//!     (the result is one `TextPropertyEntry`).
//!   * `Col` — children stacked vertically (the result is one
//!     `TextPropertyEntry` per child output line).
//!   * `HintBar` — keyboard-hint footer (one `TextPropertyEntry`).
//!   * `Raw` — pass-through (zero interpretation; plugin's entries
//!     flow through unchanged).
//!
//! Future kinds (`Toggle`, `Button`, `TextInput`, `List`, `Tree`,
//! `Layer`, `Transient`, `Table`) extend the dispatch without
//! changing the public function signature.

use fresh_core::api::{ButtonKind, HintEntry, OverlayColorSpec, OverlayOptions, WidgetSpec};
use fresh_core::text_property::{InlineOverlay, TextPropertyEntry};

// Theme keys used by the v1 widget renderers. Centralized so future
// "role-based" theming (§7 of the design doc) has one place to
// substitute the role→key mapping.
const KEY_HELP_KEY_FG: &str = "ui.help_key_fg";
const KEY_TOGGLE_ON_FG: &str = "ui.tab_active_fg";
const KEY_FOCUSED_FG: &str = "ui.menu_active_fg";
const KEY_FOCUSED_BG: &str = "ui.menu_active_bg";
const KEY_DANGER_FG: &str = "ui.status_error_indicator_fg";

/// Render a spec to a flat `Vec<TextPropertyEntry>` ready for
/// `set_virtual_buffer_content`. The entries do not contain trailing
/// newlines; the caller composes lines exactly as
/// `setVirtualBufferContent` already expects.
pub fn render_spec(spec: &WidgetSpec) -> Vec<TextPropertyEntry> {
    let mut out = Vec::new();
    render_into(spec, &mut out);
    out
}

fn render_into(spec: &WidgetSpec, out: &mut Vec<TextPropertyEntry>) {
    match spec {
        WidgetSpec::Row { children, .. } => {
            // v1: rows containing only inline-stylable children
            // collapse to one entry. Children that themselves emit
            // multiple lines (e.g. Col, Raw) are flattened in
            // declaration order — this matches the line-oriented
            // layout described in §3 of the design doc.
            let mut acc: Option<TextPropertyEntry> = None;
            for child in children {
                let child_entries = render_spec(child);
                if child_entries.is_empty() {
                    continue;
                }
                if child_entries.len() == 1 {
                    let mut entry = child_entries.into_iter().next().unwrap();
                    match acc.as_mut() {
                        Some(merged) => merge_inline(merged, &mut entry),
                        None => acc = Some(entry),
                    }
                } else {
                    // Multi-line child inside a Row: flush whatever
                    // we've accumulated, then emit the child's lines
                    // straight through. Mixing inline-row + block
                    // children in the same Row is only meaningful
                    // when the block child is the last item; this is
                    // good enough for v1 and avoids reflow logic.
                    if let Some(merged) = acc.take() {
                        out.push(merged);
                    }
                    out.extend(child_entries);
                }
            }
            if let Some(merged) = acc {
                out.push(merged);
            }
        }
        WidgetSpec::Col { children, .. } => {
            for child in children {
                render_into(child, out);
            }
        }
        WidgetSpec::HintBar { entries, .. } => {
            out.push(render_hint_bar(entries));
        }
        WidgetSpec::Toggle {
            checked,
            label,
            focused,
            ..
        } => {
            out.push(render_toggle(*checked, label, *focused));
        }
        WidgetSpec::Button {
            label,
            focused,
            intent,
            ..
        } => {
            out.push(render_button(label, *focused, *intent));
        }
        WidgetSpec::Spacer { cols, .. } => {
            // In an inline-row context a Spacer is N spaces; in a
            // block context (top-level / Col) it's a blank line per
            // `cols`. The Row collapse path treats a single-line
            // entry as inline, so emitting a single entry with N
            // spaces does the right thing in both contexts: in a
            // Col it becomes one short blank line; in a Row it
            // collapses inline alongside neighbours.
            let cols = (*cols).min(4096) as usize;
            let mut text = String::with_capacity(cols);
            for _ in 0..cols {
                text.push(' ');
            }
            out.push(TextPropertyEntry {
                text,
                properties: Default::default(),
                style: None,
                inline_overlays: Vec::new(),
            });
        }
        WidgetSpec::Raw { entries, .. } => {
            out.extend(entries.iter().cloned());
        }
    }
}

/// Render a HintBar into a single `TextPropertyEntry`.
///
/// Layout: `<keys> <label>  <keys> <label>  …`. The key portion of
/// each entry is highlighted with the `ui.help_key_fg` theme key;
/// labels use the buffer's default foreground.
///
/// This replaces the per-plugin hand-rolled footer at e.g.
/// `crates/fresh-editor/plugins/search_replace.ts:535–541`,
/// `audit_mode.ts:1068–1158`, `pkg.ts:2136–2145`.
pub fn render_hint_bar(entries: &[HintEntry]) -> TextPropertyEntry {
    let separator = "  ";
    let mut text = String::new();
    let mut overlays = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        if i > 0 {
            text.push_str(separator);
        }
        let key_start = text.len();
        text.push_str(&entry.keys);
        let key_end = text.len();
        if key_end > key_start {
            overlays.push(InlineOverlay {
                start: key_start,
                end: key_end,
                style: OverlayOptions {
                    fg: Some(OverlayColorSpec::theme_key(KEY_HELP_KEY_FG)),
                    bold: true,
                    ..Default::default()
                },
                properties: Default::default(),
            });
        }
        if !entry.label.is_empty() {
            text.push(' ');
            text.push_str(&entry.label);
        }
    }
    TextPropertyEntry {
        text,
        properties: Default::default(),
        style: None,
        inline_overlays: overlays,
    }
}

/// Render a `Toggle` to a single `TextPropertyEntry`.
///
/// Layout: `[v] label` when checked, `[ ] label` when not. The check
/// glyph is colored via `ui.tab_active_fg` when checked (no override
/// when unchecked). When focused, the entire entry is given a focused
/// fg/bg pair (`ui.menu_active_fg`/`ui.menu_active_bg`) plus bold —
/// matching the Settings UI's selected-control affordance.
pub fn render_toggle(checked: bool, label: &str, focused: bool) -> TextPropertyEntry {
    let glyph = if checked { "[v]" } else { "[ ]" };
    let mut text = String::with_capacity(glyph.len() + 1 + label.len());
    text.push_str(glyph);
    text.push(' ');
    text.push_str(label);

    let mut overlays = Vec::new();

    // Check-glyph color (only when checked — leaves default fg
    // when unchecked, which is what plugins do today).
    if checked {
        overlays.push(InlineOverlay {
            start: 0,
            end: glyph.len(),
            style: OverlayOptions {
                fg: Some(OverlayColorSpec::theme_key(KEY_TOGGLE_ON_FG)),
                bold: true,
                ..Default::default()
            },
            properties: Default::default(),
        });
    }

    // Focused: full-entry fg/bg + bold.
    if focused {
        overlays.push(InlineOverlay {
            start: 0,
            end: text.len(),
            style: OverlayOptions {
                fg: Some(OverlayColorSpec::theme_key(KEY_FOCUSED_FG)),
                bg: Some(OverlayColorSpec::theme_key(KEY_FOCUSED_BG)),
                bold: true,
                ..Default::default()
            },
            properties: Default::default(),
        });
    }

    TextPropertyEntry {
        text,
        properties: Default::default(),
        style: None,
        inline_overlays: overlays,
    }
}

/// Render a `Button` to a single `TextPropertyEntry`.
///
/// Layout: `[ Label ]` (with explicit space padding so the label
/// is visually inset from the brackets). Styling depends on `kind`
/// and `focused`:
///
/// * `Normal`     — default fg; focused → fg/bg flip + bold.
/// * `Primary`    — bold; focused → fg/bg flip.
/// * `Danger`     — red fg (theme `ui.status_error_indicator_fg`);
///                  focused → bold.
pub fn render_button(label: &str, focused: bool, kind: ButtonKind) -> TextPropertyEntry {
    let text = format!("[ {} ]", label);
    let mut overlays = Vec::new();

    let base_style = match kind {
        ButtonKind::Normal => OverlayOptions::default(),
        ButtonKind::Primary => OverlayOptions {
            bold: true,
            ..Default::default()
        },
        ButtonKind::Danger => OverlayOptions {
            fg: Some(OverlayColorSpec::theme_key(KEY_DANGER_FG)),
            ..Default::default()
        },
    };

    let style = if focused {
        OverlayOptions {
            fg: Some(OverlayColorSpec::theme_key(KEY_FOCUSED_FG)),
            bg: Some(OverlayColorSpec::theme_key(KEY_FOCUSED_BG)),
            bold: true,
            ..base_style
        }
    } else {
        base_style
    };

    // Only emit an overlay if the style is non-default — keeps the
    // serialized entry tight.
    if style.fg.is_some()
        || style.bg.is_some()
        || style.bold
        || style.italic
        || style.underline
        || style.strikethrough
    {
        overlays.push(InlineOverlay {
            start: 0,
            end: text.len(),
            style,
            properties: Default::default(),
        });
    }

    TextPropertyEntry {
        text,
        properties: Default::default(),
        style: None,
        inline_overlays: overlays,
    }
}

/// Merge `next` into `merged` for the inline-row collapse path.
/// `next`'s overlays are byte-shifted to account for the merged
/// text length so far.
fn merge_inline(merged: &mut TextPropertyEntry, next: &mut TextPropertyEntry) {
    let shift = merged.text.len();
    merged.text.push_str(&next.text);
    for overlay in next.inline_overlays.drain(..) {
        merged.inline_overlays.push(InlineOverlay {
            start: overlay.start + shift,
            end: overlay.end + shift,
            style: overlay.style,
            properties: overlay.properties,
        });
    }
    // `style` and `properties` from `next` are dropped — Row inline
    // collapse only preserves inline_overlays. Whole-entry style on
    // an inline-row child has no meaningful semantics here; if a
    // plugin needs whole-line styling it should produce a Col with
    // the styled child as its sole element.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_bar_renders_entries_with_key_overlays() {
        let entries = vec![
            HintEntry {
                keys: "Tab".into(),
                label: "next".into(),
            },
            HintEntry {
                keys: "Esc".into(),
                label: "close".into(),
            },
        ];
        let entry = render_hint_bar(&entries);
        assert_eq!(entry.text, "Tab next  Esc close");
        assert_eq!(entry.inline_overlays.len(), 2);
        // First overlay covers "Tab" (bytes 0..3).
        assert_eq!(entry.inline_overlays[0].start, 0);
        assert_eq!(entry.inline_overlays[0].end, 3);
        // Second overlay covers "Esc" (bytes 10..13).
        assert_eq!(entry.inline_overlays[1].start, 10);
        assert_eq!(entry.inline_overlays[1].end, 13);
    }

    #[test]
    fn hint_bar_omits_label_when_empty() {
        let entries = vec![HintEntry {
            keys: "?".into(),
            label: "".into(),
        }];
        let entry = render_hint_bar(&entries);
        assert_eq!(entry.text, "?");
    }

    #[test]
    fn col_stacks_children_top_to_bottom() {
        let spec = WidgetSpec::Col {
            children: vec![
                WidgetSpec::HintBar {
                    entries: vec![HintEntry {
                        keys: "A".into(),
                        label: "alpha".into(),
                    }],
                    key: None,
                },
                WidgetSpec::HintBar {
                    entries: vec![HintEntry {
                        keys: "B".into(),
                        label: "beta".into(),
                    }],
                    key: None,
                },
            ],
            key: None,
        };
        let out = render_spec(&spec);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].text, "A alpha");
        assert_eq!(out[1].text, "B beta");
    }

    #[test]
    fn raw_passes_through_unchanged() {
        let spec = WidgetSpec::Raw {
            entries: vec![TextPropertyEntry::text("hello")],
            key: None,
        };
        let out = render_spec(&spec);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].text, "hello");
    }

    #[test]
    fn toggle_checked_emits_glyph_overlay() {
        let entry = render_toggle(true, "Case", false);
        assert_eq!(entry.text, "[v] Case");
        // One overlay for the glyph, no focused overlay.
        assert_eq!(entry.inline_overlays.len(), 1);
        assert_eq!(entry.inline_overlays[0].start, 0);
        assert_eq!(entry.inline_overlays[0].end, 3);
    }

    #[test]
    fn toggle_unchecked_no_glyph_overlay() {
        let entry = render_toggle(false, "Case", false);
        assert_eq!(entry.text, "[ ] Case");
        assert_eq!(entry.inline_overlays.len(), 0);
    }

    #[test]
    fn toggle_focused_adds_full_entry_overlay() {
        let entry = render_toggle(true, "Case", true);
        // Glyph overlay + focused overlay.
        assert_eq!(entry.inline_overlays.len(), 2);
        // Focused overlay spans the full entry.
        assert_eq!(entry.inline_overlays[1].start, 0);
        assert_eq!(entry.inline_overlays[1].end, entry.text.len());
        assert!(entry.inline_overlays[1].style.bold);
    }

    #[test]
    fn button_normal_unfocused_has_no_overlay() {
        let entry = render_button("Replace All", false, ButtonKind::Normal);
        assert_eq!(entry.text, "[ Replace All ]");
        assert!(entry.inline_overlays.is_empty());
    }

    #[test]
    fn button_primary_is_bold() {
        let entry = render_button("Submit", false, ButtonKind::Primary);
        assert_eq!(entry.inline_overlays.len(), 1);
        assert!(entry.inline_overlays[0].style.bold);
    }

    #[test]
    fn button_danger_uses_error_theme_key() {
        let entry = render_button("Delete", false, ButtonKind::Danger);
        assert_eq!(entry.inline_overlays.len(), 1);
        let fg = entry.inline_overlays[0].style.fg.as_ref().unwrap();
        assert_eq!(fg.as_theme_key(), Some("ui.status_error_indicator_fg"));
    }

    #[test]
    fn button_focused_overrides_with_menu_active_keys() {
        let entry = render_button("OK", true, ButtonKind::Normal);
        let style = &entry.inline_overlays[0].style;
        assert_eq!(
            style.fg.as_ref().and_then(|c| c.as_theme_key()),
            Some("ui.menu_active_fg")
        );
        assert_eq!(
            style.bg.as_ref().and_then(|c| c.as_theme_key()),
            Some("ui.menu_active_bg")
        );
        assert!(style.bold);
    }

    #[test]
    fn spacer_in_row_pads_with_spaces() {
        let spec = WidgetSpec::Row {
            children: vec![
                WidgetSpec::Toggle {
                    checked: false,
                    label: "A".into(),
                    focused: false,
                    key: None,
                },
                WidgetSpec::Spacer { cols: 4, key: None },
                WidgetSpec::Button {
                    label: "Go".into(),
                    focused: false,
                    intent: ButtonKind::Normal,
                    key: None,
                },
            ],
            key: None,
        };
        let out = render_spec(&spec);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].text, "[ ] A    [ Go ]");
    }

    #[test]
    fn row_collapses_inline_children_with_shifted_overlays() {
        let spec = WidgetSpec::Row {
            children: vec![
                WidgetSpec::HintBar {
                    entries: vec![HintEntry {
                        keys: "Tab".into(),
                        label: "x".into(),
                    }],
                    key: None,
                },
                WidgetSpec::HintBar {
                    entries: vec![HintEntry {
                        keys: "Esc".into(),
                        label: "y".into(),
                    }],
                    key: None,
                },
            ],
            key: None,
        };
        let out = render_spec(&spec);
        assert_eq!(out.len(), 1);
        // Two adjacent HintBars are concatenated; the second's overlay shifts.
        assert_eq!(out[0].text, "Tab xEsc y");
        assert_eq!(out[0].inline_overlays.len(), 2);
        assert_eq!(out[0].inline_overlays[1].start, 5);
        assert_eq!(out[0].inline_overlays[1].end, 8);
    }
}
