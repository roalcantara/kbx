//! The overlays (D9, D15): the action panel above the footer's Actions, the filter under the
//! header's pill, both the `ui::action_panel` component in a GPUI Kit `Popover`, sharing one
//! search field at their bottom.

use std::rc::Rc;

use gpui_kit::base::actions::Confirm;
use gpui_kit::component::button::Button;
use gpui_kit::component::input::{Input, InputEvent, InputState, MoveDown, MoveUp};
use gpui_kit::component::popover::Popover;
use gpui_kit::{
    Anchor, AnyElement, App, Context, Entity, Focusable as _, InteractiveElement as _, IntoElement as _,
    ParentElement as _, Styled as _, Window, div,
};
use kb_domain::search::tags_for_types;

use super::SearchView;
use crate::actions::{
    Page, PanelCommand, Submenu, filter_page, links_page, more_page, refresh_page, report_page, report_search_page,
    root_page, search_page, theme_page,
};
use crate::catalog::Screen as CatalogScreen;
use crate::keys::Back;
use crate::navigator::{Overlay, Place};
use crate::ui::action_panel::{OnRow, matches, panel};
use crate::ui::design::{design, pt};

/// A section's label and its rows' (label, caps) pairs, for tests.
pub type PanelSectionDetails = (Option<String>, Vec<(String, Option<String>)>);

impl SearchView {
    /// Whether the action panel is open.
    #[must_use]
    pub fn is_actions_open(&self) -> bool {
        matches!(self.navigator.overlay(), Overlay::Actions(_))
    }

    /// Whether the filter is open.
    #[must_use]
    pub fn is_filter_open(&self) -> bool {
        self.navigator.overlay() == Overlay::Filter
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

    /// The open page's visible rows (label, caps, section label) by section, for tests.
    #[must_use]
    pub fn panel_row_details(&self, cx: &App) -> Vec<PanelSectionDetails> {
        let page = self.current_page(cx);
        page.page
            .sections
            .iter()
            .map(|s| {
                (
                    s.label.clone(),
                    s.rows.iter().map(|r| (r.label.clone(), r.caps.clone())).collect(),
                )
            })
            .collect()
    }

    /// The open overlay's page.
    pub(super) fn current_page(&self, cx: &App) -> Page {
        let list = self.list.read(cx).delegate();
        let entry = match self.navigator.top() {
            Place::Refresh | Place::ReportDetail => None,
            Place::List | Place::Detail => list.selected_entry(),
        };
        let screen = self.key_screen(cx);
        let query_empty = self.panel_query.read(cx).value().is_empty();
        if let Some(sync) = self.report(cx) {
            let row = sync.read(cx).selected_row(cx);
            return match self.navigator.overlay() {
                Overlay::Actions(None) if query_empty => report_page(&self.catalog, screen, row),
                Overlay::Actions(None) => report_search_page(&self.catalog, screen, row, &self.themes),
                Overlay::Actions(Some(Submenu::More)) => more_page(&self.catalog, None),
                Overlay::Actions(Some(Submenu::Refresh)) => refresh_page(&self.catalog),
                Overlay::Actions(Some(Submenu::Theme)) => theme_page(&self.catalog, &self.themes),
                _ => Page::default(),
            };
        }
        match self.navigator.overlay() {
            Overlay::None => Page::default(),
            Overlay::Actions(None) if query_empty => root_page(&self.catalog, screen, entry),
            Overlay::Actions(None) => search_page(&self.catalog, screen, entry, &self.themes),
            Overlay::Actions(Some(Submenu::More)) => more_page(&self.catalog, entry),
            Overlay::Actions(Some(Submenu::Refresh)) => refresh_page(&self.catalog),
            Overlay::Actions(Some(Submenu::Theme)) => theme_page(&self.catalog, &self.themes),
            Overlay::Actions(Some(Submenu::Links)) => {
                entry.map_or_else(Page::default, |e| links_page(&self.catalog, e))
            }
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
        self.navigator.set_overlay(overlay);
        self.panel_selected = 0;
        if matches!(overlay, Overlay::Actions(_)) {
            self.reload_themes();
        }
        if overlay == Overlay::Actions(Some(Submenu::Theme)) {
            self.begin_theme_preview();
        }
        self.panel_query.update(cx, |input, cx| input.set_value("", window, cx));
        match overlay {
            Overlay::None => self.focus_screen(window, cx),
            // A submenu has no search field: the panel itself takes the keyboard there.
            Overlay::Actions(Some(_)) => {
                let focus = self.panel_focus.clone();
                window.defer(cx, move |window, cx| focus.focus(window, cx));
            }
            _ => {
                let query = self.panel_query.clone();
                window.defer(cx, move |window, cx| {
                    query.update(cx, |input, cx| input.focus(window, cx));
                });
            }
        }
        cx.notify();
    }

    pub(super) fn set_panel_query(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.panel_query
            .update(cx, |input, cx| input.set_value(text, window, cx));
        self.panel_selected = 0;
        cx.notify();
    }

    pub(super) fn on_panel_query_event(
        &mut self,
        _: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // ↵ and ⌘↵ are handled as the panel's `Confirm` action (see `render_panel`), so they work
        // on the submenus too, where there is no search field to emit this event.
        if let InputEvent::Change = event {
            self.panel_selected = 0;
            cx.notify();
        }
    }

    fn move_panel(&mut self, delta: isize, cx: &mut Context<Self>) {
        let count = matches(&self.current_page(cx).page, &self.panel_query.read(cx).value()).len();
        if count > 0 {
            self.panel_selected = self.panel_selected.saturating_add_signed(delta).min(count - 1);
            self.panel_scroll.scroll_to_item(self.panel_selected);
            self.preview_selected_theme(cx);
            cx.notify();
        }
    }

    /// Runs visible row `ix`: ↵ (`secondary` false) or ⌘↵.
    fn confirm(&mut self, ix: usize, secondary: bool, window: &mut Window, cx: &mut Context<Self>) {
        let page = self.current_page(cx);
        let visible = matches(&page.page, &self.panel_query.read(cx).value());
        let target_ix = if secondary {
            // ⌘↵: if the selected row is a Filter or Link row, ⌘↵ acts on it (Only / Copy link).
            // Otherwise, ⌘↵ runs the visible row whose caps show ⌘↵ (Raycast parity).
            let is_special = visible
                .get(ix)
                .and_then(|&(s, r)| page.command(s, r))
                .is_some_and(|c| matches!(c, PanelCommand::Filter { .. } | PanelCommand::Link(_)));
            if is_special {
                Some(ix)
            } else {
                let cmd_enter_caps = crate::ui::keycap::caps("cmd-enter");
                visible
                    .iter()
                    .position(|&(s, r)| page.page.sections[s].rows[r].caps.as_deref() == Some(&cmd_enter_caps))
                    .or(Some(ix))
            }
        } else {
            Some(ix)
        };
        let Some(target_ix) = target_ix else { return };
        let Some(command) = visible.get(target_ix).and_then(|&(s, r)| page.command(s, r)).cloned() else {
            return;
        };
        match command {
            PanelCommand::Open(submenu) => self.show_overlay(Overlay::Actions(Some(submenu)), window, cx),
            PanelCommand::Theme(id) => self.keep_theme(&id, window, cx),
            PanelCommand::Action(id) => {
                self.show_overlay(Overlay::None, window, cx);
                self.run_action(&id, window, cx);
            }
            PanelCommand::Link(url) => {
                self.show_overlay(Overlay::None, window, cx);
                let link_action = self
                    .catalog
                    .keys_for(CatalogScreen::Links, None)
                    .into_iter()
                    .find(|k| k.keys == if secondary { "cmd-enter" } else { "enter" })
                    .map(|k| k.action);
                match link_action.as_deref() {
                    Some("copy-link") => self.copy(url, window, cx),
                    _ => cx.open_url(&url),
                }
            }
            PanelCommand::Copy(text) => {
                self.show_overlay(Overlay::None, window, cx);
                self.copy(text, window, cx);
            }
            PanelCommand::Filter { toggle, only } => {
                let filter_action = self
                    .catalog
                    .keys_for(CatalogScreen::Filter, None)
                    .into_iter()
                    .find(|k| k.keys == if secondary { "cmd-enter" } else { "enter" })
                    .map(|k| k.action);
                if filter_action.as_deref() == Some("only") {
                    self.apply_filter(only, window, cx);
                    self.show_overlay(Overlay::None, window, cx);
                } else {
                    // The toggle can add or drop the Clear All section, so the row is found again
                    // by its labels; index 0 would be Clear All, one ↵ away from losing the filter.
                    let picked = visible.get(target_ix).map(|&(s, r)| {
                        let section = &page.page.sections[s];
                        (section.label.clone(), section.rows[r].label.clone())
                    });
                    self.apply_filter(toggle, window, cx);
                    self.set_panel_query("", window, cx);
                    let page = self.current_page(cx);
                    self.panel_selected = matches(&page.page, "")
                        .into_iter()
                        .position(|(s, r)| {
                            let section = &page.page.sections[s];
                            picked.as_ref() == Some(&(section.label.clone(), section.rows[r].label.clone()))
                        })
                        .unwrap_or(0);
                }
            }
        }
    }

    /// Runs the visible row whose caps show `chord` (e.g. ⇧⌘↵).
    fn confirm_chord(&mut self, chord: &str, window: &mut Window, cx: &mut Context<Self>) {
        let page = self.current_page(cx);
        let visible = matches(&page.page, &self.panel_query.read(cx).value());
        let chord_caps = crate::ui::keycap::caps(chord);
        if let Some(target_ix) = visible
            .iter()
            .position(|&(s, r)| page.page.sections[s].rows[r].caps.as_deref() == Some(&chord_caps))
        {
            self.confirm(target_ix, false, window, cx);
        }
    }

    /// The Actions button inside its popover, opening upwards above the footer.
    pub(super) fn actions_popover(&self, trigger: Button, cx: &mut Context<Self>) -> Popover {
        self.anchored(
            "actions-popover",
            Anchor::BottomRight,
            self.is_actions_open(),
            Overlay::Actions(None),
            trigger,
            cx,
        )
    }

    /// The filter pill inside its popover, opening under the header.
    pub(super) fn filter_popover(&self, trigger: Button, cx: &mut Context<Self>) -> Popover {
        self.anchored(
            "filter-popover",
            Anchor::TopRight,
            self.is_filter_open(),
            Overlay::Filter,
            trigger,
            cx,
        )
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
        // The popover moves the keyboard to the handle it tracks when it opens, after this view's
        // own deferred focus, so it must track where the keys belong: the search field on a page
        // that has one (the root, the filter), the panel itself on a submenu, which has none.
        // Tracking the panel everywhere left the root's search field without the keyboard.
        let focus = if matches!(self.navigator.overlay(), Overlay::Actions(Some(_))) {
            self.panel_focus.clone()
        } else {
            self.panel_query.read(cx).focus_handle(cx)
        };
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
        let width = if self.is_filter_open() {
            d.components.panel.filter_width
        } else {
            d.components.panel.width
        };
        let view = cx.entity().downgrade();
        let on_row: OnRow = Rc::new(move |ix, window, cx| {
            let _ = view.update(cx, |this, cx| this.confirm(ix, false, window, cx));
        });
        let search = div()
            .capture_action(cx.listener(|this, _: &MoveUp, _, cx| {
                this.move_panel(-1, cx);
                cx.stop_propagation(); // or the panel's own ↑ would move it again
            }))
            .capture_action(cx.listener(|this, _: &MoveDown, _, cx| {
                this.move_panel(1, cx);
                cx.stop_propagation();
            }))
            .child(Input::new(&self.panel_query).appearance(false).p_0())
            .into_any_element();
        panel(&page.page, &visible, self.panel_selected, &self.panel_scroll, Some(search), width, &on_row)
            .id("action-panel")
            .key_context(crate::keys::PANEL_CONTEXT)
            .track_focus(&self.panel_focus)
            // The panel handles its own keys, so they work on the submenus (no search field)
            // and never reach the popover's own `Confirm`/`Cancel`, which would toggle it shut.
            .on_action(cx.listener(|this, event: &Confirm, window, cx| {
                this.confirm(this.panel_selected, event.secondary, window, cx);
                cx.stop_propagation();
            }))
            .on_action(cx.listener(|this, _: &crate::keys::ScreenShiftCmdEnter, window, cx| {
                this.confirm_chord("shift-cmd-enter", window, cx);
                cx.stop_propagation();
            }))
            .on_action(cx.listener(|this, event: &Back, window, cx| {
                this.back(event, window, cx);
                cx.stop_propagation();
            }))
            // On a searchable page the search field's capture handles ↑↓ and stops them; these move
            // the selection when the panel itself has the keyboard (a submenu).
            .on_action(cx.listener(|this, _: &MoveUp, _, cx| this.move_panel(-1, cx)))
            .on_action(cx.listener(|this, _: &MoveDown, _, cx| this.move_panel(1, cx)))
            .into_any_element()
    }
}
