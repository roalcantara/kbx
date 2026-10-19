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

/// The `↗ n` link count (spec 2026-10-19 D4): a round pill with a faint outline and no fill, the
/// same on every row, so it never reads as a key. Its digits are mono, so already tabular.
#[must_use]
pub fn link_chip(count: usize) -> Div {
    let d = design();
    let c = &d.components.chip;
    h_flex()
        .flex_none()
        .h(pt(c.height))
        .px(pt(d.spacing.chip.link_pad_x))
        .gap(pt(d.spacing.chip.link_gap))
        .items_center()
        .rounded_full()
        .border(pt(d.components.border.hairline))
        .border_color(d.colors.faint.opacity(c.link_outline_alpha))
        .text_color(d.colors.faint)
        .text_role(&d.typography.roles.count)
        .child(
            svg()
                .path("icons/arrow-up-right.svg")
                .size(pt(c.link_icon))
                .text_color(d.colors.faint),
        )
        .child(count.to_string())
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
