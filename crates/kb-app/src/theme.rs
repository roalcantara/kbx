//! Themes (spec 2026-10-19-themes): base24 scheme files in `~/.config/kb/themes/`, mapped onto kb's colour roles,
//! picked from More… → Theme… and saved by name in config.toml.

pub mod color;
pub mod folder;
pub mod gpui;
pub mod legibility;
pub mod scheme;

use std::path::Path;
use std::sync::{LazyLock, PoisonError, RwLock};

use crate::theme::scheme::Palette;

/// The palette kb draws with.
static ACTIVE: LazyLock<RwLock<Palette>> = LazyLock::new(|| RwLock::new(folder::native()));

/// The active palette.
#[must_use]
pub fn active_palette() -> Palette {
    ACTIVE.read().unwrap_or_else(PoisonError::into_inner).clone()
}

/// Makes `palette` the one kb draws with; the caller rebuilds GPUI Kit's theme (`crate::init_theme`).
pub fn set_active(palette: Palette) {
    crate::ui::design::set_colors(palette.colors());
    *ACTIVE.write().unwrap_or_else(PoisonError::into_inner) = palette;
}

/// The theme to start with (spec D4, D8): `id` (else `default`) from `dir`, with `overrides` on top, as (id, palette,
/// notices); kb-native, with a notice, when that theme cannot be used.
#[must_use]
pub fn resolve(dir: &Path, id: Option<&str>, default: &str, overrides: &toml::Table) -> (String, Palette, Vec<String>) {
    let wanted = id.unwrap_or(default);
    let mut notices = Vec::new();
    let (id, mut palette) = match folder::load(dir, wanted) {
        Ok(scheme) => (scheme.id, scheme.palette),
        Err(error) => {
            notices.push(format!("Using the kb native theme: {error}"));
            (folder::NATIVE.to_owned(), folder::native())
        }
    };
    notices.extend(palette.apply(overrides));
    (id, palette, notices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_kb_native_with_a_notice() {
        let dir = tempfile::tempdir().unwrap();
        let (id, palette, notices) = resolve(dir.path(), Some("nope"), "kb-native", &toml::Table::new());
        assert_eq!(id, "kb-native");
        assert_eq!(palette.role("window").hex(), "#24273A");
        assert_eq!(notices.len(), 1);
        let (id, _, notices) = resolve(dir.path(), Some("archwave"), "kb-native", &toml::Table::new());
        assert_eq!(
            (id.as_str(), notices.len()),
            ("archwave", 0),
            "shipped themes load without a seeded folder"
        );
    }
}
