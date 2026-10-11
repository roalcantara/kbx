//! The actions popover (DESIGN.md, Actions popover): anchored above the footer's Actions button,
//! opening upwards, the search field at the bottom; every row shows its keys, one keycap per key;
//! a `›` row opens its page (Links, Copy…) in place, and Esc comes back.

use std::rc::Rc;

use gpui_kit::component::button::Button;
use gpui_kit::component::command::{Command, CommandGroup, CommandItem};
use gpui_kit::component::popover::Popover;
use gpui_kit::component::{ActiveTheme as _, Icon, IndexPath, Sizable as _, h_flex};
use gpui_kit::{
    Anchor, App, ClipboardItem, Context, IntoElement as _, ParentElement as _, Styled as _, WeakEntity, Window, div, px,
};

use super::footer::{action_keycaps, keycap_row};
use super::{Overlay, Screen, SearchView};
use crate::palette::{Page, PaletteCommand, PaletteContext, PaletteGroup, entry_palette, page};

impl SearchView {
    /// Whether the actions popover is open.
    #[must_use]
    pub fn is_actions_open(&self) -> bool {
        matches!(self.overlay, Overlay::Actions(_))
    }

    /// The labels of the popover's current page, by section, for tests.
    #[must_use]
    pub fn actions_labels(&self, cx: &App) -> Vec<Vec<String>> {
        self.palette_groups(cx)
            .iter()
            .map(|group| group.rows.iter().map(|row| row.label.clone()).collect())
            .collect()
    }

    /// The current page's sections.
    fn palette_groups(&self, cx: &App) -> Vec<PaletteGroup> {
        let entry = match self.screen {
            Screen::Rebuild(_) => None,
            _ => self.list.read(cx).delegate().selected_entry(),
        };
        let detail = self.is_showing_detail();
        let current = match self.overlay {
            Overlay::Actions(page) => page,
            _ => Page::Main,
        };
        match (current, entry) {
            (Page::Main, _) | (_, None) => entry_palette(
                entry,
                PaletteContext {
                    detail,
                    showing_details: if detail { self.show_metadata } else { self.show_details },
                },
            ),
            (current, Some(entry)) => page(entry, current),
        }
    }

    /// Opens the popover on its main page with an empty search, or closes it, handing the
    /// keyboard back to the panel.
    pub(super) fn set_actions_open(&mut self, open: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.overlay = if open {
            Overlay::Actions(Page::Main)
        } else {
            Overlay::None
        };
        self.palette.update(cx, |state, cx| state.set_query("", window, cx));
        if open {
            let palette = self.palette.clone();
            window.defer(cx, move |window, cx| {
                palette.update(cx, |state, cx| state.focus(window, cx));
            });
        } else {
            self.focus_screen(window, cx);
        }
        cx.notify();
    }

    /// Shows `page` with an empty search.
    pub(super) fn show_actions_page(&mut self, page: Page, window: &mut Window, cx: &mut Context<Self>) {
        self.overlay = Overlay::Actions(page);
        self.palette.update(cx, |state, cx| state.set_query("", window, cx));
        cx.notify();
    }

    /// The Actions button inside its popover.
    pub(super) fn actions_popover(&self, trigger: Button, cx: &mut Context<Self>) -> Popover {
        let view = cx.entity().downgrade();
        let on_change = view.clone();
        Popover::new("actions-popover")
            .anchor(Anchor::BottomRight)
            .open(self.is_actions_open())
            .on_open_change(move |open, window, cx| {
                let _ = on_change.update(cx, |this, cx| this.set_actions_open(*open, window, cx));
            })
            .trigger(trigger)
            .content(move |_, window, cx| actions_content(&view, window, cx))
    }
}

/// The popover's content: the current page as a `Command`, its search field at the bottom.
fn actions_content(view: &WeakEntity<SearchView>, window: &mut Window, cx: &mut App) -> gpui_kit::AnyElement {
    let Some(entity) = view.upgrade() else {
        return div().into_any_element();
    };
    let (groups, palette) = {
        let this = entity.read(cx);
        (this.palette_groups(cx), this.palette.clone())
    };
    let mut commands: Vec<Vec<PaletteCommand>> = Vec::new();
    let mut command = Command::new(&palette)
        .bordered(false)
        .placeholder("Search for Actions…")
        .max_h(px(360.))
        .w(px(340.))
        // The search field under the rows, as Raycast's.
        .flex_col_reverse();
    for group in groups {
        let mut items = Vec::new();
        let mut row_commands = Vec::new();
        for row in group.rows {
            let keys: Vec<String> = match (&row.command, row.keys.is_empty()) {
                (PaletteCommand::Run(action), true) => action_keycaps(action.as_ref(), window),
                _ => row.keys.iter().map(|key| (*key).to_owned()).collect(),
            };
            let drills_in = matches!(row.command, PaletteCommand::OpenPage(_));
            let (label, icon) = (row.label.clone(), row.icon);
            items.push(CommandItem::new().label(row.label).child(move |_, cx| {
                h_flex()
                    .w_full()
                    .items_center()
                    .gap(px(8.))
                    .child(Icon::new(icon).small().text_color(cx.theme().muted_foreground))
                    .child(div().flex_1().min_w_0().truncate().child(label.clone()))
                    .child(if drills_in {
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child("›")
                            .into_any_element()
                    } else {
                        keycap_row(keys.clone(), cx).into_any_element()
                    })
            }));
            row_commands.push(row.command);
        }
        let mut section = CommandGroup::new().items(items);
        if !group.label.is_empty() {
            section = section.label(group.label);
        }
        command = command.group(section);
        commands.push(row_commands);
    }
    let commands = Rc::new(commands);
    let view = view.clone();
    command
        .on_confirm(move |ix: IndexPath, window, cx| {
            if let Some(command) = commands.get(ix.section).and_then(|rows| rows.get(ix.row)) {
                run(command, &view, window, cx);
            }
        })
        .into_any_element()
}

/// Runs a confirmed row: a page opens in place; anything else closes the popover first, so an
/// Action reaches the panel as its key binding would.
fn run(command: &PaletteCommand, view: &WeakEntity<SearchView>, window: &mut Window, cx: &mut App) {
    if let PaletteCommand::OpenPage(page) = command {
        let page = *page;
        let _ = view.update(cx, |this, cx| this.show_actions_page(page, window, cx));
        return;
    }
    let _ = view.update(cx, |this, cx| this.set_actions_open(false, window, cx));
    match command {
        PaletteCommand::Run(action) => {
            let action = action.boxed_clone();
            window.defer(cx, move |window, cx| window.dispatch_action(action, cx));
        }
        PaletteCommand::OpenUrl(url) => cx.open_url(url),
        PaletteCommand::Copy(text) => cx.write_to_clipboard(ClipboardItem::new_string(text.clone())),
        PaletteCommand::Filter(filter) => {
            let filter = filter.clone();
            let _ = view.update(cx, |this, cx| this.apply_filter(filter, cx));
        }
        PaletteCommand::OpenPage(_) => {}
    }
}
