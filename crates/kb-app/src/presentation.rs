//! What the screens show for an entry: labels, icons, the primary action and the metadata, as
//! plain data. Kept apart from rendering so the Raycast behaviour it reproduces is tested directly.

use gpui_kit::assets::IconName;
use kb_domain::entry::Entry;

/// An entry type, singular, as the actions and icons distinguish it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), expect(dead_code, reason = "kept for type categorization in tests"))]
pub enum TypeKey {
    /// `bookmarks`
    Bookmark,
    /// `commands`
    Command,
    /// `cheats`
    Cheat,
    /// `notes`
    Note,
    /// `tasks`
    Task,
    /// Any other type.
    Other,
}

/// The type, trimmed, lowercased and singular: `Bookmarks` → `Bookmark`.
#[cfg_attr(not(test), expect(dead_code, reason = "kept for type categorization in tests"))]
#[must_use]
pub fn type_key(entry_type: &str) -> TypeKey {
    let normalized = entry_type.trim().to_lowercase();
    match normalized.strip_suffix('s').unwrap_or(&normalized) {
        "bookmark" => TypeKey::Bookmark,
        "command" => TypeKey::Command,
        "cheat" => TypeKey::Cheat,
        "note" => TypeKey::Note,
        "task" => TypeKey::Task,
        _ => TypeKey::Other,
    }
}

/// The type as people read it: the known names, else the type with its first letter capitalized.
#[must_use]
pub fn type_label(entry_type: &str) -> String {
    let normalized = entry_type.trim().to_lowercase();
    let known = match normalized.as_str() {
        "bookmarks" => "Bookmarks",
        "bookmark" => "Bookmark",
        "commands" => "Commands",
        "command" => "Command",
        "cheats" => "Cheats",
        "cheat" => "Cheat",
        "notes" => "Notes",
        "note" => "Note",
        "tasks" => "Tasks",
        "task" => "Task",
        _ => {
            let mut chars = normalized.chars();
            return chars
                .next()
                .map_or_else(String::new, |first| first.to_uppercase().chain(chars).collect());
        }
    };
    known.to_owned()
}

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

/// The tinted icon that leads a list row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), expect(dead_code, reason = "kept for type_badge parity"))]
pub struct TypeBadge {
    /// The type's icon, as Raycast's.
    pub icon: IconName,
    /// The type's hue.
    pub hue: Hue,
}

/// The badge for an entry type: Raycast's icon and tint; an unknown type hashes its singular
/// name, as the Raycast extension's `typeColor`.
#[cfg_attr(not(test), expect(dead_code, reason = "kept for type_badge parity"))]
#[must_use]
pub fn type_badge(entry_type: &str) -> TypeBadge {
    let (icon, hue) = match type_key(entry_type) {
        TypeKey::Bookmark => (IconName::Globe, Hue::Purple),
        TypeKey::Command => (IconName::SquareTerminal, Hue::Green),
        TypeKey::Cheat => (IconName::FileText, Hue::Blue),
        TypeKey::Note => (IconName::NotebookText, Hue::Orange),
        TypeKey::Task => (IconName::CircleCheck, Hue::Magenta),
        TypeKey::Other => {
            let name = entry_type.trim().to_lowercase();
            (IconName::FileText, tag_hue(name.strip_suffix('s').unwrap_or(&name)))
        }
    };
    TypeBadge { icon, hue }
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
    fn reads_types_singular_and_case_insensitively() {
        assert_eq!(type_key(" Bookmarks "), TypeKey::Bookmark);
        assert_eq!(type_key("task"), TypeKey::Task);
        assert_eq!(type_key("snippets"), TypeKey::Other);
        assert_eq!(type_label("commands"), "Commands");
        assert_eq!(type_label("snippets"), "Snippets");
        assert_eq!(type_label(""), "");
    }

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
    fn tints_each_type_as_raycast() {
        assert_eq!(
            type_badge("bookmarks"),
            TypeBadge {
                icon: IconName::Globe,
                hue: Hue::Purple
            }
        );
        assert_eq!(
            type_badge("Commands"),
            TypeBadge {
                icon: IconName::SquareTerminal,
                hue: Hue::Green
            }
        );
        assert_eq!(
            type_badge("cheats"),
            TypeBadge {
                icon: IconName::FileText,
                hue: Hue::Blue
            }
        );
        assert_eq!(
            type_badge("notes"),
            TypeBadge {
                icon: IconName::NotebookText,
                hue: Hue::Orange
            }
        );
        assert_eq!(
            type_badge("tasks"),
            TypeBadge {
                icon: IconName::CircleCheck,
                hue: Hue::Magenta
            }
        );
        assert_eq!(type_badge("snippets").hue, tag_hue("snippet"));
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
