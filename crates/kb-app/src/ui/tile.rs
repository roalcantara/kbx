//! The type tile (01-list): the type's accent at 15 % behind its icon in the accent; a bookmark's
//! favicon in the same box, the tile when it fails to load.

use gpui_kit::{Div, Hsla, IntoElement as _, ParentElement as _, Styled as _, StyledImage as _, div, img, svg};

use crate::ui::design::{design, pt};

/// Where a tile sits, which sets its size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileSize {
    /// A list row: 24, icon 16.
    Row,
    /// The footer and the pane header: 20, icon 13.
    Small,
    /// The filter's rows: 18, icon 12.
    Mini,
}

impl TileSize {
    /// The box and the icon, in points.
    #[must_use]
    pub fn points(self) -> (f32, f32) {
        let t = &design().components.tile;
        match self {
            Self::Row => (t.size, t.icon),
            Self::Small => (t.small, t.small_icon),
            Self::Mini => (t.mini, t.mini_icon),
        }
    }
}

/// The tinted tile of `icon` (a Lucide name) in `tint`.
#[must_use]
pub fn tile(icon: &str, tint: Hsla, size: TileSize) -> Div {
    let (box_size, glyph) = size.points();
    div()
        .flex_none()
        .size(pt(box_size))
        .rounded(pt(design().rounded.tile))
        .flex()
        .items_center()
        .justify_center()
        .bg(tint.opacity(design().components.tile.alpha))
        .child(svg().path(format!("icons/{icon}.svg")).size(pt(glyph)).text_color(tint))
}

/// A bookmark's favicon from `url`, centred in the tile's box; the tile of `icon` if it fails.
#[must_use]
pub fn favicon(url: String, icon: String, tint: Hsla, size: TileSize) -> Div {
    let (box_size, _) = size.points();
    let tokens = design();
    div()
        .flex_none()
        .size(pt(box_size))
        .flex()
        .items_center()
        .justify_center()
        .child(
            img(url)
                .size(pt(tokens.components.favicon.size))
                .rounded(pt(tokens.rounded.favicon))
                .with_fallback(move || tile(&icon, tint, size).into_any_element()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_tiles_by_where_they_sit() {
        assert_eq!(TileSize::Row.points(), (24., 16.));
        assert_eq!(TileSize::Small.points(), (20., 13.));
        assert_eq!(TileSize::Mini.points(), (18., 12.));
    }
}
