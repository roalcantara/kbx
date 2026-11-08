//! kbx's legibility rules for a palette (formerly `design:validate`'s checks of DESIGN.md's colours).

use crate::theme::scheme::Palette;

/// The first rule `p` breaks, as "muted on badge is 3.9:1, below 4.5:1"; `None` when it keeps them all: text and
/// muted 4.5:1 on window, mantle, selected, badge and crust; faint 3:1 on window, mantle and crust, 2.5:1 on badge;
/// each accent 3:1 on its tile (the accent at 15 % over the window).
#[must_use]
pub fn first_failure(p: &Palette) -> Option<String> {
    let mut rules: Vec<(String, f64, f64)> = Vec::new();
    for surface in ["window", "mantle", "selected", "badge", "crust"] {
        for ink in ["text", "muted"] {
            rules.push((
                format!("{ink} on {surface}"),
                p.role(ink).contrast(p.role(surface)),
                4.5,
            ));
        }
    }
    for surface in ["window", "mantle", "crust"] {
        rules.push((
            format!("faint on {surface}"),
            p.role("faint").contrast(p.role(surface)),
            3.,
        ));
    }
    rules.push(("faint on badge".into(), p.role("faint").contrast(p.role("badge")), 2.5));
    for accent in ["mauve", "blue", "green", "peach", "pink", "yellow", "red"] {
        let tile = p.role(accent).mix(p.role("window"), 0.15);
        rules.push((format!("{accent} on its tile"), p.role(accent).contrast(tile), 3.));
    }
    rules
        .into_iter()
        .find(|(_, ratio, min)| ratio < min)
        .map(|(name, ratio, min)| format!("{name} is {ratio:.1}:1, below {min}:1"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::scheme::parse;

    /// kbx's roles as a base16 file plus its `kbx:` section, the values DESIGN.md held.
    const NATIVE: &str = "palette:\n  base00: '#24273A'\n  base01: '#2E3248'\n  base02: '#363A4F'\n  base03: '#6E738D'\n  \
        base04: '#A5ADCB'\n  base05: '#CAD3F5'\n  base06: '#CAD3F5'\n  base07: '#CAD3F5'\n  base08: '#ED8796'\n  \
        base09: '#F5A97F'\n  base0A: '#EED49F'\n  base0B: '#A6DA95'\n  base0C: '#F5BDE6'\n  base0D: '#8AADF4'\n  \
        base0E: '#C6A0F6'\n  base0F: '#ED8796'\n  base10: '#1E2030'\n  base11: '#181926'\n";

    #[test]
    fn passes_kb_native_and_names_the_first_rule_a_palette_breaks() {
        assert_eq!(first_failure(&parse("t", NATIVE).unwrap().palette), None);
        let weak = NATIVE.replace("base04: '#A5ADCB'", "base04: '#4A4F66'");
        let failure = first_failure(&parse("t", &weak).unwrap().palette).unwrap();
        assert!(failure.starts_with("muted on "), "{failure}");
        assert!(failure.ends_with("below 4.5:1"), "{failure}");
    }
}
