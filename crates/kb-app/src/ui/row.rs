//! D5's row (01-list): 56 tall, 4 apart, 6 pt corners; its lead (tile or favicon), title over
//! description, then whatever trails it; the selected row on `selected` with the 2.5 pt mauve bar
//! and its glow. The refresh report's rows are the same component (11-14-refresh).

use gpui_kit::base::Selectable;
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, Role, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};

use crate::ui::design::{TextRoleExt as _, design, glow, no_ligatures, pt};

/// A list row.
#[derive(IntoElement)]
pub struct Row {
    id: SharedString,
    label: SharedString,
    lead: Option<AnyElement>,
    title: SharedString,
    mono: bool,
    description: Option<SharedString>,
    trailing: Vec<AnyElement>,
    compact: bool,
    selected: bool,
}

impl std::fmt::Debug for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Row")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("mono", &self.mono)
            .field("compact", &self.compact)
            .field("selected", &self.selected)
            .finish_non_exhaustive()
    }
}

impl Row {
    /// A row `id` reading `title`.
    #[must_use]
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        let title = title.into();
        Self {
            id: id.into(),
            label: title.clone(),
            lead: None,
            title,
            mono: false,
            description: None,
            trailing: Vec::new(),
            compact: false,
            selected: false,
        }
    }

    /// What assistive technology reads for the row.
    #[must_use]
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// The tile or favicon before the title.
    #[must_use]
    pub fn lead(mut self, lead: impl IntoElement) -> Self {
        self.lead = Some(lead.into_any_element());
        self
    }

    /// Draws the title in `JetBrains` Mono (commands).
    #[must_use]
    pub fn mono(mut self, mono: bool) -> Self {
        self.mono = mono;
        self
    }

    /// The line under the title: one line, whatever line breaks the description has (a Skill's
    /// often lists its uses on lines of their own), cut with an ellipsis where it overflows.
    #[must_use]
    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = description.map(|d| one_line(&d).into());
        self
    }

    /// What follows the text, 12 apart: tags, the link chip, the keycap.
    #[must_use]
    pub fn trailing(mut self, trailing: Vec<AnyElement>) -> Self {
        self.trailing = trailing;
        self
    }

    /// Beside the split pane: the narrower inset of `04-entry-template.html`.
    #[must_use]
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
}

impl Selectable for Row {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl RenderOnce for Row {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let d = design();
        let (colors, spacing, size) = (&d.colors, &d.spacing.row, &d.components.row);
        let (pad_x, gap) = if self.compact {
            (spacing.split_pad_x, spacing.split_gap)
        } else {
            (spacing.pad_x, spacing.gap)
        };
        let mono = self.mono;
        div()
            .id(self.id)
            .w_full()
            // Half the list's gap above and below: rows sit 4 apart.
            .py(pt(d.spacing.list.gap / 2.))
            .role(Role::Label)
            .aria_label(self.label)
            .child(
                h_flex()
                    .relative()
                    .w_full()
                    .h(pt(size.height))
                    .items_center()
                    .gap(pt(gap))
                    .px(pt(pad_x))
                    .rounded(pt(d.rounded.row))
                    .map(|row| {
                        if self.selected {
                            row.bg(colors.selected).child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .top(pt(size.bar_inset))
                                    .bottom(pt(size.bar_inset))
                                    .w(pt(size.bar_width))
                                    .rounded_full()
                                    .bg(colors.mauve)
                                    .shadow(glow(colors.mauve, size.glow, size.glow_alpha)),
                            )
                        } else {
                            row.hover(|style| style.bg(colors.hover))
                        }
                    })
                    .children(self.lead)
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(pt(spacing.text_gap))
                            .child(
                                div()
                                    .truncate()
                                    .text_role(&d.typography.roles.title)
                                    .text_color(colors.text)
                                    .when(mono, |title| {
                                        title.font_family(d.typography.mono.clone()).font_features(no_ligatures())
                                    })
                                    .child(self.title),
                            )
                            .when_some(self.description, |text, description| {
                                text.child(
                                    div()
                                        .truncate()
                                        .text_role(&d.typography.roles.description)
                                        .text_color(colors.muted)
                                        .child(description),
                                )
                            }),
                    )
                    .child(
                        h_flex()
                            .flex_none()
                            .items_center()
                            .gap(pt(spacing.trailing_gap))
                            .children(self.trailing),
                    ),
            )
    }
}

/// `text` with every run of whitespace, line breaks included, as one space: `truncate` keeps a row
/// to one line only when the text has no line break of its own.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_description_on_one_line() {
        assert_eq!(
            one_line("Patterns for agents. Use when:\n- Building agents\r\n-  Auditing\tthem\n"),
            "Patterns for agents. Use when: - Building agents - Auditing them"
        );
        assert_eq!(one_line("Plain"), "Plain");
    }
}
