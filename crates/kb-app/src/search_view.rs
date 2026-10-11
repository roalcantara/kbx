//! The panel's root view: the search list, the split pane, the detail and rebuild screens, and
//! every command's handler.

use std::fs;

use gpui_kit::component::command::CommandState;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::list::{List, ListEvent, ListState};
use gpui_kit::component::{ActiveTheme as _, IndexPath, Theme, WindowExt as _, h_flex, v_flex};
use gpui_kit::{
    AnyElement, App, AppContext as _, ClipboardItem, Context, Entity, FocusHandle, InteractiveElement as _,
    IntoElement, ParentElement, Pixels, Render, ScrollHandle, Styled as _, Subscription, Window, div, point, px,
    relative,
};
use kb_domain::entry::Entry;
use kb_domain::search::FilterState;
use kb_domain::sync::open_readonly;

mod footer;
mod header;
mod overlays;

use crate::config::Config;
use crate::entry_list::EntryList;
use crate::keys::{
    Back, CONTEXT, CopyUrl, DETAIL_CONTEXT, HideDetails, OpenActions, OpenDetail, OpenFilter, OpenPreferences,
    OpenSourceFile, PageNotesDown, PageNotesUp, Quit, Rebuild, RunPrimary, ScrollNotesDown, ScrollNotesUp, ShowDetails,
    ToggleDetails, ToggleWindowSize, caret_at_end,
};
use crate::notes::{PaneLayout, next_offset, notes_pane};
use crate::palette::Page;
use crate::presentation::{PrimaryKind, TypeKey, primary_action, type_key};
use crate::sync_view::{SyncFinished, SyncView};
use crate::tag_images::TagImages;
use crate::trace;
use crate::window_size::{COMPACT, WindowSize};

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

/// How far ↓ and ↑ scroll the detail screen's notes: about three lines.
const NOTES_LINE_STEP: Pixels = px(60.);

/// What is open over the screen: one overlay at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Overlay {
    /// Nothing.
    None,
    /// The actions popover, on this page.
    Actions(Page),
    /// The filter dropdown.
    Filter,
}

/// The panel's root view.
pub struct SearchView {
    config: Config,
    list: Entity<ListState<EntryList>>,
    query_input: Entity<InputState>,
    palette: Entity<CommandState>,
    filter_palette: Entity<CommandState>,
    /// Focus for the screens without the list, so their keys reach this view.
    focus: FocusHandle,
    screen: Screen,
    show_details: bool,
    show_metadata: bool,
    window_size: WindowSize,
    /// The detail screen's notes, scrolled by the keyboard.
    detail_scroll: ScrollHandle,
    /// The overlay open over the screen, if any.
    overlay: Overlay,
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
    pub fn new(config: Config, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let connection = open_readonly(config.database()).ok();
        let tag_images = TagImages::scan(config.tag_images());
        let list = cx.new(|cx| {
            ListState::new(EntryList::new(connection).with_tag_images(tag_images), window, cx).searchable(false)
        });
        let query_input = cx.new(|cx| InputState::new(window, cx).placeholder("Search Knowledge Base…"));
        // The first row starts selected, as in Raycast, so Enter and ⌘K act on it at once.
        list.update(cx, |list, cx| {
            if !list.delegate().entries().is_empty() {
                list.set_selected_index(Some(IndexPath::default()), window, cx);
            }
        });
        query_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
        let palette = cx.new(|cx| CommandState::new(window, cx));
        let filter_palette = cx.new(|cx| CommandState::new(window, cx));
        let list_sub = cx.subscribe_in(&list, window, Self::on_list_event);
        let input_sub = cx.subscribe_in(&query_input, window, Self::on_query_input_event);
        // The theme follows the system's light or dark appearance while the panel is open.
        let appearance = cx.observe_window_appearance(window, |_, window, cx| {
            Theme::sync_system_appearance(Some(window), cx);
        });
        Self {
            config,
            list,
            query_input,
            palette,
            filter_palette,
            focus: cx.focus_handle(),
            screen: Screen::Search,
            // The list alone, as Raycast's kb Search opens; ⌘D shows the split pane.
            show_details: false,
            show_metadata: false,
            window_size: WindowSize::default(),
            detail_scroll: ScrollHandle::new(),
            overlay: Overlay::None,
            subscriptions: vec![list_sub, input_sub, appearance],
        }
    }

    /// The list state, for tests.
    #[must_use]
    pub fn list(&self) -> &Entity<ListState<EntryList>> {
        &self.list
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

    /// Whether the detail screen shows the metadata beside the notes.
    #[must_use]
    pub fn is_showing_metadata(&self) -> bool {
        self.show_metadata
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
        self.show_metadata = false;
        self.detail_scroll.set_offset(point(px(0.), px(0.)));
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
                if *secondary {
                    self.run_primary(&RunPrimary, window, cx);
                } else {
                    self.show_detail_screen(window, cx);
                }
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
                if self.list.read(cx).delegate().is_secondary_confirm() {
                    self.run_primary(&RunPrimary, window, cx);
                } else {
                    self.show_detail_screen(window, cx);
                }
            }
            // Esc in the list leaves the panel, as it closes the Raycast command.
            ListEvent::Cancel => cx.hide(),
        }
    }

    fn run_primary(&mut self, _: &RunPrimary, _: &mut Window, cx: &mut Context<Self>) {
        let Some(action) = self.list.read(cx).delegate().selected_entry().map(primary_action) else {
            return;
        };
        match action.kind {
            PrimaryKind::OpenInBrowser => cx.open_url(&action.content),
            PrimaryKind::Copy => cx.write_to_clipboard(ClipboardItem::new_string(action.content)),
        }
    }

    fn copy_url(&mut self, _: &CopyUrl, _: &mut Window, cx: &mut Context<Self>) {
        let url = self
            .list
            .read(cx)
            .delegate()
            .selected_entry()
            .filter(|e| type_key(&e.entry_type) == TypeKey::Bookmark)
            .map(|e| e.primary_content.clone());
        if let Some(url) = url {
            cx.write_to_clipboard(ClipboardItem::new_string(url));
        }
    }

    fn toggle_details(&mut self, _: &ToggleDetails, _: &mut Window, cx: &mut Context<Self>) {
        match self.screen {
            Screen::Detail => self.show_metadata = !self.show_metadata,
            _ => self.show_details = !self.show_details,
        }
        cx.notify();
    }

    fn show_details(&mut self, _: &ShowDetails, _: &mut Window, cx: &mut Context<Self>) {
        match self.screen {
            Screen::Detail => self.show_metadata = true,
            _ => self.show_details = true,
        }
        cx.notify();
    }

    fn hide_details(&mut self, _: &HideDetails, _: &mut Window, cx: &mut Context<Self>) {
        match self.screen {
            Screen::Detail => self.show_metadata = false,
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
        (self.detail_scroll.bounds().size.height - NOTES_LINE_STEP).max(NOTES_LINE_STEP)
    }

    fn scroll_notes_down(&mut self, _: &ScrollNotesDown, _: &mut Window, cx: &mut Context<Self>) {
        self.scroll_notes(NOTES_LINE_STEP, cx);
    }

    fn scroll_notes_up(&mut self, _: &ScrollNotesUp, _: &mut Window, cx: &mut Context<Self>) {
        self.scroll_notes(-NOTES_LINE_STEP, cx);
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
            .map_or(COMPACT, |display| display.visible_bounds().size);
        window.resize(self.window_size.size(display));
        cx.notify();
    }

    fn open_source_file(&mut self, _: &OpenSourceFile, _: &mut Window, cx: &mut Context<Self>) {
        let path = self
            .list
            .read(cx)
            .delegate()
            .selected_entry()
            .map(|e| e.source_file.clone());
        if let Some(path) = path {
            cx.open_with_system(std::path::Path::new(&path));
        }
    }

    fn open_preferences(&mut self, _: &OpenPreferences, window: &mut Window, cx: &mut Context<Self>) {
        let file = self.config.preferences().to_path_buf();
        if !file.exists() {
            let template = format!(
                "# kb preferences\n# sources_path = \"{}\"\n",
                self.config.sources_dir().display()
            );
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
        let sync = cx.new(|cx| SyncView::start(sources, database, cx));
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
        // Esc leaves a page of the actions popover, then the popover, before the screen.
        match self.overlay {
            Overlay::Actions(Page::Main) | Overlay::Filter => return self.show_overlay(Overlay::None, window, cx),
            Overlay::Actions(_) => return self.show_overlay(Overlay::Actions(Page::Main), window, cx),
            Overlay::None => {}
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
            Overlay::Actions(Page::Main)
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
    /// The screen: the list and its split pane, the detail screen, or the rebuild.
    fn render_content(&self, selected: Option<&Entry>, cx: &mut Context<Self>) -> AnyElement {
        match &self.screen {
            Screen::Rebuild(sync) => sync.clone().into_any_element(),
            // The notes alone, without a title block: the footer names the entry (UI-12).
            Screen::Detail => selected.map_or_else(
                || div().into_any_element(),
                |entry| {
                    notes_pane(
                        entry,
                        self.show_metadata,
                        PaneLayout::Detail,
                        Some(&self.detail_scroll),
                        cx,
                    )
                },
            ),
            // Raycast's split: the list a third of the width, the pane the rest (DESIGN.md, D3).
            Screen::Search => match selected.filter(|_| self.show_details) {
                Some(entry) => h_flex()
                    .size_full()
                    .child(
                        div()
                            .h_full()
                            .w(relative(1. / 3.))
                            .flex_none()
                            .border_r_1()
                            .border_color(cx.theme().border)
                            .child(List::new(&self.list)),
                    )
                    .child(div().h_full().flex_1().min_w_0().child(notes_pane(
                        entry,
                        true,
                        PaneLayout::Split,
                        None,
                        cx,
                    )))
                    .into_any_element(),
                None => List::new(&self.list).into_any_element(),
            },
        }
    }
}

impl Render for SearchView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.list.read(cx).delegate().selected_entry().cloned();
        let context = if self.is_showing_detail() {
            format!("{CONTEXT} {DETAIL_CONTEXT}")
        } else {
            CONTEXT.to_owned()
        };
        v_flex()
            .track_focus(&self.focus)
            .key_context(context.as_str())
            .on_action(cx.listener(Self::open_actions))
            .on_action(cx.listener(Self::open_filter))
            .on_action(cx.listener(Self::run_primary))
            .on_action(cx.listener(Self::copy_url))
            .on_action(cx.listener(Self::toggle_details))
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
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_header(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(self.render_content(selected.as_ref(), cx)),
            )
            .child(self.render_footer(selected.as_ref(), window, cx))
    }
}
