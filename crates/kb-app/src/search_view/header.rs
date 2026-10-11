//! The header (DESIGN.md, Header): no divider; the back chevron on the row-icon axis; the query
//! field where row titles start; the filter button with its funnel, the current filter and a
//! chevron that turns up while the dropdown is open. The detail screen shows only the chevron.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, MoveDown, MoveLeft, MoveRight, MoveUp};
use gpui_kit::component::{Icon, Sizable as _, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{Context, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _, div, px};

use super::{Overlay, SearchView};
use crate::filter::filter_summary;
use crate::keys::{Back, HideDetails, OpenFilter, ShowDetails};

/// The header's height, in points (`tools/design/layout.json`, header).
const HEIGHT: f32 = 56.;
/// The header's side inset: the list's selection inset (8) plus the row inset (16), so the
/// chevron sits on the row-icon axis.
const INSET: f32 = 24.;
/// The icon column, as the rows'.
const ICON_COLUMN: f32 = 24.;
/// The gap between the chevron and the field: the rows' icon-to-title gap, so the query starts
/// where titles do.
const GAP: f32 = 12.;

/// What the header shows on the current screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderParts {
    /// The back chevron: always, as Raycast's.
    pub back: bool,
    /// The query field: not on the detail screen.
    pub query: bool,
    /// The filter button: not on the detail screen.
    pub filter: bool,
}

impl SearchView {
    /// What the header shows on the current screen.
    #[must_use]
    pub fn header_parts(&self) -> HeaderParts {
        let detail = self.is_showing_detail();
        HeaderParts {
            back: true,
            query: !detail,
            filter: !detail,
        }
    }

    pub(super) fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let parts = self.header_parts();
        let filter_label = filter_summary(self.list.read(cx).delegate().filter());
        let chevron = if self.overlay == Overlay::Filter {
            IconName::ChevronUp
        } else {
            IconName::ChevronDown
        };
        h_flex()
            .h(px(HEIGHT))
            .w_full()
            .flex_none()
            .items_center()
            .px(px(INSET))
            .gap(px(GAP))
            .child(
                div().w(px(ICON_COLUMN)).flex().justify_center().child(
                    Button::new("back-button")
                        .ghost()
                        .xsmall()
                        .icon(IconName::ChevronLeft)
                        .accessibility_label("Back")
                        .tooltip("Back (Esc)")
                        .on_click(cx.listener(|this, _, window, cx| this.back(&Back, window, cx))),
                ),
            )
            .child(
                // A single-line input consumes ↑ and ↓ without propagating them, so they are
                // captured here, before it, to move the list selection.
                div().flex_1().min_w_0().when(parts.query, |field| {
                    field
                            .text_lg()
                            .capture_action(
                                cx.listener(|this, _: &MoveUp, window, cx| this.move_selection(-1, window, cx)),
                            )
                            .capture_action(
                                cx.listener(|this, _: &MoveDown, window, cx| this.move_selection(1, window, cx)),
                            )
                            // → and ← toggle the split pane, as in Raycast, but only with the caret
                            // at the query's end, so they still move it while editing.
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
                            .child(Input::new(&self.query_input).cleanable(true).appearance(false))
                }),
            )
            .when(parts.filter, |header| {
                header.child(
                    Button::new("filter-button")
                        .ghost()
                        .small()
                        .icon(IconName::Funnel)
                        .label(filter_label)
                        .child(Icon::new(chevron).xsmall())
                        .on_click(cx.listener(|this, _, window, cx| this.open_filter(&OpenFilter, window, cx))),
                )
            })
    }
}
