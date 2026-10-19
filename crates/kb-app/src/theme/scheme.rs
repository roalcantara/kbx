//! A base24 (or base16) scheme file mapped onto kb's 19 colour roles (spec D1, D2).

use serde_json::Value;

use crate::theme::color::Rgba8;

/// kb's colour roles, in DESIGN.md's order.
pub const ROLES: [&str; 19] = [
    "window", "mantle", "crust", "selected", "hover", "badge", "code", "edge", "ring", "text", "muted", "faint",
    "mauve", "blue", "green", "peach", "pink", "yellow", "red",
];

/// A value for every role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette([Rgba8; 19]);

impl Palette {
    fn index(role: &str) -> Option<usize> {
        ROLES.iter().position(|r| *r == role)
    }

    /// A role's colour; `None` for a name that is not a role.
    #[must_use]
    pub fn get(&self, role: &str) -> Option<Rgba8> {
        Self::index(role).map(|i| self.0[i])
    }

    /// A role's colour, for the names in [`ROLES`] (black for any other).
    #[must_use]
    pub fn role(&self, role: &str) -> Rgba8 {
        self.get(role).unwrap_or(Rgba8::BLACK)
    }

    fn set(&mut self, role: &str, value: Rgba8) -> bool {
        Self::index(role).map(|i| self.0[i] = value).is_some()
    }

    /// Overrides from a table of `role = "#hex"` (config.toml's `[colors]`); a notice for each key it cannot use.
    pub fn apply(&mut self, table: &toml::Table) -> Vec<String> {
        let mut notices = Vec::new();
        for (key, value) in table {
            match value.as_str().and_then(Rgba8::parse) {
                Some(colour) if self.set(key, colour) => {}
                Some(_) => notices.push(format!("Ignored colors.{key} in config.toml: not a colour role")),
                None => notices.push(format!("Ignored colors.{key} in config.toml: not #RRGGBB or #RRGGBBAA")),
            }
        }
        notices
    }

    /// The colours kb's views draw with.
    #[must_use]
    #[expect(dead_code, reason = "the active theme uses it from the theme tasks that follow")]
    pub(crate) fn colors(&self) -> crate::ui::design::Colors {
        let c = |role: &str| self.role(role).hsla();
        crate::ui::design::Colors {
            window: c("window"),
            mantle: c("mantle"),
            crust: c("crust"),
            selected: c("selected"),
            hover: c("hover"),
            badge: c("badge"),
            code: c("code"),
            edge: c("edge"),
            ring: c("ring"),
            text: c("text"),
            muted: c("muted"),
            faint: c("faint"),
            mauve: c("mauve"),
            blue: c("blue"),
            green: c("green"),
            peach: c("peach"),
            pink: c("pink"),
            yellow: c("yellow"),
            red: c("red"),
        }
    }
}

/// A theme file, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scheme {
    /// The file stem: what config.toml names.
    pub id: String,
    /// The file's `name`, else the id.
    pub label: String,
    /// Its colours.
    pub palette: Palette,
}

/// Why a file is not a theme.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SchemeError {
    /// Not YAML.
    #[error("not YAML: {0}")]
    Yaml(String),
    /// A required colour is absent.
    #[error("no {0}")]
    Missing(String),
    /// A value is not a colour.
    #[error("{key} is not a colour: {value}")]
    BadColour {
        /// Where.
        key: String,
        /// What it held.
        value: String,
    },
}

/// base24's fallbacks for base16 files (base24's styling.md): base10/11 → base00, base12–17 → their normal colours.
const FALLBACK: [(&str, &str); 8] = [
    ("base10", "base00"),
    ("base11", "base00"),
    ("base12", "base08"),
    ("base13", "base0A"),
    ("base14", "base0B"),
    ("base15", "base0C"),
    ("base16", "base0D"),
    ("base17", "base0E"),
];

fn colour(key: String, value: &Value) -> Result<Rgba8, SchemeError> {
    let text = value.as_str().map_or_else(|| value.to_string(), str::to_owned);
    Rgba8::parse(&text).ok_or(SchemeError::BadColour { key, value: text })
}

/// A palette entry by name, as written or lower-cased (`base0A`, `base0a`).
fn entry<'a>(palette: &'a Value, name: &str) -> Option<&'a Value> {
    palette.get(name).or_else(|| palette.get(name.to_lowercase()))
}

fn base(palette: &Value, name: &str) -> Result<Rgba8, SchemeError> {
    if let Some(value) = entry(palette, name) {
        return colour(format!("palette.{name}"), value);
    }
    let to = FALLBACK
        .iter()
        .find(|(from, _)| *from == name)
        .map(|(_, to)| *to)
        .ok_or_else(|| SchemeError::Missing(name.into()))?;
    let value = entry(palette, to).ok_or_else(|| SchemeError::Missing(to.into()))?;
    colour(format!("palette.{to}"), value)
}

/// Reads theme `id` from a base24 or base16 YAML `text`: the D2 mapping, then the file's `kb:` section.
///
/// # Errors
///
/// The text is not YAML, lacks one of base00–base0F, or holds a value that is not a colour.
pub fn parse(id: &str, text: &str) -> Result<Scheme, SchemeError> {
    let node = kb_domain::yaml::load(text)
        .map_err(|e| SchemeError::Yaml(e.to_string()))?
        .ok_or_else(|| SchemeError::Yaml("empty".into()))?;
    let json = kb_domain::yaml::to_json(&node);
    let palette = &json["palette"];
    for i in 0..16 {
        base(palette, &format!("base{i:02X}"))?;
    }
    let b = |name: &str| base(palette, name);
    let mut p = Palette([Rgba8::BLACK; 19]);
    let mapping: [(&str, Rgba8); 19] = [
        ("window", b("base00")?),
        ("badge", b("base01")?),
        ("selected", b("base02")?),
        ("hover", b("base02")?.mix(b("base00")?, 0.5)),
        ("mantle", b("base10")?),
        ("code", b("base10")?),
        ("crust", b("base11")?),
        ("text", b("base05")?),
        ("muted", b("base04")?),
        ("faint", b("base03")?),
        ("edge", b("base03")?.with_alpha(0.5)),
        ("ring", b("base05")?.with_alpha(0.08)),
        ("red", b("base08")?),
        ("peach", b("base09")?),
        ("yellow", b("base0A")?),
        ("green", b("base0B")?),
        ("pink", b("base0C")?),
        ("blue", b("base0D")?),
        ("mauve", b("base0E")?),
    ];
    for (role, value) in mapping {
        p.set(role, value);
    }
    if let Some(kb) = json.get("kb").and_then(Value::as_object) {
        for (role, value) in kb {
            let value = colour(format!("kb.{role}"), value)?;
            if !p.set(role, value) {
                return Err(SchemeError::BadColour {
                    key: format!("kb.{role}"),
                    value: "not a colour role".into(),
                });
            }
        }
    }
    let label = json.get("name").and_then(Value::as_str).unwrap_or(id).to_owned();
    Ok(Scheme {
        id: id.into(),
        label,
        palette: p,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE16: &str = "system: base16\nname: Test Dark\npalette:\n  base00: '#101010'\n  base01: '#202020'\n  \
        base02: '#303030'\n  base03: '#808080'\n  base04: '#B0B0B0'\n  base05: '#E0E0E0'\n  base06: '#F0F0F0'\n  \
        base07: '#FFFFFF'\n  base08: '#FF0000'\n  base09: '#FF8000'\n  base0A: '#FFFF00'\n  base0B: '#00FF00'\n  \
        base0C: '#00FFFF'\n  base0D: '#0000FF'\n  base0E: '#FF00FF'\n  base0F: '#800000'\n";

    #[test]
    fn maps_a_base16_file_by_each_colours_meaning() {
        let s = parse("test", BASE16).unwrap();
        assert_eq!((s.id.as_str(), s.label.as_str()), ("test", "Test Dark"));
        let hex = |role: &str| s.palette.get(role).unwrap().hex();
        assert_eq!(hex("window"), "#101010");
        assert_eq!(hex("mantle"), "#101010", "base16: base10 falls back to base00");
        assert_eq!(hex("crust"), "#101010");
        assert_eq!(hex("selected"), "#303030");
        assert_eq!(hex("hover"), "#202020", "base00 and base02 half and half");
        assert_eq!(hex("badge"), "#202020");
        assert_eq!(hex("text"), "#E0E0E0");
        assert_eq!(hex("muted"), "#B0B0B0");
        assert_eq!(hex("faint"), "#808080");
        assert_eq!(hex("edge"), "#80808080");
        assert_eq!(hex("ring"), "#E0E0E014");
        assert_eq!(hex("mauve"), "#FF00FF");
        assert_eq!(hex("pink"), "#00FFFF");
        assert_eq!(hex("blue"), "#0000FF");
    }

    #[test]
    fn uses_base24s_darker_backgrounds_and_the_kb_section() {
        let text = format!("{BASE16}  base10: '#080808'\n  base11: '#040404'\nkb:\n  mauve: '#123456'\n");
        let p = parse("t", &text).unwrap().palette;
        assert_eq!(p.get("mantle").unwrap().hex(), "#080808");
        assert_eq!(p.get("crust").unwrap().hex(), "#040404");
        assert_eq!(p.get("mauve").unwrap().hex(), "#123456");
    }

    #[test]
    fn says_why_a_file_is_not_a_theme() {
        assert!(parse("t", "palette: [").is_err());
        let missing = parse("t", "palette:\n  base00: '#000000'\n").unwrap_err().to_string();
        assert!(missing.contains("base01"), "{missing}");
        let bad = format!("{BASE16}kb:\n  mauve: 'pink'\n");
        assert!(parse("t", &bad).unwrap_err().to_string().contains("kb.mauve"));
    }

    #[test]
    fn applies_config_overrides_and_reports_bad_ones() {
        let mut p = parse("t", BASE16).unwrap().palette;
        let table: toml::Table = toml::from_str("mauve = '#ABCDEF'\nteal = '#000000'\nred = 'nope'\n").unwrap();
        let notices = p.apply(&table);
        assert_eq!(p.get("mauve").unwrap().hex(), "#ABCDEF");
        assert_eq!(notices.len(), 2, "{notices:?}");
    }
}
