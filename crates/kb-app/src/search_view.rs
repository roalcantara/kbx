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
use kb_domain::sync::open_readonly;

mod footer;
mod header;
mod panel;

use crate::catalog::{Catalog, KeyItem, Screen as KeyScreen};
use crate::config::Config;
use crate::entry_list::EntryList;
use crate::keys::{
    Back, CONTEXT, CopyPath, DETAIL_CONTEXT, HideDetails, OpenActions, OpenDetail, OpenFilter, OpenPreferences,
    OpenSourceFile, PageNotesDown, PageNotesUp, Quit, REFRESH_CONTEXT, Rebuild, RunPrimary, RunRow1, RunRow2, RunRow3,
    RunRow4, RunRow5, RunRow6, RunRow7, RunRow8, RunRow9, RunSecondary, ScreenCmdEnter, ScreenEnter,
    ScreenShiftCmdEnter, ScrollNotesDown, ScrollNotesUp, ShowDetails, ToggleWindowSize, caret_at_end,
};
use crate::sync_view::{Phase, SyncFinished, SyncView};
use crate::tag_images::TagImages;
use crate::trace;
use crate::ui::design::{design, pixels, pt};
use crate::ui::entry_view::{EntryView, Layout, columns_for};
use crate::ui::notice;
use crate::window_size::{WindowSize, compact};

/// What fills the panel.
#[derive(Debug)]
enum Screen {
    /// The list, with the split pane when shown.
    Search,
    /// One entry's notes, full width.
    Detail,
    /// A rebuild and its report.
    Rebuild(Entity<SyncView>),
}

fn notes_line_step() -> Pixels {
    pixels(design().components.detail.scroll_step)
}

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
    screen: Screen,
    show_details: bool,
    window_size: WindowSize,
    /// The detail screen's notes, scrolled by the keyboard.
    detail_scroll: ScrollHandle,
    /// The action panel's rows, scrolled as the selection moves.
    panel_scroll: ScrollHandle,
    /// The overlay open over the screen, if any.
    overlay: Overlay,
    /// The last notice the view showed (D17).
    last_notice: Option<String>,
    subscriptions: Vec<Subscription>,
}

impl std::fmt::Debug for SearchView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchView")
            .field("screen", &self.screen)
            .field("show_details", &self.show_details)
            .finish_non_exhaustive()
    }
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
        Self {
            config,
            catalog,
            list,
            query_input,
            panel_query,
            panel_selected: 0,
            focus: cx.focus_handle(),
            panel_focus: cx.focus_handle(),
            screen: Screen::Search,
            // The list alone, as Raycast's kb Search opens; ⌘D shows the split pane.
            show_details: false,
            window_size: WindowSize::default(),
            detail_scroll: ScrollHandle::new(),
            panel_scroll: ScrollHandle::new(),
            overlay: Overlay::None,
            last_notice: None,
            subscriptions: vec![list_sub, input_sub, panel_sub],
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
        matches!(self.screen, Screen::Detail)
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
    /// Whether the rebuild screen is showing.
    #[must_use]
    pub fn is_rebuilding(&self) -> bool {
        matches!(self.screen, Screen::Rebuild(_))
    }

    /// Moves the list selection by `delta` rows, clamped to the results, and scrolls to it.
    fn move_selection(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        self.list.update(cx, |list, cx| {
            let count = list.delegate().entries().len();
            if count == 0 {
                return;
            }
            let current = list.selected_index().map_or(0, |ix| ix.row);
            let row = current.saturating_add_signed(delta).min(count - 1);
            list.set_selected_index(Some(IndexPath::new(row)), window, cx);
            list.scroll_to_selected_item(window, cx);
        });
        cx.notify();
    }

    /// Searches for `text`: the list debounces it and selects the first result.
    fn search(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.list.update(cx, |list, cx| list.set_query(text, window, cx));
    }

    fn show_detail_screen(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.list.read(cx).delegate().selected_entry().is_none() {
            return;
        }
        self.screen = Screen::Detail;
        self.detail_scroll.set_offset(gpui_kit::Point::default());
        self.focus.focus(window, cx);
        cx.notify();
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
            ListEvent::Cancel => cx.hide(),
        }
    }

    /// The screen whose keys apply now (8g adds the empty and refresh screens).
    fn key_screen(&self, cx: &App) -> KeyScreen {
        // An open filter or Links… page has keys of its own (catalog `screens.filter`, `.links`).
        match self.overlay {
            Overlay::Filter => return KeyScreen::Filter,
            Overlay::Actions(Some(crate::actions::Submenu::Links)) => return KeyScreen::Links,
            _ => {}
        }
        match &self.screen {
            Screen::Detail => KeyScreen::Detail,
            Screen::Rebuild(sync) => match sync.read(cx).phase() {
                Phase::Running => KeyScreen::Refreshing,
                Phase::Report => KeyScreen::Report,
                Phase::Stopped => KeyScreen::Stopped,
                Phase::Failed => KeyScreen::Failed,
            },
            Screen::Search => {
                let list = self.list.read(cx).delegate();
                if !list.has_index() {
                    KeyScreen::NoIndex
                } else if list.entries().is_empty() && self.query_input.read(cx).value() == list.query() {
                    KeyScreen::Empty
                } else {
                    KeyScreen::List
                }
            }
        }
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
        if self.overlay != Overlay::None {
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

    /// Runs a catalog action by id on the selected entry.
    fn run_action(&mut self, action: &str, window: &mut Window, cx: &mut Context<Self>) {
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
            ("refresh", _) => self.rebuild(&Rebuild, window, cx),
            ("configure", _) => self.open_preferences(&OpenPreferences, window, cx),
            ("resize", _) => self.toggle_window_size(&ToggleWindowSize, window, cx),
            ("edit", _) => self.open_source_file(&OpenSourceFile, window, cx),
            ("copy-path", _) => self.copy_path(&CopyPath, window, cx),
            ("stop", _) => {
                if let Screen::Rebuild(sync) = &self.screen {
                    sync.read(cx).cancel();
                }
            }
            ("back", _) => self.back(&Back, window, cx),
            ("retry", _) => {
                // A finished or failed sync lets `rebuild` start a new one.
                self.rebuild(&Rebuild, window, cx);
            }
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
        self.run_entry(false, window, cx);
    }

    fn run_secondary(&mut self, _: &RunSecondary, window: &mut Window, cx: &mut Context<Self>) {
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
        if matches!(self.screen, Screen::Detail) {
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
            return;
        }
        if !matches!(self.screen, Screen::Search) {
            return;
        }
        let exists = self.list.read(cx).delegate().entries().len() >= n;
        if !exists {
            return;
        }
        self.list.update(cx, |list, cx| {
            list.set_selected_index(Some(IndexPath::new(n - 1)), window, cx);
        });
        self.run_entry(false, window, cx);
    }

    fn copy_path(&mut self, _: &CopyPath, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = self.selected_source_file(cx) {
            self.copy(path, window, cx);
        }
    }

    /// The selected entry's source file, absolute, for the copy and open commands.
    fn selected_source_file(&self, cx: &App) -> Option<String> {
        self.list
            .read(cx)
            .delegate()
            .selected_entry()
            .map(|e| e.source_file.clone())
    }

    fn show_details(&mut self, _: &ShowDetails, _: &mut Window, cx: &mut Context<Self>) {
        match self.screen {
            Screen::Detail => {}
            _ => self.show_details = true,
        }
        cx.notify();
    }

    fn hide_details(&mut self, _: &HideDetails, _: &mut Window, cx: &mut Context<Self>) {
        match self.screen {
            Screen::Detail => {}
            _ => self.show_details = false,
        }
        cx.notify();
    }

    /// Whether → and ← should act on the split pane: the caret is at the query's end.
    fn caret_at_end(&self, cx: &App) -> bool {
        let input = self.query_input.read(cx);
        caret_at_end(input.cursor(), input.selected_range(), input.value().len())
    }

    /// Scrolls the detail screen's notes by `delta` (negative: up), within their ends.
    fn scroll_notes(&mut self, delta: Pixels, cx: &mut Context<Self>) {
        let offset = self.detail_scroll.offset();
        let max = self.detail_scroll.max_offset().y;
        self.detail_scroll
            .set_offset(point(offset.x, next_offset(offset.y, delta, max)));
        cx.notify();
    }

    /// A page: the visible height, less a few lines kept for context.
    fn notes_page(&self) -> Pixels {
        (self.detail_scroll.bounds().size.height - notes_line_step()).max(notes_line_step())
    }

    fn scroll_notes_down(&mut self, _: &ScrollNotesDown, _: &mut Window, cx: &mut Context<Self>) {
        self.scroll_notes(notes_line_step(), cx);
    }

    fn scroll_notes_up(&mut self, _: &ScrollNotesUp, _: &mut Window, cx: &mut Context<Self>) {
        self.scroll_notes(-notes_line_step(), cx);
    }

    fn page_notes_down(&mut self, _: &PageNotesDown, _: &mut Window, cx: &mut Context<Self>) {
        self.scroll_notes(self.notes_page(), cx);
    }

    fn page_notes_up(&mut self, _: &PageNotesUp, _: &mut Window, cx: &mut Context<Self>) {
        self.scroll_notes(-self.notes_page(), cx);
    }

    fn toggle_window_size(&mut self, _: &ToggleWindowSize, window: &mut Window, cx: &mut Context<Self>) {
        self.window_size = self.window_size.toggled();
        let display = window
            .display(cx)
            .map_or_else(compact, |display| display.visible_bounds().size);
        window.resize(self.window_size.size(display));
        cx.notify();
    }

    fn open_source_file(&mut self, _: &OpenSourceFile, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = self.selected_source_file(cx) {
            cx.open_with_system(std::path::Path::new(&path));
        }
    }

    fn open_preferences(&mut self, _: &OpenPreferences, window: &mut Window, cx: &mut Context<Self>) {
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
        if matches!(&self.screen, Screen::Rebuild(sync) if sync.read(cx).is_running()) {
            return;
        }
        let (sources, database) = (
            self.config.sources_dir().to_path_buf(),
            self.config.database().to_path_buf(),
        );
        let catalog = self.catalog.clone();
        let sync = cx.new(|cx| SyncView::start(sources, database, catalog, cx));
        let subscription = cx.subscribe_in(&sync, window, |this, _, finished: &SyncFinished, window, cx| {
            window.push_notification(finished.message.clone(), cx);
            if finished.rebuilt {
                let connection = open_readonly(this.config.database()).ok();
                this.list.update(cx, |list, cx| {
                    list.delegate_mut().set_connection(connection);
                    cx.notify();
                });
            }
        });
        self.subscriptions.push(subscription);
        self.screen = Screen::Rebuild(sync);
        self.focus.focus(window, cx);
        cx.notify();
    }

    fn back(&mut self, _: &Back, window: &mut Window, cx: &mut Context<Self>) {
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
        match &self.screen {
            Screen::Rebuild(sync) if sync.read(cx).is_running() => sync.read(cx).cancel(),
            Screen::Search => {
                if self.query_input.read(cx).value().is_empty() {
                    cx.hide();
                } else {
                    self.query_input.update(cx, |input, cx| {
                        input.set_value("", window, cx);
                    });
                    self.search("", window, cx);
                }
            }
            Screen::Detail | Screen::Rebuild(_) => {
                self.screen = Screen::Search;
                self.query_input.update(cx, |input, cx| input.focus(window, cx));
            }
        }
        cx.notify();
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
        self.show_detail_screen(window, cx);
    }

    /// Hands the keyboard to the current screen: the query field in the list, the panel elsewhere.
    fn focus_screen(&self, window: &mut Window, cx: &mut Context<Self>) {
        match self.screen {
            Screen::Search => self.query_input.update(cx, |input, cx| input.focus(window, cx)),
            _ => self.focus.focus(window, cx),
        }
    }

    fn open_filter(&mut self, _: &OpenFilter, window: &mut Window, cx: &mut Context<Self>) {
        let next = if self.is_filter_open() {
            Overlay::None
        } else {
            Overlay::Filter
        };
        self.show_overlay(next, window, cx);
    }

    fn apply_filter(&mut self, filter: FilterState, cx: &mut Context<Self>) {
        self.list.update(cx, |list, cx| {
            list.delegate_mut().set_filter(filter);
            cx.notify();
        });
        cx.notify();
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
                d.spacing.pane.split_pad_x,
                &d.typography.roles.card_split,
            ),
            Layout::Detail => (1., d.spacing.pane.detail_pad_x, &d.typography.roles.card_detail),
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
            cx,
        );
        self.screen = Screen::Search;
        self.query_input.update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
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
        match &self.screen {
            Screen::Rebuild(sync) => sync.clone().into_any_element(),
            Screen::Detail => selected.map_or_else(
                || div().into_any_element(),
                |entry| {
                    self.entry_view(entry, Layout::Detail, window, cx)
                        .track_scroll(self.detail_scroll.clone())
                        .into_any_element()
                },
            ),
            Screen::Search => match selected.filter(|_| self.show_details) {
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
                        div()
                            .h_full()
                            .flex_1()
                            .min_w_0()
                            .child(self.entry_view(entry, Layout::Split, window, cx)),
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
        // Rows show their titles alone while the split pane is open beside them.
        let compact = matches!(self.screen, Screen::Search) && self.show_details && selected.is_some();
        if self.list.read(cx).delegate().is_compact() != compact {
            self.list.update(cx, |list, _| list.delegate_mut().set_compact(compact));
        }
        let context = if self.is_showing_detail() {
            format!("{CONTEXT} {DETAIL_CONTEXT}")
        } else if self.is_rebuilding() {
            format!("{CONTEXT} {REFRESH_CONTEXT}")
        } else {
            CONTEXT.to_owned()
        };
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
            .on_action(cx.listener(Self::scroll_notes_down))
            .on_action(cx.listener(Self::scroll_notes_up))
            .on_action(cx.listener(Self::page_notes_down))
            .on_action(cx.listener(Self::page_notes_up))
            .on_action(cx.listener(Self::open_detail))
            .on_action(cx.listener(Self::open_source_file))
            .on_action(cx.listener(Self::rebuild))
            .on_action(cx.listener(Self::open_preferences))
            .on_action(cx.listener(Self::back))
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
