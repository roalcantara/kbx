//! What the screens show for an entry: labels, icons, the primary action and the metadata, as
//! plain data. Kept apart from rendering so the Raycast behaviour it reproduces is tested directly.

use kb_domain::entry::Entry;

/// A colour role among the seven accents of DESIGN.md. Rendering resolves it for the current
/// appearance (`accents::accent`), so badges and tags follow light and dark alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hue {
    /// Cheats; tags.
    Blue,
    /// Commands; tags.
    Green,
    /// Notes; tags.
    Orange,
    /// Bookmarks; tags.
    Purple,
    /// Tasks; tags.
    Magenta,
    /// Tags.
    Yellow,
    /// Tags; the notes' description bar.
    Red,
}

/// The order tags and unknown types hash into, as the Raycast extension's `accessories.ts`.
const TAG_ORDER: [Hue; 7] = [
    Hue::Blue,
    Hue::Orange,
    Hue::Green,
    Hue::Purple,
    Hue::Magenta,
    Hue::Yellow,
    Hue::Red,
];

/// The Raycast extension's hash: `acc × 31 + code point`, modulo 100 003, over the characters.
fn hash(value: &str) -> usize {
    value.chars().fold(0_usize, |acc, c| (acc * 31 + c as usize) % 100_003)
}

/// A tag's hue: its lowercased name hashed, so a tag keeps its colour everywhere.
#[must_use]
pub fn tag_hue(tag: &str) -> Hue {
    TAG_ORDER[hash(&tag.to_lowercase()) % TAG_ORDER.len()]
}

/// What assistive technology reads for a list row: its visible text, in reading order, because
/// a row's children do not name it. "docker ps -a, Commands, List every container…, tags docker,
/// containers".
#[must_use]
pub fn row_label(entry: &Entry, type_label: &str) -> String {
    let mut parts = vec![entry.title.clone(), type_label.to_owned()];
    parts.extend(entry.description.clone());
    if !entry.tags.is_empty() {
        parts.push(format!("tags {}", entry.tags.join(", ")));
    }
    if !entry.links.is_empty() {
        parts.push(format!("{} links", entry.links.len()));
    }
    parts.join(", ")
}

/// A bookmark's favicon from `DuckDuckGo`'s icon service (DESIGN.md, Adaptations: Raycast's own
/// service serves Raycast); `None` when the URL has no host.
#[must_use]
pub fn favicon_url(url: &str) -> Option<String> {
    let rest = url.split_once("://")?.1;
    let host = rest.split(['/', '?', '#']).next().filter(|host| !host.is_empty())?;
    Some(format!("https://icons.duckduckgo.com/ip3/{host}.ico"))
}

/// A URL as the Links list shows it (D20): no scheme, `www.`, query or fragment, no trailing
/// slash; longer than `max` characters, the host and the last path segment with `/…/` between.
#[must_use]
pub fn trim_url(url: &str, max: usize) -> String {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let rest = rest.split(['?', '#']).next().unwrap_or(rest);
    let rest = rest.strip_prefix("www.").unwrap_or(rest).trim_end_matches('/');
    if rest.chars().count() <= max {
        return rest.to_owned();
    }
    let mut parts = rest.split('/');
    let host = parts.next().unwrap_or(rest);
    match parts.next_back() {
        Some(last) => format!("{host}/…/{last}"),
        None => rest.to_owned(),
    }
}

/// The tags a row shows (at most `max`) and how many more there are.
#[must_use]
pub fn row_tags(tags: &[String], max: usize) -> (&[String], usize) {
    let shown = tags.len().min(max);
    (&tags[..shown], tags.len() - shown)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::fixture::entry;

    #[test]
    fn names_a_row_by_its_visible_text() {
        let command = entry(
            "commands",
            "docker ps -a",
            "desc: List containers.\ntags: [docker, ops]\n",
        );
        assert_eq!(
            row_label(&command, "Commands"),
            "docker ps -a, Commands, List containers., tags docker, ops"
        );
    }

    #[test]
    fn collapses_tags_after_three() {
        let tags: Vec<String> = ["a", "b", "c", "d", "e"].map(String::from).to_vec();
        assert_eq!(row_tags(&tags, 3), (&tags[..3], 2));
        assert_eq!(row_tags(&tags[..2], 3), (&tags[..2], 0));
    }

    #[test]
    fn asks_duckduckgo_for_a_bookmark_favicon() {
        assert_eq!(
            favicon_url("https://www.youtube.com/watch?v=x").as_deref(),
            Some("https://icons.duckduckgo.com/ip3/www.youtube.com.ico")
        );
        assert_eq!(favicon_url("not a url"), None);
        assert_eq!(favicon_url("https://"), None);
    }

    #[test]
    fn hashes_tags_as_the_raycast_extension_does() {
        let fixture: std::collections::BTreeMap<String, String> =
            serde_json::from_str(include_str!("../tests/fixtures/tag-accents.json")).unwrap();
        for (tag, accent) in fixture {
            assert_eq!(format!("{:?}", tag_hue(&tag)).to_lowercase(), accent, "#{tag}");
        }
        assert_eq!(tag_hue("Git"), tag_hue("git"));
    }

    #[test]
    fn trims_a_url_to_its_host_and_last_segment() {
        assert_eq!(
            trim_url("https://www.github.com/tokio-rs/mini-redis?tab=1#x", 32),
            "github.com/tokio-rs/mini-redis"
        );
        assert_eq!(
            trim_url("https://doc.rust-lang.org/std/task/struct.Waker.html", 32),
            "doc.rust-lang.org/…/struct.Waker.html"
        );
        assert_eq!(trim_url("https://tokio.rs/", 32), "tokio.rs");
    }
}
