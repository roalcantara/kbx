//! D3's chips (01-list, 01-chips), 20 tall with 4 pt corners: tag pills, tag icon chips, the
//! `↗ n` link chip and text chips (a task's status and priority).

use gpui_kit::component::h_flex;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::{
    Div, Hsla, InteractiveElement as _, IntoElement as _, ParentElement as _, SharedString, Stateful,
    StatefulInteractiveElement as _, Styled as _, div, img, svg,
};

use crate::tag_images::TagImage;
use crate::ui::design::{TextRoleExt as _, design, pt};
use crate::ui::keycap::ChipState;

/// A text chip: `#tag`, `+2`, a task's status. Its text is muted in both states.
#[must_use]
pub fn pill(text: impl Into<SharedString>, state: ChipState) -> Div {
    let d = design();
    div()
        .flex_none()
        .h(pt(d.components.chip.height))
        .px(pt(d.spacing.chip.pad_x))
        .flex()
        .items_center()
        .rounded(pt(d.rounded.chip))
        .bg(state.fill())
        .text_color(d.colors.muted)
        .text_role(&d.typography.roles.chip)
        .child(text.into())
}

/// The `↗ n` link chip: its text muted when live, faint at rest.
#[must_use]
pub fn link_chip(count: usize, state: ChipState) -> Div {
    let d = design();
    let color = link_text(state);
    h_flex()
        .flex_none()
        .h(pt(d.components.chip.height))
        .px(pt(d.spacing.chip.link_pad_x))
        .gap(pt(d.spacing.chip.link_gap))
        .items_center()
        .rounded(pt(d.rounded.chip))
        .bg(state.fill())
        .text_color(color)
        .text_role(&d.typography.roles.count)
        .child(
            svg()
                .path("icons/arrow-up-right.svg")
                .size(pt(d.components.chip.link_icon))
                .text_color(color),
        )
        .child(count.to_string())
}

/// The link chip's text colour in `state`.
#[must_use]
pub fn link_text(state: ChipState) -> Hsla {
    let colors = &design().colors;
    match state {
        ChipState::Live => colors.muted,
        ChipState::Rest => colors.faint,
    }
}

/// A tag drawn as its image: a shape takes the tag's hue, a brand keeps its colours; its name in
/// a tooltip, since the image carries no text.
#[must_use]
pub fn icon_chip(id: SharedString, image: &TagImage, hue: Hsla, state: ChipState, name: String) -> Stateful<Div> {
    let d = design();
    let glyph = pt(d.components.chip.icon);
    let drawn = if image.coloured {
        img(image.path.clone()).size(glyph).into_any_element()
    } else {
        svg()
            .external_path(image.path.to_string_lossy().into_owned())
            .size(glyph)
            .text_color(hue)
            .into_any_element()
    };
    div()
        .id(id)
        .flex_none()
        .size(pt(d.components.chip.height))
        .flex()
        .items_center()
        .justify_center()
        .rounded(pt(d.rounded.chip))
        .bg(state.fill())
        .child(drawn)
        .tooltip(move |window, cx| Tooltip::new(name.clone()).build(window, cx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dims_the_link_count_at_rest_only() {
        let colors = &design().colors;
        assert_eq!(link_text(ChipState::Live), colors.muted);
        assert_eq!(link_text(ChipState::Rest), colors.faint);
    }
}
