//! D4's header (01-list, 07-filter): 56 tall, a hairline under it; the magnifier on the list, the
//! back chevron elsewhere; the query at 15; the filter pill (sliders, the filter's name, `⌘P`).

use gpui_kit::component::h_flex;
use gpui_kit::{Div, ParentElement as _, SharedString, Styled as _, div, svg};

use crate::ui::design::{TextRoleExt as _, design, pt};
use crate::ui::keycap::{ChipState, keycap};

/// The header's row; the caller adds the lead icon, the query and the pill.
#[must_use]
pub fn header() -> Div {
    let d = design();
    h_flex()
        .h(pt(d.components.header.height))
        .w_full()
        .flex_none()
        .items_center()
        .px(pt(d.spacing.header.pad_x))
        .gap(pt(d.spacing.header.gap))
        .border_b(pt(d.components.border.hairline))
        .border_color(d.colors.edge)
}

/// The lead icon (`search` on the list, `chevron-left` elsewhere), 20 faint.
#[must_use]
pub fn lead_icon(icon: &str) -> Div {
    let d = design();
    div().flex_none().child(
        svg()
            .path(format!("icons/{icon}.svg"))
            .size(pt(d.components.header.icon))
            .text_color(d.colors.faint),
    )
}

/// The query field's box: 15 pt text, the rest of the row.
#[must_use]
pub fn query_box() -> Div {
    let d = design();
    div()
        .flex_1()
        .min_w_0()
        .text_role(&d.typography.roles.search)
        .text_color(d.colors.text)
}

/// The filter pill: sliders, `label` (truncated), and the filter's keycap.
#[must_use]
pub fn filter_pill(label: impl Into<SharedString>, keys_caps: impl Into<SharedString>) -> Div {
    let d = design();
    h_flex()
        .flex_none()
        .items_center()
        .gap(pt(d.spacing.pill.gap))
        .px(pt(d.spacing.pill.pad_x))
        .py(pt(d.spacing.pill.pad_y))
        .rounded(pt(d.rounded.pill))
        .bg(d.colors.badge)
        .text_color(d.colors.muted)
        .text_role(&d.typography.roles.label)
        .child(
            svg()
                .path("icons/sliders-horizontal.svg")
                .size(pt(d.components.header.pill_icon))
                .text_color(d.colors.faint),
        )
        .child(
            div()
                .max_w(pt(d.components.header.pill_max_width))
                .truncate()
                .child(label.into()),
        )
        .child(keycap(keys_caps, ChipState::Live))
}
