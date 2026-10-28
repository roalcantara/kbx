//! The result list: one page of entries for the query and filter, read from the index.

use std::rc::Rc;
use std::time::Duration;

use gpui_kit::component::list::{ListDelegate, ListState};
use gpui_kit::component::{IndexPath, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, Context, IntoElement as _, ParentElement as _, SharedString, Styled as _, Task, Window, div,
};
use kb_domain::Connection;
use kb_domain::entry::Entry;
use kb_domain::row::row_to_entry;
use kb_domain::search::{FilterState, PAGE_SIZE, SearchQuery, count, entry_types, search};

use crate::catalog::{Catalog, Hero, Messages};
use crate::presentation::{favicon_url, row_label, row_tags, tag_hue};
use crate::tag_images::TagImages;
use crate::trace;
use crate::ui::chip::{icon_chip, link_chip, pill};
use crate::ui::design::{design, pt};
use crate::ui::empty_view::{EmptyView, Sentence};
use crate::ui::keycap::{ChipState, caps, keycap};
use crate::ui::row::Row;
use crate::ui::tile::{TileSize, favicon, tile};

/// Keystrokes settle this long before a query runs, as in the Raycast command.
pub const DEBOUNCE: Duration = Duration::from_millis(150);

/// Owns the query, filter and page of the list, and the read-only index connection.
#[derive(Debug)]
pub struct EntryList {
    connection: Option<Connection>,
    query: String,
    filter: FilterState,
    /// The catalog's entry types (D6); other types never show.
    known_types: Vec<String>,
    /// The index's types the catalog knows: what an empty type filter means.
    visible_types: Vec<String>,
    /// How many entries the query and filter match, whatever the page.
    count: usize,
    limit: usize,
    entries: Vec<Entry>,
    selected: Option<IndexPath>,
    secondary_confirm: bool,
    tag_images: TagImages,
    compact: bool,
    catalog: Rc<Catalog>,
}

impl EntryList {
    /// A list over `connection`; `None` when there is no index yet.
    #[must_use]
    pub fn new(connection: Option<Connection>) -> Self {
        let mut list = Self {
            connection,
            query: String::new(),
            filter: FilterState::default(),
            known_types: Vec::new(),
            visible_types: Vec::new(),
            count: 0,
            limit: PAGE_SIZE,
            entries: Vec::new(),
            selected: None,
            secondary_confirm: false,
            tag_images: TagImages::default(),
            compact: false,
            catalog: Rc::new(Catalog::default()),
        };
        list.refresh();
        list
    }

    /// Draws the tags that have an image as that image.
    #[must_use]
    pub fn with_tag_images(mut self, tag_images: TagImages) -> Self {
        self.tag_images = tag_images;
        self
    }

    /// Shows only the catalog's types (D6), drawn as the catalog says.
    #[must_use]
    pub fn with_catalog(mut self, catalog: Rc<Catalog>) -> Self {
        self.known_types = catalog.types.keys().cloned().collect();
        self.catalog = catalog;
        self.refresh();
        self
    }

    /// The index's types the list shows, sorted: the filter's type choices.
    #[must_use]
    pub fn visible_types(&self) -> &[String] {
        &self.visible_types
    }

    /// How many entries match the query and filter (the status line).
    #[must_use]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Whether rows show their titles alone, as beside the split pane.
    #[must_use]
    pub fn is_compact(&self) -> bool {
        self.compact
    }

    /// Shows titles alone (`true`, beside the split pane) or with descriptions, tags and links.
    pub fn set_compact(&mut self, compact: bool) {
        self.compact = compact;
    }

    /// Whether an index is open.
    #[must_use]
    pub fn has_index(&self) -> bool {
        self.connection.is_some()
    }

    /// Replaces the connection after a sync and runs the query again.
    pub fn set_connection(&mut self, connection: Option<Connection>) {
        self.connection = connection;
        self.refresh();
    }

    /// The read-only connection, for the filter's choices.
    #[must_use]
    pub fn connection(&self) -> Option<&Connection> {
        self.connection.as_ref()
    }

    /// The current filter.
    #[must_use]
    pub fn filter(&self) -> &FilterState {
        &self.filter
    }

    /// The tag-images folder's images, for the filter's rows.
    #[must_use]
    pub fn tag_images(&self) -> &TagImages {
        &self.tag_images
    }

    /// Replaces the filter and runs the query again from the first page.
    pub fn set_filter(&mut self, filter: FilterState) {
        self.filter = filter;
        self.limit = PAGE_SIZE;
        self.refresh();
    }

    /// The entries shown.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The query the list currently reflects.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// The selected entry.
    #[must_use]
    pub fn selected_entry(&self) -> Option<&Entry> {
        self.selected.and_then(|ix| self.entries.get(ix.row))
    }

    /// The row of the entry with `id`, if the results list it.
    #[must_use]
    pub fn row_of(&self, id: &str) -> Option<usize> {
        self.entries.iter().position(|entry| entry.id == id)
    }

    /// Whether the last confirmation was the secondary one (⌘↩), which runs the primary action.
    #[must_use]
    pub fn is_secondary_confirm(&self) -> bool {
        self.secondary_confirm
    }

    /// The keycap each row shows, in order, for tests: `↵` on the selected row, `⌘n` on the
    /// first nine (R7).
    #[must_use]
    pub fn row_keycaps(&self) -> Vec<String> {
        (0..self.entries.len())
            .filter_map(|ix| Self::row_keycap(ix, Some(IndexPath::new(ix)) == self.selected))
            .collect()
    }

    fn row_keycap(ix: usize, selected: bool) -> Option<String> {
        if selected {
            Some(caps("enter"))
        } else if ix < 9 {
            Some(caps(&format!("cmd-{}", ix + 1)))
        } else {
            None
        }
    }

    /// Runs the query for the current text, filter and page window, over the visible types only.
    fn refresh(&mut self) {
        let Some(connection) = &self.connection else {
            self.entries.clear();
            self.visible_types.clear();
            self.count = 0;
            return;
        };
        self.visible_types = entry_types(connection)
            .unwrap_or_default()
            .into_iter()
            .filter(|t| self.known_types.contains(&t.trim().to_lowercase()))
            .collect();
        let types = if self.filter.types.is_empty() {
            self.visible_types.clone()
        } else {
            self.filter.types.clone()
        };
        if types.is_empty() {
            self.entries.clear();
            self.count = 0;
            return;
        }
        let query = SearchQuery {
            text: self.query.clone(),
            filter: FilterState {
                types,
                tags: self.filter.tags.clone(),
            },
            limit: self.limit,
            offset: 0,
        };
        // A failed query keeps the previous results on screen rather than emptying the list.
        if let Ok(rows) = search(connection, &query) {
            self.entries = rows.iter().map(row_to_entry).collect();
        }
        if let Ok(matched) = count(connection, &query) {
            self.count = matched;
        }
    }

    /// A row as `01-list.html` draws it.
    fn render_row(&self, entry: &Entry, ix: usize, selected: bool) -> Row {
        let d = design();
        let kind = self.catalog.entry_type(&entry.entry_type);
        let (icon, tint, label, mono, media) = kind.map_or(("file-text", d.colors.faint, "", false, false), |kind| {
            (
                kind.icon.as_str(),
                d.colors.named(&kind.tint).unwrap_or(d.colors.faint),
                kind.label.as_str(),
                kind.mono_title,
                kind.hero == Some(Hero::Media),
            )
        });
        let lead = match media.then(|| favicon_url(&entry.primary_content)).flatten() {
            Some(url) => favicon(url, icon.to_owned(), tint, TileSize::Row).into_any_element(),
            None => tile(icon, tint, TileSize::Row).into_any_element(),
        };
        let state = ChipState::of(selected);
        let mut trailing: Vec<AnyElement> = Vec::new();
        if !self.compact {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a token of a few tags"
            )]
            let max = d.components.row.max_tags as usize;
            let (tags, more) = row_tags(&entry.tags, max);
            if !tags.is_empty() {
                trailing.push(
                    h_flex()
                        .items_center()
                        .gap(pt(d.spacing.row.tags_gap))
                        .children(tags.iter().map(|tag| self.render_tag(&entry.id, tag, state)))
                        .when(more > 0, |group| group.child(pill(format!("+{more}"), state)))
                        .into_any_element(),
                );
            }
            if !entry.links.is_empty() {
                trailing.push(link_chip(entry.links.len()).into_any_element());
            }
        }
        // A fixed, right-aligned slot: `↵` and `⌘ n` take the same room, so the chips beside them
        // never move with the selection (spec D3). Rows past the ninth keep the empty slot.
        trailing.push(
            div()
                .flex_none()
                .w(pt(design().components.row.key_slot))
                .flex()
                .justify_end()
                .children(Self::row_keycap(ix, selected).map(|key| keycap(key, ChipState::Live)))
                .into_any_element(),
        );
        Row::new(SharedString::from(entry.id.clone()), entry.title.clone())
            .label(row_label(entry, label))
            .lead(lead)
            .mono(mono)
            .description(entry.description.clone())
            .trailing(trailing)
    }

    /// A tag: its image when the tag-images folder has one, else its grey pill.
    fn render_tag(&self, entry_id: &str, tag: &str, state: ChipState) -> AnyElement {
        match self.tag_images.image(tag) {
            Some(image) => icon_chip(
                SharedString::from(format!("tag-{entry_id}-{tag}")),
                image,
                design().colors.hue(tag_hue(tag)),
                state,
                format!("#{tag}"),
            )
            .into_any_element(),
            None => pill(format!("#{tag}"), state).into_any_element(),
        }
    }
}

impl ListDelegate for EntryList {
    type Item = Row;

    fn perform_search(&mut self, query: &str, window: &mut Window, cx: &mut Context<ListState<Self>>) -> Task<()> {
        let query = query.to_owned();
        // The list drops the previous task when the query changes again, which cancels it, so
        // only a query that has been stable for DEBOUNCE runs.
        cx.spawn_in(window, async move |list, cx| {
            cx.background_executor().timer(DEBOUNCE).await;
            let _ = list.update_in(cx, |list, window, cx| {
                let delegate = list.delegate_mut();
                delegate.query = query;
                delegate.limit = PAGE_SIZE;
                delegate.refresh();
                trace::searched(window, cx);
                cx.notify();
            });
        })
    }

    fn items_count(&self, _: usize, _: &App) -> usize {
        self.entries.len()
    }

    fn render_item(&mut self, ix: IndexPath, _: &mut Window, _: &mut Context<ListState<Self>>) -> Option<Row> {
        let entry = self.entries.get(ix.row)?;
        Some(self.render_row(entry, ix.row, Some(ix) == self.selected))
    }

    fn render_empty(&mut self, _: &mut Window, _: &mut Context<ListState<Self>>) -> impl gpui_kit::IntoElement {
        let d = design();
        let m = &self.catalog.messages;
        let view = if !self.has_index() {
            EmptyView::new(
                "database",
                d.colors.faint,
                m.no_index_title.clone(),
                Sentence::of(&m.no_index, ""),
            )
        } else if self.query.trim().is_empty() && self.filter == FilterState::default() {
            EmptyView::new(
                "search",
                d.colors.faint,
                m.no_match_title.clone(),
                Sentence::of(&m.no_entries, ""),
            )
        } else {
            let template = no_match_template(m, &self.query, self.filter != FilterState::default());
            EmptyView::new(
                "search",
                d.colors.faint,
                m.no_match_title.clone(),
                Sentence::of(template, self.query.trim()),
            )
        };
        view.into_any_element()
    }

    fn set_selected_index(&mut self, ix: Option<IndexPath>, _: &mut Window, cx: &mut Context<ListState<Self>>) {
        self.selected = ix;
        cx.notify();
    }

    fn confirm(&mut self, secondary: bool, _: &mut Window, _: &mut Context<ListState<Self>>) {
        self.secondary_confirm = secondary;
    }

    fn has_more(&self, _: &App) -> bool {
        self.entries.len() == self.limit
    }

    fn load_more(&mut self, _: &mut Window, cx: &mut Context<ListState<Self>>) {
        self.limit += PAGE_SIZE;
        self.refresh();
        cx.notify();
    }
}

/// The sentence for a query or filter that matched nothing: with an empty query, only the filter is named.
fn no_match_template<'a>(m: &'a Messages, query: &str, filtered: bool) -> &'a str {
    match (query.trim().is_empty(), filtered) {
        (true, true) => &m.no_entries_filtered,
        (_, true) => &m.no_match_filtered,
        (_, false) => &m.no_match,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn says_no_entries_with_this_filter_when_the_query_is_empty() {
        let m = &Catalog::default().messages;
        assert_eq!(no_match_template(m, "", true), m.no_entries_filtered);
        assert_eq!(no_match_template(m, "  ", true), m.no_entries_filtered);
        assert_eq!(no_match_template(m, "kube", true), m.no_match_filtered);
        assert_eq!(no_match_template(m, "kube", false), m.no_match);
    }

    #[test]
    fn every_row_key_fits_the_row_slot() {
        let slot = design().components.row.key_slot;
        for ix in 0..12 {
            for selected in [true, false] {
                if let Some(key) = EntryList::row_keycap(ix, selected) {
                    assert!(crate::ui::keycap::keycap_width(&key) <= slot, "{key} in row {ix}");
                }
            }
        }
    }

    fn list_with_entries(ids: &[&str]) -> EntryList {
        let mut list = EntryList::new(None);
        list.entries = ids
            .iter()
            .map(|id| kb_domain::entry::Entry {
                id: (*id).to_owned(),
                title: String::new(),
                description: None,
                entry_type: "commands".into(),
                tags: Vec::new(),
                notes: None,
                notes_markdown: String::new(),
                links: Vec::new(),
                source_file: String::new(),
                source_rel_path: String::new(),
                primary_content: String::new(),
                task_status: None,
                task_priority: None,
                raw_yaml: String::new(),
            })
            .collect();
        list
    }

    #[test]
    fn finds_an_entry_s_row_by_id() {
        let list = list_with_entries(&["a#commands#x", "b#commands#y"]);
        assert_eq!(list.row_of("b#commands#y"), Some(1));
        assert_eq!(list.row_of("gone"), None);
    }
}
