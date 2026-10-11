//! Tag images (DESIGN.md, Tag pill): a tag is drawn as `<tag>.svg` when the tag-images folder
//! has that file, as in the Raycast extension. The folder is read once, when the panel opens.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The SVGs of the tag-images folder, by lowercased tag.
#[derive(Debug, Clone, Default)]
pub struct TagImages(HashMap<String, PathBuf>);

impl TagImages {
    /// Reads `dir`; a missing or unreadable folder means no images, not an error.
    #[must_use]
    pub fn scan(dir: &Path) -> Self {
        let entries = std::fs::read_dir(dir).into_iter().flatten().flatten();
        Self(
            entries
                .filter_map(|entry| {
                    let path = entry.path();
                    if path.extension()? != "svg" {
                        return None;
                    }
                    let tag = path.file_stem()?.to_str()?.to_lowercase();
                    Some((tag, path))
                })
                .collect(),
        )
    }

    /// The image for `tag`, if any.
    #[must_use]
    pub fn get(&self, tag: &str) -> Option<&Path> {
        self.0.get(&tag.to_lowercase()).map(PathBuf::as_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_tag_image_whatever_the_case_and_ignores_other_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("linux.svg"), "<svg/>").unwrap();
        std::fs::write(dir.path().join("notes.txt"), "").unwrap();
        let images = TagImages::scan(dir.path());
        assert_eq!(images.get("Linux"), Some(dir.path().join("linux.svg").as_path()));
        assert_eq!(images.get("notes"), None);
    }

    #[test]
    fn a_missing_folder_means_no_images() {
        assert_eq!(TagImages::scan(Path::new("/nonexistent")).get("linux"), None);
    }
}
