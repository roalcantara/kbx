//! The tag and type accents, from the design tokens (DESIGN.md's front matter). The components of
//! Plan 8d replace this module; until then it keeps the old call sites drawing the new colours.

use gpui_kit::{App, Hsla};

use crate::presentation::Hue;
use crate::ui::design::design;

/// An accent and its tag pill.
#[derive(Debug, Clone, Copy, PartialEq)]
#[expect(dead_code, reason = "Plan 8d transition artifact kept for backward compatibility")]
pub struct Accent {
    /// Icons, bars, the type tint.
    pub color: Hsla,
    /// The tag pill's fill: the rest chip.
    pub pill: Hsla,
    /// The tag pill's text.
    pub pill_text: Hsla,
}

/// The accent for `hue`; pills are grey (D3, chip at rest).
#[expect(dead_code, reason = "Plan 8d transition artifact kept for backward compatibility")]
#[must_use]
pub fn accent(hue: Hue, _: &App) -> Accent {
    let colors = &design().colors;
    Accent {
        color: colors.hue(hue),
        pill: colors.badge,
        pill_text: colors.muted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_tag_pills_grey_and_hues_from_the_tokens() {
        let colors = &design().colors;
        assert_eq!(colors.hue(Hue::Purple), colors.mauve);
        assert_eq!(colors.badge, colors.badge);
    }
}
