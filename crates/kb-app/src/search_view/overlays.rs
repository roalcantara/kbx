//! The two overlays, each a popover anchored to the control that opens it (DESIGN.md, §4):
//!
//! - **Actions**, above the footer's Actions button and opening upwards, the search field at the
//!   bottom; every row shows its keys, one keycap per key; a `›` row opens its page (Links,
//!   Copy…) in place, and Esc comes back.
//! - **Filter**, under the header's filter button: a section counting the active filters with
//!   Clear All Filters, then the types and tags with ✓ / ○. It stays open while picking, so
//!   several picks take one visit; Esc or ⌘P closes it.

use std::rc::Rc;

use gpui_kit::assets::IconName;
use gpui_kit::component::button::Button;
use gpui_kit::component::command::{Command, CommandGroup, CommandItem};
use gpui_kit::component::popover::Popover;
use gpui_kit::component::{ActiveTheme as _, Icon, IndexPath, Sizable as _, h_flex};
use gpui_kit::{
    Anchor, AnyElement, App, ClipboardItem, Context, Div, Hsla, IntoElement as _, ParentElement as _, Styled as _,
    WeakEntity, Window, div, px,
};
use kb_domain::search::{FilterState, entry_types, tags};

use super::footer::{action_keycaps, keycap_row};
use super::{Overlay, Screen, SearchView};
use crate::filter::{FilterGroup, filter_groups};
use crate::palette::{Page, PaletteCommand, PaletteContext, PaletteGroup, entry_palette, page};

/// Builds an overlay's content from the view, on every render of its popover.
type Content = fn(&SearchView, &WeakEntity<SearchView>, &mut Window, &App) -> AnyElement;

impl SearchView {
    /// Whether the actions popover is open.
    #[must_use]
    pub fn is_actions_open(&self) -> bool {
        matches!(self.overlay, Overlay::Actions(_))
    }

    /// Whether the filter dropdown is open.
    #[must_use]
    pub fn is_filter_open(&self) -> bool {
        self.overlay == Overlay::Filter
    }

    /// The labels of the actions popover's current page, by section, for tests.
    #[must_use]
    pub fn actions_labels(&self, cx: &App) -> Vec<Vec<String>> {
        self.palette_groups(cx)
            .iter()
            .map(|group| group.rows.iter().map(|row| row.label.clone()).collect())
            .collect()
    }

    /// Shows `overlay` with an empty search and hands it the keyboard; `Overlay::None` hands the
    /// keyboard back to the screen.
    pub(super) fn show_overlay(&mut self, overlay: Overlay, window: &mut Window, cx: &mut Context<Self>) {
        self.overlay = overlay;
        let palette = match overlay {
            Overlay::None => {
                self.focus_screen(window, cx);
                cx.notify();
                return;
            }
            Overlay::Actions(_) => self.palette.clone(),
            Overlay::Filter => self.filter_palette.clone(),
        };
        palette.update(cx, |state, cx| state.set_query("", window, cx));
        window.defer(cx, move |window, cx| {
            palette.update(cx, |state, cx| state.focus(window, cx));
        });
        cx.notify();
    }

    /// The Actions button inside its popover.
    pub(super) fn actions_popover(&self, trigger: Button, cx: &mut Context<Self>) -> Popover {
        let open = self.is_actions_open();
        anchored(
            "actions-popover",
            Anchor::BottomRight,
            open,
            Overlay::Actions(Page::Main),
            trigger,
            actions_content,
            cx,
        )
    }

    /// The filter button inside its dropdown.
    pub(super) fn filter_popover(&self, trigger: Button, cx: &mut Context<Self>) -> Popover {
        let open = self.is_filter_open();
        anchored(
            "filter-popover",
            Anchor::TopRight,
            open,
            Overlay::Filter,
            trigger,
            filter_content,
            cx,
        )
    }

    /// The actions popover's current page.
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

    /// The filter's sections for the index's types and tags; none without an index.
    fn filter_sections(&self, cx: &App) -> Vec<FilterGroup> {
        let delegate = self.list.read(cx).delegate();
        let Some(connection) = delegate.connection() else {
            return Vec::new();
        };
        // A failed read offers no choices rather than stale ones; the list shows the index.
        match (entry_types(connection), tags(connection)) {
            (Ok(types), Ok(tags)) => filter_groups(&types, &tags, delegate.filter()),
            _ => Vec::new(),
        }
    }
}

/// `trigger` inside a popover showing `overlay` while `open`.
fn anchored(
    id: &'static str,
    anchor: Anchor,
    open: bool,
    overlay: Overlay,
    trigger: Button,
    content: Content,
    cx: &mut Context<SearchView>,
) -> Popover {
    let view = cx.entity().downgrade();
    let on_change = view.clone();
    Popover::new(id)
        .anchor(anchor)
        .open(open)
        .on_open_change(move |open, window, cx| {
            let next = if *open { overlay } else { Overlay::None };
            let _ = on_change.update(cx, |this, cx| this.show_overlay(next, window, cx));
        })
        .trigger(trigger)
        .content(move |_, window, cx| match view.upgrade() {
            Some(entity) => content(entity.read(cx), &view, window, cx),
            None => div().into_any_element(),
        })
}

/// A row: its icon in `tint`, then its label; the caller appends what trails it.
fn row(icon: IconName, tint: Hsla, label: &str) -> Div {
    h_flex()
        .w_full()
        .items_center()
        .gap(px(8.))
        .child(Icon::new(icon).small().text_color(tint))
        .child(div().flex_1().min_w_0().truncate().child(label.to_owned()))
}

/// The actions popover: the current page as a `Command`, its search field at the bottom.
fn actions_content(this: &SearchView, view: &WeakEntity<SearchView>, window: &mut Window, cx: &App) -> AnyElement {
    let (groups, palette) = (this.palette_groups(cx), this.palette.clone());
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
        for palette_row in group.rows {
            let keys: Vec<String> = match (&palette_row.command, palette_row.keys.is_empty()) {
                (PaletteCommand::Run(action), true) => action_keycaps(action.as_ref(), window),
                _ => palette_row.keys.iter().map(|key| (*key).to_owned()).collect(),
            };
            let drills_in = matches!(palette_row.command, PaletteCommand::OpenPage(_));
            let (label, icon) = (palette_row.label.clone(), palette_row.icon);
            items.push(CommandItem::new().label(palette_row.label).child(move |_, cx| {
                let muted = cx.theme().muted_foreground;
                row(icon, muted, &label).child(if drills_in {
                    div().text_color(muted).child("›").into_any_element()
                } else {
                    keycap_row(keys.clone(), cx).into_any_element()
                })
            }));
            row_commands.push(palette_row.command);
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

/// Runs a confirmed action: a page opens in place; anything else closes the popover first, so
/// an Action reaches the panel as its key binding would.
fn run(command: &PaletteCommand, view: &WeakEntity<SearchView>, window: &mut Window, cx: &mut App) {
    let next = match command {
        PaletteCommand::OpenPage(page) => Overlay::Actions(*page),
        _ => Overlay::None,
    };
    let _ = view.update(cx, |this, cx| this.show_overlay(next, window, cx));
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

/// The filter dropdown: the sections as a `Command`, each choice marked ✓ or ○.
fn filter_content(this: &SearchView, view: &WeakEntity<SearchView>, _: &mut Window, cx: &App) -> AnyElement {
    let (groups, palette) = (this.filter_sections(cx), this.filter_palette.clone());
    let mut choices: Vec<Vec<FilterState>> = Vec::new();
    let mut command = Command::new(&palette)
        .bordered(false)
        .placeholder("Filter by Type or Tag…")
        .max_h(px(360.))
        .w(px(280.));
    for group in groups {
        let mut items = Vec::new();
        let mut nexts = Vec::new();
        for choice in group.choices {
            let (label, checked) = (choice.label.clone(), choice.checked);
            items.push(CommandItem::new().label(choice.label).child(move |_, cx| {
                let theme = cx.theme();
                if checked {
                    row(IconName::CircleCheck, theme.foreground, &label)
                } else {
                    row(IconName::Circle, theme.muted_foreground, &label)
                }
            }));
            nexts.push(choice.next);
        }
        command = command.group(CommandGroup::new().label(group.label).items(items));
        choices.push(nexts);
    }
    let choices = Rc::new(choices);
    let view = view.clone();
    command
        .on_confirm(move |ix: IndexPath, window, cx| {
            let Some(next) = choices.get(ix.section).and_then(|nexts| nexts.get(ix.row)).cloned() else {
                return;
            };
            // It stays open for the next pick, with a fresh search (UI-18).
            let _ = view.update(cx, |this, cx| {
                this.apply_filter(next, cx);
                this.show_overlay(Overlay::Filter, window, cx);
            });
        })
        .into_any_element()
}
