//! The result list: one page of entries for the query and filter, read from the index.

use std::time::Duration;

use gpui_kit::assets::IconName;
use gpui_kit::component::empty::{Empty, EmptyDescription, EmptyHeader, EmptyTitle};
use gpui_kit::component::list::{ListDelegate, ListItem, ListState};
use gpui_kit::component::tag::Tag;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme as _, Icon, IndexPath, Sizable as _, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, Context, InteractiveElement as _, IntoElement as _, ParentElement, Role, SharedString,
    StatefulInteractiveElement as _, Styled as _, StyledImage as _, Task, Window, div, img, px, relative, svg,
};
use kb_domain::Connection;
use kb_domain::entry::Entry;
use kb_domain::row::row_to_entry;
use kb_domain::search::{FilterState, PAGE_SIZE, SearchQuery, search};

use crate::accents::accent;
use crate::presentation::{TypeKey, favicon_url, row_label, row_tags, tag_hue, type_badge, type_key};
use crate::tag_images::TagImages;
use crate::trace;

/// Keystrokes settle this long before a query runs, as in the Raycast command.
pub const DEBOUNCE: Duration = Duration::from_millis(150);

/// Owns the query, filter and page of the list, and the read-only index connection.
#[derive(Debug)]
pub struct EntryList {
    connection: Option<Connection>,
    query: String,
    filter: FilterState,
    limit: usize,
    entries: Vec<Entry>,
    selected: Option<IndexPath>,
    secondary_confirm: bool,
    tag_images: TagImages,
}

impl EntryList {
    /// A list over `connection`; `None` when there is no index yet.
    #[must_use]
    pub fn new(connection: Option<Connection>) -> Self {
        let mut list = Self {
            connection,
            query: String::new(),
            filter: FilterState::default(),
            limit: PAGE_SIZE,
            entries: Vec::new(),
            selected: None,
            secondary_confirm: false,
            tag_images: TagImages::default(),
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

    /// The selected entry.
    #[must_use]
    pub fn selected_entry(&self) -> Option<&Entry> {
        self.selected.and_then(|ix| self.entries.get(ix.row))
    }

    /// Whether the last confirmation was the secondary one (⌘↩), which runs the primary action.
    #[must_use]
    pub fn is_secondary_confirm(&self) -> bool {
        self.secondary_confirm
    }

    /// Runs the query for the current text, filter and page window.
    fn refresh(&mut self) {
        let Some(connection) = &self.connection else {
            self.entries.clear();
            return;
        };
        let query = SearchQuery {
            text: self.query.clone(),
            filter: self.filter.clone(),
            limit: self.limit,
            offset: 0,
        };
        // A failed query keeps the previous results on screen rather than emptying the list.
        if let Ok(rows) = search(connection, &query) {
            self.entries = rows.iter().map(row_to_entry).collect();
        }
    }

    /// A row as Raycast's (DESIGN.md, List row): the type icon or the bookmark's favicon, the
    /// title, the muted description at the same size, then up to three tags (pills, or their
    /// images), `+N` and the link count.
    fn render_row(&self, entry: &Entry, selected: bool, cx: &App) -> ListItem {
        let (tags, more) = row_tags(&entry.tags);
        let badge = type_badge(&entry.entry_type);
        let tint = accent(badge.hue, cx).color;
        let type_icon = move || Icon::new(badge.icon).small().text_color(tint).into_any_element();
        let favicon = (type_key(&entry.entry_type) == TypeKey::Bookmark)
            .then(|| favicon_url(&entry.primary_content))
            .flatten();
        let muted = cx.theme().muted_foreground;

        ListItem::new(SharedString::from(entry.id.clone()))
            // The List wraps each row in its own ListItem node and reports only nodes with a role,
            // so the row needs one to carry its name to assistive technology.
            .role(Role::Label)
            .accessibility_label(row_label(entry))
            .selected(selected)
            // Inset and rounded, as Raycast's selection (tools/design/layout.json, resultRow).
            .mx(px(8.))
            .px(px(16.))
            .h(px(50.))
            .rounded(px(12.))
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .flex_none()
                            .w(px(24.))
                            .flex()
                            .justify_center()
                            .map(|column| match favicon {
                                Some(url) => column.child(img(url).size_4().with_fallback(type_icon)),
                                None => column.child(type_icon()),
                            }),
                    )
                    .child(
                        div()
                            .flex_none()
                            .max_w(relative(0.6))
                            .truncate()
                            .text_color(cx.theme().foreground)
                            .child(entry.title.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_color(muted)
                            .when_some(entry.description.clone(), ParentElement::child),
                    )
                    .children(tags.iter().map(|tag| self.render_tag(tag, cx)))
                    .when(more > 0, |row| {
                        row.child(div().text_color(muted).text_xs().child(format!("+{more}")))
                    })
                    .when(!entry.links.is_empty(), |row| {
                        row.child(
                            h_flex()
                                .gap_1()
                                .items_center()
                                .text_color(muted)
                                .text_xs()
                                .child(Icon::new(IconName::Link).xsmall())
                                .child(entry.links.len().to_string()),
                        )
                    }),
            )
    }

    /// A tag: its filled pill, holding its image when the tag-images folder has one. The image is drawn as a mask in
    /// the pill's text colour (Raycast's `tintColor`), so every image has the pill's size and contrast whatever its
    /// own colours and margins.
    fn render_tag(&self, tag: &str, cx: &App) -> AnyElement {
        let accent = accent(tag_hue(tag), cx);
        let pill = Tag::custom(accent.pill, accent.pill_text, accent.pill)
            .xsmall()
            .rounded(px(10.));
        let Some(path) = self.tag_images.get(tag) else {
            return pill.child(format!("#{tag}")).into_any_element();
        };
        let name = format!("#{tag}");
        div()
            .id(SharedString::from(format!("tag-{tag}")))
            .child(
                pill.child(
                    svg()
                        .external_path(path.to_string_lossy().into_owned())
                        .size_3()
                        .text_color(accent.pill_text),
                ),
            )
            .tooltip(move |window, cx| Tooltip::new(name.clone()).build(window, cx))
            .into_any_element()
    }
}

impl ListDelegate for EntryList {
    type Item = ListItem;

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

    fn render_item(&mut self, ix: IndexPath, _: &mut Window, cx: &mut Context<ListState<Self>>) -> Option<ListItem> {
        let entry = self.entries.get(ix.row)?;
        Some(self.render_row(entry, Some(ix) == self.selected, cx))
    }

    fn render_empty(&mut self, _: &mut Window, cx: &mut Context<ListState<Self>>) -> impl gpui_kit::IntoElement {
        let (title, description) = if self.has_index() {
            ("No matching entries", "Adjust the query or clear the filters.")
        } else {
            (
                "No entries indexed yet",
                "Press ⌘R to rebuild the index from your sources.",
            )
        };
        Empty::new()
            .header(
                EmptyHeader::new()
                    .title(EmptyTitle::new().child(title))
                    .description(EmptyDescription::new().child(description)),
            )
            .text_color(cx.theme().muted_foreground)
            .into_any_element()
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
