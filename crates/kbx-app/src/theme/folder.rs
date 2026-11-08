//! The themes folder: kbx's shipped themes, seeded into `~/.config/kbx/themes/`, and the files found there.

use std::fs;
use std::io;
use std::path::Path;

use crate::theme::scheme::{Palette, Scheme, parse};

/// The theme kbx uses when config.toml names none.
pub const NATIVE: &str = "kbx";

/// kbx's shipped themes, compiled in: (id, file).
pub const SHIPPED: [(&str, &str); 8] = [
    (NATIVE, include_str!("../../assets/themes/kbx.yaml")),
    ("neon-dusk", include_str!("../../assets/themes/neon-dusk.yaml")),
    ("archwave", include_str!("../../assets/themes/archwave.yaml")),
    ("azure-glow", include_str!("../../assets/themes/azure-glow.yaml")),
    ("aetheria", include_str!("../../assets/themes/aetheria.yaml")),
    ("monokai", include_str!("../../assets/themes/monokai.yaml")),
    ("sapphire", include_str!("../../assets/themes/sapphire.yaml")),
    (
        "nagai-poolside",
        include_str!("../../assets/themes/nagai-poolside.yaml"),
    ),
];

/// kbx's palette, compiled in: the fallback of every fallback.
///
/// # Panics
///
/// The compiled-in `kbx.yaml` is not a theme; a unit test proves it is.
#[must_use]
#[expect(clippy::expect_used, reason = "the shipped theme is checked by a unit test")]
pub fn native() -> Palette {
    parse(NATIVE, SHIPPED[0].1).expect("kbx.yaml is a theme").palette
}

/// Writes each shipped theme into `dir` unless a theme of that id is there (`.yaml` or `.yml`).
///
/// # Errors
///
/// `dir` cannot be created or a file cannot be written.
pub fn seed(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    for (id, text) in SHIPPED {
        if !dir.join(format!("{id}.yaml")).exists() && !dir.join(format!("{id}.yml")).exists() {
            fs::write(dir.join(format!("{id}.yaml")), text)?;
        }
    }
    Ok(())
}

/// One theme file: its id, and the theme or why it is not one.
#[derive(Debug, Clone)]
pub struct Entry {
    /// The file stem.
    pub id: String,
    /// The theme, or why the file is not one.
    pub scheme: Result<Scheme, String>,
}

impl Entry {
    /// The theme's label, else the id.
    #[must_use]
    pub fn label(&self) -> &str {
        self.scheme.as_ref().map_or(self.id.as_str(), |s| s.label.as_str())
    }
}

/// Every `*.yaml` and `*.yml` in `dir`, by label; the shipped themes when `dir` cannot be read.
#[must_use]
pub fn list(dir: &Path) -> Vec<Entry> {
    let mut entries: Vec<Entry> = match fs::read_dir(dir) {
        Ok(read) => read
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "yaml" || x == "yml"))
            .filter_map(|path| {
                let id = path.file_stem()?.to_string_lossy().into_owned();
                let scheme = fs::read_to_string(&path)
                    .map_err(|e| e.to_string())
                    .and_then(|text| parse(&id, &text).map_err(|e| e.to_string()));
                Some(Entry { id, scheme })
            })
            .collect(),
        Err(_) => SHIPPED
            .iter()
            .map(|(id, text)| Entry {
                id: (*id).into(),
                scheme: parse(id, text).map_err(|e| e.to_string()),
            })
            .collect(),
    };
    entries.sort_by_key(|e| e.label().to_lowercase());
    entries
}

/// Theme `id` from `dir`, else the shipped theme of that id.
///
/// # Errors
///
/// No such theme, or its file is not a theme.
pub fn load(dir: &Path, id: &str) -> Result<Scheme, String> {
    for ext in ["yaml", "yml"] {
        if let Ok(text) = fs::read_to_string(dir.join(format!("{id}.{ext}"))) {
            return parse(id, &text).map_err(|e| format!("{id}.{ext}: {e}"));
        }
    }
    match SHIPPED.iter().find(|(shipped, _)| *shipped == id) {
        Some((_, text)) => parse(id, text).map_err(|e| e.to_string()),
        None => Err(format!("no theme “{id}” in {}", dir.display())),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn every_shipped_theme_reads_and_is_legible() {
        for (id, text) in SHIPPED {
            let scheme = crate::theme::scheme::parse(id, text).unwrap();
            assert_eq!(crate::theme::legibility::first_failure(&scheme.palette), None, "{id}");
        }
        assert_eq!(native().role("window").hex(), "#24273A");
    }

    #[test]
    fn seeds_missing_themes_and_never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("neon-dusk.yaml"), "edited").unwrap();
        seed(dir.path()).unwrap();
        assert_eq!(fs::read_to_string(dir.path().join("neon-dusk.yaml")).unwrap(), "edited");
        assert!(dir.path().join("kbx.yaml").is_file());
    }

    #[test]
    fn lists_files_by_label_with_broken_ones_and_loads_by_id() {
        let dir = tempfile::tempdir().unwrap();
        seed(dir.path()).unwrap();
        fs::write(dir.path().join("broken.yml"), "palette: [").unwrap();
        fs::write(dir.path().join("notes.txt"), "not a theme").unwrap();
        let entries = list(dir.path());
        assert_eq!(entries.len(), 9);
        assert!(entries.iter().any(|e| e.id == "broken" && e.scheme.is_err()));
        assert_eq!(load(dir.path(), "archwave").unwrap().label, "Archwave");
        assert!(load(dir.path(), "nope").is_err());
    }
}
