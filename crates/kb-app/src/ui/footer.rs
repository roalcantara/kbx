//! D14's footer (01-list, 04-entry-template): 40 tall, a hairline over it; left the status line
//! or the entry's tile, file and `⌥⌘C`; right the screen's keys and _Actions ⌘K_.

use gpui_kit::component::h_flex;
use gpui_kit::{Div, IntoElement, ParentElement as _, SharedString, Styled as _, div};

use crate::ui::design::{TextRoleExt as _, design, glow, pt};
use crate::ui::keycap::{ChipState, keycap};

/// The status dot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dot {
    /// Green with its glow: the index is ready.
    Ready,
    /// Yellow: a refresh runs.
    Busy,
    /// Faint, no glow: no index.
    Off,
}

/// The footer's row; `entry` (a pane or the detail screen is open) narrows its right inset and
/// key gap, as the template page does.
#[must_use]
pub fn footer(left: impl IntoElement, right: Vec<gpui_kit::AnyElement>, entry: bool) -> Div {
    let d = design();
    let s = &d.spacing.footer;
    h_flex()
        .h(pt(d.components.footer.height))
        .w_full()
        .flex_none()
        .items_center()
        .justify_between()
        .pl(pt(s.pad_left))
        .pr(pt(if entry { s.entry_pad_right } else { s.pad_right }))
        .border_t(pt(d.components.border.hairline))
        .border_color(d.colors.edge)
        .text_role(&d.typography.roles.footer)
        .text_color(d.colors.faint)
        .child(div().min_w_0().child(left))
        .child(
            h_flex()
                .flex_none()
                .items_center()
                .gap(pt(if entry { s.entry_gap } else { s.gap }))
                .children(right),
        )
}

/// `● Ready • 12 results`: the dot, the word in muted text, then the count.
#[must_use]
pub fn status(dot: Dot, word: impl Into<SharedString>, count: Option<String>) -> Div {
    let d = design();
    let size = pt(d.components.footer.dot);
    let dot = match dot {
        Dot::Ready => div().size(size).rounded_full().bg(d.colors.green).shadow(glow(
            d.colors.green,
            d.components.footer.dot_glow,
            d.components.footer.dot_glow_alpha,
        )),
        Dot::Busy => div().size(size).rounded_full().bg(d.colors.yellow).shadow(glow(
            d.colors.yellow,
            d.components.footer.dot_glow,
            d.components.footer.dot_glow_alpha,
        )),
        Dot::Off => div().size(size).rounded_full().bg(d.colors.faint),
    };
    h_flex()
        .items_center()
        .gap(pt(d.spacing.footer.left_gap))
        .child(dot)
        .child(
            div()
                .text_role(&d.typography.roles.footer)
                .text_color(d.colors.muted)
                .child(word.into()),
        )
        .when_some_count(count)
}

/// Adds `• count` to the status line when there is a count.
trait WhenSomeCount {
    fn when_some_count(self, count: Option<String>) -> Self;
}

impl WhenSomeCount for Div {
    fn when_some_count(self, count: Option<String>) -> Self {
        match count {
            Some(count) => self.child("•").child(count),
            None => self,
        }
    }
}

/// The entry's tile, its file in mono and the copy-path keycap.
#[must_use]
pub fn entry_status(tile: impl IntoElement, file: impl Into<SharedString>, copy_caps: impl Into<SharedString>) -> Div {
    let d = design();
    h_flex()
        .min_w_0()
        .items_center()
        .gap(pt(d.spacing.footer.left_gap))
        .child(tile)
        .child(
            div()
                .truncate()
                .font_family(d.typography.mono.clone())
                .text_color(d.colors.muted)
                .child(file.into()),
        )
        .child(keycap(copy_caps, ChipState::Live))
}

/// A key on the right: its label in muted text and its keycap.
#[must_use]
pub fn key_item(label: impl Into<SharedString>, keys_caps: impl Into<SharedString>) -> Div {
    let d = design();
    h_flex()
        .items_center()
        .gap(pt(d.spacing.footer.item_gap))
        .text_role(&d.typography.roles.footer)
        .text_color(d.colors.muted)
        .child(label.into())
        .child(keycap(keys_caps, ChipState::Live))
}
