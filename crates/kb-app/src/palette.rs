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

use crate::keys::{CopyUrl, HideDetails, OpenPreferences, OpenSourceFile, Quit, Rebuild, RunPrimary, ShowDetails};
use crate::presentation::{PrimaryKind, TypeKey, primary_action, type_key};

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
}

/// One labelled palette section.
#[derive(Debug)]
pub struct PaletteGroup {
    /// The section label.
    pub label: &'static str,
    /// Its rows, never empty.
    pub rows: Vec<PaletteRow>,
}

fn row(label: impl Into<String>, icon: IconName, command: PaletteCommand) -> PaletteRow {
    PaletteRow {
        label: label.into(),
        icon,
        command,
    }
}

/// The palette for the selected entry, in the Raycast action panel's order; without an entry,
/// only the administration commands.
#[must_use]
pub fn entry_palette(entry: Option<&Entry>, showing_details: bool) -> Vec<PaletteGroup> {
    let mut groups = Vec::new();
    if let Some(entry) = entry {
        let primary = primary_action(entry);
        let icon = if primary.kind == PrimaryKind::OpenInBrowser {
            IconName::Globe
        } else {
            IconName::Copy
        };
        groups.push(PaletteGroup {
            label: "Entry",
            rows: vec![row(primary.title, icon, PaletteCommand::Run(Box::new(RunPrimary)))],
        });

        let mut source = Vec::new();
        if type_key(&entry.entry_type) == TypeKey::Bookmark {
            source.push(row("Copy URL", IconName::Copy, PaletteCommand::Run(Box::new(CopyUrl))));
        }
        // The row carries → or ←, as Raycast's; ⌘D toggles too, unlisted.
        source.push(if showing_details {
            row(
                "Hide Details",
                IconName::PanelRight,
                PaletteCommand::Run(Box::new(HideDetails)),
            )
        } else {
            row(
                "Show Details",
                IconName::PanelRight,
                PaletteCommand::Run(Box::new(ShowDetails)),
            )
        });
        source.push(row(
            "Open Source File",
            IconName::FolderOpen,
            PaletteCommand::Run(Box::new(OpenSourceFile)),
        ));
        groups.push(PaletteGroup {
            label: "Source & Clipboard",
            rows: source,
        });

        if !entry.links.is_empty() {
            let mut links = Vec::new();
            for link in &entry.links {
                let name = link.title.as_deref().unwrap_or(&link.url);
                links.push(row(
                    format!("Open {name}"),
                    IconName::ExternalLink,
                    PaletteCommand::OpenUrl(link.url.clone()),
                ));
                links.push(row(
                    format!("Copy URL of {name}"),
                    IconName::Copy,
                    PaletteCommand::Copy(link.url.clone()),
                ));
            }
            groups.push(PaletteGroup {
                label: "Links",
                rows: links,
            });
        }

        let mut copy = vec![row(
            "Copy Title",
            IconName::Copy,
            PaletteCommand::Copy(entry.title.clone()),
        )];
        if let Some(description) = &entry.description {
            copy.push(row(
                "Copy Description",
                IconName::Copy,
                PaletteCommand::Copy(description.clone()),
            ));
        }
        if !entry.notes_markdown.is_empty() {
            copy.push(row(
                "Copy Notes",
                IconName::Copy,
                PaletteCommand::Copy(entry.notes_markdown.clone()),
            ));
        }
        groups.push(PaletteGroup {
            label: "Copy",
            rows: copy,
        });
    }
    groups.push(PaletteGroup {
        label: "Administration",
        rows: vec![
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
            row("Quit kb", IconName::Power, PaletteCommand::Run(Box::new(Quit))),
        ],
    });
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::entry;

    fn labels(groups: &[PaletteGroup]) -> Vec<(&str, Vec<&str>)> {
        groups
            .iter()
            .map(|g| (g.label, g.rows.iter().map(|r| r.label.as_str()).collect()))
            .collect()
    }

    #[test]
    fn offers_the_raycast_actions_for_a_bookmark_with_links() {
        let bookmark = entry(
            "bookmarks",
            "https://b.example",
            "desc: D\nlinks: [{Guide: https://g.example}]\n",
        );
        assert_eq!(
            labels(&entry_palette(Some(&bookmark), false)),
            [
                ("Entry", vec!["Open in Browser"]),
                (
                    "Source & Clipboard",
                    vec!["Copy URL", "Show Details", "Open Source File"]
                ),
                ("Links", vec!["Open Guide", "Copy URL of Guide"]),
                ("Copy", vec!["Copy Title", "Copy Description", "Copy Notes"]),
                ("Administration", vec!["Rebuild Index…", "Open Preferences", "Quit kb"]),
            ]
        );
    }

    #[test]
    fn leaves_out_what_an_entry_does_not_have() {
        let command = entry("commands", "ls", "title: List\n");
        let groups = entry_palette(Some(&command), true);
        assert_eq!(
            labels(&groups)[1],
            ("Source & Clipboard", vec!["Hide Details", "Open Source File"])
        );
        assert!(groups.iter().all(|g| g.label != "Links"));
    }

    #[test]
    fn offers_only_administration_without_a_selection() {
        assert_eq!(
            labels(&entry_palette(None, false)),
            [("Administration", vec!["Rebuild Index…", "Open Preferences", "Quit kb"])]
        );
    }
}
