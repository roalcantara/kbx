//! Keycaps (spec 2026-10-19 D1): one 20 × 20 cap per key (`⌘` `K`), 2 apart, Inter 10; a word key
//! (`esc`) 28 wide.

use gpui_kit::component::h_flex;
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

/// The modifier glyphs `caps` puts before the key.
const MODIFIERS: [char; 4] = ['⌃', '⌥', '⇧', '⌘'];

/// A shortcut as `caps` spells it, split into its keys: each leading modifier is a key, the rest
/// is one key (`⇧⌘,` → `⇧` `⌘` `,`; `esc` → `esc`).
#[must_use]
pub fn keys(text: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut rest = String::new();
    for ch in text.chars() {
        if rest.is_empty() && MODIFIERS.contains(&ch) {
            keys.push(ch.to_string());
        } else {
            rest.push(ch);
        }
    }
    if !rest.is_empty() {
        keys.push(rest);
    }
    keys
}

/// One key's cap width: a glyph's, or a word's (`esc`).
fn cap_width(key: &str) -> f32 {
    let k = &design().components.keycap;
    if key.chars().count() > 1 { k.word_width } else { k.width }
}

/// The width `keycap(text)` draws: its caps and the gaps between them (the tests check slots with it).
#[cfg(test)]
#[must_use]
pub(crate) fn keycap_width(text: &str) -> f32 {
    let keys = keys(text);
    #[expect(clippy::cast_precision_loss, reason = "a shortcut has at most five keys")]
    let gaps = keys.len().saturating_sub(1) as f32 * design().spacing.keycap.gap;
    keys.iter().map(|key| cap_width(key)).sum::<f32>() + gaps
}

/// A shortcut's keycaps: one cap per key, 2 apart. Live (mantle, muted text) or, on the popover
/// and notice surfaces, rest (badge, faint text).
#[must_use]
pub fn keycap(text: impl Into<SharedString>, state: ChipState) -> Div {
    let d = design();
    let text_color = match state {
        ChipState::Live => d.colors.muted,
        ChipState::Rest => d.colors.faint,
    };
    let text: SharedString = text.into();
    h_flex()
        .flex_none()
        .items_center()
        .gap(pt(d.spacing.keycap.gap))
        .children(keys(&text).into_iter().map(|key| {
            div()
                .flex_none()
                .h(pt(d.components.keycap.height))
                .w(pt(cap_width(&key)))
                .flex()
                .items_center()
                .justify_center()
                .rounded(pt(d.rounded.chip))
                .bg(state.fill())
                .text_color(text_color)
                .text_role(&d.typography.roles.keycap)
                .child(key)
        }))
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
    fn splits_a_shortcut_into_its_keys() {
        assert_eq!(keys("⌘↵"), ["⌘", "↵"]);
        assert_eq!(keys("⇧⌘,"), ["⇧", "⌘", ","]);
        assert_eq!(keys("⌥⌘C"), ["⌥", "⌘", "C"]);
        assert_eq!(keys("⌘1"), ["⌘", "1"]);
        assert_eq!(keys("↵"), ["↵"]);
        assert_eq!(keys("esc"), ["esc"]);
    }

    #[test]
    fn gives_every_key_the_same_cap() {
        let k = &design().components.keycap;
        let gap = design().spacing.keycap.gap;
        for one in ["↵", "⌘", "K", "3", ","] {
            assert!((keycap_width(one) - k.width).abs() < f32::EPSILON, "{one}");
        }
        for two in ["⌘↵", "⌘1", "⌘K", "⌘P"] {
            assert!((keycap_width(two) - (2. * k.width + gap)).abs() < f32::EPSILON, "{two}");
        }
        assert!((keycap_width("esc") - k.word_width).abs() < f32::EPSILON);
        assert!((keycap_width("⇧⌘↵") - (3. * k.width + 2. * gap)).abs() < f32::EPSILON);
    }
}
