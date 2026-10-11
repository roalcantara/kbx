//! The seven accents of DESIGN.md, which GPUI Kit's theme has no keys for: each with its tag
//! pill's fill and text, for the dark and light appearances. `tools/design/theme.mjs` writes them
//! into `assets/themes/accents.json` from the chosen palette.

use std::collections::HashMap;
use std::sync::LazyLock;

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{App, Hsla, rgb};
use serde::Deserialize;

use crate::presentation::Hue;

/// An accent and its tag pill.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Accent {
    /// Icons, bars, the type tint.
    pub color: Hsla,
    /// The tag pill's fill.
    pub pill: Hsla,
    /// The tag pill's text.
    pub pill_text: Hsla,
}

#[derive(Debug, Deserialize)]
struct Raw {
    accent: String,
    background: String,
    text: String,
}

#[derive(Debug, Default, Deserialize)]
struct Sets {
    dark: HashMap<String, Raw>,
    light: HashMap<String, Raw>,
}

/// The file is compiled in and a test loads it: should it ever fail to parse, `accent` falls back
/// to the theme's foreground.
static SETS: LazyLock<Sets> =
    LazyLock::new(|| serde_json::from_str(include_str!("../assets/themes/accents.json")).unwrap_or_default());

fn hsla(hex: &str) -> Option<Hsla> {
    u32::from_str_radix(hex.trim_start_matches('#'), 16)
        .ok()
        .map(|n| rgb(n).into())
}

fn key(hue: Hue) -> &'static str {
    match hue {
        Hue::Blue => "blue",
        Hue::Green => "green",
        Hue::Orange => "orange",
        Hue::Purple => "purple",
        Hue::Magenta => "magenta",
        Hue::Yellow => "yellow",
        Hue::Red => "red",
    }
}

/// The accent for `hue` in the current appearance.
#[must_use]
pub fn accent(hue: Hue, cx: &App) -> Accent {
    let theme = cx.theme();
    let set = if theme.is_dark() { &SETS.dark } else { &SETS.light };
    let fallback = Accent {
        color: theme.foreground,
        pill: theme.muted,
        pill_text: theme.foreground,
    };
    set.get(key(hue)).map_or(fallback, |raw| Accent {
        color: hsla(&raw.accent).unwrap_or(fallback.color),
        pill: hsla(&raw.background).unwrap_or(fallback.pill),
        pill_text: hsla(&raw.text).unwrap_or(fallback.pill_text),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_every_accent_for_both_appearances() {
        for hue in [
            Hue::Blue,
            Hue::Green,
            Hue::Orange,
            Hue::Purple,
            Hue::Magenta,
            Hue::Yellow,
            Hue::Red,
        ] {
            for set in [&SETS.dark, &SETS.light] {
                let raw = set.get(key(hue)).unwrap_or_else(|| panic!("{hue:?}"));
                assert!(
                    hsla(&raw.accent)
                        .and(hsla(&raw.background))
                        .and(hsla(&raw.text))
                        .is_some()
                );
            }
        }
    }
}
