//! The panel's two sizes (DESIGN.md, Layout): compact, and expanded to most of the display.
//! ⌘⇧F switches between them; the size holds until the app quits.

use gpui_kit::{Pixels, Size, px, size};

/// The compact size, in points. Raycast's is 750 × 475; this one is larger so the split pane
/// leaves the notes room.
pub const COMPACT: Size<Pixels> = Size {
    width: px(860.),
    height: px(520.),
};

/// The share of the display's visible area the expanded panel takes.
const EXPANDED_SHARE: f32 = 0.85;

/// The panel's size.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WindowSize {
    /// [`COMPACT`].
    #[default]
    Compact,
    /// 85% of the display's visible area, never smaller than compact.
    Expanded,
}

impl WindowSize {
    /// The other size.
    #[must_use]
    pub fn toggled(self) -> Self {
        match self {
            Self::Compact => Self::Expanded,
            Self::Expanded => Self::Compact,
        }
    }

    /// The window size on a display whose visible area is `display`.
    #[must_use]
    pub fn size(self, display: Size<Pixels>) -> Size<Pixels> {
        match self {
            Self::Compact => COMPACT,
            Self::Expanded => size(
                (display.width * EXPANDED_SHARE).max(COMPACT.width),
                (display.height * EXPANDED_SHARE).max(COMPACT.height),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_to_85_percent_of_the_display_and_back() {
        let display = size(px(1800.), px(1100.));
        assert_eq!(WindowSize::Compact.toggled().size(display), size(px(1530.), px(935.)));
        assert_eq!(WindowSize::Expanded.toggled().size(display), COMPACT);
    }

    #[test]
    fn never_expands_below_the_compact_size() {
        assert_eq!(WindowSize::Expanded.size(size(px(900.), px(560.))), COMPACT);
    }
}
