//! D2's keycap (01-chips): one cap per shortcut (`⌘K`, `⇧⌘,`), 20 tall, `JetBrains` Mono 11.

use gpui_kit::{Div, Hsla, Keystroke, ParentElement as _, SharedString, Styled as _, div};

use crate::ui::design::{TextRoleExt as _, design, pt};

/// A chip's or keycap's state (D3): live on every keycap and on the selected row's chips; rest
/// elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipState {
    /// Mantle fill, muted text.
    Live,
    /// Grey (badge) fill.
    Rest,
}

impl ChipState {
    /// `Live` on the selected row, `Rest` elsewhere.
    #[must_use]
    pub fn of(selected: bool) -> Self {
        if selected { Self::Live } else { Self::Rest }
    }

    /// The fill.
    #[must_use]
    pub fn fill(self) -> Hsla {
        let colors = &design().colors;
        match self {
            Self::Live => colors.mantle,
            Self::Rest => colors.badge,
        }
    }
}

/// GPUI's key spelling as one keycap's text: modifiers in macOS order (⌃ ⌥ ⇧ ⌘), then the key.
#[must_use]
pub fn caps(keys: &str) -> String {
    let Ok(keystroke) = Keystroke::parse(keys) else {
        return keys.to_owned();
    };
    let m = keystroke.modifiers;
    let mut text: String = [(m.control, "⌃"), (m.alt, "⌥"), (m.shift, "⇧"), (m.platform, "⌘")]
        .into_iter()
        .filter_map(|(held, cap)| held.then_some(cap))
        .collect();
    text.push_str(&match keystroke.key.as_str() {
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
    text
}

/// The visual width bucket for a keycap: normalize all shortcuts to a few widths so rows stay
/// aligned even when labels differ in glyph count.
#[must_use]
pub fn keycap_width(text: &str) -> f32 {
    match text.chars().count() {
        1 => 28.0,
        2 => 36.0,
        3 => 42.0,
        _ => 48.0,
    }
}

/// A keycap reading `text`: live (mantle, muted text) or, on the popover and notice surfaces,
/// rest (badge, faint text).
#[must_use]
pub fn keycap(text: impl Into<SharedString>, state: ChipState) -> Div {
    let d = design();
    let text_color = match state {
        ChipState::Live => d.colors.muted,
        ChipState::Rest => d.colors.faint,
    };
    let text = text.into();
    div()
        .flex_none()
        .h(pt(d.components.keycap.height))
        .w(pt(keycap_width(text.as_ref())))
        .px(pt(d.spacing.keycap.pad_x))
        .flex()
        .items_center()
        .justify_center()
        .rounded(pt(d.rounded.chip))
        .bg(state.fill())
        .text_color(text_color)
        .text_role(&d.typography.roles.keycap)
        .child(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_one_cap_per_shortcut_in_macos_order() {
        assert_eq!(caps("cmd-k"), "⌘K");
        assert_eq!(caps("shift-cmd-enter"), "⇧⌘↵");
        assert_eq!(caps("cmd-shift-,"), "⇧⌘,");
        assert_eq!(caps("cmd-alt-c"), "⌥⌘C");
        assert_eq!(caps("escape"), "esc");
        assert_eq!(caps("cmd-1"), "⌘1");
    }

    #[test]
    fn fills_live_with_mantle_and_rest_with_the_badge_grey() {
        let colors = &design().colors;
        assert_eq!(ChipState::of(true).fill(), colors.mantle);
        assert_eq!(ChipState::of(false).fill(), colors.badge);
    }

    #[test]
    fn normalizes_keycap_width_by_visual_bucket() {
        assert_eq!(keycap_width("⌘"), 28.0);
        assert_eq!(keycap_width("⌘1"), 36.0);
        assert_eq!(keycap_width("⌘K"), 36.0);
        assert_eq!(keycap_width("⇧⌘↵"), 42.0);
        assert_eq!(keycap_width("esc"), 42.0);
    }
}
