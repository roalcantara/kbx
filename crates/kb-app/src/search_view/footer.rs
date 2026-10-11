//! The footer (DESIGN.md, Footer): two floating pills over the panel's bottom edge, the entry on
//! the left; the primary action and Actions on the right, one keycap per key.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Action, App, AsKeystroke as _, Context, IntoElement, KeyContext, Keystroke, ParentElement as _, Styled as _,
    Window, div, px,
};
use kb_domain::entry::Entry;

use super::{Screen, SearchView};
use crate::accents::accent;
use crate::keys::{CONTEXT, OpenActions, RunPrimary};
use crate::presentation::{primary_action, type_badge};

/// The keycaps of `keystroke`, one per key: modifiers in macOS order (⌃ ⌥ ⇧ ⌘), then the key.
#[must_use]
pub fn keycaps(keystroke: &Keystroke) -> Vec<String> {
    let modifiers = keystroke.modifiers;
    let mut caps: Vec<String> = [
        (modifiers.control, "⌃"),
        (modifiers.alt, "⌥"),
        (modifiers.shift, "⇧"),
        (modifiers.platform, "⌘"),
    ]
    .into_iter()
    .filter(|(held, _)| *held)
    .map(|(_, cap)| cap.to_owned())
    .collect();
    caps.push(match keystroke.key.as_str() {
        "enter" => "↵".to_owned(),
        "escape" => "esc".to_owned(),
        "right" => "→".to_owned(),
        "left" => "←".to_owned(),
        "up" => "↑".to_owned(),
        "down" => "↓".to_owned(),
        "backspace" => "⌫".to_owned(),
        "tab" => "⇥".to_owned(),
        "space" => "␣".to_owned(),
        key => key.to_uppercase(),
    });
    caps
}

/// The keycaps of `action`'s binding in the panel, if it has one.
pub fn action_keycaps(action: &dyn Action, window: &Window) -> Vec<String> {
    KeyContext::parse(CONTEXT)
        .ok()
        .and_then(|context| window.highest_precedence_binding_for_action_in_context(action, context))
        .and_then(|binding| binding.keystrokes().first().map(|key| keycaps(key.as_keystroke())))
        .unwrap_or_default()
}

/// One keycap per key, as Raycast draws them.
pub fn keycap_row(caps: Vec<String>, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    h_flex().gap(px(4.)).children(caps.into_iter().map(|cap| {
        div()
            .min_w(px(18.))
            .h(px(18.))
            .px(px(4.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(4.))
            .bg(theme.list_active)
            .text_color(theme.foreground)
            .text_xs()
            .child(cap)
    }))
}

/// A floating pill.
fn pill(cx: &App) -> gpui_kit::Div {
    let theme = cx.theme();
    h_flex()
        .h(px(30.))
        .items_center()
        .gap(px(8.))
        .px(px(12.))
        .rounded_full()
        .bg(theme.popover)
        .border_1()
        .border_color(theme.border)
        .shadow_sm()
        .text_xs()
}

impl SearchView {
    /// The footer: the entry on the left; the primary action and Actions on the right.
    pub(super) fn render_footer(
        &self,
        selected: Option<&Entry>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let (icon, tint, context) = match (&self.screen, selected) {
            (Screen::Rebuild(_), _) => (
                IconName::RefreshCw,
                theme.muted_foreground,
                "Rebuilding the Index…".to_owned(),
            ),
            (Screen::Detail, Some(entry)) => {
                let badge = type_badge(&entry.entry_type);
                (badge.icon, accent(badge.hue, cx).color, entry.title.clone())
            }
            _ => (IconName::BookOpen, theme.red, "Search Knowledge Base".to_owned()),
        };
        let primary_label = if self.is_showing_detail() {
            selected.map_or("Run", |e| primary_action(e).title)
        } else {
            "Show Details"
        };
        let enter = vec!["↵".to_owned()];
        let actions = action_keycaps(&OpenActions, window);
        h_flex()
            .w_full()
            .flex_none()
            .items_center()
            .justify_between()
            .gap(px(8.))
            .p(px(8.))
            .child(
                pill(cx)
                    .min_w_0()
                    .child(Icon::new(icon).small().text_color(tint))
                    .child(div().truncate().max_w(px(320.)).child(context)),
            )
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(
                        pill(cx).child(
                            Button::new("footer-primary-action")
                                .ghost()
                                .xsmall()
                                .label(primary_label)
                                .child(keycap_row(enter, cx))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    if this.is_showing_detail() {
                                        this.run_primary(&RunPrimary, window, cx);
                                    } else {
                                        this.show_detail_screen(window, cx);
                                    }
                                })),
                        ),
                    )
                    .child(
                        pill(cx).child(
                            Button::new("footer-open-actions")
                                .ghost()
                                .xsmall()
                                .label("Actions")
                                .when(!actions.is_empty(), |button| button.child(keycap_row(actions, cx)))
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.open_actions(&OpenActions, window, cx)),
                                ),
                        ),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caps(binding: &str) -> Vec<String> {
        keycaps(&Keystroke::parse(binding).unwrap())
    }

    #[test]
    fn draws_one_keycap_per_key() {
        assert_eq!(caps("cmd-k"), ["⌘", "K"]);
        assert_eq!(caps("cmd-enter"), ["⌘", "↵"]);
        assert_eq!(caps("cmd-shift-,"), ["⇧", "⌘", ","]);
        assert_eq!(caps("right"), ["→"]);
        assert_eq!(caps("escape"), ["esc"]);
    }
}
