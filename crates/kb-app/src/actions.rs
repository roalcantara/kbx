//! The action panel's pages for a screen and an entry, from the catalog (D9), and the filter's
//! page (D15): each row paired with what it does.

use kb_domain::entry::Entry;
use kb_domain::markdown::notes_body;
use kb_domain::search::FilterState;

use crate::catalog::{Catalog, Screen, fill};
use crate::presentation::{tag_hue, trim_url};
use crate::tag_images::TagImages;
use crate::ui::action_panel::{PanelPage, PanelRow, PanelSection, RowLead};
use crate::ui::design::design;
use crate::ui::keycap::caps;

/// A submenu of the root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Submenu {
    /// One row per link.
    Links,
    /// COPY and ADMINISTRATION.
    More,
}

/// What a row does when confirmed (↵, or ⌘↵ where the screen gives ⌘↵ a meaning).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PanelCommand {
    /// Run a catalog action on the selected entry.
    Action(String),
    /// Open a submenu in place.
    Open(Submenu),
    /// A link: ↵ opens it, ⌘↵ copies it (the Links screen's keys).
    Link(String),
    /// Copy this text.
    Copy(String),
    /// A filter row: ↵ gives `toggle` and keeps the filter open; ⌘↵ gives `only` and closes it.
    Filter {
        /// The filter after toggling the row.
        toggle: FilterState,
        /// The filter with this choice alone.
        only: FilterState,
    },
}

/// A page and, section by section, what each row does.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Page {
    /// What the panel draws.
    pub page: PanelPage,
    /// What each row does, by section then row.
    pub commands: Vec<Vec<PanelCommand>>,
}

impl Page {
    fn section(&mut self, label: Option<String>, rows: Vec<(PanelRow, PanelCommand)>) {
        if rows.is_empty() {
            return;
        }
        let (rows, commands): (Vec<_>, Vec<_>) = rows.into_iter().unzip();
        self.page.sections.push(PanelSection { label, rows });
        self.commands.push(commands);
    }

    /// The command of row (`section`, `row`).
    #[must_use]
    pub fn command(&self, section: usize, row: usize) -> Option<&PanelCommand> {
        self.commands.get(section)?.get(row)
    }
}

fn action_row(catalog: &Catalog, id: &str) -> (PanelRow, PanelCommand) {
    (
        PanelRow {
            label: catalog.label(id).to_owned(),
            caps: catalog.actions.get(id).and_then(|a| a.keys.as_deref()).map(caps),
            ..PanelRow::default()
        },
        PanelCommand::Action(id.to_owned()),
    )
}

/// The root: the screen's keys, the catalog's root rows, Links… (with links) and More….
#[must_use]
pub fn root_page(catalog: &Catalog, screen: Screen, entry: Option<&Entry>) -> Page {
    let kind = entry.and_then(|e| catalog.entry_type(&e.entry_type));
    let mut rows: Vec<(PanelRow, PanelCommand)> = catalog
        .keys_for(screen, kind)
        .into_iter()
        .filter(|item| entry.is_some() || item.action != "details")
        .map(|item| {
            (
                PanelRow {
                    label: item.label,
                    caps: Some(item.caps),
                    ..PanelRow::default()
                },
                PanelCommand::Action(item.action),
            )
        })
        .collect();
    rows.extend(catalog.panel.root.iter().map(|id| action_row(catalog, id)));
    if entry.is_some_and(|e| !e.links.is_empty()) {
        rows.push((
            PanelRow {
                label: catalog.panel.links.clone(),
                submenu: true,
                ..PanelRow::default()
            },
            PanelCommand::Open(Submenu::Links),
        ));
    }
    rows.push((
        PanelRow {
            label: catalog.panel.more.clone(),
            submenu: true,
            ..PanelRow::default()
        },
        PanelCommand::Open(Submenu::More),
    ));
    let mut page = Page::default();
    page.page.searchable = true;
    page.section(None, rows);
    page
}

/// More…: COPY (what the entry has) │ ADMINISTRATION.
#[must_use]
pub fn more_page(catalog: &Catalog, entry: Option<&Entry>) -> Page {
    let mut page = Page::default();
    page.page.crumb = Some(catalog.panel.more.trim_end_matches('…').to_owned());
    let copy: Vec<(PanelRow, PanelCommand)> = entry.map_or_else(Vec::new, |entry| {
        catalog
            .panel
            .more_copy
            .iter()
            .filter_map(|id| {
                let (row, command) = action_row(catalog, id);
                let command = match id.as_str() {
                    "copy-description" => PanelCommand::Copy(entry.description.clone()?),
                    "copy-notes" => {
                        let body = notes_body(entry);
                        (!body.is_empty()).then(|| PanelCommand::Copy(body.to_owned()))?
                    }
                    _ => command,
                };
                Some((row, command))
            })
            .collect()
    });
    page.section(Some(catalog.panel.copy_section.clone()), copy);
    let admin = catalog
        .panel
        .more_admin
        .iter()
        .map(|id| action_row(catalog, id))
        .collect();
    page.section(Some(catalog.panel.admin_section.clone()), admin);
    page
}

/// Links…: one row per link, its title or its trimmed URL.
#[must_use]
pub fn links_page(catalog: &Catalog, entry: &Entry) -> Page {
    let mut page = Page::default();
    page.page.crumb = Some(catalog.panel.links.trim_end_matches('…').to_owned());
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a token of a few dozen characters"
    )]
    let chars = design().components.links.url_chars as usize;
    let cmd_enter_caps = catalog
        .keys_for(Screen::Links, None)
        .into_iter()
        .find(|k| k.keys == "cmd-enter")
        .map(|k| k.caps);
    let rows = entry
        .links
        .iter()
        .map(|link| {
            let label = link
                .title
                .clone()
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| trim_url(&link.url, chars));
            (
                PanelRow {
                    label,
                    caps: cmd_enter_caps.clone(),
                    ..PanelRow::default()
                },
                PanelCommand::Link(link.url.clone()),
            )
        })
        .collect();
    page.section(None, rows);
    page
}

/// The root's search page: the root's rows (Links… and More… included), then More…'s and Links…'s
/// rows, marked with their submenu (`more_origin`, `links_origin`).
#[must_use]
pub fn search_page(catalog: &Catalog, screen: Screen, entry: Option<&Entry>) -> Page {
    let mut page = root_page(catalog, screen, entry);
    let mut marked = |sub: Page, origin: &str| {
        for (section, commands) in sub.page.sections.into_iter().zip(sub.commands) {
            let rows = section
                .rows
                .into_iter()
                .map(|row| PanelRow {
                    origin: Some(origin.to_owned()),
                    ..row
                })
                .zip(commands)
                .collect();
            page.section(section.label, rows);
        }
    };
    marked(more_page(catalog, entry), &catalog.messages.more_origin);
    if let Some(entry) = entry.filter(|e| !e.links.is_empty()) {
        marked(links_page(catalog, entry), &catalog.messages.links_origin);
    }
    page
}

/// The filter (D15): _n filters · Clear All_ when filters are set, TYPES, TAGS.
#[must_use]
pub fn filter_page(
    catalog: &Catalog,
    types: &[String],
    tags: &[String],
    current: &FilterState,
    images: &TagImages,
) -> Page {
    let mut page = Page::default();
    page.page.searchable = true;
    let active = current.types.len() + current.tags.len();
    if active > 0 {
        let label = if active == 1 {
            catalog.messages.one_filter.clone()
        } else {
            fill(&catalog.messages.filters, &[("count", &active.to_string())])
        };
        let clear = (
            PanelRow {
                label: catalog.label("clear-filters").to_owned(),
                lead: Some(RowLead::Clear),
                muted: true,
                ..PanelRow::default()
            },
            PanelCommand::Filter {
                toggle: FilterState::default(),
                only: FilterState::default(),
            },
        );
        page.section(Some(label), vec![clear]);
    }
    let colors = &design().colors;
    let type_rows = types
        .iter()
        .filter_map(|t| {
            let kind = catalog.entry_type(t)?;
            Some((
                PanelRow {
                    label: kind.label.clone(),
                    lead: Some(RowLead::Tile {
                        icon: kind.icon.clone(),
                        tint: colors.named(&kind.tint).unwrap_or(colors.faint),
                    }),
                    checked: current.types.contains(t),
                    ..PanelRow::default()
                },
                PanelCommand::Filter {
                    toggle: FilterState {
                        types: toggled(&current.types, t),
                        tags: current.tags.clone(),
                    },
                    only: FilterState {
                        types: vec![t.clone()],
                        tags: Vec::new(),
                    },
                },
            ))
        })
        .collect();
    page.section(Some(catalog.panel.types_section.clone()), type_rows);
    let tag_rows = tags
        .iter()
        .map(|t| {
            (
                PanelRow {
                    label: t.clone(),
                    lead: Some(images.image(t).map_or(RowLead::Hash, |image| {
                        RowLead::Image(image.clone(), colors.hue(tag_hue(t)))
                    })),
                    checked: current.tags.contains(t),
                    ..PanelRow::default()
                },
                PanelCommand::Filter {
                    toggle: FilterState {
                        types: current.types.clone(),
                        tags: toggled(&current.tags, t),
                    },
                    only: FilterState {
                        types: Vec::new(),
                        tags: vec![t.clone()],
                    },
                },
            )
        })
        .collect();
    page.section(Some(catalog.panel.tags_section.clone()), tag_rows);
    page
}

fn toggled(values: &[String], value: &str) -> Vec<String> {
    if values.iter().any(|v| v == value) {
        values.iter().filter(|v| *v != value).cloned().collect()
    } else {
        let mut next = values.to_vec();
        next.push(value.to_owned());
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::entry;

    fn labels(page: &Page) -> Vec<Vec<String>> {
        page.page
            .sections
            .iter()
            .map(|s| s.rows.iter().map(|r| r.label.clone()).collect())
            .collect()
    }

    fn bookmark() -> Entry {
        entry(
            "bookmarks",
            "https://b.example",
            "desc: D\nnotes: N\nlinks: [{Guide: https://g.example}]\n",
        )
    }

    #[test]
    fn roots_the_screen_keys_resize_links_and_more() {
        let catalog = Catalog::default();
        let page = root_page(&catalog, Screen::List, Some(&bookmark()));
        assert_eq!(
            labels(&page),
            [["Details", "Open", "Copy", "Resize", "Links…", "More…"]]
        );
        assert_eq!(page.page.sections[0].rows[2].caps.as_deref(), Some("⇧⌘↵"));
        let command = entry("commands", "ls", "title: List\n");
        assert_eq!(
            labels(&root_page(&catalog, Screen::Detail, Some(&command))),
            [["Copy", "Resize", "More…"]]
        );
    }

    #[test]
    fn copies_and_administers_from_more() {
        let catalog = Catalog::default();
        let page = more_page(&catalog, Some(&bookmark()));
        assert_eq!(page.page.crumb.as_deref(), Some("More"));
        assert_eq!(
            labels(&page),
            [
                vec!["Description", "Notes", "Path"],
                vec!["Edit", "Refresh", "Configure"]
            ]
        );
        assert_eq!(page.command(0, 1), Some(&PanelCommand::Copy("N".into())));
        assert_eq!(labels(&more_page(&catalog, None)), [["Edit", "Refresh", "Configure"]]);
    }

    #[test]
    fn finds_the_submenus_rows_from_the_root() {
        let catalog = Catalog::default();
        let page = search_page(&catalog, Screen::List, Some(&bookmark()));
        let all = labels(&page);
        assert_eq!(all[0], ["Details", "Open", "Copy", "Resize", "Links…", "More…"]);
        assert!(all.iter().flatten().any(|l| l == "Path"));
        assert!(all.iter().flatten().any(|l| l == "Guide"));
        assert_eq!(page.page.sections[1].rows[0].origin.as_deref(), Some("More"));
    }

    #[test]
    fn toggles_or_isolates_a_filter_choice() {
        let catalog = Catalog::default();
        let current = FilterState {
            types: vec!["commands".into()],
            tags: Vec::new(),
        };
        let page = filter_page(
            &catalog,
            &["bookmarks".into(), "commands".into()],
            &["git".into()],
            &current,
            &TagImages::default(),
        );
        assert_eq!(
            labels(&page),
            [vec!["Clear All"], vec!["Bookmarks", "Commands"], vec!["git"]]
        );
        assert_eq!(page.page.sections[0].label.as_deref(), Some("1 filter"));
        assert!(page.page.sections[1].rows[1].checked);
        let Some(PanelCommand::Filter { toggle, only }) = page.command(2, 0) else {
            panic!()
        };
        assert_eq!(toggle.tags, ["git"]);
        assert_eq!(toggle.types, ["commands"]);
        assert_eq!(
            (only.types.len(), only.tags.as_slice()),
            (0, ["git".to_owned()].as_slice())
        );
    }
}
