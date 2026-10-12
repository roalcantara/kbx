//! What the screens show for an entry: labels, icons, the primary action and the metadata, as
//! plain data. Kept apart from rendering so the Raycast behaviour it reproduces is tested directly.

use gpui_kit::assets::IconName;
use kb_domain::entry::Entry;

/// The notes pane's text when an entry has no notes.
pub const NO_NOTES: &str = "*No additional notes available for this entry.*";

/// Tags shown on a list row before the rest collapse into `+N`.
pub const MAX_ROW_TAGS: usize = 3;

/// An entry type, singular, as the actions and icons distinguish it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub struct TypeBadge {
    /// The type's icon, as Raycast's.
    pub icon: IconName,
    /// The type's hue.
    pub hue: Hue,
}

/// The badge for an entry type: Raycast's icon and tint; an unknown type hashes its singular
/// name, as the Raycast extension's `typeColor`.
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

/// What the primary action does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryKind {
    /// Open the content in the browser.
    OpenInBrowser,
    /// Copy the content.
    Copy,
}

/// An entry type's primary action, as data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimaryAction {
    /// What it does.
    pub kind: PrimaryKind,
    /// Its label.
    pub title: &'static str,
    /// What it opens or copies.
    pub content: String,
}

/// A bookmark opens; commands, cheats, notes and tasks copy.
#[must_use]
pub fn primary_action(entry: &Entry) -> PrimaryAction {
    let (kind, title) = match type_key(&entry.entry_type) {
        TypeKey::Bookmark => (PrimaryKind::OpenInBrowser, "Open in Browser"),
        TypeKey::Command => (PrimaryKind::Copy, "Copy Command Snippet"),
        TypeKey::Cheat => (PrimaryKind::Copy, "Copy Cheat Sheet"),
        TypeKey::Note => (PrimaryKind::Copy, "Copy Notes"),
        TypeKey::Task => (PrimaryKind::Copy, "Copy Task"),
        TypeKey::Other => (PrimaryKind::Copy, "Copy Primary Content"),
    };
    PrimaryAction {
        kind,
        title,
        content: entry.primary_content.clone(),
    }
}

/// What assistive technology reads for a list row: its visible text, in reading order, because
/// a row's children do not name it. "docker ps -a, Commands, List every container…, tags docker,
/// containers".
#[must_use]
pub fn row_label(entry: &Entry) -> String {
    let mut parts = vec![entry.title.clone(), type_label(&entry.entry_type)];
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

/// The tags a row shows and how many more there are.
#[must_use]
pub fn row_tags(tags: &[String]) -> (&[String], usize) {
    let shown = tags.len().min(MAX_ROW_TAGS);
    (&tags[..shown], tags.len() - shown)
}

/// The Markdown the notes pane renders.
#[must_use]
pub fn notes_markdown(entry: &Entry) -> &str {
    if entry.notes_markdown.is_empty() {
        NO_NOTES
    } else {
        &entry.notes_markdown
    }
}

/// The notes Markdown split around the description quote, which the pane draws itself with the
/// red bar of DESIGN.md (`TextView` draws every quote bar in the border colour).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotesParts<'a> {
    /// What precedes the quote: the thumbnail, a command's code block.
    pub before: &'a str,
    /// The description, when the Markdown carries its quote.
    pub description: Option<&'a str>,
    /// The rest: notes, links, source.
    pub after: &'a str,
}

/// Finds the quote `kb_domain::markdown` writes for `description` (each line prefixed `> `);
/// without it, the Markdown stays whole in `after` and keeps its own quote.
#[must_use]
pub fn split_description<'a>(markdown: &'a str, description: Option<&'a str>) -> NotesParts<'a> {
    let whole = NotesParts {
        before: "",
        description: None,
        after: markdown,
    };
    let Some(description) = description.map(str::trim).filter(|d| !d.is_empty()) else {
        return whole;
    };
    let quote = description.lines().fold(String::new(), |mut quote, line| {
        quote.push_str("> ");
        quote.push_str(line);
        quote.push('\n');
        quote
    });
    match markdown.find(&quote) {
        Some(at) => NotesParts {
            before: &markdown[..at],
            description: Some(description),
            after: &markdown[at + quote.len()..],
        },
        None => whole,
    }
}

/// The metadata pane's rows, in the Raycast order: target, type, workflow, description, tags,
/// links, source.
#[must_use]
pub fn metadata(entry: &Entry) -> Vec<(&'static str, String)> {
    let mut rows = Vec::new();
    // A bookmark's URL; a command has none, as the notes show it as a code block.
    if type_key(&entry.entry_type) == TypeKey::Bookmark {
        rows.push(("URL", entry.primary_content.clone()));
    }
    rows.push(("Type", type_label(&entry.entry_type)));
    if let Some(status) = &entry.task_status {
        rows.push(("Status", status.clone()));
    }
    if let Some(priority) = &entry.task_priority {
        rows.push(("Priority", priority.clone()));
    }
    // No Description row: the notes open with it, as a quote.
    if !entry.tags.is_empty() {
        rows.push((
            "Tags",
            entry.tags.iter().map(|t| format!("#{t}")).collect::<Vec<_>>().join(" "),
        ));
    }
    // One link by its title; several by their count, which the Links page of the actions lists.
    match entry.links.as_slice() {
        [] => {}
        [link] => rows.push(("Links", link.title.clone().unwrap_or_else(|| link.url.clone()))),
        links => rows.push(("Links", format!("{} links", links.len()))),
    }
    rows.push(("Source", entry.source_rel_path.clone()));
    rows
}

#[cfg(test)]
mod tests {
    use kb_domain::entry::EntryLink;

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
    fn matches_the_raycast_primary_action_matrix() {
        let cases = [
            ("bookmarks", PrimaryKind::OpenInBrowser, "Open in Browser"),
            ("commands", PrimaryKind::Copy, "Copy Command Snippet"),
            ("cheats", PrimaryKind::Copy, "Copy Cheat Sheet"),
            ("notes", PrimaryKind::Copy, "Copy Notes"),
            ("tasks", PrimaryKind::Copy, "Copy Task"),
            ("snippets", PrimaryKind::Copy, "Copy Primary Content"),
        ];
        for (entry_type, kind, title) in cases {
            let action = primary_action(&entry(entry_type, "key", "title: t\n"));
            assert_eq!(
                (action.kind, action.title, action.content.as_str()),
                (kind, title, "key")
            );
        }
    }

    #[test]
    fn names_a_row_by_its_visible_text() {
        let command = entry(
            "commands",
            "docker ps -a",
            "desc: List containers.\ntags: [docker, ops]\n",
        );
        assert_eq!(
            row_label(&command),
            "docker ps -a, Commands, List containers., tags docker, ops"
        );
    }

    #[test]
    fn collapses_tags_after_three() {
        let tags: Vec<String> = ["a", "b", "c", "d", "e"].map(String::from).to_vec();
        assert_eq!(row_tags(&tags), (&tags[..3], 2));
        assert_eq!(row_tags(&tags[..2]), (&tags[..2], 0));
    }

    #[test]
    fn falls_back_to_a_placeholder_without_notes() {
        let mut bare = entry("notes", "k", "title: t\n");
        bare.notes_markdown.clear();
        assert_eq!(notes_markdown(&bare), NO_NOTES);
    }

    #[test]
    fn leaves_a_command_to_the_notes_code_block() {
        let command = entry("commands", "ls -la", "desc: List all\n");
        let labels: Vec<&str> = metadata(&command).iter().map(|(label, _)| *label).collect();
        assert_eq!(labels, ["Type", "Source"]);
    }

    #[test]
    fn lists_metadata_in_raycast_order_without_the_description_the_notes_quote() {
        let mut task = entry(
            "tasks",
            "port",
            "desc: Port it\nstatus: todo\npriority: high\ntags: [rust, kb]\nlinks: [{Spec: https://s.example}]\n",
        );
        task.links.push(EntryLink {
            title: None,
            url: "https://u.example".into(),
        });
        let labels: Vec<&str> = metadata(&task).iter().map(|(label, _)| *label).collect();
        assert_eq!(labels, ["Type", "Status", "Priority", "Tags", "Links", "Source"]);
        assert_eq!(metadata(&task)[3].1, "#rust #kb");
        assert_eq!(metadata(&task)[4].1, "2 links", "the Links page lists them");
        assert_eq!(
            metadata(&entry("bookmarks", "https://b.example", "title: B\n"))[0],
            ("URL", "https://b.example".into())
        );
    }

    #[test]
    fn splits_the_notes_around_the_description_quote() {
        let bookmark = entry(
            "bookmarks",
            "https://youtu.be/aqz-KE-bpKQ",
            "desc: A talk\nnotes: Watch it.\n",
        );
        let parts = split_description(&bookmark.notes_markdown, bookmark.description.as_deref());
        assert!(parts.before.starts_with("![]("), "{}", parts.before);
        assert_eq!(parts.description, Some("A talk"));
        assert!(parts.after.contains("Watch it."), "{}", parts.after);
        assert!(!parts.after.contains("> A talk"));
    }

    #[test]
    fn keeps_the_notes_whole_without_a_description() {
        let note = entry("notes", "n", "notes: Hi\n");
        let parts = split_description(&note.notes_markdown, None);
        assert_eq!((parts.before, parts.description), ("", None));
        assert_eq!(parts.after, note.notes_markdown);
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
}
