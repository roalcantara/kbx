//! D17's notice (01-list): bottom right above the footer, through GPUI Kit's Notification. The
//! copy notice reads _Copied to clipboard_ with a `⌘V` hint. Its restyle is parked (TODO MOB4).

use gpui_kit::base::Edges;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Theme, h_flex};
use gpui_kit::{Anchor, IntoElement as _, ParentElement as _, Styled as _, div, svg};

use crate::catalog::Messages;
use crate::ui::design::{TextRoleExt as _, design, pixels, pt};
use crate::ui::keycap::{ChipState, keycap};

/// The copy notice's id: a new copy replaces the one on screen.
#[derive(Debug)]
pub struct CopyNotice;

/// Puts every notice bottom right, above the footer (D17).
pub fn configure(theme: &mut Theme) {
    let d = design();
    let s = &d.spacing.notice;
    theme.notification.placement = Anchor::BottomRight;
    theme.notification.margins = Edges {
        // The window's content edge, so the notice lines up with the footer.
        top: pixels(d.content_inset()),
        right: pixels(d.content_inset()),
        bottom: pixels(s.bottom),
        left: pixels(d.content_inset()),
    };
    theme.notification.width = pixels(d.components.notice.width);
}

/// _Copied to clipboard_, a check in a green ring, and the `⌘V to paste` hint on a rest keycap.
#[must_use]
pub fn copied(messages: &Messages) -> Notification {
    let (message, hint) = (messages.copied.clone(), messages.paste_hint.clone());
    Notification::new().id::<CopyNotice>().content(move |_, _, _| {
        let d = design();
        let (c, n) = (&d.colors, &d.components.notice);
        h_flex()
            .items_center()
            .gap(pt(d.spacing.notice.gap))
            .text_role(&d.typography.roles.notice)
            .text_color(c.text)
            .child(
                div()
                    .size(pt(n.icon))
                    .flex_none()
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(c.green.opacity(n.ring_fill))
                    .border_1()
                    .border_color(c.green.opacity(n.ring_border))
                    .child(svg().path("icons/check.svg").size(pt(n.check)).text_color(c.green)),
            )
            .child(message.clone())
            .child(
                h_flex()
                    .items_center()
                    .gap(pt(d.spacing.notice.hint_gap))
                    .text_role(&d.typography.roles.description)
                    .text_color(c.muted)
                    .child(keycap("⌘V", ChipState::Rest))
                    .child(hint.clone()),
            )
            .into_any_element()
    })
}
