//! The panel's root view: the search list, the split pane, the detail and rebuild screens, and
//! every command's handler.

use std::fs;
use std::rc::Rc;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::list::{List, ListEvent, ListState};
use gpui_kit::component::{IndexPath, WindowExt as _, h_flex, v_flex};
use gpui_kit::{
    AnyElement, App, AppContext as _, ClipboardItem, Context, Entity, FocusHandle, InteractiveElement as _,
    IntoElement, ParentElement, Pixels, Render, ScrollHandle, Styled as _, Subscription, Window, div, point, relative,
};
use kb_domain::entry::Entry;
use kb_domain::search::FilterState;
use kb_domain::selection::Selection;
use kb_domain::sync::open_readonly;

mod footer;
mod header;
mod panel;
mod theme;

use crate::catalog::{Catalog, KeyItem, Screen as KeyScreen, fill};
use crate::config::Config;
use crate::entry_list::EntryList;
use crate::keys::{
    Back, CopyError, CopyLocation, CopyPath, FirstRow, HideDetails, LastRow, NextRow, OpenActions, OpenDetail,
    OpenFilter, OpenPreferences, OpenSourceFile, PageDown, PageUp, PreviousRow, Quit, Rebuild, RevealFile, RunPrimary,
    RunRow1, RunRow2, RunRow3, RunRow4, RunRow5, RunRow6, RunRow7, RunRow8, RunRow9, RunSecondary, ScreenCmdEnter,
    ScreenEnter, ScreenShiftCmdEnter, ShowDetails, TogglePane, ToggleWindowSize, caret_at_end,
};
use crate::navigator::{BackName, Nav, Navigator, Overlay, Place};
use crate::sync_view::{Phase, SyncFinished, SyncJob, SyncView};
use crate::tag_images::TagImages;
use crate::trace;
use crate::ui::action_panel::matches;
use crate::ui::design::{design, pixels, pt};
use crate::ui::entry_view::{EntryView, Layout, columns_for};
use crate::ui::notice;
use crate::ui::report_pane::report_pane;
use crate::window_size::{WindowSize, compact};

fn notes_line_step() -> Pixels {
    pixels(design().components.detail.scroll_step)
}

/// The panel's root view.
pub struct SearchView {
    config: Config,
    catalog: Rc<Catalog>,
    list: Entity<ListState<EntryList>>,
    query_input: Entity<InputState>,
    /// The panel's shared search field (D9, D15).
    panel_query: Entity<InputState>,
    /// The visible row the panel highlights.
    panel_selected: usize,
    /// Focus for the screens without the list, so their keys reach this view.
    focus: FocusHandle,
    /// Focus for the panel itself, so its keys reach it on the submenus (no search field).
    panel_focus: FocusHandle,
    /// Where the user is: the places and the overlay (navigation spec D2).
    navigator: Navigator,
    /// The refresh while `Place::Refresh` is on the stack.
    sync: Option<Entity<SyncView>>,
    /// Its `SyncFinished` subscription, dropped with it.
    sync_subscription: Option<Subscription>,
    show_details: bool,
    window_size: WindowSize,
    /// The detail screen's notes, scrolled by the keyboard.
    detail_scroll: ScrollHandle,
    /// The action panel's rows, scrolled as the selection moves.
    panel_scroll: ScrollHandle,
    /// The last notice the view showed (D17).
    last_notice: Option<String>,
    /// The theme files, as Theme… lists them (read when the action panel opens).
    themes: Vec<crate::actions::ThemeRow>,
    /// The theme in use: config.toml's, else the default.
    theme_id: String,
    /// The palette in use when Theme… opened, restored unless a theme is kept.
    theme_preview: Option<crate::theme::scheme::Palette>,
    /// Held only to keep the list, query and panel subscriptions alive.
    _subscriptions: Vec<Subscription>,
}

impl std::fmt::Debug for SearchView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchView")
            .field("navigator", &self.navigator)
            .field("show_details", &self.show_details)
            .finish_non_exhaustive()
    }
}

/// The theme in use at start: config.toml's, else DESIGN.md's default (themes spec D4).
fn theme_id_of(config: &Config) -> String {
    config
        .theme()
        .map_or_else(|| crate::ui::design::design().theme.clone(), str::to_owned)
}

impl SearchView {
    /// Opens the index if there is one and builds the list over it.
    pub fn new(config: Config, catalog: Catalog, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let connection = open_readonly(config.database()).ok();
        let tag_images = TagImages::scan(config.tag_images());
        let placeholder = catalog.messages.search_placeholder.clone();
        let catalog = Rc::new(catalog);
        let panel_query = cx.new(|cx| InputState::new(window, cx).placeholder(catalog.panel.search.clone()));
        let panel_sub = cx.subscribe_in(&panel_query, window, Self::on_panel_query_event);
        let list = cx.new(|cx| {
            ListState::new(
                EntryList::new(connection)
                    .with_tag_images(tag_images)
                    .with_catalog(catalog.clone()),
                window,
                cx,
            )
            .searchable(false)
        });
        let query_input = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder));
        // The first row starts selected, as in Raycast, so Enter and ⌘K act on it at once.
        list.update(cx, |list, cx| {
            if !list.delegate().entries().is_empty() {
                list.set_selected_index(Some(IndexPath::default()), window, cx);
            }
        });
        query_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
        let list_sub = cx.subscribe_in(&list, window, Self::on_list_event);
        let input_sub = cx.subscribe_in(&query_input, window, Self::on_query_input_event);
        let theme_id = theme_id_of(&config);
        Self {
            config,
            catalog,
            list,
            query_input,
            panel_query,
            panel_selected: 0,
            focus: cx.focus_handle(),
            panel_focus: cx.focus_handle(),
            navigator: Navigator::default(),
            sync: None,
            sync_subscription: None,
            // The list alone, as Raycast's kb Search opens; ⌘D shows the split pane.
            show_details: false,
            window_size: WindowSize::default(),
            detail_scroll: ScrollHandle::new(),
            panel_scroll: ScrollHandle::new(),
            last_notice: None,
            themes: Vec::new(),
            theme_id,
            theme_preview: None,
            _subscriptions: vec![list_sub, input_sub, panel_sub],
        }
    }

    /// The list state, for tests.
    #[must_use]
    pub fn list(&self) -> &Entity<ListState<EntryList>> {
        &self.list
    }

    /// The catalog the panel was built with: the entry types, actions, keys and words it shows.
    #[must_use]
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// Whether the detail screen is showing.
    #[must_use]
    pub fn is_showing_detail(&self) -> bool {
        self.navigator.top() == Place::Detail
    }

    /// Whether the list shows the split pane with the selected entry's notes.
    #[must_use]
    pub fn is_showing_details(&self) -> bool {
        self.show_details
    }

    /// The panel's current size.
    #[must_use]
    pub fn window_size(&self) -> WindowSize {
        self.window_size
    }

    /// How far the detail screen's notes are scrolled: 0 at the top, negative below it.
    #[must_use]
    pub fn detail_scroll_offset(&self) -> Pixels {
        self.detail_scroll.offset().y
    }

    /// How far the panel's rows are scrolled: 0 at the top, negative below it.
    #[must_use]
    pub fn panel_scroll_offset(&self) -> Pixels {
        self.panel_scroll.offset().y
    }

    /// The panel rows' scroll handle (audit branch only: lets a test check a row is in view).
    #[must_use]
    pub fn panel_scroll_handle(&self) -> ScrollHandle {
        self.panel_scroll.clone()
    }

    /// Whether the split pane currently has the keyboard (close-out D3).
    #[must_use]
    pub fn pane_has_keys(&self) -> bool {
        self.navigator.pane_has_keys()
    }

    /// Whether the rebuild screen is showing.
    #[must_use]
    pub fn is_rebuilding(&self) -> bool {
        self.navigator.contains(Place::Refresh)
    }

    /// The name `place` shows when a place opened from it says where back goes (D2, D5): the list is its query, or
    /// "Search" without one; an entry is its title; a refresh is "Refresh". The one function that names a place.
    fn place_name(&self, place: Place, cx: &App) -> BackName {
        let m = &self.catalog.messages;
        let plain = |text: &str| BackName {
            text: text.to_owned(),
            mono: false,
        };
        match place {
            Place::List => {
                let query = self.query_input.read(cx).value().to_string();
                if query.is_empty() {
                    plain(&m.back_search)
                } else {
                    BackName {
                        text: query,
                        mono: true,
                    }
                }
            }
            Place::Detail => self
                .list
                .read(cx)
                .delegate()
                .selected_entry()
                .map_or_else(|| plain(&m.back_search), |entry| plain(&entry.title)),
            Place::Refresh | Place::ReportDetail => plain(&m.back_refresh),
        }
    }

    /// Opens `place` on top of the current one: it remembers where it came from, starts scrolled to the top, and
    /// takes the keyboard.
    fn push(&mut self, place: Place, window: &mut Window, cx: &mut Context<Self>) {
        let back = self.place_name(self.navigator.top(), cx);
        self.navigator.push(place, back);
        self.detail_scroll.set_offset(gpui_kit::Point::default());
        self.apply_list_state(cx);
        self.focus_screen(window, cx);
        cx.notify();
    }

    /// Back one place, to the one underneath as it was left (D3).
    fn pop(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.navigator.pop();
        self.settle(window, cx);
    }

    /// Back to the list, from anywhere.
    fn pop_to_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.navigator.pop_to_list();
        self.settle(window, cx);
    }

    /// After a pop: the refresh and its subscription go with their place, and the place on top takes the keyboard.
    /// The one place that keeps `sync` in step with the stack.
    fn settle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.navigator.contains(Place::Refresh) {
            self.sync = None;
            self.sync_subscription = None;
        }
        self.apply_list_state(cx);
        self.focus_screen(window, cx);
        cx.notify();
    }

    /// Selects row `row` of the list (clamped; nothing with no results) and scrolls to it (D1). The list's one way to
    /// select: every move, filter change and refresh comes through here.
    fn select_entry(&mut self, row: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.navigator.set_pane_has_keys(false);
        self.detail_scroll.set_offset(gpui_kit::Point::default());
        self.list.update(cx, |list, cx| {
            let count = list.delegate().entries().len();
            let ix = (count > 0).then(|| IndexPath::new(row.min(count - 1)));
            list.set_selected_index(ix, window, cx);
            if ix.is_some() {
                list.scroll_to_selected_item(window, cx);
            }
        });
        self.apply_list_state(cx);
        cx.notify();
    }

    /// The list's selected row, if any.
    fn selected_row(&self, cx: &App) -> Option<usize> {
        self.list.read(cx).selected_index().map(|ix| ix.row)
    }

    /// Searches for `text`: the list debounces it and selects the first result.
    fn search(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.list.update(cx, |list, cx| list.set_query(text, window, cx));
    }

    /// Opens the selected entry's detail: only from the list, and only with an entry selected.
    fn show_detail_screen(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.navigator.top() != Place::List || self.list.read(cx).delegate().selected_entry().is_none() {
            return;
        }
        self.push(Place::Detail, window, cx);
    }

    fn on_query_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                trace::typed(cx);
                let text = state.read(cx).value().to_string();
                self.search(&text, window, cx);
            }
            InputEvent::PressEnter { secondary, .. } => {
                self.ladder(if *secondary { "cmd-enter" } else { "enter" }, window, cx);
            }
            _ => {}
        }
    }

    fn on_list_event(
        &mut self,
        _: &Entity<ListState<EntryList>>,
        event: &ListEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            ListEvent::Select(_) => cx.notify(),
            ListEvent::Confirm(_) => {
                let keys = if self.list.read(cx).delegate().is_secondary_confirm() {
                    "cmd-enter"
                } else {
                    "enter"
                };
                self.ladder(keys, window, cx);
            }
            // Esc in the list leaves the panel, as it closes the Raycast command.
            ListEvent::Cancel => {
                self.cancel_theme_preview(cx);
                cx.hide();
            }
        }
    }

    /// The screen whose keys apply now (8g adds the empty and refresh screens).
    fn key_screen(&self, cx: &App) -> KeyScreen {
        let list = {
            let list = self.list.read(cx).delegate();
            if !list.has_index() {
                KeyScreen::NoIndex
            } else if list.entries().is_empty() && self.query_input.read(cx).value() == list.query() {
                KeyScreen::Empty
            } else {
                KeyScreen::List
            }
        };
        let phase = self.sync.as_ref().map(|sync| sync.read(cx).phase());
        self.navigator.key_screen(list, phase)
    }

    /// The current screen's keys, resolved for the selected entry.
    fn screen_keys(&self, cx: &App) -> Vec<KeyItem> {
        let selected = self.list.read(cx).delegate().selected_entry();
        let kind = selected.and_then(|entry| self.catalog.entry_type(&entry.entry_type));
        self.catalog.keys_for(self.key_screen(cx), kind)
    }

    /// Runs what the current screen binds to `keys` (`enter`, `cmd-enter`, `shift-cmd-enter`).
    fn ladder(&mut self, keys: &str, window: &mut Window, cx: &mut Context<Self>) {
        // While the panel is open it owns ↵ (its own `Confirm` action), so the screen ladder must
        // not also run and switch the screen under it.
        if !self.navigator.screen_has_keys() {
            return;
        }
        let action = self
            .screen_keys(cx)
            .into_iter()
            .find(|item| item.keys == keys)
            .map(|item| item.action);
        if let Some(action) = action {
            self.run_action(&action, window, cx);
        }
    }

    /// The refresh screen's view while it shows a report, else `None`.
    fn report(&self, cx: &App) -> Option<Entity<SyncView>> {
        self.sync
            .clone()
            .filter(|_| matches!(self.navigator.top(), Place::Refresh | Place::ReportDetail))
            .filter(|sync| sync.read(cx).phase() == Phase::Report)
    }

    /// The selected report row's path, for tests.
    #[must_use]
    pub fn report_row_path(&self, cx: &App) -> Option<String> {
        self.report(cx)?.read(cx).selected_row(cx).map(|r| r.path.clone())
    }

    /// The current place, for tests.
    #[must_use]
    pub fn place(&self) -> Place {
        self.navigator.top()
    }

    /// Whether the report shows its pane beside the rows, for tests.
    #[must_use]
    pub fn report_pane_open(&self, cx: &App) -> bool {
        self.sync.as_ref().is_some_and(|sync| sync.read(cx).has_pane())
    }

    /// A key bound to catalog action `id`: it acts on the top place through `run_action`, and not at all while an
    /// overlay owns the keys (D4). The one entry point for every keyed command.
    fn key_action(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.navigator.screen_has_keys() {
            self.run_action(id, window, cx);
        }
    }

    /// Runs catalog action `id` on the top place (D4): the selected entry on the list and its detail, the selected
    /// report row on a refresh and its detail. A row action with nothing to act on does nothing.
    fn run_action(&mut self, action: &str, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.navigator.top(), Place::Refresh | Place::ReportDetail) {
            let row = self.report(cx).and_then(|sync| sync.read(cx).selected_row(cx).cloned());
            match (action, row) {
                ("details", Some(_)) => return self.push(Place::ReportDetail, window, cx),
                ("edit", Some(row)) => {
                    if let Some(file) = row.file() {
                        let (line, column) = row.line_and_column();
                        self.open_in_editor(std::path::Path::new(file), line, column, window, cx);
                    } else if row.configures() {
                        self.configure(window, cx);
                    }
                    return;
                }
                ("copy-location", Some(row)) => {
                    if let Some(text) = row.location() {
                        self.copy(text, window, cx);
                    }
                    return;
                }
                ("copy-error", Some(row)) => {
                    if let Some(text) = row.error() {
                        self.copy(text, window, cx);
                    }
                    return;
                }
                ("copy-path", Some(row)) => {
                    if let Some(file) = row.file() {
                        self.copy(file.to_owned(), window, cx);
                    }
                    return;
                }
                ("reveal", Some(row)) => {
                    if let Some(file) = row.file().map(std::path::Path::new) {
                        if file.exists() {
                            cx.reveal_path(file);
                        } else {
                            let notice = fill(
                                &self.catalog.messages.open_missing,
                                &[("file", &file.display().to_string())],
                            );
                            self.notify(notice, window, cx);
                        }
                    }
                    return;
                }
                ("refresh" | "retry", _) => {
                    let selection = self
                        .sync
                        .as_ref()
                        .map_or(Selection::All, |sync| sync.read(cx).selection());
                    return self.refresh(selection, window, cx);
                }
                ("details" | "edit" | "copy-location" | "copy-error" | "copy-path" | "reveal" | "open" | "copy", _) => {
                    return;
                }
                _ => {}
            }
        }
        let content = self
            .list
            .read(cx)
            .delegate()
            .selected_entry()
            .map(|entry| entry.primary_content.clone());
        match (action, content) {
            ("details", _) => self.show_detail_screen(window, cx),
            ("open", Some(url)) => cx.open_url(&url),
            ("copy", Some(text)) => self.copy(text, window, cx),
            ("refresh", _) => self.refresh(Selection::All, window, cx),
            ("refresh-yaml", _) => self.refresh(Selection::Yaml, window, cx),
            ("refresh-skills", _) => self.refresh(Selection::Skills, window, cx),
            ("configure", _) => self.configure(window, cx),
            ("resize", _) => self.resize(window, cx),
            ("edit", _) => {
                if let Some(path) = self.selected_source_file(cx) {
                    self.open_in_editor(std::path::Path::new(&path), None, None, window, cx);
                }
            }
            ("copy-path", _) => {
                if let Some(path) = self.selected_source_file(cx) {
                    self.copy(path, window, cx);
                }
            }
            ("stop", _) => {
                if let Some(sync) = &self.sync {
                    sync.read(cx).cancel();
                }
            }
            ("back", _) => self.back(&Back, window, cx),
            _ => {}
        }
    }

    /// Copies `text` and says so (D17).
    pub(super) fn copy(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.notice_copied(window, cx);
    }

    /// The last notice the view showed, for tests.
    #[must_use]
    pub fn last_notice(&self) -> Option<&str> {
        self.last_notice.as_deref()
    }

    /// The selected entry's primary (`secondary` false) or secondary action.
    fn run_entry(&mut self, secondary: bool, window: &mut Window, cx: &mut Context<Self>) {
        let action = self.list.read(cx).delegate().selected_entry().and_then(|entry| {
            let kind = self.catalog.entry_type(&entry.entry_type)?;
            if secondary {
                kind.secondary.clone()
            } else {
                Some(kind.primary.clone())
            }
        });
        if let Some(action) = action {
            self.run_action(&action, window, cx);
        }
    }

    fn run_primary(&mut self, _: &RunPrimary, window: &mut Window, cx: &mut Context<Self>) {
        if !self.navigator.screen_has_keys() {
            return;
        }
        self.run_entry(false, window, cx);
    }

    fn run_secondary(&mut self, _: &RunSecondary, window: &mut Window, cx: &mut Context<Self>) {
        if !self.navigator.screen_has_keys() {
            return;
        }
        self.run_entry(true, window, cx);
    }

    fn screen_enter(&mut self, _: &ScreenEnter, window: &mut Window, cx: &mut Context<Self>) {
        self.ladder("enter", window, cx);
    }

    fn screen_cmd_enter(&mut self, _: &ScreenCmdEnter, window: &mut Window, cx: &mut Context<Self>) {
        self.ladder("cmd-enter", window, cx);
    }

    fn screen_shift_cmd_enter(&mut self, _: &ScreenShiftCmdEnter, window: &mut Window, cx: &mut Context<Self>) {
        self.ladder("shift-cmd-enter", window, cx);
    }

    /// ⌘n: on the detail screen opens link n − 1 (D20); on the list selects row n − 1 and runs its primary action (R7).
    fn run_row(&mut self, n: usize, window: &mut Window, cx: &mut Context<Self>) {
        if !self.navigator.screen_has_keys() {
            return; // A07: ⌘n under the action panel or the filter
        }
        match self.navigator.top() {
            Place::Detail => {
                let url = self
                    .list
                    .read(cx)
                    .delegate()
                    .selected_entry()
                    .and_then(|entry| entry.links.get(n - 1))
                    .map(|link| link.url.clone());
                if let Some(url) = url {
                    cx.open_url(&url);
                }
            }
            Place::List => {
                if self.list.read(cx).delegate().entries().len() >= n {
                    self.list.update(cx, |list, cx| {
                        list.set_selected_index(Some(IndexPath::new(n - 1)), window, cx);
                    });
                    self.run_entry(false, window, cx);
                }
            }
            Place::Refresh | Place::ReportDetail => {}
        }
    }

    fn copy_path(&mut self, _: &CopyPath, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("copy-path", window, cx);
    }

    /// The selected entry's source file, absolute, for the copy and open commands.
    fn selected_source_file(&self, cx: &App) -> Option<String> {
        self.list
            .read(cx)
            .delegate()
            .selected_entry()
            .map(|e| e.source_file.clone())
    }

    /// → (`open`) or ← on the place that has a pane: the list's split, or the report's (A13).
    fn set_pane(&mut self, open: bool, cx: &mut Context<Self>) {
        if !self.navigator.screen_has_keys() {
            return;
        }
        if !open {
            self.navigator.set_pane_has_keys(false);
            self.detail_scroll.set_offset(gpui_kit::Point::default());
        }
        match self.navigator.top() {
            Place::List => self.show_details = open,
            Place::Refresh => {
                if let Some(sync) = self.report(cx) {
                    sync.update(cx, |s, cx| s.set_pane(open, cx));
                }
            }
            Place::Detail | Place::ReportDetail => {}
        }
        self.apply_list_state(cx);
        cx.notify();
    }

    /// ⇥ / ⇧⇥ on a split: the keys to its pane, or back to its list (close-out D3). Nothing without a split.
    fn toggle_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.navigator.screen_has_keys() {
            return;
        }
        let split = match self.navigator.top() {
            Place::List => self.show_details && self.list.read(cx).delegate().selected_entry().is_some(),
            Place::Refresh => self.report(cx).is_some_and(|sync| sync.read(cx).has_pane()),
            Place::Detail | Place::ReportDetail => false,
        };
        if !split {
            return;
        }
        let pane = !self.navigator.pane_has_keys();
        self.navigator.set_pane_has_keys(pane);
        self.apply_list_state(cx);
        self.focus_screen(window, cx);
        cx.notify();
    }

    /// The list's display state, derived from the place and the split: compact rows beside an open pane, and a faint
    /// selection while the pane has the keys. Called where they change (close-out D7, A16), never from `render`.
    fn apply_list_state(&mut self, cx: &mut Context<Self>) {
        let on_list = self.navigator.top() == Place::List;
        let compact = on_list && self.show_details && self.list.read(cx).delegate().selected_entry().is_some();
        let idle = self.navigator.pane_has_keys();
        self.list.update(cx, |list, cx| {
            list.delegate_mut().set_compact(compact);
            list.delegate_mut().set_idle(idle && on_list);
            cx.notify();
        });
        if let Some(sync) = self.sync.clone() {
            sync.update(cx, |s, cx| s.set_idle_and_focus(idle, cx));
        }
    }

    fn show_details(&mut self, _: &ShowDetails, _: &mut Window, cx: &mut Context<Self>) {
        self.set_pane(true, cx);
    }

    fn hide_details(&mut self, _: &HideDetails, _: &mut Window, cx: &mut Context<Self>) {
        self.set_pane(false, cx);
    }

    /// Whether → and ← should act on the split pane: the caret is at the query's end.
    fn caret_at_end(&self, cx: &App) -> bool {
        let input = self.query_input.read(cx);
        caret_at_end(input.cursor(), input.selected_range(), input.value().len())
    }

    /// Whole rows that fit `visible` points of rows `row` points tall; at least one (close-out D2).
    fn page_rows(visible: f32, row: f32) -> usize {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a few dozen rows"
        )]
        let rows = (visible / row).floor() as usize;
        rows.max(1)
    }

    /// The step `key` makes over `count` rows from `current`, `page` rows to a page.
    fn step(key: Nav, current: usize, count: usize, page: usize) -> usize {
        let last = count.saturating_sub(1);
        match key {
            Nav::Up => current.saturating_sub(1),
            Nav::Down => (current + 1).min(last),
            Nav::First => 0,
            Nav::Last => last,
            Nav::PageUp => current.saturating_sub(page),
            Nav::PageDown => (current + page).min(last),
        }
    }

    /// Every navigation key, on every surface (close-out D2): the open overlay's rows, a scrolling detail or pane,
    /// the report's rows, or the list. The one place a navigation key is routed.
    fn nav(&mut self, key: Nav, window: &mut Window, cx: &mut Context<Self>) {
        let d = design();
        if !self.navigator.screen_has_keys() {
            let page = Self::page_rows(d.components.panel.max_height, d.components.panel.row);
            let count = matches(&self.current_page(cx).page, &self.panel_query.read(cx).value()).len();
            return self.select_panel_row(Self::step(key, self.panel_selected, count, page), cx);
        }
        let scrolls =
            matches!(self.navigator.top(), Place::Detail | Place::ReportDetail) || self.navigator.pane_has_keys();
        if scrolls {
            return self.scroll(key, cx);
        }
        let rows_height =
            f32::from(window.viewport_size().height) - d.components.header.height - d.components.footer.height;
        let page = Self::page_rows(rows_height, d.components.row.height);
        match self.navigator.top() {
            Place::Refresh => {
                if let Some(sync) = self.report(cx) {
                    sync.update(cx, |s, cx| s.step_selection(key, page, window, cx));
                    self.detail_scroll.set_offset(gpui_kit::Point::default());
                }
            }
            Place::List => {
                let count = self.list.read(cx).delegate().entries().len();
                let row = Self::step(key, self.selected_row(cx).unwrap_or(0), count, page);
                self.select_entry(row, window, cx);
            }
            Place::Detail | Place::ReportDetail => {}
        }
    }

    /// A scrolling detail or pane: ↑↓ a line, ⌘↑↓ its ends, Page Up/Down a page.
    fn scroll(&mut self, key: Nav, cx: &mut Context<Self>) {
        let max = self.detail_scroll.max_offset().y;
        let offset = self.detail_scroll.offset();
        let y = match key {
            Nav::Up => next_offset(offset.y, -notes_line_step(), max),
            Nav::Down => next_offset(offset.y, notes_line_step(), max),
            Nav::First => pixels(0.),
            Nav::Last => -max,
            Nav::PageUp => next_offset(offset.y, -self.notes_page(), max),
            Nav::PageDown => next_offset(offset.y, self.notes_page(), max),
        };
        self.detail_scroll.set_offset(point(offset.x, y));
        cx.notify();
    }

    /// A page: the visible height, less a few lines kept for context.
    fn notes_page(&self) -> Pixels {
        (self.detail_scroll.bounds().size.height - notes_line_step()).max(notes_line_step())
    }

    fn toggle_window_size(&mut self, _: &ToggleWindowSize, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("resize", window, cx);
    }

    /// The size toggle (⌘⇧F): compact or expanded, keeping the centre.
    fn resize(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.window_size = self.window_size.toggled();
        let display = window
            .display(cx)
            .map_or_else(compact, |display| display.visible_bounds().size);
        window.resize(self.window_size.size(display));
        cx.notify();
    }

    fn open_source_file(&mut self, _: &OpenSourceFile, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("edit", window, cx);
    }

    /// Opens `file` in the editor at `line`:`column` (D6), saying so when it cannot.
    pub(super) fn open_in_editor(
        &mut self,
        file: &std::path::Path,
        line: Option<usize>,
        column: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let m = &self.catalog.messages;
        let file_text = file.display().to_string();
        let notice = match crate::editor::open(self.config.editor(), file, line, column, self.config.home(), cx) {
            crate::editor::Opened::Editor | crate::editor::Opened::DefaultApp => None,
            crate::editor::Opened::Missing => Some(fill(&m.open_missing, &[("file", &file_text)])),
            crate::editor::Opened::Fallback { program, error } => {
                Some(fill(&m.editor_failed, &[("program", &program), ("error", &error)]))
            }
        };
        if let Some(notice) = notice {
            self.notify(notice, window, cx);
        }
    }

    fn notify(&mut self, notice: String, window: &mut Window, cx: &mut Context<Self>) {
        self.last_notice = Some(notice.clone());
        window.push_notification(notice, cx);
    }

    fn open_preferences(&mut self, _: &OpenPreferences, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("configure", window, cx);
    }

    /// Opens config.toml, writing its template first when there is none.
    fn configure(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let file = self.config.preferences().to_path_buf();
        if !file.exists() {
            let template = crate::catalog::preferences_template(self.config.sources_dir());
            let written = file
                .parent()
                .map_or(Ok(()), fs::create_dir_all)
                .and_then(|()| fs::write(&file, template));
            if let Err(error) = written {
                window.push_notification(format!("Couldn’t create {}: {error}", file.display()), cx);
                return;
            }
        }
        cx.open_with_system(&file);
    }

    fn rebuild(&mut self, _: &Rebuild, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("refresh", window, cx);
    }

    /// Refreshes `selection`'s sources on the refresh screen, unless a refresh is running.
    fn refresh(&mut self, selection: Selection, window: &mut Window, cx: &mut Context<Self>) {
        if self.sync.as_ref().is_some_and(|sync| sync.read(cx).is_running()) {
            return;
        }
        let job = SyncJob {
            selection,
            sources: self.config.sources_dir().to_path_buf(),
            skills_command: self.config.skills_command(),
            database: self.config.database().to_path_buf(),
        };
        let catalog = self.catalog.clone();
        let sync = cx.new(|cx| SyncView::start(job, catalog, self.detail_scroll.clone(), window, cx));
        let subscription = cx.subscribe_in(&sync, window, |this, _, finished: &SyncFinished, window, cx| {
            window.push_notification(finished.message.clone(), cx);
            if finished.changed {
                let kept = this.list.read(cx).delegate().selected_entry().map(|e| e.id.clone());
                let connection = open_readonly(this.config.database()).ok();
                this.list.update(cx, |list, cx| {
                    list.delegate_mut().set_connection(connection);
                    cx.notify();
                });
                let row = kept
                    .and_then(|id| this.list.read(cx).delegate().row_of(&id))
                    .unwrap_or(0);
                this.select_entry(row, window, cx);
            }
        });
        // The previous run's subscription goes with it.
        self.sync_subscription = Some(subscription);
        self.sync = Some(sync);
        match self.navigator.top() {
            // A rerun replaces the refresh in place; from a row's detail, back to the report first.
            Place::Refresh => {
                self.focus_screen(window, cx);
                cx.notify();
            }
            Place::ReportDetail => {
                self.navigator.pop();
                self.focus_screen(window, cx);
                cx.notify();
            }
            // A refresh always opens on the list: the index it rebuilds may move or drop the entry a detail shows.
            Place::List | Place::Detail => {
                self.navigator.pop_to_list();
                self.push(Place::Refresh, window, cx);
            }
        }
    }

    /// The sources of the refresh on screen, for tests.
    #[must_use]
    pub fn refresh_selection(&self, cx: &App) -> Option<Selection> {
        self.sync.as_ref().map(|sync| sync.read(cx).selection())
    }

    fn back(&mut self, _: &Back, window: &mut Window, cx: &mut Context<Self>) {
        // 1. An overlay: clear its search, then leave the submenu, then close it (Raycast).
        if !self.navigator.screen_has_keys() {
            if !self.panel_query.read(cx).value().is_empty() {
                return self.set_panel_query("", window, cx);
            }
            // Leaving Theme… without ↵ restores the theme it opened on.
            self.cancel_theme_preview(cx);
            let next = match self.navigator.overlay() {
                Overlay::Actions(Some(crate::actions::Submenu::Refresh | crate::actions::Submenu::Theme)) => {
                    Overlay::Actions(Some(crate::actions::Submenu::More))
                }
                Overlay::Actions(Some(_)) => Overlay::Actions(None),
                _ => Overlay::None,
            };
            return self.show_overlay(next, window, cx);
        }
        // The pane: Esc gives the keys back to its list (close-out D3).
        if self.navigator.pane_has_keys() {
            self.navigator.set_pane_has_keys(false);
            self.apply_list_state(cx);
            self.focus_screen(window, cx);
            cx.notify();
            return;
        }
        // 2. A running refresh: Esc stops it.
        if let Some(sync) = self.sync.clone().filter(|_| self.navigator.top() == Place::Refresh)
            && sync.read(cx).is_running()
        {
            sync.read(cx).cancel();
            cx.notify();
            return;
        }
        // 3. The list: clear the query, then hide the panel.
        if self.navigator.top() == Place::List {
            if self.query_input.read(cx).value().is_empty() {
                self.cancel_theme_preview(cx);
                cx.hide();
            } else {
                self.query_input.update(cx, |input, cx| input.set_value("", window, cx));
                self.search("", window, cx);
            }
            cx.notify();
            return;
        }
        // 4. Anything else: back one place.
        self.pop(window, cx);
    }

    fn copy_location(&mut self, _: &CopyLocation, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("copy-location", window, cx);
    }

    fn copy_error(&mut self, _: &CopyError, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("copy-error", window, cx);
    }

    fn reveal_file(&mut self, _: &RevealFile, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("reveal", window, cx);
    }

    fn open_actions(&mut self, _: &OpenActions, window: &mut Window, cx: &mut Context<Self>) {
        let next = if self.is_actions_open() {
            Overlay::None
        } else {
            Overlay::Actions(None)
        };
        self.show_overlay(next, window, cx);
    }

    fn open_detail(&mut self, _: &OpenDetail, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("details", window, cx);
    }

    /// Hands the keyboard to the current screen: the query field in the list, the panel elsewhere.
    fn focus_screen(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.navigator.top() == Place::List && !self.navigator.pane_has_keys() {
            self.query_input.update(cx, |input, cx| input.focus(window, cx));
        } else {
            self.focus.focus(window, cx);
        }
    }

    fn open_filter(&mut self, _: &OpenFilter, window: &mut Window, cx: &mut Context<Self>) {
        let next = if self.is_filter_open() {
            Overlay::None
        } else if self.navigator.top() == Place::List {
            Overlay::Filter
        } else {
            return;
        };
        self.show_overlay(next, window, cx);
    }

    fn apply_filter(&mut self, filter: FilterState, window: &mut Window, cx: &mut Context<Self>) {
        self.list.update(cx, |list, cx| {
            list.delegate_mut().set_filter(filter);
            cx.notify();
        });
        self.select_entry(0, window, cx);
    }
}
impl SearchView {
    /// The entry view of `entry` in `layout`, its command card as wide as the pane allows.
    fn entry_view(&self, entry: &Entry, layout: Layout, window: &Window, cx: &mut Context<Self>) -> EntryView {
        let d = design();
        let viewport = f32::from(window.viewport_size().width);
        let (share, pad, role) = match layout {
            Layout::Split => (
                1. - d.components.split.list,
                d.content_inset(),
                &d.typography.roles.card_split,
            ),
            Layout::Detail => (1., d.content_inset(), &d.typography.roles.card_detail),
        };
        let width = viewport * share
            - 2. * pad
            - d.spacing.card.pad_x
            - d.spacing.card.pad_right
            - 2. * d.components.border.hairline;
        let font = gpui_kit::font(d.typography.mono.clone());
        let advance = window
            .text_system()
            .em_advance(window.text_system().resolve_font(&font), pixels(role.size))
            .map_or(role.size * 0.6, f32::from);
        let view = cx.entity().downgrade();
        let (on_tag_view, on_copy_view) = (view.clone(), view.clone());
        let images = self.list.read(cx).delegate().tag_images().clone();
        EntryView::new(
            entry.clone(),
            self.catalog.clone(),
            images,
            layout,
            columns_for(width, advance),
            Rc::new(move |tag, window, cx| {
                let tag = tag.to_owned();
                let _ = on_tag_view.update(cx, |this, cx| this.filter_by_tag(&tag, window, cx));
            }),
            Rc::new(|url, _, cx| cx.open_url(url)),
            Rc::new(move |_, window, cx| {
                let _ = on_copy_view.update(cx, |this, cx| this.notice_copied(window, cx));
            }),
        )
    }

    /// A header tag clicked: the list, filtered by that tag alone (D20).
    fn filter_by_tag(&mut self, tag: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_filter(
            FilterState {
                types: Vec::new(),
                tags: vec![tag.to_owned()],
            },
            window,
            cx,
        );
        self.pop_to_list(window, cx);
    }

    /// The copy notice, after the command card's own button copied (D17).
    fn notice_copied(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.last_notice = Some(self.catalog.messages.copied.clone());
        window.push_notification(notice::copied(&self.catalog.messages), cx);
    }

    /// The screen: the list and its split pane, the detail screen, or the rebuild.
    fn render_content(&self, selected: Option<&Entry>, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let tokens = design();
        let list_panel = || {
            List::new(&self.list)
                .px(pt(tokens.spacing.list.pad))
                .py(pt(tokens.spacing.list.pad - tokens.spacing.list.gap / 2.))
                .into_any_element()
        };
        match self.navigator.top() {
            Place::Refresh => self
                .sync
                .clone()
                .map_or_else(|| div().into_any_element(), IntoElement::into_any_element),
            Place::ReportDetail => match self.report(cx).and_then(|sync| sync.read(cx).selected_row(cx).cloned()) {
                Some(row) => {
                    report_pane(&row, &self.catalog.messages, true, Some(&self.detail_scroll)).into_any_element()
                }
                None => div().into_any_element(),
            },
            Place::Detail => selected.map_or_else(
                || div().into_any_element(),
                |entry| {
                    self.entry_view(entry, Layout::Detail, window, cx)
                        .track_scroll(self.detail_scroll.clone())
                        .into_any_element()
                },
            ),
            Place::List => match selected.filter(|_| self.show_details) {
                Some(entry) => h_flex()
                    .size_full()
                    .child(
                        div()
                            .h_full()
                            .w(relative(tokens.components.split.list))
                            .flex_none()
                            .border_r(pt(tokens.components.border.hairline))
                            .border_color(tokens.colors.edge)
                            .child(list_panel()),
                    )
                    .child(
                        div().h_full().flex_1().min_w_0().child(
                            self.entry_view(entry, Layout::Split, window, cx)
                                .track_scroll(self.detail_scroll.clone())
                                .focused(self.navigator.pane_has_keys()),
                        ),
                    )
                    .into_any_element(),
                None => list_panel(),
            },
        }
    }
}

impl Render for SearchView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = design();
        let selected = self.list.read(cx).delegate().selected_entry().cloned();
        let context = self.navigator.key_context();
        v_flex()
            .track_focus(&self.focus)
            .key_context(context.as_str())
            .on_action(cx.listener(Self::open_actions))
            .on_action(cx.listener(Self::open_filter))
            .on_action(cx.listener(Self::run_primary))
            .on_action(cx.listener(Self::run_secondary))
            .on_action(cx.listener(Self::screen_enter))
            .on_action(cx.listener(Self::screen_cmd_enter))
            .on_action(cx.listener(Self::screen_shift_cmd_enter))
            .on_action(cx.listener(|this, _: &RunRow1, window, cx| this.run_row(1, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow2, window, cx| this.run_row(2, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow3, window, cx| this.run_row(3, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow4, window, cx| this.run_row(4, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow5, window, cx| this.run_row(5, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow6, window, cx| this.run_row(6, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow7, window, cx| this.run_row(7, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow8, window, cx| this.run_row(8, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow9, window, cx| this.run_row(9, window, cx)))
            .on_action(cx.listener(Self::copy_path))
            .on_action(cx.listener(Self::show_details))
            .on_action(cx.listener(Self::hide_details))
            .on_action(cx.listener(Self::toggle_window_size))
            .on_action(cx.listener(|this, _: &TogglePane, w, cx| this.toggle_pane(w, cx)))
            .on_action(cx.listener(Self::open_detail))
            .on_action(cx.listener(Self::open_source_file))
            .on_action(cx.listener(Self::rebuild))
            .on_action(cx.listener(Self::open_preferences))
            .on_action(cx.listener(Self::back))
            .on_action(cx.listener(Self::copy_location))
            .on_action(cx.listener(Self::copy_error))
            .on_action(cx.listener(Self::reveal_file))
            .on_action(cx.listener(|this, _: &PreviousRow, w, cx| this.nav(Nav::Up, w, cx)))
            .on_action(cx.listener(|this, _: &NextRow, w, cx| this.nav(Nav::Down, w, cx)))
            .on_action(cx.listener(|this, _: &FirstRow, w, cx| this.nav(Nav::First, w, cx)))
            .on_action(cx.listener(|this, _: &LastRow, w, cx| this.nav(Nav::Last, w, cx)))
            .on_action(cx.listener(|this, _: &PageUp, w, cx| this.nav(Nav::PageUp, w, cx)))
            .on_action(cx.listener(|this, _: &PageDown, w, cx| this.nav(Nav::PageDown, w, cx)))
            .on_action(|_: &Quit, _, cx| cx.quit())
            .size_full()
            .rounded(pt(tokens.rounded.window))
            .bg(tokens.colors.window)
            .border_1()
            .border_color(tokens.colors.ring)
            .overflow_hidden()
            .text_color(tokens.colors.text)
            .child(self.render_header(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(self.render_content(selected.as_ref(), window, cx)),
            )
            .child(self.render_footer(selected.as_ref(), window, cx))
    }
}

/// The notes' next scroll offset after moving `delta` down (negative: up), kept between the top (0)
/// and the bottom (`-max`). A scroll offset grows more negative as the content moves up.
fn next_offset(current: Pixels, delta: Pixels, max: Pixels) -> Pixels {
    let next = current - delta;
    if next > pixels(0.) {
        pixels(0.)
    } else if next < -max {
        -max
    } else {
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolls_within_the_notes_and_stops_at_their_ends() {
        assert_eq!(next_offset(pixels(0.), pixels(40.), pixels(100.)), pixels(-40.));
        assert_eq!(
            next_offset(pixels(-80.), pixels(40.), pixels(100.)),
            pixels(-100.),
            "stops at the bottom"
        );
        assert_eq!(
            next_offset(pixels(-20.), pixels(-40.), pixels(100.)),
            pixels(0.),
            "stops at the top"
        );
        assert_eq!(
            next_offset(pixels(0.), pixels(40.), pixels(0.)),
            pixels(0.),
            "short notes do not scroll"
        );
    }
}
