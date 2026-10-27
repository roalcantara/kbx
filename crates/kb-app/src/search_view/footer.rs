//! The footer on each screen (D14): the status line on the list, the entry's file while a pane or
//! the detail screen is open; the screen's keys and Actions.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::{AnyElement, App, Context, IntoElement, ParentElement as _, SharedString, Styled as _, Window};
use kb_domain::entry::Entry;

use super::SearchView;
use super::header::plain_button;
use crate::catalog::{KeyItem, fill};
use crate::navigator::Place;
use crate::ui::design::design;
use crate::ui::footer::{Dot, entry_status, footer, key_item, status};
use crate::ui::keycap::caps;
use crate::ui::tile::{TileSize, tile};

impl SearchView {
    /// Whether the footer names the entry rather than the index: a pane or the detail screen.
    fn footer_names_entry(&self, selected: Option<&Entry>) -> bool {
        selected.is_some() && (self.is_showing_detail() || (self.navigator.top() == Place::List && self.show_details))
    }

    /// The status line's text, for tests: "Ready • 8 results".
    #[must_use]
    pub fn status_line(&self, cx: &App) -> String {
        let (word, count) = self.status_parts(cx);
        count.map_or_else(|| word.clone(), |count| format!("{word} • {count}"))
    }

    fn status_parts(&self, cx: &App) -> (String, Option<String>) {
        let messages = &self.catalog.messages;
        if let Some(sync) = &self.sync
            && sync.read(cx).is_running()
        {
            return (messages.refreshing_status.clone(), None);
        }
        let list = self.list.read(cx).delegate();
        if !list.has_index() {
            return (messages.no_index_status.clone(), None);
        }
        let count = list.count();
        let count = if count == 1 {
            messages.one_result.clone()
        } else {
            fill(&messages.results, &[("count", &count.to_string())])
        };
        (messages.ready.clone(), Some(count))
    }

    /// The screen's keys the footer shows: all but ⇧⌘↵ (spec D9), which would come and go with
    /// the entry type and move the others; it still runs, and the action panel lists it.
    fn footer_items(&self, cx: &App) -> Vec<KeyItem> {
        self.screen_keys(cx)
            .into_iter()
            .filter(|item| item.keys != "shift-cmd-enter")
            .collect()
    }

    /// The footer's right side as text, for tests: "Details ↵", …, "Actions ⌘K".
    #[must_use]
    pub fn footer_keys(&self, cx: &App) -> Vec<String> {
        let mut keys: Vec<String> = self
            .footer_items(cx)
            .into_iter()
            .map(|item| format!("{} {}", item.label, item.caps))
            .collect();
        keys.push(format!("{} {}", self.catalog.label("actions"), self.actions_caps()));
        keys
    }

    fn actions_caps(&self) -> String {
        self.catalog.actions["actions"]
            .keys
            .as_deref()
            .map(caps)
            .unwrap_or_default()
    }

    pub(super) fn render_footer(
        &self,
        selected: Option<&Entry>,
        _: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let names_entry = self.footer_names_entry(selected);
        let report_row = self
            .report(cx)
            .filter(|sync| sync.read(cx).has_pane() || self.navigator.top() == Place::ReportDetail)
            .and_then(|sync| sync.read(cx).selected_row(cx).cloned());
        let left: AnyElement = if let Some(row) = report_row {
            let colors = &design().colors;
            let (icon, tint) = crate::report_list::kind_tile(row.kind, colors);
            let copy_caps = self.catalog.actions["copy-path"]
                .keys
                .as_deref()
                .map(caps)
                .unwrap_or_default();
            entry_status(tile(icon, tint, TileSize::Small), row.path.clone(), copy_caps).into_any_element()
        } else if let Some(entry) = selected.filter(|_| names_entry) {
            let colors = &design().colors;
            let kind = self.catalog.entry_type(&entry.entry_type);
            let (icon, tint) = kind.map_or(("file-text", colors.faint), |kind| {
                (kind.icon.as_str(), colors.named(&kind.tint).unwrap_or(colors.faint))
            });
            let copy_caps = self.catalog.actions["copy-path"]
                .keys
                .as_deref()
                .map(caps)
                .unwrap_or_default();
            entry_status(
                tile(icon, tint, TileSize::Small),
                entry.source_rel_path.clone(),
                copy_caps,
            )
            .into_any_element()
        } else {
            let (word, count) = self.status_parts(cx);
            let dot = if self.sync.as_ref().is_some_and(|sync| sync.read(cx).is_running()) {
                Dot::Busy
            } else if self.list.read(cx).delegate().has_index() {
                Dot::Ready
            } else {
                Dot::Off
            };
            status(dot, word, count).into_any_element()
        };
        let plain = plain_button(cx);
        let mut right: Vec<AnyElement> = self
            .footer_items(cx)
            .into_iter()
            .enumerate()
            .map(|(ix, item)| {
                let id = if ix == 0 {
                    SharedString::from("footer-primary-action")
                } else {
                    SharedString::from(format!("footer-key-{ix}"))
                };
                let keys = item.keys;
                Button::new(id)
                    .custom(plain)
                    // No button padding: the contents sit on the window's content edge (spec 2026-10-19 spacing D2).
                    .px_0()
                    .child(key_item(item.label, item.caps))
                    .on_click(cx.listener(move |this, _, window, cx| this.ladder(keys, window, cx)))
                    .into_any_element()
            })
            .collect();
        right.push(
            self.actions_popover(
                Button::new("footer-open-actions")
                    .custom(plain)
                    // No button padding: the contents sit on the window's content edge (spec 2026-10-19 spacing D2).
                    .px_0()
                    // The popover opens on its trigger's click; a handler here would close it again.
                    .child(key_item(self.catalog.label("actions").to_owned(), self.actions_caps())),
                cx,
            )
            .into_any_element(),
        );
        footer(left, right)
    }
}
