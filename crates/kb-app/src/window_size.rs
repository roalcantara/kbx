//! The panel's two sizes (DESIGN.md, Layout): compact, and expanded to most of the display.
//! ⌘⇧F switches between them; the size holds until the app quits.

use gpui_kit::{Pixels, Size, size};

use crate::ui::design::{design, pixels};

/// The compact size, in points (DESIGN.md, `components.window`).
#[must_use]
pub fn compact() -> Size<Pixels> {
    let w = &design().components.window;
    size(pixels(w.width), pixels(w.height))
}

/// The panel's size.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WindowSize {
    /// [`compact()`].
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
        let compact = compact();
        let expanded_share = design().components.window.expanded_share;
        match self {
            Self::Compact => compact,
            Self::Expanded => size(
                (display.width * expanded_share).max(compact.width),
                (display.height * expanded_share).max(compact.height),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_to_85_percent_of_the_display_and_back() {
        let display = size(pixels(1800.), pixels(1100.));
        assert_eq!(
            WindowSize::Compact.toggled().size(display),
            size(pixels(1530.), pixels(935.))
        );
        assert_eq!(WindowSize::Expanded.toggled().size(display), compact());
    }

    #[test]
    fn never_expands_below_the_compact_size() {
        assert_eq!(WindowSize::Expanded.size(size(pixels(900.), pixels(560.))), compact());
    }
}
