//! The header on each screen (D4): the magnifier, the query and the filter pill on the list; the
//! back chevron alone elsewhere.

use crate::navigator::Nav;
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants as _};
use gpui_kit::component::input::{
    IndentInline, Input, MoveDown, MoveLeft, MovePageDown, MovePageUp, MoveRight, MoveToEnd, MoveToStart, MoveUp,
    OutdentInline,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{App, Context, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _};

use super::SearchView;
use crate::filter::filter_summary;
use crate::keys::{Back, HideDetails, ShowDetails};
use crate::ui::header::{filter_pill, header, lead_icon, query_box};
use crate::ui::keycap::caps;

use crate::navigator::{BackName, Place};
use crate::ui::crumb::{CrumbSize, crumb};

/// What the header shows on the current place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderParts {
    /// Where ‹ goes: every place but the list, which shows the magnifier.
    pub back: Option<BackName>,
    /// The query field: the list only.
    pub query: bool,
    /// The filter pill: the list only.
    pub filter: bool,
}

/// A button drawn as its children alone: no fill, even on hover (the header, the footer).
pub(super) fn plain_button(cx: &App) -> ButtonCustomVariant {
    ButtonCustomVariant::new(cx)
        .color(gpui_kit::transparent_black())
        .hover(gpui_kit::transparent_black())
        .active(gpui_kit::transparent_black())
        .shadow(false)
}

impl SearchView {
    /// A text field over a list: its caret keys move the list instead (close-out D2). The query field and ⌘K's
    /// search use it, so both answer the same keys.
    pub(super) fn nav_captures(el: gpui_kit::Div, cx: &mut Context<Self>) -> gpui_kit::Div {
        el.capture_action(cx.listener(|this, _: &MoveUp, w, cx| {
            this.nav(Nav::Up, w, cx);
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|this, _: &MoveDown, w, cx| {
            this.nav(Nav::Down, w, cx);
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|this, _: &MoveToStart, w, cx| {
            this.nav(Nav::First, w, cx);
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|this, _: &MoveToEnd, w, cx| {
            this.nav(Nav::Last, w, cx);
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|this, _: &MovePageUp, w, cx| {
            this.nav(Nav::PageUp, w, cx);
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|this, _: &MovePageDown, w, cx| {
            this.nav(Nav::PageDown, w, cx);
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|this, _: &IndentInline, w, cx| {
            this.toggle_pane(w, cx);
            cx.stop_propagation();
        }))
        .capture_action(cx.listener(|this, _: &OutdentInline, w, cx| {
            this.toggle_pane(w, cx);
            cx.stop_propagation();
        }))
    }

    /// What the header shows on the current screen.
    #[must_use]
    pub fn header_parts(&self) -> HeaderParts {
        let list = self.navigator.top() == Place::List;
        HeaderParts {
            back: self.navigator.back_name().cloned(),
            query: list,
            filter: list,
        }
    }

    pub(super) fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let parts = self.header_parts();
        let row = header();
        if let Some(back) = parts.back {
            return row.child(
                Button::new("back-button")
                    .custom(plain_button(cx))
                    // No button padding: the contents sit on the window's content edge (spec 2026-10-19 spacing D2).
                    .px_0()
                    .min_w_0()
                    .child(crumb(back.text, back.mono, CrumbSize::Header))
                    .accessibility_label(self.catalog.label("back").to_owned())
                    .tooltip(format!("{} ({})", self.catalog.label("back"), caps("escape")))
                    .on_click(cx.listener(|this, _, window, cx| this.back(&Back, window, cx))),
            );
        }
        let filter = self.list.read(cx).delegate().filter();
        let label = filter_summary(filter, &self.catalog);
        let filter_caps = self.catalog.actions["filter"]
            .keys
            .as_deref()
            .map(caps)
            .unwrap_or_default();
        row.child(lead_icon("search"))
            .child(
                // A single-line input consumes ↑ and ↓ without propagating them, so they are
                // captured here, before it, to move the list selection.
                Self::nav_captures(query_box(), cx)
                    // → and ← toggle the split pane, as in Raycast, but only with the caret at the
                    // query's end, so they still move it while editing.
                    .capture_action(cx.listener(|this, _: &MoveRight, window, cx| {
                        if !this.show_details && this.caret_at_end(cx) {
                            this.show_details(&ShowDetails, window, cx);
                            cx.stop_propagation();
                        }
                    }))
                    .capture_action(cx.listener(|this, _: &MoveLeft, window, cx| {
                        if this.show_details && this.caret_at_end(cx) {
                            this.hide_details(&HideDetails, window, cx);
                            cx.stop_propagation();
                        }
                    }))
                    .child(Input::new(&self.query_input).cleanable(true).appearance(false).p_0()),
            )
            .when(parts.filter, |row| {
                row.child(
                    self.filter_popover(
                        Button::new("filter-button")
                            .custom(plain_button(cx))
                    // No button padding: the contents sit on the window's content edge (spec 2026-10-19 spacing D2).
                    .px_0()
                            .child(filter_pill(label, filter_caps)),
                        cx,
                    ),
                )
            })
    }
}
