//! An entry's notes and metadata, as the split pane and the detail screen show them
//! (DESIGN.md, Notes and Metadata card).

use gpui_kit::component::text::TextView;
use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, px, relative,
};
use kb_domain::entry::Entry;

use crate::accents::accent;
use crate::metadata::metadata_card;
use crate::presentation::{Hue, notes_markdown, split_description};

/// Where the pane sits, which decides where its metadata card goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneLayout {
    /// The split pane: the card under the notes.
    Split,
    /// The detail screen: the card in a column on the right, a third of the width.
    Detail,
}

/// The notes, rendered from the index's Markdown with highlighted code blocks, and the metadata
/// card when `show_metadata`.
pub fn notes_pane(entry: &Entry, show_metadata: bool, layout: PaneLayout, cx: &App) -> AnyElement {
    let notes = notes_column(entry, cx);
    let card = show_metadata.then(|| div().p(px(12.)).child(metadata_card(entry, cx)));
    match layout {
        PaneLayout::Split => v_flex()
            .size_full()
            .min_h_0()
            .child(notes)
            .when_some(card, |pane, card| pane.child(card.flex_none()))
            .into_any_element(),
        PaneLayout::Detail => h_flex()
            .size_full()
            .min_h_0()
            .items_start()
            .child(notes)
            .when_some(card, |pane, card| {
                pane.child(
                    card.w(relative(1. / 3.))
                        .flex_none()
                        .h_full()
                        .border_l_1()
                        .border_color(cx.theme().border),
                )
            })
            .into_any_element(),
    }
}

/// The notes, scrolling: what precedes the description, the description quote with its red bar,
/// then the rest.
fn notes_column(entry: &Entry, cx: &App) -> impl IntoElement {
    let markdown = notes_markdown(entry);
    let parts = split_description(markdown, entry.description.as_deref());
    // Keyed by the entry: a TextView keeps its parse while its id is unchanged.
    let id = |part: &str| SharedString::from(format!("notes-{part}-{}", entry.id));
    div()
        .id(id("scroll"))
        .flex_1()
        .min_w_0()
        .min_h_0()
        .h_full()
        .overflow_y_scroll()
        .p(px(16.))
        .when(!parts.before.trim().is_empty(), |column| {
            column.child(TextView::markdown(id("before"), parts.before.to_owned()))
        })
        .when_some(parts.description, |column, description| {
            column.child(
                div()
                    .my_2()
                    .pl_3()
                    .border_l_3()
                    .border_color(accent(Hue::Red, cx).color)
                    .text_color(cx.theme().muted_foreground)
                    .child(description.to_owned()),
            )
        })
        .child(TextView::markdown(id("after"), parts.after.to_owned()))
}
