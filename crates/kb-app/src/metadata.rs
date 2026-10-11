//! The metadata card (DESIGN.md, Metadata card): alternate rows a shade lighter, the label left
//! and the value right; Type and Tags as pills, the URL with ↗, the source with a file icon.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::tag::Tag;
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{App, IntoElement, ParentElement as _, SharedString, Styled as _, div, px};
use kb_domain::entry::Entry;

use crate::accents::accent;
use crate::presentation::{Hue, metadata, tag_hue, type_badge};

/// A value in the card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaValue {
    /// Plain text.
    Text(String),
    /// Pills, each with its accent.
    Pills(Vec<(String, Hue)>),
    /// A URL, opened by clicking it.
    Link(String),
    /// The source file's path.
    File(String),
}

/// One card row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaRow {
    /// The label.
    pub label: &'static str,
    /// The value.
    pub value: MetaValue,
}

/// The card's rows, in `presentation::metadata`'s order.
#[must_use]
pub fn metadata_rows(entry: &Entry) -> Vec<MetaRow> {
    metadata(entry)
        .into_iter()
        .map(|(label, value)| MetaRow {
            label,
            value: match label {
                "URL" => MetaValue::Link(value),
                "Type" => MetaValue::Pills(vec![(value, type_badge(&entry.entry_type).hue)]),
                "Tags" => MetaValue::Pills(entry.tags.iter().map(|t| (format!("#{t}"), tag_hue(t))).collect()),
                "Source" => MetaValue::File(value),
                _ => MetaValue::Text(value),
            },
        })
        .collect()
}

/// The card for `entry`.
pub fn metadata_card(entry: &Entry, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    v_flex()
        .w_full()
        .flex_none()
        .rounded(px(10.))
        .border_1()
        .border_color(theme.border)
        .overflow_hidden()
        .text_xs()
        .children(metadata_rows(entry).into_iter().enumerate().map(|(ix, row)| {
            h_flex()
                .w_full()
                .items_start()
                .justify_between()
                .gap(px(12.))
                .px(px(12.))
                .py(px(6.))
                // Alternate rows a shade lighter, as Raycast's card.
                .when(ix % 2 == 1, |row| row.bg(theme.muted.opacity(0.5)))
                .child(div().flex_none().text_color(theme.muted_foreground).child(row.label))
                .child(
                    h_flex()
                        .flex_1()
                        .min_w_0()
                        .justify_end()
                        .flex_wrap()
                        .gap_1()
                        .text_right()
                        .child(render_value(row.label, row.value, cx)),
                )
        }))
}

fn render_value(label: &'static str, value: MetaValue, cx: &App) -> gpui_kit::AnyElement {
    match value {
        MetaValue::Text(text) => div().child(text).into_any_element(),
        MetaValue::Pills(pills) => h_flex()
            .flex_wrap()
            .justify_end()
            .gap_1()
            .children(pills.into_iter().map(|(text, hue)| {
                let accent = accent(hue, cx);
                Tag::custom(accent.pill, accent.pill_text, accent.pill)
                    .xsmall()
                    .rounded(px(10.))
                    .child(text)
            }))
            .into_any_element(),
        MetaValue::Link(url) => Button::new(SharedString::from(format!("metadata-{label}")))
            .link()
            .xsmall()
            .label(url.clone())
            .icon(IconName::ArrowUpRight)
            .on_click(move |_, _, cx| cx.open_url(&url))
            .into_any_element(),
        MetaValue::File(path) => h_flex()
            .gap_1()
            .items_center()
            .child(Icon::new(IconName::File).xsmall())
            .child(path)
            .into_any_element(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::entry;

    #[test]
    fn marks_pills_links_and_the_source_in_the_card() {
        let bookmark = entry("bookmarks", "https://b.example", "tags: [git]\n");
        let rows = metadata_rows(&bookmark);
        assert_eq!(
            rows[0],
            MetaRow {
                label: "URL",
                value: MetaValue::Link("https://b.example".into())
            }
        );
        assert_eq!(rows[1].value, MetaValue::Pills(vec![("Bookmarks".into(), Hue::Purple)]));
        assert_eq!(rows[2].value, MetaValue::Pills(vec![("#git".into(), tag_hue("git"))]));
        assert_eq!(rows.last().map(|r| &r.value), Some(&MetaValue::File("f.yml".into())));
    }
}
