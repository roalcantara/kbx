//! The app's assets: GPUI Kit's default component icons, plus the bundled icons kb draws that
//! the default set leaves out. An icon missing from both draws as nothing, silently.

use std::borrow::Cow;

use gpui_kit::assets::{Assets, icon_assets};
use gpui_kit::{AssetSource, Result, SharedString};

icon_assets!(ExtraIcons, [ArrowUpRight, Circle, Funnel, Link, NotebookText, Power]);

/// GPUI Kit's default assets, then kb's extra icons.
#[derive(Debug, Clone, Copy, Default)]
pub struct KbAssets;

impl AssetSource for KbAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        // The default bundle answers a path it does not embed with an error, not `None`.
        match Assets.load(path) {
            Ok(Some(bytes)) => Ok(Some(bytes)),
            _ => ExtraIcons.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = Assets.list(path).unwrap_or_default();
        paths.extend(ExtraIcons.list(path)?);
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `IconName::FooBar` as its asset path, `icons/foo-bar.svg`.
    fn icon_path(name: &str) -> String {
        let mut kebab = String::new();
        for (ix, c) in name.char_indices() {
            if ix > 0
                && (c.is_ascii_uppercase()
                    || (c.is_ascii_digit() && !name[..ix].ends_with(|p: char| p.is_ascii_digit())))
            {
                kebab.push('-');
            }
            kebab.push(c.to_ascii_lowercase());
        }
        format!("icons/{kebab}.svg")
    }

    #[test]
    fn loads_every_icon_the_app_names() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut names = std::collections::BTreeSet::new();
        for file in walk(&src) {
            let text = std::fs::read_to_string(&file).unwrap();
            for (at, _) in text.match_indices("IconName::") {
                let name: String = text[at + "IconName::".len()..]
                    .chars()
                    .take_while(char::is_ascii_alphanumeric)
                    .collect();
                if !name.is_empty() && name != "FooBar" {
                    names.insert(name);
                }
            }
        }
        let missing: Vec<_> = names
            .iter()
            .filter(|name| !matches!(KbAssets.load(&icon_path(name)), Ok(Some(_))))
            .collect();
        assert!(missing.is_empty(), "icons that would draw as nothing: {missing:?}");
    }

    fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .flat_map(|entry| {
                let path = entry.path();
                if path.is_dir() { walk(&path) } else { vec![path] }
            })
            .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
            .collect()
    }
}
