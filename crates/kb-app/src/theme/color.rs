//! Colour arithmetic on `#RRGGBB(AA)` values, as `tools/design/color.ts` does it (in `f64`, rounding half up), so a
//! palette's GPUI Kit theme matches the one the design kit wrote.

use gpui_kit::Hsla;

/// A colour as 8-bit channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba8 {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
    /// Opacity, 255 opaque.
    pub a: u8,
}

/// A channel computed in `f64`, rounded half up and clamped, as `color.ts`'s `toHex` and `tint` do.
fn channel(value: f64) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "rounded and clamped to 0..=255 first"
    )]
    {
        value.round().clamp(0., 255.) as u8
    }
}

impl Rgba8 {
    /// Opaque black: a placeholder before a palette is filled.
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    /// `#RRGGBB` or `#RRGGBBAA`; the `#` is optional, as older base16 files omit it.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let digits = text.trim().trim_start_matches('#');
        if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let value = u32::from_str_radix(digits, 16).ok()?;
        match digits.len() {
            6 => {
                let [_, r, g, b] = value.to_be_bytes();
                Some(Self { r, g, b, a: 255 })
            }
            8 => {
                let [r, g, b, a] = value.to_be_bytes();
                Some(Self { r, g, b, a })
            }
            _ => None,
        }
    }

    /// `#RRGGBB` when opaque, else `#RRGGBBAA`, upper-case.
    #[must_use]
    pub fn hex(self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    /// The same colour at `alpha` (0–1).
    #[must_use]
    pub fn with_alpha(self, alpha: f64) -> Self {
        Self {
            a: channel(alpha * 255.),
            ..self
        }
    }

    fn channels(self) -> [f64; 3] {
        [f64::from(self.r), f64::from(self.g), f64::from(self.b)]
    }

    /// `self` drawn at `alpha` over `under`, opaque (`color.ts`'s `mix`).
    #[must_use]
    pub fn mix(self, under: Self, alpha: f64) -> Self {
        let (f, u) = (self.channels(), under.channels());
        let at = |i: usize| channel(f[i] * alpha + u[i] * (1. - alpha));
        Self {
            r: at(0),
            g: at(1),
            b: at(2),
            a: 255,
        }
    }

    /// The colour that, drawn at `alpha` over `under`, shows as `self` (`color.ts`'s `tint`), clamped to what can be
    /// drawn instead of failing: GPUI Kit caps some highlights' opacity.
    #[must_use]
    pub fn tint(self, under: Self, alpha: f64) -> Self {
        let (t, u) = (self.channels(), under.channels());
        let at = |i: usize| channel(u[i] + (t[i] - u[i]) / alpha);
        Self {
            r: at(0),
            g: at(1),
            b: at(2),
            a: channel(alpha * 255.),
        }
    }

    fn luminance(self) -> f64 {
        let lin = |v: f64| {
            let c = v / 255.;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let [r, g, b] = self.channels().map(lin);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// The WCAG 2.2 contrast ratio of two opaque colours.
    #[must_use]
    pub fn contrast(self, other: Self) -> f64 {
        let (x, y) = (self.luminance(), other.luminance());
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    /// The colour GPUI draws.
    #[must_use]
    pub fn hsla(self) -> Hsla {
        gpui_kit::Rgba {
            r: f32::from(self.r) / 255.,
            g: f32::from(self.g) / 255.,
            b: f32::from(self.b) / 255.,
            a: f32::from(self.a) / 255.,
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(text: &str) -> Rgba8 {
        Rgba8::parse(text).unwrap()
    }

    #[test]
    fn parses_with_or_without_a_hash_and_writes_upper_case() {
        assert_eq!(c("#24273a").hex(), "#24273A");
        assert_eq!(c("24273A").hex(), "#24273A", "older base16 files omit the #");
        assert_eq!(c("#6E738D1A").hex(), "#6E738D1A");
        assert_eq!(Rgba8::parse("#123"), None);
    }

    #[test]
    fn mixes_tints_and_measures_as_the_design_kit_does() {
        // color.ts: mix('#8AADF4', '#24273A', 0.2), tint('#363A4F', '#24273A', 0.2), contrast('#CAD3F5', '#24273A').
        assert_eq!(c("#8AADF4").mix(c("#24273A"), 0.2).hex(), "#38425F");
        assert_eq!(c("#363A4F").tint(c("#24273A"), 0.2).hex(), "#7E86A333");
        assert!((c("#CAD3F5").contrast(c("#24273A")) - 9.92).abs() < 0.01);
    }

    #[test]
    fn clamps_a_tint_no_colour_can_reach() {
        // Neon Dusk's first selection colour: the blue channel would need 292.
        let t = c("#38265C").tint(c("#16122A"), 0.2);
        assert_eq!(&t.hex()[5..7], "FF");
    }
}
