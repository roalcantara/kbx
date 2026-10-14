//! D10's empty view (03-empty, 11-14-refresh): one large icon, a title, a sentence whose query
//! (if any) reads in mauve mono, and an optional extra (the refresh's progress bar). Six states
//! use it: no match, no entries, no index, refreshing, stopped, failed.

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{AnyElement, App, Hsla, IntoElement, ParentElement as _, RenderOnce, Styled as _, Window, div, svg};

use crate::ui::design::{TextRoleExt as _, design, no_ligatures, pt};

/// A sentence with an optional highlighted part: `before`, `query`, `after`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    /// The text before the query.
    pub before: String,
    /// The query, drawn in mauve mono; empty for none.
    pub query: String,
    /// The text after it.
    pub after: String,
}

impl Sentence {
    /// `template` with `{query}` replaced by `query`, split around it; without `{query}`, plain.
    #[must_use]
    pub fn of(template: &str, query: &str) -> Self {
        match template.split_once("{query}") {
            Some((before, after)) => Self {
                before: before.to_owned(),
                query: query.to_owned(),
                after: after.to_owned(),
            },
            None => Self {
                before: template.to_owned(),
                query: String::new(),
                after: String::new(),
            },
        }
    }
}

/// The empty view.
#[derive(IntoElement)]
pub struct EmptyView {
    icon: String,
    icon_color: Hsla,
    title: String,
    sentence: Sentence,
    extra: Option<AnyElement>,
}

impl EmptyView {
    /// `icon` (a Lucide name) in `icon_color`, `title`, and `sentence`.
    #[must_use]
    pub fn new(icon: &str, icon_color: Hsla, title: impl Into<String>, sentence: Sentence) -> Self {
        Self {
            icon: icon.to_owned(),
            icon_color,
            title: title.into(),
            sentence,
            extra: None,
        }
    }

    /// Something under the sentence: the refresh's progress bar.
    #[expect(dead_code, reason = "Plan 8g Task 3 wires EmptyView::extra into SyncView")]
    #[must_use]
    pub fn extra(mut self, extra: impl IntoElement) -> Self {
        self.extra = Some(extra.into_any_element());
        self
    }
}

impl RenderOnce for EmptyView {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let d = design();
        let (c, s, k) = (&d.colors, &d.spacing.empty, &d.components.empty);
        let sentence = self.sentence;
        v_flex()
            .size_full()
            .flex_1()
            .items_center()
            .justify_center()
            .px(pt(s.pad_x))
            .child(
                svg()
                    .path(format!("icons/{}.svg", self.icon))
                    .size(pt(k.icon))
                    .text_color(self.icon_color)
                    .mb(pt(s.icon_gap)),
            )
            .child(
                div()
                    .text_role(&d.typography.roles.empty_title)
                    .text_color(c.text)
                    .mb(pt(s.title_gap))
                    .child(self.title),
            )
            .child(
                h_flex()
                    .flex_wrap()
                    .justify_center()
                    .max_w(pt(k.text_max_width))
                    .text_role(&d.typography.roles.empty_text)
                    .text_color(c.muted)
                    .child(sentence.before)
                    .when(!sentence.query.is_empty(), |line| {
                        line.child(
                            div()
                                .font_family(d.typography.mono.clone())
                                .font_features(no_ligatures())
                                .text_color(c.mauve)
                                .child(sentence.query),
                        )
                    })
                    .child(sentence.after),
            )
            .when_some(self.extra, |view, extra| {
                view.child(div().mt(pt(s.bar_gap)).child(extra))
            })
    }
}

/// The refresh's thin progress bar, `fraction` (0–1) filled.
#[expect(dead_code, reason = "Plan 8g Task 3 wires progress_bar into SyncView")]
#[must_use]
pub fn progress_bar(fraction: f32) -> impl IntoElement {
    let d = design();
    let k = &d.components.empty;
    div()
        .w(pt(k.bar_width))
        .h(pt(k.bar_height))
        .rounded_full()
        .bg(d.colors.selected)
        .overflow_hidden()
        .child(
            div()
                .h_full()
                .w(pt(k.bar_width * fraction.clamp(0., 1.)))
                .bg(d.colors.blue),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_a_message_around_its_query() {
        assert_eq!(
            Sentence::of("No entries match “{query}”.", "/terraform-plan"),
            Sentence {
                before: "No entries match “".into(),
                query: "/terraform-plan".into(),
                after: "”.".into()
            }
        );
        assert_eq!(
            Sentence::of("The previous index is unchanged.", "x"),
            Sentence {
                before: "The previous index is unchanged.".into(),
                query: String::new(),
                after: String::new()
            }
        );
    }
}
