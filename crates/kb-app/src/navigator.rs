//! Where the user is (spec 2026-10-27-navigation-model): a stack of places, the list at the bottom, and the overlay
//! open over the top one. Plain Rust: `SearchView` asks it which place is current, which keys show and which bindings
//! apply. One rule for every screen; a new screen is a new `Place` and nothing else.

use crate::actions::Submenu;
use crate::catalog::Screen as KeyScreen;
use crate::keys::{CONTEXT, DETAIL_CONTEXT, REFRESH_CONTEXT, REPORT_CONTEXT};
use crate::sync_view::Phase;

/// A screen on the stack (D1). A split is not a place: it is a pane of the list or of the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// The search list, always at the bottom.
    List,
    /// One entry, full screen.
    Detail,
    /// A refresh: running, stopped, failed, or its report.
    Refresh,
    /// One report row, full screen.
    ReportDetail,
}

/// What floats over the top place; while one is open it owns the keys (D1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    /// Nothing.
    None,
    /// The action panel: the root, or a submenu.
    Actions(Option<Submenu>),
    /// The filter.
    Filter,
}

/// The name a pushed place shows after `‹`: the place it was opened from (D5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackName {
    /// The words.
    pub text: String,
    /// Whether they are a query, drawn in mono.
    pub mono: bool,
}

/// The stack of places and the overlay.
#[derive(Debug)]
pub struct Navigator {
    /// Never empty: `List` at the bottom, with no back name.
    stack: Vec<(Place, Option<BackName>)>,
    overlay: Overlay,
}

impl Default for Navigator {
    fn default() -> Self {
        Self {
            stack: vec![(Place::List, None)],
            overlay: Overlay::None,
        }
    }
}

impl Navigator {
    /// The current place.
    #[must_use]
    pub fn top(&self) -> Place {
        self.stack.last().map_or(Place::List, |(place, _)| *place)
    }

    /// Whether `place` is anywhere on the stack.
    #[must_use]
    pub fn contains(&self, place: Place) -> bool {
        self.stack.iter().any(|(p, _)| *p == place)
    }

    /// Opens `place` on top, remembering where it was opened from. `List` is never pushed.
    pub fn push(&mut self, place: Place, back: BackName) {
        if place != Place::List {
            self.stack.push((place, Some(back)));
        }
    }

    /// Back one place; `None` on the list, which is never popped.
    pub fn pop(&mut self) -> Option<Place> {
        if self.stack.len() > 1 {
            self.stack.pop().map(|(place, _)| place)
        } else {
            None
        }
    }

    /// Back to the list, from anywhere.
    pub fn pop_to_list(&mut self) {
        self.stack.truncate(1);
    }

    /// The overlay open over the top place.
    #[must_use]
    pub fn overlay(&self) -> Overlay {
        self.overlay
    }

    /// Opens `overlay` (or closes it, with `Overlay::None`).
    pub fn set_overlay(&mut self, overlay: Overlay) {
        self.overlay = overlay;
    }

    /// Whether the top place may act on a key: no overlay is open (D4).
    #[must_use]
    pub fn screen_has_keys(&self) -> bool {
        self.overlay == Overlay::None
    }

    /// The catalog screen whose keys apply: the overlay's, else the top place's. `list` is what the list answers
    /// (list, empty or no index); `phase` is the refresh's, when one is on the stack.
    #[must_use]
    pub fn key_screen(&self, list: KeyScreen, phase: Option<Phase>) -> KeyScreen {
        match self.overlay {
            Overlay::Filter => return KeyScreen::Filter,
            Overlay::Actions(Some(Submenu::Links)) => return KeyScreen::Links,
            _ => {}
        }
        match self.top() {
            Place::List => list,
            Place::Detail => KeyScreen::Detail,
            Place::ReportDetail => KeyScreen::ReportDetail,
            Place::Refresh => match phase {
                Some(Phase::Report) => KeyScreen::Report,
                Some(Phase::Stopped) => KeyScreen::Stopped,
                Some(Phase::Failed) => KeyScreen::Failed,
                Some(Phase::Running) | None => KeyScreen::Refreshing,
            },
        }
    }

    /// The key context `render` sets, from shared parts, so places that behave alike share bindings (D4): both
    /// detail screens scroll with `Detail`'s keys; the report and its detail take the report-only keys.
    #[must_use]
    pub fn key_context(&self) -> String {
        let parts: &[&str] = match self.top() {
            Place::List => &[CONTEXT],
            Place::Detail => &[CONTEXT, DETAIL_CONTEXT],
            Place::Refresh => &[CONTEXT, REFRESH_CONTEXT, REPORT_CONTEXT],
            Place::ReportDetail => &[CONTEXT, DETAIL_CONTEXT, REPORT_CONTEXT],
        };
        parts.join(" ")
    }

    /// Where back goes from the top place; `None` on the list.
    #[must_use]
    pub fn back_name(&self) -> Option<&BackName> {
        self.stack.last()?.1.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(text: &str) -> BackName {
        BackName {
            text: text.to_owned(),
            mono: false,
        }
    }

    #[test]
    fn starts_on_the_list_and_never_pops_it() {
        let mut nav = Navigator::default();
        assert_eq!(nav.top(), Place::List);
        assert_eq!(nav.pop(), None);
        assert_eq!(nav.top(), Place::List);
        assert_eq!(nav.back_name(), None);
    }

    #[test]
    fn pushes_and_pops_one_place_at_a_time() {
        let mut nav = Navigator::default();
        nav.push(Place::Detail, name("/k8s"));
        nav.push(Place::Refresh, name("Tokio"));
        assert_eq!(nav.top(), Place::Refresh);
        assert_eq!(nav.back_name(), Some(&name("Tokio")));
        assert!(nav.contains(Place::Detail));
        assert_eq!(nav.pop(), Some(Place::Refresh));
        assert_eq!(nav.back_name(), Some(&name("/k8s")));
        nav.pop_to_list();
        assert_eq!(nav.top(), Place::List);
    }

    #[test]
    fn never_pushes_the_list() {
        let mut nav = Navigator::default();
        nav.push(Place::List, name("x"));
        assert_eq!(nav.pop(), None, "still one place");
    }

    #[test]
    fn gives_the_keys_to_an_open_overlay() {
        let mut nav = Navigator::default();
        assert!(nav.screen_has_keys());
        nav.set_overlay(Overlay::Actions(None));
        assert!(!nav.screen_has_keys());
        nav.set_overlay(Overlay::None);
        assert!(nav.screen_has_keys());
    }

    #[test]
    fn maps_each_place_and_overlay_to_its_key_screen() {
        let mut nav = Navigator::default();
        assert_eq!(
            nav.key_screen(KeyScreen::Empty, None),
            KeyScreen::Empty,
            "the list answers for itself"
        );
        nav.push(Place::Refresh, name("Search"));
        assert_eq!(
            nav.key_screen(KeyScreen::List, Some(Phase::Running)),
            KeyScreen::Refreshing
        );
        assert_eq!(nav.key_screen(KeyScreen::List, Some(Phase::Report)), KeyScreen::Report);
        assert_eq!(
            nav.key_screen(KeyScreen::List, Some(Phase::Stopped)),
            KeyScreen::Stopped
        );
        assert_eq!(nav.key_screen(KeyScreen::List, Some(Phase::Failed)), KeyScreen::Failed);
        nav.push(Place::ReportDetail, name("Refresh"));
        assert_eq!(
            nav.key_screen(KeyScreen::List, Some(Phase::Report)),
            KeyScreen::ReportDetail
        );
        nav.set_overlay(Overlay::Filter);
        assert_eq!(nav.key_screen(KeyScreen::List, None), KeyScreen::Filter);
        nav.set_overlay(Overlay::Actions(Some(Submenu::Links)));
        assert_eq!(nav.key_screen(KeyScreen::List, None), KeyScreen::Links);
    }

    #[test]
    fn builds_the_key_context_from_shared_parts() {
        let mut nav = Navigator::default();
        assert_eq!(nav.key_context(), "Search");
        nav.push(Place::Detail, name("/k8s"));
        assert_eq!(nav.key_context(), "Search Detail");
        nav.pop();
        nav.push(Place::Refresh, name("Search"));
        assert_eq!(nav.key_context(), "Search Refresh Report");
        nav.push(Place::ReportDetail, name("Refresh"));
        assert_eq!(
            nav.key_context(),
            "Search Detail Report",
            "the report's detail scrolls as an entry's does"
        );
    }
}
