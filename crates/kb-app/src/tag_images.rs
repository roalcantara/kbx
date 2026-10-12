//! Tag images (DESIGN.md, Tag pill): a tag is drawn as `<tag>.svg` when the tag-images folder
//! has that file, as in the Raycast extension. The folder is read once, when the panel opens.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A tag's image: its file, and whether it carries its own colours (a brand's) or is a shape to
/// draw in the text colour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagImage {
    /// The SVG file.
    pub path: PathBuf,
    /// Whether the SVG paints in colours of its own; black, `none` and `currentColor` do not count.
    pub coloured: bool,
}

/// The SVGs of the tag-images folder, by lowercased tag.
#[derive(Debug, Clone, Default)]
pub struct TagImages(HashMap<String, TagImage>);

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
                    let coloured = std::fs::read_to_string(&path).is_ok_and(|svg| is_coloured(&svg));
                    Some((tag, TagImage { path, coloured }))
                })
                .collect(),
        )
    }

    /// The image for `tag`, if any.
    #[must_use]
    pub fn image(&self, tag: &str) -> Option<&TagImage> {
        self.0.get(&tag.to_lowercase())
    }
}

/// Whether `svg` paints with a colour of its own: a raster image, or a `fill`, `stroke` or
/// `stop-color` (attribute or style) other than black, `none`, `currentColor` and the like.
fn is_coloured(svg: &str) -> bool {
    if svg.contains("<image") {
        return true;
    }
    ["fill", "stroke", "stop-color"].iter().any(|property| {
        svg.match_indices(property).any(|(at, _)| {
            let value = svg[at + property.len()..]
                .trim_start_matches(['=', ':', ' ', '"', '\''])
                .split(['"', '\'', ';', ')', ' ', '>'])
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase();
            paints(&value)
        })
    })
}

/// Whether a colour value paints something other than black or nothing.
fn paints(value: &str) -> bool {
    match value {
        "" | "none" | "currentcolor" | "transparent" | "inherit" | "black" => false,
        hex if hex.starts_with('#') => {
            let digits = &hex[1..];
            let channels: Vec<u8> = match digits.len() {
                3 | 4 => digits
                    .chars()
                    .take(3)
                    .filter_map(|c| u8::from_str_radix(&c.to_string(), 16).ok())
                    .map(|v| v * 17)
                    .collect(),
                6 | 8 => (0..3)
                    .filter_map(|i| u8::from_str_radix(digits.get(i * 2..i * 2 + 2)?, 16).ok())
                    .collect(),
                _ => return false,
            };
            channels.iter().any(|&c| c > 0x30)
        }
        other => {
            other.starts_with("rgb") || other.starts_with("url(") || other.chars().all(|c| c.is_ascii_alphabetic())
        }
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
        assert_eq!(
            images.image("Linux").map(|i| i.path.clone()),
            Some(dir.path().join("linux.svg"))
        );
        assert_eq!(images.image("notes"), None);
    }

    #[test]
    fn tells_a_coloured_image_from_a_monochrome_one() {
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, svg: &str| std::fs::write(dir.path().join(name), svg).unwrap();
        write("brand.svg", r##"<svg><path fill="#FF9900" d="M0 0"/></svg>"##);
        write(
            "styled.svg",
            r#"<svg><path style="fill: rgb(1, 2, 3)" d="M0 0"/></svg>"#,
        );
        write(
            "lucide.svg",
            r#"<svg stroke="currentColor" fill="none"><path d="M0 0"/></svg>"#,
        );
        write("plain.svg", r#"<svg><path d="M0 0"/></svg>"#);
        write(
            "black.svg",
            r##"<svg fill="#000000"><path stroke="#000" d="M0 0"/></svg>"##,
        );
        let images = TagImages::scan(dir.path());
        let coloured = |tag| images.image(tag).map(|image| image.coloured);
        assert_eq!(coloured("brand"), Some(true));
        assert_eq!(coloured("styled"), Some(true));
        assert_eq!(coloured("lucide"), Some(false));
        assert_eq!(coloured("plain"), Some(false));
        assert_eq!(coloured("black"), Some(false), "black would vanish on the dark window");
    }

    #[test]
    fn a_missing_folder_means_no_images() {
        assert_eq!(TagImages::scan(Path::new("/nonexistent")).image("linux"), None);
    }
}
