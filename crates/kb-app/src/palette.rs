//! The ⌘K palette: the commands for the selected entry, as data, in the Raycast action panel's
//! order.
//!
//! A row runs an Action of [`crate::keys`], so its label, shortcut and result cannot drift from
//! the key binding. Commands that carry an argument (open this link, copy this text) exist only
//! in the palette and run directly.

use gpui_kit::Action;
use gpui_kit::assets::IconName;
use kb_domain::entry::Entry;
use kb_domain::search::FilterState;

use crate::keys::{
    CopyUrl, HideDetails, OpenDetail, OpenPreferences, OpenSourceFile, Quit, Rebuild, RunPrimary, ShowDetails,
    ToggleWindowSize,
};
use crate::presentation::{PrimaryKind, TypeKey, primary_action, type_key};

/// A page of the palette: the main actions, or one of the submenus a `›` row opens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Page {
    /// The main actions.
    #[default]
    Main,
    /// Open or copy each attached link.
    Links,
    /// Copy the title, description, notes or raw YAML.
    Copy,
}

/// What a palette row does when confirmed.
#[derive(Debug)]
pub enum PaletteCommand {
    /// Dispatch an Action from the panel, as its key binding does.
    Run(Box<dyn Action>),
    /// Open a URL in the browser.
    OpenUrl(String),
    /// Copy text to the clipboard.
    Copy(String),
    /// Replace the list's filter.
    Filter(FilterState),
    /// Show another page of the palette, keeping it open.
    OpenPage(Page),
}

/// One palette row.
#[derive(Debug)]
pub struct PaletteRow {
    /// Its label.
    pub label: String,
    /// Its icon.
    pub icon: IconName,
    /// What it does.
    pub command: PaletteCommand,
    /// Its keys, for the rows whose key is not a binding of their Action (Return and ⌘Return
    /// belong to the list); empty means the Action's binding, if any.
    pub keys: Vec<&'static str>,
}

/// One palette section; an empty label draws none.
#[derive(Debug)]
pub struct PaletteGroup {
    /// The section label.
    pub label: String,
    /// Its rows, never empty.
    pub rows: Vec<PaletteRow>,
}

fn row(label: impl Into<String>, icon: IconName, command: PaletteCommand) -> PaletteRow {
    PaletteRow {
        label: label.into(),
        icon,
        command,
        keys: Vec::new(),
    }
}

fn group(label: impl Into<String>, rows: Vec<PaletteRow>) -> PaletteGroup {
    PaletteGroup {
        label: label.into(),
        rows,
    }
}

/// Where the palette opens, which decides its first rows and their keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaletteContext {
    /// On the detail screen rather than the list.
    pub detail: bool,
    /// The split pane (list) or the metadata (detail screen) is shown.
    pub showing_details: bool,
}

/// The main page for the selected entry, in the Raycast action panel's order; without an entry,
/// only the administration commands.
#[must_use]
pub fn entry_palette(entry: Option<&Entry>, context: PaletteContext) -> Vec<PaletteGroup> {
    let mut groups = Vec::new();
    if let Some(entry) = entry {
        let primary = primary_action(entry);
        let icon = if primary.kind == PrimaryKind::OpenInBrowser {
            IconName::Globe
        } else {
            IconName::Copy
        };
        let run_primary = row(primary.title, icon, PaletteCommand::Run(Box::new(RunPrimary)));
        // In the list Return shows the details and ⌘Return runs the primary action; on the
        // detail screen Return runs it (Raycast's `EntryActionPanel`).
        groups.push(group(
            "",
            if context.detail {
                vec![PaletteRow {
                    keys: vec!["↵"],
                    ..run_primary
                }]
            } else {
                vec![
                    PaletteRow {
                        keys: vec!["↵"],
                        ..row("Show Details", IconName::Eye, PaletteCommand::Run(Box::new(OpenDetail)))
                    },
                    PaletteRow {
                        keys: vec!["⌘", "↵"],
                        ..run_primary
                    },
                ]
            },
        ));

        let mut source = Vec::new();
        if type_key(&entry.entry_type) == TypeKey::Bookmark {
            source.push(row("Copy URL", IconName::Copy, PaletteCommand::Run(Box::new(CopyUrl))));
        }
        // "Details Pane", so the row reads apart from Show Details (↵), which opens the detail screen.
        let noun = if context.detail { "Metadata" } else { "Details Pane" };
        // The row carries → or ←, as Raycast's; ⌘D toggles too, unlisted.
        source.push(if context.showing_details {
            row(
                format!("Hide {noun}"),
                IconName::PanelRight,
                PaletteCommand::Run(Box::new(HideDetails)),
            )
        } else {
            row(
                format!("Show {noun}"),
                IconName::PanelRight,
                PaletteCommand::Run(Box::new(ShowDetails)),
            )
        });
        source.push(row(
            "Open Source File",
            IconName::FolderOpen,
            PaletteCommand::Run(Box::new(OpenSourceFile)),
        ));
        groups.push(group("Source & Clipboard", source));

        let mut pages = Vec::new();
        if !entry.links.is_empty() {
            pages.push(row("Links", IconName::Link, PaletteCommand::OpenPage(Page::Links)));
        }
        pages.push(row("Copy…", IconName::Copy, PaletteCommand::OpenPage(Page::Copy)));
        groups.push(group("", pages));
    }
    groups.push(group(
        "Administration",
        vec![
            row(
                "Rebuild Index…",
                IconName::RefreshCw,
                PaletteCommand::Run(Box::new(Rebuild)),
            ),
            row(
                "Open Preferences",
                IconName::Settings,
                PaletteCommand::Run(Box::new(OpenPreferences)),
            ),
            row(
                "Toggle Window Size",
                IconName::Maximize,
                PaletteCommand::Run(Box::new(ToggleWindowSize)),
            ),
            row("Quit kb", IconName::Power, PaletteCommand::Run(Box::new(Quit))),
        ],
    ));
    groups
}

/// A submenu page for `entry`: each link with its open and copy rows, or the copy rows.
#[must_use]
pub fn page(entry: &Entry, page: Page) -> Vec<PaletteGroup> {
    match page {
        Page::Main => Vec::new(),
        Page::Links => entry
            .links
            .iter()
            .map(|link| {
                let name = link.title.as_deref().filter(|t| !t.is_empty()).unwrap_or(&link.url);
                group(
                    name,
                    vec![
                        row(
                            format!("Open {name}"),
                            IconName::ExternalLink,
                            PaletteCommand::OpenUrl(link.url.clone()),
                        ),
                        row("Copy URL", IconName::Copy, PaletteCommand::Copy(link.url.clone())),
                    ],
                )
            })
            .collect(),
        Page::Copy => {
            let mut rows = vec![row(
                "Copy Title",
                IconName::Copy,
                PaletteCommand::Copy(entry.title.clone()),
            )];
            if let Some(description) = &entry.description {
                rows.push(row(
                    "Copy Description",
                    IconName::Copy,
                    PaletteCommand::Copy(description.clone()),
                ));
            }
            if !entry.notes_markdown.is_empty() {
                rows.push(row(
                    "Copy Notes",
                    IconName::Copy,
                    PaletteCommand::Copy(entry.notes_markdown.clone()),
                ));
            }
            rows.push(row(
                "Copy Raw YAML",
                IconName::Copy,
                PaletteCommand::Copy(entry.raw_yaml.clone()),
            ));
            vec![group("Copy…", rows)]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::entry;

    const LIST: PaletteContext = PaletteContext {
        detail: false,
        showing_details: false,
    };

    fn labels(groups: &[PaletteGroup]) -> Vec<(&str, Vec<&str>)> {
        groups
            .iter()
            .map(|g| (g.label.as_str(), g.rows.iter().map(|r| r.label.as_str()).collect()))
            .collect()
    }

    fn bookmark() -> Entry {
        entry(
            "bookmarks",
            "https://b.example",
            "desc: D\nlinks: [{Guide: https://g.example}]\n",
        )
    }

    const ADMINISTRATION: [&str; 4] = ["Rebuild Index…", "Open Preferences", "Toggle Window Size", "Quit kb"];

    #[test]
    fn offers_the_raycast_actions_for_a_bookmark_with_links() {
        assert_eq!(
            labels(&entry_palette(Some(&bookmark()), LIST)),
            [
                ("", vec!["Show Details", "Open in Browser"]),
                (
                    "Source & Clipboard",
                    vec!["Copy URL", "Show Details Pane", "Open Source File"]
                ),
                ("", vec!["Links", "Copy…"]),
                ("Administration", ADMINISTRATION.to_vec()),
            ]
        );
    }

    #[test]
    fn gives_return_and_cmd_return_to_the_first_rows() {
        let list = entry_palette(Some(&bookmark()), LIST);
        assert_eq!(
            (list[0].rows[0].keys.clone(), list[0].rows[1].keys.clone()),
            (vec!["↵"], vec!["⌘", "↵"])
        );
        let detail = entry_palette(
            Some(&bookmark()),
            PaletteContext {
                detail: true,
                showing_details: true,
            },
        );
        assert_eq!(labels(&detail)[0], ("", vec!["Open in Browser"]));
        assert_eq!(detail[0].rows[0].keys, ["↵"]);
        assert_eq!(labels(&detail)[1].1[1], "Hide Metadata");
    }

    #[test]
    fn opens_copy_and_links_as_pages() {
        let bookmark = bookmark();
        assert_eq!(
            labels(&page(&bookmark, Page::Copy)),
            [(
                "Copy…",
                vec!["Copy Title", "Copy Description", "Copy Notes", "Copy Raw YAML"]
            )]
        );
        assert_eq!(
            labels(&page(&bookmark, Page::Links)),
            [("Guide", vec!["Open Guide", "Copy URL"])]
        );
    }

    #[test]
    fn leaves_out_what_an_entry_does_not_have() {
        let command = entry("commands", "ls", "title: List\n");
        let groups = entry_palette(
            Some(&command),
            PaletteContext {
                detail: false,
                showing_details: true,
            },
        );
        assert_eq!(
            labels(&groups)[1],
            ("Source & Clipboard", vec!["Hide Details Pane", "Open Source File"])
        );
        assert_eq!(labels(&groups)[2], ("", vec!["Copy…"]));
    }

    #[test]
    fn offers_only_administration_without_a_selection() {
        assert_eq!(
            labels(&entry_palette(None, LIST)),
            [("Administration", ADMINISTRATION.to_vec())]
        );
    }
}
