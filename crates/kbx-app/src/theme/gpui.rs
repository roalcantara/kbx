//! GPUI Kit's dark theme from a palette: the mapping `design:theme` wrote into `kbx.json`, in Rust, so any
//! theme and any override reach GPUI Kit's own parts too.

use serde_json::{Map, Value, json};

use crate::theme::color::Rgba8;
use crate::theme::scheme::Palette;

/// The dark theme's name inside the JSON.
pub const THEME_NAME: &str = "kbx Dark";

/// The opacity GPUI Kit allows the selected list row's highlight (theme/schema.rs, `clamp_alpha`).
const LIST_ACTIVE_ALPHA: f64 = 0.2;

/// GPUI Kit's theme JSON for `p`.
#[must_use]
pub fn theme_json(p: &Palette) -> String {
    json!({
        "name": "kbx",
        "author": "kbx",
        "themes": [{ "name": THEME_NAME, "mode": "dark", "colors": colors(p), "highlight": highlight(p) }]
    })
    .to_string()
}

/// The theme's colour keys.
fn colors(p: &Palette) -> Map<String, Value> {
    let t = |role: &str| p.role(role);
    let mut colors: Map<String, Value> = Map::new();
    let mut put = |key: &str, value: String| {
        colors.insert(key.into(), Value::String(value));
    };
    // The panel draws its own rounded surface on a transparent window (DESIGN.md, Shape), so the root and list
    // backgrounds are clear; kbx's components draw rows, chips and panels from the palette themselves.
    for key in [
        "background",
        "list.background",
        "list.even.background",
        "list.head.background",
    ] {
        put(key, "#00000000".into());
    }
    put("table.background", t("window").hex());
    put("title_bar.background", t("window").hex());
    put(
        "list.active.background",
        t("selected").tint(t("window"), LIST_ACTIVE_ALPHA).hex(),
    );
    put("list.active.border", t("selected").hex());
    put("accent.background", t("selected").hex());
    put("accent.foreground", t("text").hex());
    put("secondary.background", t("badge").hex());
    put("secondary.hover.background", t("selected").hex());
    put("secondary.active.background", t("selected").hex());
    put("secondary.foreground", t("text").hex());
    put("popover.background", t("mantle").hex());
    put("popover.foreground", t("text").hex());
    put("muted.background", t("code").hex());
    put("muted.foreground", t("muted").hex());
    for key in ["border", "input.border", "title_bar.border", "window.border"] {
        put(key, t("selected").hex());
    }
    put("foreground", t("text").hex());
    put("caret", t("mauve").hex());
    // 0x4D: 30 %, the most GPUI Kit allows a text selection.
    put("selection.background", Rgba8 { a: 0x4D, ..t("mauve") }.hex());
    put("ring", t("mauve").hex());
    put("link.foreground", t("blue").hex());
    put("progress_bar.background", t("blue").hex());
    for (hue, role) in [
        ("blue", "blue"),
        ("cyan", "blue"),
        ("green", "green"),
        ("red", "red"),
        ("yellow", "yellow"),
        ("magenta", "pink"),
    ] {
        put(&format!("base.{hue}"), t(role).hex());
        put(&format!("base.{hue}.light"), t(role).mix(t("window"), 0.2).hex());
    }
    colors.sort_keys();
    colors
}

/// Code in the notes: each syntax colour lifted toward `text` only as far as AA (4.5:1) needs on `code`.
fn highlight(p: &Palette) -> Value {
    let t = |role: &str| p.role(role);
    let readable = |accent: Rgba8| {
        let mut c = accent;
        let mut lift = 0.05;
        while c.contrast(t("code")) < 4.5 && lift <= 1. {
            c = t("text").mix(accent, lift);
            lift += 0.05;
        }
        c.hex()
    };
    let color = |role: &str| json!({ "color": readable(t(role)) });
    let with = |role: &str, extra: &Value| {
        let mut v = color(role);
        if let (Some(o), Some(e)) = (v.as_object_mut(), extra.as_object()) {
            o.extend(e.clone());
        }
        v
    };
    json!({
        "editor.foreground": t("text").hex(),
        "editor.background": t("code").hex(),
        "editor.active_line.background": t("selected").hex(),
        "editor.line_number": readable(t("faint")),
        "editor.active_line_number": t("muted").hex(),
        "syntax": {
            "attribute": color("blue"),
            "boolean": color("peach"),
            "comment": color("faint"),
            "comment.doc": color("faint"),
            "constant": color("peach"),
            "constructor": color("mauve"),
            "embedded": color("text"),
            "emphasis": { "font_style": "italic" },
            "emphasis.strong": { "font_weight": 700 },
            "function": color("blue"),
            "keyword": color("mauve"),
            "link_text": with("blue", &json!({ "font_style": "normal" })),
            "link_uri": with("blue", &json!({ "font_style": "italic" })),
            "number": color("peach"),
            "operator": color("muted"),
            "property": color("text"),
            "punctuation": color("muted"),
            "string": color("green"),
            "string.escape": color("yellow"),
            "string.regex": color("yellow"),
            "string.special": color("green"),
            "string.special.symbol": color("green"),
            "tag": color("red"),
            "text.literal": color("green"),
            "text.code.span": color("green"),
            "title": with("text", &json!({ "font_weight": 600 })),
            "type": color("yellow"),
            "variable": color("text"),
            "variable.special": color("red")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_what_the_design_kit_wrote_for_kbx() {
        let ours: serde_json::Value = serde_json::from_str(&theme_json(&crate::theme::folder::native())).unwrap();
        let theirs: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/kbx-gpui-theme.json")).unwrap();
        assert_eq!(ours, theirs);
    }
}
