//! The one way kbx says "back to X": a left chevron and a name. ⌘K's submenus use it in their
//! crumb row (`‹ More`), and every pushed place uses it in the header (`‹ /k8s`); only the size differs.

use gpui_kit::{Div, ParentElement as _, SharedString, Styled as _, component::h_flex, div, svg};

use crate::ui::design::{TextRoleExt as _, design, pt};

/// Where the crumb sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrumbSize {
    /// A submenu's crumb row in the action panel.
    Panel,
    /// The header of a pushed place.
    Header,
}

/// `‹ name`; `mono` for a query.
#[must_use]
pub fn crumb(name: impl Into<SharedString>, mono: bool, size: CrumbSize) -> Div {
    let d = design();
    let (icon, gap, role) = match size {
        CrumbSize::Panel => (
            d.components.panel.crumb_icon,
            d.spacing.panel.crumb_gap,
            &d.typography.roles.label,
        ),
        CrumbSize::Header => (
            d.components.header.icon,
            d.spacing.header.gap,
            &d.typography.roles.search,
        ),
    };
    let label = div().min_w_0().truncate().child(name.into());
    let label = if mono {
        label.font_family(d.typography.mono.clone()).text_color(d.colors.muted)
    } else {
        label.text_color(d.colors.faint)
    };
    h_flex()
        .min_w_0()
        .items_center()
        .gap(pt(gap))
        .text_role(role)
        .child(
            svg()
                .path("icons/chevron-left.svg")
                .size(pt(icon))
                .flex_none()
                .text_color(d.colors.faint),
        )
        .child(label)
}
