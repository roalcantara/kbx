//! The design tokens: DESIGN.md's front matter, parsed once at start into typed values. The only
//! place a token becomes a GPUI length or colour.

use std::sync::{Arc, LazyLock};

use gpui_kit::{BoxShadow, FontFeatures, FontWeight, Hsla, Pixels, Rems, Styled, point, px, rems, rgb, rgba};
use serde::Deserialize;

use crate::presentation::Hue;

/// DESIGN.md, compiled in.
const DESIGN_MD: &str = include_str!("../../../../DESIGN.md");

/// Points per rem: GPUI Kit's base font size, which kb keeps at its default.
pub const POINTS_PER_REM: f32 = 16.;

/// `points` as rems, so every size follows the base font (R9).
#[must_use]
pub fn pt(points: f32) -> Rems {
    rems(points / POINTS_PER_REM)
}

/// `points` as GPUI pixels, for the few APIs that take pixels (shadows, window frames, notice
/// margins). One point is one pixel at the base font size.
#[must_use]
pub fn pixels(points: f32) -> Pixels {
    px(points)
}

/// A glow around an element: `color` at `alpha`, blurred `blur` points (the selection bar, the
/// ready dot).
#[must_use]
pub fn glow(color: Hsla, blur: f32, alpha: f32) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: color.opacity(alpha),
        offset: point(px(0.), px(0.)),
        blur_radius: px(blur),
        spread_radius: px(0.),
        inset: false,
    }]
}

/// Why the front matter could not be read.
#[derive(Debug, thiserror::Error)]
pub enum DesignError {
    /// DESIGN.md does not start with a `---` block.
    #[error("DESIGN.md has no front matter")]
    NoFrontMatter,
    /// The block is not YAML.
    #[error("DESIGN.md's front matter is not YAML: {0}")]
    Yaml(String),
    /// The block does not have the expected tokens.
    #[error("DESIGN.md's front matter: {0}")]
    Shape(String),
}

/// The tokens, parsed once. The front matter is compiled in and a test parses it, so a failure
/// here is a broken build, not a user's mistake.
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "the compiled-in front matter is checked by a unit test"
)]
pub fn design() -> &'static Design {
    static DESIGN: LazyLock<Design> = LazyLock::new(|| parse(DESIGN_MD).expect("DESIGN.md's front matter is valid"));
    &DESIGN
}

/// A colour token: `#RRGGBB` or `#RRGGBBAA`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub Hsla);

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        hex(&text)
            .map(Color)
            .ok_or_else(|| serde::de::Error::custom(format!("{text} is not #RRGGBB or #RRGGBBAA")))
    }
}

/// `#RRGGBB` or `#RRGGBBAA` as a colour.
#[must_use]
pub fn hex(text: &str) -> Option<Hsla> {
    let digits = text.strip_prefix('#')?;
    let value = u32::from_str_radix(digits, 16).ok()?;
    match digits.len() {
        6 => Some(rgb(value).into()),
        8 => Some(rgba(value).into()),
        _ => None,
    }
}

/// The colour roles (DESIGN.md, Color Palette & Roles).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Colors {
    #[serde(deserialize_with = "color")]
    pub window: Hsla,
    #[serde(deserialize_with = "color")]
    pub mantle: Hsla,
    #[serde(deserialize_with = "color")]
    pub selected: Hsla,
    #[serde(deserialize_with = "color")]
    pub hover: Hsla,
    #[serde(deserialize_with = "color")]
    pub badge: Hsla,
    #[serde(deserialize_with = "color")]
    pub edge: Hsla,
    #[serde(deserialize_with = "color")]
    pub ring: Hsla,
    #[serde(deserialize_with = "color")]
    pub text: Hsla,
    #[serde(deserialize_with = "color")]
    pub muted: Hsla,
    #[serde(deserialize_with = "color")]
    pub faint: Hsla,
    #[serde(deserialize_with = "color")]
    pub mauve: Hsla,
    #[serde(deserialize_with = "color")]
    pub blue: Hsla,
    #[serde(deserialize_with = "color")]
    pub green: Hsla,
    #[serde(deserialize_with = "color")]
    pub peach: Hsla,
    #[serde(deserialize_with = "color")]
    pub pink: Hsla,
    #[serde(deserialize_with = "color")]
    pub yellow: Hsla,
    #[serde(deserialize_with = "color")]
    pub red: Hsla,
}

fn color<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Hsla, D::Error> {
    Color::deserialize(deserializer).map(|c| c.0)
}

impl Colors {
    /// A tag hue's colour: the Raycast hash's seven hues in Catppuccin's names.
    #[must_use]
    pub fn hue(&self, hue: Hue) -> Hsla {
        match hue {
            Hue::Blue => self.blue,
            Hue::Orange => self.peach,
            Hue::Green => self.green,
            Hue::Purple => self.mauve,
            Hue::Magenta => self.pink,
            Hue::Yellow => self.yellow,
            Hue::Red => self.red,
        }
    }

    /// An accent by its token name (the catalog's `tint`): `mauve`, `blue`, `green`, `peach`,
    /// `pink`, `yellow` or `red`.
    #[must_use]
    pub fn named(&self, name: &str) -> Option<Hsla> {
        Some(match name {
            "mauve" => self.mauve,
            "blue" => self.blue,
            "green" => self.green,
            "peach" => self.peach,
            "pink" => self.pink,
            "yellow" => self.yellow,
            "red" => self.red,
            _ => return None,
        })
    }
}

/// Which bundled family a role uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Family {
    /// Inter.
    Ui,
    /// `JetBrains` Mono.
    Mono,
}

/// One type role.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextRole {
    /// The family.
    pub family: Family,
    /// The size, in points.
    pub size: f32,
    /// The weight, 100–900.
    pub weight: f32,
    /// The line height, in points; the family's own when absent.
    pub line_height: Option<f32>,
    /// Italic.
    #[serde(default)]
    pub italic: bool,
}

/// The type roles (DESIGN.md, Typography Rules).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Roles {
    pub search: TextRole,
    pub title: TextRole,
    pub description: TextRole,
    pub label: TextRole,
    pub section: TextRole,
    pub footer: TextRole,
    pub chip: TextRole,
    pub count: TextRole,
    pub keycap: TextRole,
    pub heading: TextRole,
    pub quote: TextRole,
    pub body: TextRole,
    pub notes_h1: TextRole,
    pub notes_h2: TextRole,
    pub notes_h3: TextRole,
    pub card_split: TextRole,
    pub card_detail: TextRole,
    pub link_title: TextRole,
    pub link_url: TextRole,
    pub link_hero: TextRole,
    pub empty_title: TextRole,
    pub empty_text: TextRole,
    pub notice: TextRole,
}

/// The families and the roles.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Typography {
    /// The interface family.
    pub ui: String,
    /// The code family.
    pub mono: String,
    /// The roles.
    pub roles: Roles,
}

/// Corner radii, in points.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rounded {
    pub window: f32,
    pub row: f32,
    pub chip: f32,
    pub tile: f32,
    pub pill: f32,
    pub popover: f32,
    pub card: f32,
    pub hero: f32,
    pub favicon: f32,
}

/// Declares a struct of `f32` points, each field documented by its name.
macro_rules! points {
    ($(#[$meta:meta])* $name:ident { $($field:ident),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            $(pub $field: f32,)*
        }
    };
}

points!(/// The header's spacing.
    HeaderSpacing { pad_x, gap });
points!(/// The list's spacing.
    ListSpacing { pad, gap });
points!(/// A row's spacing.
    RowSpacing { pad_x, gap, text_gap, trailing_gap, tags_gap, split_pad_x, split_gap });
points!(/// The entry view's spacing.
    PaneSpacing { split_pad_x, split_pad_y, detail_pad_x, detail_pad_y, gap, header_gap });
points!(/// The footer's spacing.
    FooterSpacing { pad_left, pad_right, entry_pad_right, gap, entry_gap, item_gap, left_gap });
points!(/// The action panel's spacing.
    PanelSpacing { pad, row_pad_x, row_gap, section_pad_top, section_pad_x, section_pad_bottom, rule_margin, search_pad_x, search_gap, crumb_pad_x, crumb_gap });
points!(/// The filter pill's spacing.
    PillSpacing { pad_x, pad_y, gap });
points!(/// A chip's spacing.
    ChipSpacing { pad_x, link_pad_x, link_gap });
points!(/// A keycap's spacing.
    KeycapSpacing { pad_x });
points!(/// The empty view's spacing.
    EmptySpacing { pad_x, icon_gap, title_gap, bar_gap, summary_pad_x, summary_pad_top, summary_pad_bottom });
points!(/// The quote's spacing.
    QuoteSpacing { pad_left, pad_y });
points!(/// The command card's spacing.
    CardSpacing { pad_x, pad_y, pad_right, copy_inset });
points!(/// The link hero's spacing.
    LinkHeroSpacing { pad_x, pad_y, gap });
points!(/// The Links section's spacing.
    LinksSpacing { gap, label_gap, label_pad_top, label_pad_bottom, row_pad_x, row_gap });
points!(/// The notice's spacing.
    NoticeSpacing { gap, hint_gap, right, bottom });

/// Spacing, by component.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spacing {
    pub header: HeaderSpacing,
    pub list: ListSpacing,
    pub row: RowSpacing,
    pub pane: PaneSpacing,
    pub footer: FooterSpacing,
    pub panel: PanelSpacing,
    pub pill: PillSpacing,
    pub chip: ChipSpacing,
    pub keycap: KeycapSpacing,
    pub empty: EmptySpacing,
    pub quote: QuoteSpacing,
    pub card: CardSpacing,
    pub link_hero: LinkHeroSpacing,
    pub links: LinksSpacing,
    pub notice: NoticeSpacing,
}

points!(/// The window's sizes.
    WindowSize { width, height, expanded_share });
points!(/// The header's sizes.
    HeaderSize { height, icon, pill_icon, pill_max_width });
points!(/// A row's sizes.
    RowSize { height, bar_width, bar_inset, glow, glow_alpha, max_tags });
points!(/// A chip's sizes.
    ChipSize { height, icon, link_icon });
points!(/// A keycap's sizes.
    KeycapSize { height, min_width });
points!(/// The type tile's sizes.
    TileSize { size, icon, small, small_icon, mini, mini_icon, alpha });
points!(/// The favicon's sizes.
    FaviconSize { size });
points!(/// The action panel's sizes.
    PanelSize { width, filter_width, row, search, crumb, max_height, chevron, search_icon, crumb_icon, check });
points!(/// The footer's sizes.
    FooterSize { height, dot, dot_glow, dot_glow_alpha });
points!(/// The split's shares.
    SplitSize { list });
points!(/// The empty view's sizes.
    EmptySize { icon, text_max_width, bar_width, bar_height });
points!(/// The detail screen's scrolling.
    DetailSize { scroll_step });
points!(/// The hero's sizes.
    HeroSize { thumb_split, thumb_detail, link_icon });
points!(/// The quote's sizes.
    QuoteSize { bar });
points!(/// The Links section's sizes.
    LinksSize { row, url_chars, title_share });
points!(/// The notice's sizes.
    NoticeSize { width, icon, check, ring_fill, ring_border });
points!(/// Borders.
    BorderSize { hairline });

/// Component sizes.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Components {
    pub window: WindowSize,
    pub header: HeaderSize,
    pub row: RowSize,
    pub chip: ChipSize,
    pub keycap: KeycapSize,
    pub tile: TileSize,
    pub favicon: FaviconSize,
    pub panel: PanelSize,
    pub footer: FooterSize,
    pub split: SplitSize,
    pub empty: EmptySize,
    pub hero: HeroSize,
    pub quote: QuoteSize,
    pub links: LinksSize,
    pub notice: NoticeSize,
    pub border: BorderSize,
    pub detail: DetailSize,
}

/// Every design token.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Design {
    /// Colour roles.
    pub colors: Colors,
    /// Families and type roles.
    pub typography: Typography,
    /// Corner radii.
    pub rounded: Rounded,
    /// Spacing.
    pub spacing: Spacing,
    /// Component sizes.
    pub components: Components,
}

/// Parses the YAML front matter of `markdown`.
///
/// # Errors
///
/// No front matter, YAML that does not parse, or tokens of the wrong shape.
pub fn parse(markdown: &str) -> Result<Design, DesignError> {
    let rest = markdown.strip_prefix("---\n").ok_or(DesignError::NoFrontMatter)?;
    let end = rest.find("\n---\n").ok_or(DesignError::NoFrontMatter)?;
    let node = kb_domain::yaml::load(&rest[..end])
        .map_err(|error| DesignError::Yaml(error.to_string()))?
        .ok_or(DesignError::NoFrontMatter)?;
    serde_json::from_value(kb_domain::yaml::to_json(&node)).map_err(|error| DesignError::Shape(error.to_string()))
}

/// `JetBrains` Mono's ligatures and contextual alternates off: code reads as it is copied.
#[must_use]
pub fn no_ligatures() -> FontFeatures {
    FontFeatures(Arc::new(vec![("liga".into(), 0), ("calt".into(), 0)]))
}

/// The bundled fonts: Inter 400, 500, 600 and 400 italic, then `JetBrains` Mono 400 and 500.
pub const FONTS: [&[u8]; 6] = [
    include_bytes!("../../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../../assets/fonts/Inter-Medium.ttf"),
    include_bytes!("../../assets/fonts/Inter-SemiBold.ttf"),
    include_bytes!("../../assets/fonts/Inter-Italic.ttf"),
    include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../../assets/fonts/JetBrainsMono-Medium.ttf"),
];

/// Registers the bundled fonts with GPUI's text system.
///
/// # Errors
///
/// The platform rejected a font.
pub fn load_fonts(cx: &gpui_kit::App) -> gpui_kit::Result<()> {
    cx.text_system()
        .add_fonts(FONTS.iter().map(|font| std::borrow::Cow::Borrowed(*font)).collect())
}

/// Applies a type role: family, size, weight, line height, italic; ligatures off for mono.
pub trait TextRoleExt: Styled + Sized {
    /// Draws text in `role`.
    #[must_use]
    fn text_role(self, role: &TextRole) -> Self {
        let typography = &design().typography;
        let family = match role.family {
            Family::Ui => typography.ui.clone(),
            Family::Mono => typography.mono.clone(),
        };
        let mut this = self
            .font_family(family)
            .text_size(pt(role.size))
            .font_weight(FontWeight(role.weight));
        if let Some(line_height) = role.line_height {
            this = this.line_height(pt(line_height));
        }
        if role.italic {
            this = this.italic();
        }
        if role.family == Family::Mono {
            this = this.font_features(no_ligatures());
        }
        this
    }
}

impl<T: Styled> TextRoleExt for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_tokens_of_design_md() {
        let d = parse(DESIGN_MD).unwrap();
        assert_eq!(d.colors.window, Hsla::from(rgb(0x0024_273A)));
        assert_eq!(d.colors.edge, Hsla::from(rgba(0x6E73_8D1A)));
        assert_eq!(d.typography.ui, "Inter");
        assert_eq!(d.typography.mono, "JetBrains Mono");
        assert!((d.typography.roles.title.size - 13.).abs() < f32::EPSILON);
        assert!((d.components.row.height - 56.).abs() < f32::EPSILON);
        assert!((d.components.split.list - 0.34).abs() < f32::EPSILON);
        assert!((d.rounded.window - 14.).abs() < f32::EPSILON);
    }

    #[test]
    fn rejects_a_file_without_front_matter_or_with_a_missing_token() {
        assert!(matches!(parse("# Design\n"), Err(DesignError::NoFrontMatter)));
        let without_window = DESIGN_MD.replacen("  window: \"#24273A\"\n", "", 1);
        assert!(matches!(parse(&without_window), Err(DesignError::Shape(_))));
    }

    #[test]
    fn converts_points_to_rems_at_sixteen_points_per_rem() {
        assert_eq!(pt(16.), rems(1.));
        assert_eq!(pt(8.), rems(0.5));
    }

    #[test]
    fn names_the_seven_hues_after_catppuccin() {
        let c = &design().colors;
        assert_eq!(c.hue(Hue::Purple), c.mauve);
        assert_eq!(c.hue(Hue::Magenta), c.pink);
        assert_eq!(c.hue(Hue::Orange), c.peach);
        assert_eq!(c.named("green"), Some(c.green));
        assert_eq!(c.named("teal"), None);
    }

    #[test]
    fn gives_gpui_kit_the_tokens() {
        let theme: serde_json::Value = serde_json::from_str(include_str!("../../assets/themes/kb.json")).unwrap();
        let colors = &theme["themes"][0]["colors"];
        let token = |key: &str| hex(colors[key].as_str().unwrap()).unwrap();
        let c = &design().colors;
        assert_eq!(token("popover.background"), c.mantle);
        assert_eq!(token("foreground"), c.text);
        assert_eq!(token("muted.foreground"), c.muted);
        assert_eq!(token("border"), c.selected);
        assert_eq!(token("caret"), c.mauve);
        assert_eq!(token("background").a, 0., "the window draws its own rounded surface");
    }

    /// Whether `font` names `family` in its name table (UTF-16 big-endian, as TrueType stores it).
    fn names_family(font: &[u8], family: &str) -> bool {
        let needle: Vec<u8> = family.encode_utf16().flat_map(u16::to_be_bytes).collect();
        font.windows(needle.len()).any(|window| window == needle.as_slice())
    }

    #[test]
    fn bundles_the_families_the_tokens_name() {
        let typography = &design().typography;
        for font in FONTS {
            assert_eq!(&font[..4], &[0, 1, 0, 0], "a TrueType font");
        }
        for font in &FONTS[..4] {
            assert!(names_family(font, &typography.ui));
        }
        for font in &FONTS[4..] {
            assert!(names_family(font, &typography.mono));
        }
    }
}
