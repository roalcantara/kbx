//! The header on each screen (D4): the magnifier, the query and the filter pill on the list; the
//! back chevron alone elsewhere.

use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants as _};
use gpui_kit::component::input::{Input, MoveDown, MoveLeft, MoveRight, MoveUp};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{App, Context, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _};

use super::SearchView;
use crate::filter::filter_summary;
use crate::keys::{Back, HideDetails, ShowDetails};
use crate::ui::header::{filter_pill, header, lead_icon, query_box};
use crate::ui::keycap::caps;

/// What the header shows on the current screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderParts {
    /// The back chevron: every screen but the list, which shows the magnifier.
    pub back: bool,
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
    /// What the header shows on the current screen.
    #[must_use]
    pub fn header_parts(&self) -> HeaderParts {
        let list = !self.is_showing_detail() && !self.is_rebuilding();
        HeaderParts {
            back: !list,
            query: list,
            filter: list,
        }
    }

    pub(super) fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let parts = self.header_parts();
        let row = header();
        if parts.back {
            return row.child(
                Button::new("back-button")
                    .custom(plain_button(cx))
                    // No button padding: the contents sit on the window's content edge (spec 2026-10-19 spacing D2).
                    .px_0()
                    .child(lead_icon("chevron-left"))
                    .accessibility_label("Back")
                    .tooltip("Back (Esc)")
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
                query_box()
                    .capture_action(cx.listener(|this, _: &MoveUp, window, cx| this.move_selection(-1, window, cx)))
                    .capture_action(cx.listener(|this, _: &MoveDown, window, cx| this.move_selection(1, window, cx)))
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
