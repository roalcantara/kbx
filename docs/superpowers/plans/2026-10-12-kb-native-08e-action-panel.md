# Plan 8e: The action panel and the filter

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ⌘K opens the action panel of `02-actions.html` (D9): 220 wide, 28 pt rows, search at the bottom, sections
by a hairline, at most two levels — the root (the screen's keys, Resize, Links… with links, More…), Links… (↵ opens a
link, ⌘↵ copies it) and More… (COPY Description, Notes, Path │ ADMINISTRATION Edit, Refresh, Configure) under a `‹`
header without search; typing in the root finds the submenus' rows, marked with their submenu. ⌘P opens the filter on
the same component (D15, `07-filter.html`): TYPES with their tile, TAGS with their icon or `#`, ✓ only on chosen
rows, _n filters · Clear All_ first, ↵ Toggle keeps it open, ⌘↵ Only sets that choice alone and closes it (R6). Esc
clears the panel's search first, then leaves a submenu, then closes (Raycast; the Asyar survey confirms it).

**Architecture:** `ui/action_panel.rs` is a pure component: a `PanelPage` (sections of `PanelRow`s), the indices the
query matches, the selection, and a search element in, a `Div` out. `actions.rs` (replacing `palette.rs`) builds the
pages from the catalog, the screen and the entry, each row paired with a `PanelCommand`. `search_view/panel.rs`
(replacing `overlays.rs`) holds the overlay state, the shared search field and the confirm logic, and wraps the panel
in GPUI Kit's `Popover` for anchoring (R3).

**Tech Stack:** Rust, GPUI Kit 0.7.1 (`Popover`, `Input`).

**Expects:** Plans 8a–8d done (`ui::{design, keycap, tile, chip}`, `Catalog::keys_for`, `SearchView::{run_action,
copy, apply_filter, key_screen}`).

**Index:** [2026-10-12-kb-native-08-design-system.md](2026-10-12-kb-native-08-design-system.md).

---

## File structure

| File                                        | Change | Responsibility                                                         |
| ------------------------------------------- | ------ | ---------------------------------------------------------------------- |
| `crates/kb-app/src/ui/action_panel.rs`      | create | `PanelPage`, `PanelSection`, `PanelRow`, `RowLead`, `matches`, `panel` |
| `crates/kb-app/src/actions.rs`              | create | `PanelCommand`, `Submenu`, `Page`, the page builders                   |
| `crates/kb-app/src/palette.rs`              | delete | Replaced by `actions.rs`                                               |
| `crates/kb-app/src/filter.rs`               | modify | Keeps `filter_summary` only; the choices move to `actions.rs`          |
| `crates/kb-app/src/search_view/panel.rs`    | create | The overlay: state, keys, confirm, popovers                            |
| `crates/kb-app/src/search_view/overlays.rs` | delete | Replaced by `panel.rs`                                                 |
| `crates/kb-app/src/search_view.rs`          | modify | `Overlay`, the shared search field, `back`                             |
| `crates/kb-app/src/keys.rs`                 | modify | `CopyUrl` goes                                                         |
| `crates/kb-app/tests/ui.rs`                 | modify | Panel and filter tests                                                 |

---

### Task 1: The pages as data

**Files:** create `crates/kb-app/src/ui/action_panel.rs` (data part), `crates/kb-app/src/actions.rs`; modify
`ui.rs`, `lib.rs`.

- [x] **Step 1: Write the data types.** Create `crates/kb-app/src/ui/action_panel.rs` with:

```rust
//! D9's action panel (02-actions, 07-filter): sections of 28 pt rows separated by a hairline, a
//! `‹` header on submenus, the search at the bottom. Pure: the page, what the query matches and
//! the selection in, an element out; `search_view/panel.rs` owns the state.

use std::rc::Rc;

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, Div, Hsla, InteractiveElement as _, IntoElement as _, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div, svg,
};

use crate::tag_images::TagImage;
use crate::ui::chip::icon_chip;
use crate::ui::design::{TextRoleExt as _, design, pt};
use crate::ui::keycap::{ChipState, keycap};
use crate::ui::tile::{TileSize, tile};

/// What leads a row (the filter's rows; the action rows have none).
#[derive(Debug, Clone, PartialEq)]
pub enum RowLead {
    /// A type's mini tile.
    Tile {
        /// The Lucide icon.
        icon: String,
        /// Its accent.
        tint: Hsla,
    },
    /// A tag's image, tinted in its hue when it is a shape.
    Image(TagImage, Hsla),
    /// A tag without an image: `#`.
    Hash,
    /// The Clear All row's ✕.
    Clear,
}

/// One row.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PanelRow {
    /// Its label.
    pub label: String,
    /// Its keycap text, if its action has keys.
    pub caps: Option<String>,
    /// Opens a submenu (`›`).
    pub submenu: bool,
    /// The submenu it comes from, when the root's search found it there.
    pub origin: Option<String>,
    /// The filter's lead.
    pub lead: Option<RowLead>,
    /// A chosen filter row (✓).
    pub checked: bool,
    /// Drawn in muted text (Clear All).
    pub muted: bool,
}

/// A section: an optional label (drawn upper-case) and its rows.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PanelSection {
    /// The label.
    pub label: Option<String>,
    /// The rows.
    pub rows: Vec<PanelRow>,
}

/// A page: the root, a submenu (with its `‹` header) or the filter.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PanelPage {
    /// The submenu's header.
    pub crumb: Option<String>,
    /// The sections, separated by a hairline.
    pub sections: Vec<PanelSection>,
    /// Whether it has a search field (submenus have none).
    pub searchable: bool,
}

/// The rows `query` matches, as (section, row), in order: a case-insensitive substring of the
/// label, or of the origin; every row for an empty query.
#[must_use]
pub fn matches(page: &PanelPage, query: &str) -> Vec<(usize, usize)> {
    let query = query.trim().to_lowercase();
    page.sections
        .iter()
        .enumerate()
        .flat_map(|(s, section)| section.rows.iter().enumerate().map(move |(r, row)| (s, r, row)))
        .filter(|(_, _, row)| {
            query.is_empty()
                || row.label.to_lowercase().contains(&query)
                || row.origin.as_deref().is_some_and(|o| o.to_lowercase().contains(&query))
        })
        .map(|(s, r, _)| (s, r))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(label: &str) -> PanelRow {
        PanelRow {
            label: label.into(),
            ..PanelRow::default()
        }
    }

    #[test]
    fn matches_labels_and_origins_case_insensitively() {
        let page = PanelPage {
            crumb: None,
            searchable: true,
            sections: vec![
                PanelSection {
                    label: None,
                    rows: vec![row("Details"), row("Copy")],
                },
                PanelSection {
                    label: Some("Copy".into()),
                    rows: vec![PanelRow {
                        origin: Some("More".into()),
                        ..row("Path")
                    }],
                },
            ],
        };
        assert_eq!(matches(&page, ""), [(0, 0), (0, 1), (1, 0)]);
        assert_eq!(matches(&page, "pa"), [(1, 0)]);
        assert_eq!(matches(&page, "MORE"), [(1, 0)]);
        assert!(matches(&page, "zz").is_empty());
    }
}
```

Add `pub mod action_panel;` to `ui.rs` (sorted, first).

- [x] **Step 2: Write the page builders with their tests.** Create `crates/kb-app/src/actions.rs`:

```rust
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
    let admin = catalog.panel.more_admin.iter().map(|id| action_row(catalog, id)).collect();
    page.section(Some(catalog.panel.admin_section.clone()), admin);
    page
}

/// Links…: one row per link, its title or its trimmed URL.
#[must_use]
pub fn links_page(catalog: &Catalog, entry: &Entry) -> Page {
    let mut page = Page::default();
    page.page.crumb = Some(catalog.panel.links.trim_end_matches('…').to_owned());
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a token of a few dozen characters")]
    let chars = design().components.links.url_chars as usize;
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
                    ..PanelRow::default()
                },
                PanelCommand::Link(link.url.clone()),
            )
        })
        .collect();
    page.section(None, rows);
    page
}

/// The root's search page: the root's rows, then More…'s and Links…'s, marked with their
/// submenu (`more_origin`, `links_origin`); the submenu rows themselves are left out.
#[must_use]
pub fn search_page(catalog: &Catalog, screen: Screen, entry: Option<&Entry>) -> Page {
    let mut page = root_page(catalog, screen, entry);
    if let Some(section) = page.page.sections.first_mut() {
        let commands = &mut page.commands[0];
        let keep: Vec<bool> = commands.iter().map(|c| !matches!(c, PanelCommand::Open(_))).collect();
        let mut ix = 0;
        section.rows.retain(|_| {
            ix += 1;
            keep[ix - 1]
        });
        let mut ix = 0;
        commands.retain(|_| {
            ix += 1;
            keep[ix - 1]
        });
    }
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
    page.section(Some("Types".into()), type_rows);
    let tag_rows = tags
        .iter()
        .map(|t| {
            (
                PanelRow {
                    label: t.clone(),
                    lead: Some(
                        images
                            .image(t)
                            .map_or(RowLead::Hash, |image| RowLead::Image(image.clone(), colors.hue(tag_hue(t)))),
                    ),
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
    page.section(Some("Tags".into()), tag_rows);
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
        entry("bookmarks", "https://b.example", "desc: D\nnotes: N\nlinks: [{Guide: https://g.example}]\n")
    }

    #[test]
    fn roots_the_screen_keys_resize_links_and_more() {
        let catalog = Catalog::default();
        let page = root_page(&catalog, Screen::List, Some(&bookmark()));
        assert_eq!(labels(&page), [["Details", "Open", "Copy", "Resize", "Links…", "More…"]]);
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
            [vec!["Description", "Notes", "Path"], vec!["Edit", "Refresh", "Configure"]]
        );
        assert_eq!(page.command(0, 1), Some(&PanelCommand::Copy("N".into())));
        assert_eq!(labels(&more_page(&catalog, None)), [["Edit", "Refresh", "Configure"]]);
    }

    #[test]
    fn finds_the_submenus_rows_from_the_root() {
        let catalog = Catalog::default();
        let page = search_page(&catalog, Screen::List, Some(&bookmark()));
        let all = labels(&page);
        assert_eq!(all[0], ["Details", "Open", "Copy", "Resize"], "no submenu rows");
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
        assert_eq!(labels(&page), [vec!["Clear All"], vec!["Bookmarks", "Commands"], vec!["git"]]);
        assert_eq!(page.page.sections[0].label.as_deref(), Some("1 filter"));
        assert!(page.page.sections[1].rows[1].checked);
        let Some(PanelCommand::Filter { toggle, only }) = page.command(2, 0) else { panic!() };
        assert_eq!(toggle.tags, ["git"]);
        assert_eq!(toggle.types, ["commands"]);
        assert_eq!((only.types.len(), only.tags.as_slice()), (0, ["git".to_owned()].as_slice()));
    }
}
```

In `lib.rs` add `mod actions;` (sorted, first). In `presentation.rs` add `trim_url` (it is 8f's, but the links page
needs it now):

```rust
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
```

with its test in `presentation.rs`'s `tests` module:

```rust
#[test]
fn trims_a_url_to_its_host_and_last_segment() {
    assert_eq!(trim_url("https://www.github.com/tokio-rs/mini-redis?tab=1#x", 32), "github.com/tokio-rs/mini-redis");
    assert_eq!(
        trim_url("https://doc.rust-lang.org/std/task/struct.Waker.html", 32),
        "doc.rust-lang.org/…/struct.Waker.html"
    );
    assert_eq!(trim_url("https://tokio.rs/", 32), "tokio.rs");
}
```

- [x] **Step 3: Run the tests.**

Run: `cargo nextest run -p kb-app actions ui::action_panel presentation`
Expected: PASS (the builders are complete; this task writes data and tests together because the page shapes are
copied from the approved pages, not discovered).

- [x] **Step 4: Gate and commit.** (`dead_code` on the not-yet-used builders: add
      `#![expect(dead_code, reason = "search_view/panel.rs uses the pages in the next task")]` at the top of
      `actions.rs` and remove it in Task 2.)

```bash
mise run gate
git add crates/kb-app/src
git commit -m "feat(app): Build the action panel's pages from the catalog" \
  -m "Root, Links…, More… and the filter, as data with what each row does (D9, D15)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: The panel drawn, and wired to the keyboard

**Files:** modify `ui/action_panel.rs` (render part); create `search_view/panel.rs`; delete
`search_view/overlays.rs`, `palette.rs`; modify `search_view.rs`, `keys.rs`, `filter.rs`, `lib.rs`, `tests/ui.rs`.

- [x] **Step 1: Write the failing UI tests.** In `crates/kb-app/tests/ui.rs`, delete
      `copies_the_title_from_the_copy_page_of_the_actions` and `leaves_a_page_of_the_actions_then_the_actions_with_escape`,
      and append:

```rust
#[gpui_kit::test]
fn lists_the_root_actions_for_a_bookmark_with_links(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "example");
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| view.read(cx).panel_labels(cx)),
        [["Details", "Open", "Copy", "Resize", "Links…", "More…"]]
    );
}

#[gpui_kit::test]
fn copies_the_path_found_from_the_root_s_search(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "rebase");
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    type_text(cx, window, "path");
    cx.run_until_parked();
    press(cx, window, "enter");
    cx.run_until_parked();
    let copied = cx.read_from_clipboard().and_then(|item| item.text()).unwrap();
    assert!(copied.ends_with("cheats.yml"), "{copied}");
    assert!(!cx.read(|cx| view.read(cx).is_actions_open()));
}

#[gpui_kit::test]
fn clears_the_search_then_leaves_more_then_closes_with_escape(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "rebase");
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    type_text(cx, window, "More");
    cx.run_until_parked();
    press(cx, window, "enter");
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| view.read(cx).panel_labels(cx)),
        [vec!["Notes", "Path"], vec!["Edit", "Refresh", "Configure"]],
        "the cheat sheet has no description"
    );
    press(cx, window, "escape");
    cx.run_until_parked();
    assert_eq!(cx.read(|cx| view.read(cx).panel_labels(cx))[0][0], "Details");
    press(cx, window, "escape");
    cx.run_until_parked();
    assert!(!cx.read(|cx| view.read(cx).is_actions_open()));
    assert_eq!(titles(&view, cx), ["Interactive rebase"], "Esc closed the panel, not the query");
}

#[gpui_kit::test]
fn opens_and_copies_a_link_from_links(cx: &mut TestAppContext) {
    let (_home, window, _view) = open_with_query(cx, "example");
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    type_text(cx, window, "Links…");
    cx.run_until_parked();
    press(cx, window, "enter");
    cx.run_until_parked();
    press(cx, window, "cmd-enter");
    cx.run_until_parked();
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("https://example.com/changelog"));
}

#[gpui_kit::test]
fn isolates_a_filter_choice_with_cmd_return(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    press(cx, window, "cmd-p");
    cx.run_until_parked();
    type_text(cx, window, "docker");
    cx.run_until_parked();
    press(cx, window, "cmd-enter");
    settle(cx);
    assert!(!cx.read(|cx| view.read(cx).is_filter_open()), "Only closes the filter");
    let filter = cx.read(|cx| view.read(cx).list().read(cx).delegate().filter().clone());
    assert_eq!((filter.types.len(), filter.tags), (0, vec!["docker".to_owned()]));
}
```

In `task_t3_filters_by_a_type_and_a_tag_in_one_visit`, the tag row's label is now `docker` (no `#`): change the
choices to `["Commands", "docker"]`.

Run: `cargo nextest run -p kb-app --test ui`
Expected: FAIL to compile: no method `panel_labels`.

- [x] **Step 2: Draw the panel.** Append to `ui/action_panel.rs` (before the tests module):

```rust
/// What runs when a row is clicked: its index among the visible rows.
pub type OnRow = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The panel: `page`'s `visible` rows (from [`matches`]) with `selected` highlighted, `search` (the
/// search field) at the bottom when the page is searchable, `width` points wide.
#[must_use]
pub fn panel(
    page: &PanelPage,
    visible: &[(usize, usize)],
    selected: usize,
    search: Option<AnyElement>,
    width: f32,
    on_row: &OnRow,
) -> Div {
    let d = design();
    let (c, s, k) = (&d.colors, &d.spacing.panel, &d.components.panel);
    let hairline = || div().h(pt(d.components.border.hairline)).bg(c.edge).flex_none();
    let mut list = v_flex().id("panel-rows").p(pt(s.pad)).max_h(pt(k.max_height)).overflow_y_scroll();
    let mut shown_section = None;
    for (ix, &(section_ix, row_ix)) in visible.iter().enumerate() {
        let section = &page.sections[section_ix];
        if shown_section != Some(section_ix) {
            if shown_section.is_some() {
                list = list.child(hairline().mx(pt(s.rule_margin)).my(pt(s.rule_margin)));
            }
            if let Some(label) = &section.label {
                list = list.child(
                    div()
                        .pt(pt(s.section_pad_top))
                        .px(pt(s.section_pad_x))
                        .pb(pt(s.section_pad_bottom))
                        .text_role(&d.typography.roles.section)
                        .text_color(c.faint)
                        .child(label.to_uppercase()),
                );
            }
            shown_section = Some(section_ix);
        }
        list = list.child(row(&section.rows[row_ix], ix, ix == selected, on_row));
    }
    v_flex()
        .w(pt(width))
        .overflow_hidden()
        .when_some(page.crumb.clone(), |panel, crumb| {
            panel.child(
                h_flex()
                    .h(pt(k.crumb))
                    .items_center()
                    .gap(pt(s.crumb_gap))
                    .px(pt(s.crumb_pad_x))
                    .text_role(&d.typography.roles.label)
                    .text_color(c.faint)
                    .border_b(pt(d.components.border.hairline))
                    .border_color(c.edge)
                    .child(svg().path("icons/chevron-left.svg").size(pt(k.crumb_icon)).text_color(c.faint))
                    .child(crumb),
            )
        })
        .child(list)
        .when_some(search.filter(|_| page.searchable), |panel, search| {
            panel.child(
                h_flex()
                    .h(pt(k.search))
                    .flex_none()
                    .items_center()
                    .gap(pt(s.search_gap))
                    .px(pt(s.search_pad_x))
                    .border_t(pt(d.components.border.hairline))
                    .border_color(c.edge)
                    .text_role(&d.typography.roles.description)
                    .text_color(c.text)
                    .child(svg().path("icons/search.svg").size(pt(k.search_icon)).text_color(c.faint))
                    .child(div().flex_1().min_w_0().child(search)),
            )
        })
}

fn row(row: &PanelRow, ix: usize, selected: bool, on_row: &OnRow) -> impl gpui_kit::IntoElement {
    let d = design();
    let (c, s, k) = (&d.colors, &d.spacing.panel, &d.components.panel);
    let on_row = on_row.clone();
    let lead = row.lead.clone().map(|lead| match lead {
        RowLead::Tile { icon, tint } => tile(&icon, tint, TileSize::Mini).into_any_element(),
        RowLead::Image(image, hue) => {
            icon_chip(SharedString::from(format!("filter-tag-{ix}")), &image, hue, ChipState::Rest, String::new())
                .size(pt(d.components.tile.mini))
                .into_any_element()
        }
        RowLead::Hash => div()
            .w(pt(d.components.tile.mini))
            .flex()
            .justify_center()
            .text_color(c.faint)
            .child("#")
            .into_any_element(),
        RowLead::Clear => div()
            .w(pt(d.components.tile.mini))
            .flex()
            .justify_center()
            .child(svg().path("icons/x.svg").size(pt(d.components.tile.mini_icon)).text_color(c.faint))
            .into_any_element(),
    });
    let caps = row.caps.clone().or_else(|| selected.then(|| crate::ui::keycap::caps("enter")));
    h_flex()
        .id(SharedString::from(format!("panel-row-{ix}")))
        .h(pt(k.row))
        .flex_none()
        .items_center()
        .gap(pt(s.row_gap))
        .px(pt(s.row_pad_x))
        .rounded(pt(d.rounded.row))
        .text_role(&d.typography.roles.label)
        .text_color(if row.muted { c.muted } else { c.text })
        .when(selected, |r| r.bg(c.selected))
        .on_click(move |_, window, cx| on_row(ix, window, cx))
        .children(lead)
        .child(div().flex_1().min_w_0().truncate().child(row.label.clone()))
        .when_some(row.origin.clone(), |r, origin| r.child(div().text_color(c.faint).child(origin)))
        .when(row.checked, |r| {
            r.child(svg().path("icons/check.svg").size(pt(k.check)).text_color(c.green))
        })
        .map(|r| {
            if row.submenu {
                r.child(svg().path("icons/chevron-right.svg").size(pt(k.chevron)).text_color(c.faint))
            } else if let Some(caps) = caps {
                r.child(keycap(caps, ChipState::of(selected)))
            } else {
                r
            }
        })
}
```

- [x] **Step 3: The overlay state.** In `search_view.rs`:
  - replace `enum Overlay` with:

```rust
/// What is open over the screen: one overlay at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Overlay {
    /// Nothing.
    None,
    /// The action panel: the root, or a submenu.
    Actions(Option<crate::actions::Submenu>),
    /// The filter.
    Filter,
}
```

- replace the fields `palette: Entity<CommandState>, filter_palette: Entity<CommandState>,` with
  `panel_query: Entity<InputState>, panel_selected: usize,`; in `new`, replace the two `CommandState::new` lines
  with:

```rust
let panel_query = cx.new(|cx| InputState::new(window, cx).placeholder(catalog.panel.search.clone()));
let panel_sub = cx.subscribe_in(&panel_query, window, Self::on_panel_query_event);
```

    (place them after `let catalog = Rc::new(catalog);`), add `panel_sub` to `subscriptions`, and set
    `panel_query, panel_selected: 0,` in the returned `Self`;

- change `mod overlays;` to `mod panel;`, and remove the `CommandState` and `Page` imports;
- in `back`, replace the first `match self.overlay { … }` with:

```rust
// Esc clears the panel's search, then leaves a submenu, then closes the panel (Raycast).
if self.overlay != Overlay::None {
    if !self.panel_query.read(cx).value().is_empty() {
        return self.set_panel_query("", window, cx);
    }
    let next = match self.overlay {
        Overlay::Actions(Some(_)) => Overlay::Actions(None),
        _ => Overlay::None,
    };
    return self.show_overlay(next, window, cx);
}
```

- in `open_actions`, `Overlay::Actions(Page::Main)` becomes `Overlay::Actions(None)`.

- [x] **Step 4: The panel module.** Create `crates/kb-app/src/search_view/panel.rs`:

```rust
//! The overlays (D9, D15): the action panel above the footer's Actions, the filter under the
//! header's pill, both the `ui::action_panel` component in a GPUI Kit `Popover`, sharing one
//! search field at their bottom.

use std::rc::Rc;

use gpui_kit::component::button::Button;
use gpui_kit::component::input::{Input, InputEvent, InputState, MoveDown, MoveUp};
use gpui_kit::component::popover::Popover;
use gpui_kit::{
    Anchor, AnyElement, App, Context, Entity, Focusable as _, InteractiveElement as _, IntoElement as _,
    ParentElement as _, Styled as _, Window, div,
};
use kb_domain::search::tags_for_types;

use super::{Overlay, Screen, SearchView};
use crate::actions::{Page, PanelCommand, Submenu, filter_page, links_page, more_page, root_page, search_page};
use crate::ui::action_panel::{OnRow, matches, panel};
use crate::ui::design::{design, pt};

impl SearchView {
    /// Whether the action panel is open.
    #[must_use]
    pub fn is_actions_open(&self) -> bool {
        matches!(self.overlay, Overlay::Actions(_))
    }

    /// Whether the filter is open.
    #[must_use]
    pub fn is_filter_open(&self) -> bool {
        self.overlay == Overlay::Filter
    }

    /// The open page's visible labels by section, for tests.
    #[must_use]
    pub fn panel_labels(&self, cx: &App) -> Vec<Vec<String>> {
        let page = self.current_page(cx);
        let mut out: Vec<Vec<String>> = Vec::new();
        let mut last = None;
        for (s, r) in matches(&page.page, &self.panel_query.read(cx).value()) {
            if last != Some(s) {
                out.push(Vec::new());
                last = Some(s);
            }
            if let Some(labels) = out.last_mut() {
                labels.push(page.page.sections[s].rows[r].label.clone());
            }
        }
        out
    }

    /// The open overlay's page.
    fn current_page(&self, cx: &App) -> Page {
        let list = self.list.read(cx).delegate();
        let entry = match self.screen {
            Screen::Rebuild(_) => None,
            _ => list.selected_entry(),
        };
        let screen = self.key_screen();
        let query_empty = self.panel_query.read(cx).value().is_empty();
        match self.overlay {
            Overlay::None => Page::default(),
            Overlay::Actions(None) if query_empty => root_page(&self.catalog, screen, entry),
            Overlay::Actions(None) => search_page(&self.catalog, screen, entry),
            Overlay::Actions(Some(Submenu::More)) => more_page(&self.catalog, entry),
            Overlay::Actions(Some(Submenu::Links)) => entry.map_or_else(Page::default, |e| links_page(&self.catalog, e)),
            Overlay::Filter => {
                let types = list.visible_types().to_vec();
                let tags = list
                    .connection()
                    .and_then(|c| tags_for_types(c, &types).ok())
                    .unwrap_or_default();
                filter_page(&self.catalog, &types, &tags, list.filter(), list.tag_images())
            }
        }
    }

    /// Shows `overlay` with an empty search and hands it the keyboard; `Overlay::None` hands the
    /// keyboard back to the screen.
    pub(super) fn show_overlay(&mut self, overlay: Overlay, window: &mut Window, cx: &mut Context<Self>) {
        self.overlay = overlay;
        self.panel_selected = 0;
        self.panel_query.update(cx, |input, cx| input.set_value("", window, cx));
        if overlay == Overlay::None {
            self.focus_screen(window, cx);
        } else {
            let query = self.panel_query.clone();
            window.defer(cx, move |window, cx| query.update(cx, |input, cx| input.focus(window, cx)));
        }
        cx.notify();
    }

    pub(super) fn set_panel_query(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.panel_query.update(cx, |input, cx| input.set_value(text, window, cx));
        self.panel_selected = 0;
        cx.notify();
    }

    pub(super) fn on_panel_query_event(
        &mut self,
        _: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                self.panel_selected = 0;
                cx.notify();
            }
            InputEvent::PressEnter { secondary, .. } => self.confirm(self.panel_selected, *secondary, window, cx),
            _ => {}
        }
    }

    fn move_panel(&mut self, delta: isize, cx: &mut Context<Self>) {
        let count = matches(&self.current_page(cx).page, &self.panel_query.read(cx).value()).len();
        if count > 0 {
            self.panel_selected = self.panel_selected.saturating_add_signed(delta).min(count - 1);
            cx.notify();
        }
    }

    /// Runs visible row `ix`: ↵ (`secondary` false) or ⌘↵.
    fn confirm(&mut self, ix: usize, secondary: bool, window: &mut Window, cx: &mut Context<Self>) {
        let page = self.current_page(cx);
        let visible = matches(&page.page, &self.panel_query.read(cx).value());
        let Some(command) = visible.get(ix).and_then(|&(s, r)| page.command(s, r)).cloned() else {
            return;
        };
        match command {
            PanelCommand::Open(submenu) => self.show_overlay(Overlay::Actions(Some(submenu)), window, cx),
            PanelCommand::Action(id) => {
                self.show_overlay(Overlay::None, window, cx);
                self.run_action(&id, window, cx);
            }
            PanelCommand::Link(url) => {
                self.show_overlay(Overlay::None, window, cx);
                if secondary { self.copy(url, window, cx) } else { cx.open_url(&url) }
            }
            PanelCommand::Copy(text) => {
                self.show_overlay(Overlay::None, window, cx);
                self.copy(text, window, cx);
            }
            PanelCommand::Filter { toggle, only } => {
                if secondary {
                    self.apply_filter(only, cx);
                    self.show_overlay(Overlay::None, window, cx);
                } else {
                    self.apply_filter(toggle, cx);
                    self.set_panel_query("", window, cx);
                }
            }
        }
    }

    /// The Actions button inside its popover, opening upwards above the footer.
    pub(super) fn actions_popover(&self, trigger: Button, cx: &mut Context<Self>) -> Popover {
        self.anchored("actions-popover", Anchor::BottomRight, self.is_actions_open(), Overlay::Actions(None), trigger, cx)
    }

    /// The filter pill inside its popover, opening under the header.
    pub(super) fn filter_popover(&self, trigger: Button, cx: &mut Context<Self>) -> Popover {
        self.anchored("filter-popover", Anchor::TopRight, self.is_filter_open(), Overlay::Filter, trigger, cx)
    }

    fn anchored(
        &self,
        id: &'static str,
        anchor: Anchor,
        open: bool,
        overlay: Overlay,
        trigger: Button,
        cx: &mut Context<Self>,
    ) -> Popover {
        let d = design();
        let view = cx.entity().downgrade();
        let on_change = view.clone();
        // Without a tracked handle the popover focuses one of its own, taking the keyboard away
        // from the search field.
        let focus = self.panel_query.read(cx).focus_handle(cx);
        Popover::new(id)
            .p_0()
            .rounded(pt(d.rounded.popover))
            .anchor(anchor)
            .open(open)
            .track_focus(&focus)
            .on_open_change(move |open, window, cx| {
                let next = if *open { overlay } else { Overlay::None };
                let _ = on_change.update(cx, |this, cx| this.show_overlay(next, window, cx));
            })
            .trigger(trigger)
            .content(move |_, window, cx| match view.upgrade() {
                Some(entity) => entity.update(cx, |this, cx| this.render_panel(window, cx)),
                None => div().into_any_element(),
            })
    }

    fn render_panel(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let d = design();
        let page = self.current_page(cx);
        let visible = matches(&page.page, &self.panel_query.read(cx).value());
        let width = if self.is_filter_open() { d.components.panel.filter_width } else { d.components.panel.width };
        let view = cx.entity().downgrade();
        let on_row: OnRow = Rc::new(move |ix, window, cx| {
            let _ = view.update(cx, |this, cx| this.confirm(ix, false, window, cx));
        });
        let search = div()
            .capture_action(cx.listener(|this, _: &MoveUp, _, cx| this.move_panel(-1, cx)))
            .capture_action(cx.listener(|this, _: &MoveDown, _, cx| this.move_panel(1, cx)))
            .child(Input::new(&self.panel_query).appearance(false).p_0())
            .into_any_element();
        panel(&page.page, &visible, self.panel_selected, Some(search), width, &on_row).into_any_element()
    }
}
```

`EntryList` needs a `tag_images()` accessor: add to `entry_list.rs`:

```rust
/// The tag-images folder's images, for the filter's rows.
#[must_use]
pub fn tag_images(&self) -> &TagImages {
    &self.tag_images
}
```

- [x] **Step 5: Remove what the panel replaces.**

```bash
git rm crates/kb-app/src/palette.rs crates/kb-app/src/search_view/overlays.rs
```

- In `lib.rs`, delete `pub mod palette;`.
- In `keys.rs`, delete `CopyUrl` from `actions!` (D7: the bookmark's Copy is its secondary, ⇧⌘↵).
- In `search_view.rs`, delete `copy_url` and its `on_action` line and `CopyUrl` from the imports; delete
  `apply_filter`'s doc-less duplicate if the compiler reports one; remove `actions_labels` (replaced by
  `panel_labels`).
- In `filter.rs`, delete `FilterChoice`, `FilterGroup`, `toggled`, `filter_groups` and their tests (moved to
  `actions.rs`); keep `filter_summary` and its test.
- In `actions.rs`, delete the `#![expect(dead_code, …)]` line of Task 1.

- [x] **Step 6: Run every kb-app test.**

Run: `cargo nextest run -p kb-app`
Expected: PASS. If `Popover` is not `Styled` (so `.p_0()` does not compile), drop `.p_0()` and add
`.appearance(false)` instead, wrapping the content in
`div().bg(d.colors.mantle).border_1().border_color(d.colors.selected).rounded(pt(d.rounded.popover)).overflow_hidden()`;
note it under "Deviation:".

- [x] **Step 7: Gate and commit.**

```bash
mise run gate
git add -A crates/kb-app
git commit -m "feat(ui): Draw the action panel and the filter as the approved pages" \
  -m "28 pt rows, search at the bottom, Links… and More… submenus; Toggle ↵ and Only ⌘↵ (D9, D15)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Deviations

Recorded while executing (2026-10-13, commit `41355c3`):

- **`search_page` keeps the Links…/More… rows.** The plan's code dropped every row whose command
  was `Open(_)`, but its own UI tests need them: typing `More…` or `Links…` and pressing ↵ must
  open the submenu. The root rows are kept and the submenus' rows are appended, marked with their
  origin, so both goals hold; `actions::tests::finds_the_submenus_rows_from_the_root` was updated.
- **The panel has its own key context and focus.** The submenus have no search field, so with the
  plan's `Input`-only handling their keys (↵, ⌘↵, Esc, ↑↓) were lost and the popover closed.
  `keys::PANEL_CONTEXT` (`ui/action_panel`'s element, `panel_focus`) now binds `enter`/`secondary-enter`
  to the `Confirm` action and `escape` to `Back`, handled on the panel element (with
  `stop_propagation`, since gpui-base's `Popover` binds ↵ to a `Confirm` that toggles it shut).
  `InputEvent::PressEnter` is no longer used.
- **`SearchView::ladder` returns early while an overlay is open**, so the screen ladder cannot switch
  the screen under the panel.
- **Removed the now-dead `presentation::{PrimaryKind, PrimaryAction, primary_action}`** (and its test):
  `palette.rs` was its only caller, so deleting the palette left it unused (`-D unused`).
- **One commit instead of two.** The plan's Task 1 tree does not pass the repo's own hooks
  (jscpd flags the `actions.rs`/`filter.rs` `toggled` clone; `-D unused` flags the not-yet-used
  builders), so Tasks 1–2 land together.

---

## Done when

- `cargo nextest run -p kb-app` and `mise run gate` pass.
- ⌘K over the showcase's `/k8s-rollout-restart` shows `Details ↵ · Copy ⌘↵ · Resize ⇧⌘F · Links… › · More… ›`, as
  `02-actions.html`.
