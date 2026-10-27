# One navigation model Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every screen in kb follows one navigation rule. A stack of places owns where the user is. One back
function goes back. One action path acts on the top place. One header draws `‹ <where back goes>` with the same crumb
component ⌘K uses.

**Architecture:** A new plain-Rust `Navigator` (`crates/kb-app/src/navigator.rs`) holds a stack of `Place`s
(`List`, `Detail`, `Refresh`, `ReportDetail`) and the open `Overlay`. `SearchView` replaces its `screen` and `overlay`
fields with it, and `SyncView` loses its own `ReportLevel` state machine: the report's detail becomes a place. Keys,
footer keys and key contexts are all derived from the navigator. A shared `ui::crumb` draws "back to X" in the
header and in ⌘K's submenus.

**Tech Stack:** Rust 2024, GPUI Kit 0.7.1 (`gpui_kit`), `cargo nextest`, hk, mise.

**Spec:** `docs/superpowers/specs/2026-10-27-navigation-model-design.md`. Read D1–D7 before starting. The approved
page is `docs/guides/design/20-navigation.html` (option B).

**Consistency over flexibility (owner, 2026-10-27).** Every concept below has exactly one home. When a task tempts you
to add a special case for one screen, stop: the answer belongs in `Navigator`, `place_name`, `run_action` or
`ui::crumb`. Do not add a second way to do any of these.

---

## Before you start

- Read `AGENTS.md` (commit rules: `git add <paths>`, then `git commit`; Conventional Commits; sentence-case subject of
  at most 72 characters; end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`).
- Read the GPUI Kit Coding Guides, "Events, actions, and focus", at `~/.claude/skills/gpui-kit/references/coding-guides.md`.
- **The suite is red on purpose.** `crates/kb-app/tests/ui.rs` has failing `audit_*` reproductions, from
  `docs/superpowers/audits/2026-10-27-mvp-ui-ux-audit.md`. Record the baseline once:

  ```bash
  cargo nextest run -p kb-app --test ui --no-fail-fast 2>&1 | grep -E "^\s+FAIL" | sort > /tmp/kb-nav-baseline.txt
  wc -l /tmp/kb-nav-baseline.txt   # expected: 10
  ```

  "No regressions" in every task means this: the failures are a subset of the baseline, minus the ones the task
  turns green. Check it with the same command and `comm -13 /tmp/kb-nav-baseline.txt <new list>`, which must print
  nothing.
- `mise run gate` fails while the audit tests fail, so commits rely on the pre-commit hook (hk) and the commands in
  each task. Run `hk fix --all` before each commit.

## File map

| File                                                           | Change                                                                                                                          |
| -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `crates/kb-app/src/navigator.rs`                               | **New.** `Place`, `Overlay`, `BackName`, `Navigator`; unit tests                                                                |
| `crates/kb-app/src/lib.rs`                                     | `pub mod navigator;`                                                                                                            |
| `crates/kb-app/src/keys.rs`                                    | `REPORT_CONTEXT`; the report-only actions are bound in it                                                                       |
| `crates/kb-app/src/search_view.rs`                             | `navigator` and `sync` fields; `place_name`, `push`, `pop`, `key_action`; `run_action` keyed by the top place; `back` rewritten |
| `crates/kb-app/src/search_view/{panel,footer,theme,header}.rs` | Read the navigator instead of `screen` and `overlay`; the header draws the crumb                                                |
| `crates/kb-app/src/sync_view.rs`                               | `ReportLevel` and `Move::Detail` removed; a `pane: bool`                                                                        |
| `crates/kb-app/src/ui/crumb.rs`                                | **New.** The one "‹ name" component                                                                                             |
| `crates/kb-app/src/ui.rs`                                      | `pub mod crumb;`                                                                                                                |
| `crates/kb-app/src/ui/action_panel.rs`                         | The submenu crumb uses `ui::crumb`                                                                                              |
| `crates/kb-app/src/catalog.rs`, `assets/catalog.toml`          | `back_search`, `back_refresh` messages                                                                                          |
| `crates/kb-app/tests/ui.rs`                                    | Accessors move to `place()`; new navigation tests; the audit tests' doc comments put back on their own tests                    |
| `DESIGN.md`, `CONTEXT.md`, the audit                           | Docs                                                                                                                            |

---

### Task 1: The `Navigator`, alone

**Files:**

- Create: `crates/kb-app/src/navigator.rs`
- Modify: `crates/kb-app/src/lib.rs` (add `pub mod navigator;` beside the other `pub mod` lines, alphabetically)
- Modify: `crates/kb-app/src/keys.rs:21` (add `REPORT_CONTEXT` after `REFRESH_CONTEXT`)

- [ ] **Step 1: Add the report context constant** to `crates/kb-app/src/keys.rs`, after `REFRESH_CONTEXT`:

```rust
/// Added to [`CONTEXT`] on the report and its detail: the report-only keys (Copy Location, Copy Error, Reveal).
pub const REPORT_CONTEXT: &str = "Report";
```

- [ ] **Step 2: Write the module with its failing tests.** Create `crates/kb-app/src/navigator.rs`:

```rust
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
        assert_eq!(nav.key_screen(KeyScreen::Empty, None), KeyScreen::Empty, "the list answers for itself");
        nav.push(Place::Refresh, name("Search"));
        assert_eq!(nav.key_screen(KeyScreen::List, Some(Phase::Running)), KeyScreen::Refreshing);
        assert_eq!(nav.key_screen(KeyScreen::List, Some(Phase::Report)), KeyScreen::Report);
        assert_eq!(nav.key_screen(KeyScreen::List, Some(Phase::Stopped)), KeyScreen::Stopped);
        assert_eq!(nav.key_screen(KeyScreen::List, Some(Phase::Failed)), KeyScreen::Failed);
        nav.push(Place::ReportDetail, name("Refresh"));
        assert_eq!(nav.key_screen(KeyScreen::List, Some(Phase::Report)), KeyScreen::ReportDetail);
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
        assert_eq!(nav.key_context(), "Search Detail Report", "the report's detail scrolls as an entry's does");
    }
}
```

- [ ] **Step 3: Register the module** in `crates/kb-app/src/lib.rs`, with the other `pub mod` lines:

```rust
pub mod navigator;
```

- [ ] **Step 4: Run the tests.**

Run: `cargo nextest run -p kb-app navigator`
Expected: 6 passed. If `catalog::Screen`, `actions::Submenu` or `sync_view::Phase` are not `Copy + PartialEq + Debug`,
the build says so. All three derive them today (`catalog.rs:103`, `actions.rs:17`, `sync_view.rs:147`).

- [ ] **Step 5: Commit.**

```bash
hk fix --all
git add crates/kb-app/src/navigator.rs crates/kb-app/src/lib.rs crates/kb-app/src/keys.rs
git commit -m "feat(app): Add a navigator: a stack of places and the overlay" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: `SearchView` on the navigator, with no change in behaviour

The structural switch. `SearchView`'s `screen` and `overlay` fields become `navigator` and `sync`. Every reader of
the old fields reads the navigator instead. The report's own detail level stays in `SyncView` until Task 3. When this
task ends, the UI suite fails exactly the baseline and nothing more.

**Files:**

- Modify: `crates/kb-app/src/search_view.rs`, `search_view/panel.rs`, `search_view/footer.rs`, `search_view/theme.rs`,
  `search_view/header.rs`
- Modify: `crates/kb-app/src/catalog.rs` (`Messages`), `crates/kb-app/assets/catalog.toml` (`[messages]`)

- [ ] **Step 1: Add the two words places are named with.** In `crates/kb-app/assets/catalog.toml`, under
      `[messages]`, after `editor_failed`:

```toml
# What ‹ says it returns to (navigation spec D5): the list without a query, and a refresh.
back_search = "Search"
back_refresh = "Refresh"
```

In `crates/kb-app/src/catalog.rs`, in `pub struct Messages`, after `pub editor_failed: String,`:

```rust
/// The list's name after ‹ when it has no query.
pub back_search: String,
/// A refresh's name after ‹.
pub back_refresh: String,
```

Run: `cargo nextest run -p kb-app catalog`
Expected: PASS. If a catalog test counts the messages, add the two names to its list. If `Messages` has a
hand-written `Default`, add the two fields there with the same words.

- [ ] **Step 2: Replace the fields.** In `crates/kb-app/src/search_view.rs`:

1. Delete the local `enum Screen { … }` (lines 42–51) and `enum Overlay { … }` (lines 57–66).
2. Add the imports. `super::Overlay` keeps working in the submodules, because a private `use` is visible to child
   modules:

```rust
use crate::navigator::{BackName, Navigator, Overlay, Place};
```

3. In `pub struct SearchView`, replace `screen: Screen,` and the `overlay: Overlay,` field (and its doc comment) with:

```rust
/// Where the user is: the places and the overlay (navigation spec D2).
navigator: Navigator,
/// The refresh while `Place::Refresh` is on the stack.
sync: Option<Entity<SyncView>>,
```

4. In `new`, replace `screen: Screen::Search,` with `navigator: Navigator::default(),`, then `sync: None,`, and
   delete `overlay: Overlay::None,`.
5. In `impl std::fmt::Debug for SearchView`, replace `.field("screen", &self.screen)` with
   `.field("navigator", &self.navigator)`.

- [ ] **Step 3: Add the one way to name, push and pop a place.** Add these to `impl SearchView`, after
      `is_rebuilding`:

```rust
    /// The name `place` shows when a place opened from it says where back goes (D2, D5): the list is its query, or
    /// "Search" without one; an entry is its title; a refresh is "Refresh". The one function that names a place.
    fn place_name(&self, place: Place, cx: &App) -> BackName {
        let m = &self.catalog.messages;
        let plain = |text: &str| BackName {
            text: text.to_owned(),
            mono: false,
        };
        match place {
            Place::List => {
                let query = self.query_input.read(cx).value().to_string();
                if query.is_empty() {
                    plain(&m.back_search)
                } else {
                    BackName { text: query, mono: true }
                }
            }
            Place::Detail => self
                .list
                .read(cx)
                .delegate()
                .selected_entry()
                .map_or_else(|| plain(&m.back_search), |entry| plain(&entry.title)),
            Place::Refresh | Place::ReportDetail => plain(&m.back_refresh),
        }
    }

    /// Opens `place` on top of the current one: it remembers where it came from, starts scrolled to the top, and
    /// takes the keyboard.
    fn push(&mut self, place: Place, window: &mut Window, cx: &mut Context<Self>) {
        let back = self.place_name(self.navigator.top(), cx);
        self.navigator.push(place, back);
        self.detail_scroll.set_offset(gpui_kit::Point::default());
        self.focus_screen(window, cx);
        cx.notify();
    }

    /// Back one place, to the one underneath as it was left (D3). The refresh goes with its place.
    fn pop(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.navigator.pop();
        if !self.navigator.contains(Place::Refresh) {
            self.sync = None;
        }
        self.focus_screen(window, cx);
        cx.notify();
    }
```

- [ ] **Step 4: Rewrite the readers in `search_view.rs`.** Make each change below. The code is the whole new body.

`is_showing_detail` / `is_rebuilding`:

```rust
pub fn is_showing_detail(&self) -> bool {
    self.navigator.top() == Place::Detail
}
```

```rust
pub fn is_rebuilding(&self) -> bool {
    self.navigator.contains(Place::Refresh)
}
```

`show_detail_screen`:

```rust
fn show_detail_screen(&mut self, window: &mut Window, cx: &mut Context<Self>) {
    if self.list.read(cx).delegate().selected_entry().is_none() {
        return;
    }
    self.push(Place::Detail, window, cx);
}
```

`key_screen` (the report's detail is still a `SyncView` level until Task 3, hence the marked line):

```rust
fn key_screen(&self, cx: &App) -> KeyScreen {
    let list = {
        let list = self.list.read(cx).delegate();
        if !list.has_index() {
            KeyScreen::NoIndex
        } else if list.entries().is_empty() && self.query_input.read(cx).value() == list.query() {
            KeyScreen::Empty
        } else {
            KeyScreen::List
        }
    };
    let phase = self.sync.as_ref().map(|sync| sync.read(cx).phase());
    let screen = self.navigator.key_screen(list, phase);
    // Until Task 3: the report's detail is a level of `SyncView`, not yet a place.
    if screen == KeyScreen::Report
        && self.sync.as_ref().is_some_and(|s| matches!(s.read(cx).level(), ReportLevel::Detail { .. }))
    {
        return KeyScreen::ReportDetail;
    }
    screen
}
```

`ladder`: replace `if self.overlay != Overlay::None {` with `if !self.navigator.screen_has_keys() {`.

`report`:

```rust
fn report(&self, cx: &App) -> Option<Entity<SyncView>> {
    self.sync
        .clone()
        .filter(|_| matches!(self.navigator.top(), Place::Refresh | Place::ReportDetail))
        .filter(|sync| sync.read(cx).phase() == Phase::Report)
}
```

In `run_action`'s list branch, change `"stop"` and `"retry"` to:

```rust
("stop", _) => {
    if let Some(sync) = &self.sync {
        sync.read(cx).cancel();
    }
}
("back", _) => self.back(&Back, window, cx),
("retry", _) => {
    // A finished or failed refresh lets `refresh` start a new one, of the same sources.
    let selection = self.sync.as_ref().map_or(Selection::All, |sync| sync.read(cx).selection());
    self.refresh(selection, window, cx);
}
```

`run_row`: replace `if matches!(self.screen, Screen::Detail) {` with `if self.navigator.top() == Place::Detail {`,
and `if !matches!(self.screen, Screen::Search) {` with `if self.navigator.top() != Place::List {`.

`show_details` and `hide_details`: replace each `match self.screen { Screen::Detail => {} _ => self.show_details = X, }`
with:

```rust
if self.navigator.top() != Place::Detail {
    self.show_details = X; // `true` in show_details, `false` in hide_details
}
```

`refresh` (a refresh from `Detail` now returns to that detail, as the spec's error table says, and a rerun
replaces the refresh in place):

```rust
fn refresh(&mut self, selection: Selection, window: &mut Window, cx: &mut Context<Self>) {
    if self.sync.as_ref().is_some_and(|sync| sync.read(cx).is_running()) {
        return;
    }
    let job = SyncJob {
        selection,
        sources: self.config.sources_dir().to_path_buf(),
        skills_command: self.config.skills_command(),
        database: self.config.database().to_path_buf(),
    };
    let catalog = self.catalog.clone();
    let sync = cx.new(|cx| SyncView::start(job, catalog, window, cx));
    let subscription = cx.subscribe_in(&sync, window, |this, _, finished: &SyncFinished, window, cx| {
        window.push_notification(finished.message.clone(), cx);
        if finished.changed {
            let connection = open_readonly(this.config.database()).ok();
            this.list.update(cx, |list, cx| {
                list.delegate_mut().set_connection(connection);
                cx.notify();
            });
        }
    });
    self.subscriptions.push(subscription);
    self.sync = Some(sync);
    // A rerun replaces the refresh in place: from its detail, back to the report first; nothing new is pushed.
    if self.navigator.top() == Place::ReportDetail {
        self.navigator.pop();
    }
    if self.navigator.top() == Place::Refresh {
        self.focus_screen(window, cx);
        cx.notify();
    } else {
        self.push(Place::Refresh, window, cx);
    }
}
```

`refresh_selection`:

```rust
pub fn refresh_selection(&self, cx: &App) -> Option<Selection> {
    self.sync.as_ref().map(|sync| sync.read(cx).selection())
}
```

`back` (one function, D3; the `SyncView::back` call stays until Task 3):

```rust
fn back(&mut self, _: &Back, window: &mut Window, cx: &mut Context<Self>) {
    // 1. An overlay: clear its search, then leave the submenu, then close it (Raycast).
    if !self.navigator.screen_has_keys() {
        if !self.panel_query.read(cx).value().is_empty() {
            return self.set_panel_query("", window, cx);
        }
        // Leaving Theme… without ↵ restores the theme it opened on.
        self.cancel_theme_preview(cx);
        let next = match self.navigator.overlay() {
            Overlay::Actions(Some(crate::actions::Submenu::Refresh | crate::actions::Submenu::Theme)) => {
                Overlay::Actions(Some(crate::actions::Submenu::More))
            }
            Overlay::Actions(Some(_)) => Overlay::Actions(None),
            _ => Overlay::None,
        };
        return self.show_overlay(next, window, cx);
    }
    // 2. A running refresh: Esc stops it.
    if let Some(sync) = self.sync.clone().filter(|_| self.navigator.top() == Place::Refresh) {
        if sync.read(cx).is_running() {
            sync.read(cx).cancel();
            cx.notify();
            return;
        }
        // Until Task 3: the report's detail is a level of `SyncView`.
        if sync.read(cx).phase() == Phase::Report && sync.update(cx, SyncView::back) {
            return;
        }
    }
    // 3. The list: clear the query, then hide the panel.
    if self.navigator.top() == Place::List {
        if self.query_input.read(cx).value().is_empty() {
            self.cancel_theme_preview(cx);
            cx.hide();
        } else {
            self.query_input.update(cx, |input, cx| input.set_value("", window, cx));
            self.search("", window, cx);
        }
        cx.notify();
        return;
    }
    // 4. Anything else: back one place.
    self.pop(window, cx);
}
```

`report_step`: replace `if self.overlay == Overlay::None` with `if self.navigator.screen_has_keys()`.

`focus_screen`:

```rust
fn focus_screen(&self, window: &mut Window, cx: &mut Context<Self>) {
    if self.navigator.top() == Place::List {
        self.query_input.update(cx, |input, cx| input.focus(window, cx));
    } else {
        self.focus.focus(window, cx);
    }
}
```

`filter_by_tag`: replace `self.screen = Screen::Search;` with:

```rust
self.navigator.pop_to_list();
self.sync = None;
```

`render_content`: replace the `match &self.screen { … }` head and arms with:

```rust
match self.navigator.top() {
    Place::Refresh | Place::ReportDetail => self
        .sync
        .clone()
        .map_or_else(|| div().into_any_element(), IntoElement::into_any_element),
    Place::Detail => selected.map_or_else(
        || div().into_any_element(),
        |entry| {
            self.entry_view(entry, Layout::Detail, window, cx)
                .track_scroll(self.detail_scroll.clone())
                .into_any_element()
        },
    ),
    Place::List => match selected.filter(|_| self.show_details) {
        // … the existing Some(entry) / None arms, unchanged …
    },
}
```

`render`: replace `let compact = matches!(self.screen, Screen::Search) && …` with
`let compact = self.navigator.top() == Place::List && self.show_details && selected.is_some();`. Replace the
`let context = if … else …;` block with `let context = self.navigator.key_context();`. Then drop `DETAIL_CONTEXT` and
`REFRESH_CONTEXT` from the `crate::keys::{…}` import if the compiler says they are unused.

- [ ] **Step 5: Rewrite the readers in the submodules.**

`search_view/panel.rs`:

- `is_actions_open`: `matches!(self.navigator.overlay(), Overlay::Actions(_))`.
- `is_filter_open`: `self.navigator.overlay() == Overlay::Filter`.
- `current_page`: replace `let entry = match self.screen { Screen::Rebuild(_) => None, _ => list.selected_entry() };`
  with:

```rust
let entry = match self.navigator.top() {
    Place::Refresh | Place::ReportDetail => None,
    Place::List | Place::Detail => list.selected_entry(),
};
```

Then replace both `match self.overlay {` with `match self.navigator.overlay() {`.

- `show_overlay`: replace `self.overlay = overlay;` with `self.navigator.set_overlay(overlay);`.
- `anchored`: replace `matches!(self.overlay, Overlay::Actions(Some(_)))` with
  `matches!(self.navigator.overlay(), Overlay::Actions(Some(_)))`.
- Imports: `use super::{Overlay, SearchView};` stays; add `use crate::navigator::Place;`.

`search_view/theme.rs:58`: `if self.navigator.overlay() != Overlay::Actions(Some(Submenu::Theme)) {`.

`search_view/footer.rs`:

- line 19: replace `matches!(self.screen, Screen::Search)` with `self.navigator.top() == Place::List`.
- line 31: replace `if let Screen::Rebuild(sync) = &self.screen` with `if let Some(sync) = &self.sync`.
- line 117: replace `matches!(&self.screen, Screen::Rebuild(sync) if sync.read(cx).is_running())` with
  `self.sync.as_ref().is_some_and(|sync| sync.read(cx).is_running())`.
- Imports: drop `Screen`; add `use crate::navigator::Place;`.

`search_view/header.rs:39`: `let list = self.navigator.top() == Place::List;` and add
`use crate::navigator::Place;`.

- [ ] **Step 6: Build, then check the whole suite against the baseline.**

Run: `cargo clippy -p kb-app --all-targets -- -D warnings`
Expected: no warnings. Fix any unused import it names.

Run:

```bash
cargo nextest run -p kb-app --no-fail-fast 2>&1 | grep -E "^\s+FAIL" | sort > /tmp/kb-nav-t2.txt
comm -13 /tmp/kb-nav-baseline.txt /tmp/kb-nav-t2.txt
```

Expected: `comm` prints nothing (no new failure). The lib tests all pass.

- [ ] **Step 7: Commit.**

```bash
hk fix --all
git add crates/kb-app/src/search_view.rs crates/kb-app/src/search_view crates/kb-app/src/catalog.rs crates/kb-app/assets/catalog.toml
git commit -m "ref(app): Drive the panel's screens from the navigator" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: The report's detail becomes a place, and scrolls like an entry's

`SyncView` keeps only a `pane` flag. ↵ on a report row pushes `Place::ReportDetail`, which `SearchView` draws with the
same `detail_scroll` handle and the same scroll actions as an entry's detail (its context includes `Detail`).
That turns `audit_k3_arrows_on_the_report_detail_keep_its_row` green.

**Files:**

- Modify: `crates/kb-app/src/sync_view.rs`
- Modify: `crates/kb-app/src/search_view.rs`, `crates/kb-app/src/search_view/footer.rs`
- Test: `crates/kb-app/tests/ui.rs` (`navigates_the_report_like_the_list`, a new accessor)

- [ ] **Step 1: Update the report navigation test first.** In `crates/kb-app/tests/ui.rs`, replace the body of
      `navigates_the_report_like_the_list` from `press(cx, window, "up");` to the end with:

```rust
    press(cx, window, "up");
    press(cx, window, "right");
    assert!(cx.read(|cx| view.read(cx).report_pane_open(cx)), "→ opens the report's pane");
    press(cx, window, "enter");
    assert_eq!(cx.read(|cx| view.read(cx).place()), Place::ReportDetail);
    assert_eq!(
        cx.read(|cx| view.read(cx).footer_keys(cx)),
        ["Edit ↵", "Copy Location ⌘↵", "Actions ⌘K"]
    );
    press(cx, window, "cmd-enter");
    let copied = cx.read_from_clipboard().and_then(|item| item.text()).unwrap();
    assert!(copied.ends_with("/errors/broken.yml:4:1"), "{copied}");
    press(cx, window, "escape");
    assert_eq!(cx.read(|cx| view.read(cx).place()), Place::Refresh, "Esc: back to the report");
    assert!(cx.read(|cx| view.read(cx).report_pane_open(cx)), "its pane as it was");
    press(cx, window, "escape");
    assert_eq!(cx.read(|cx| view.read(cx).place()), Place::List, "Esc: back to the list");
}
```

Replace its first line `use kb_app::sync_view::ReportLevel;` with `use kb_app::navigator::Place;`.

- [ ] **Step 2: Run it to see it fail.**

Run: `cargo nextest run -p kb-app --test ui navigates_the_report_like_the_list`
Expected: FAIL to compile: `no method named place` / `report_pane_open`.

- [ ] **Step 3: Reduce `SyncView` to a pane flag.** In `crates/kb-app/src/sync_view.rs`:

1. Delete `pub enum ReportLevel { … }`, `impl ReportLevel { … }`, the `Move::Detail` variant, and the test
   `steps_back_one_level_at_a_time`.
2. In `pub struct SyncView`, replace `level: ReportLevel,` with:

```rust
/// Whether the report shows the selected row's pane beside the rows (→ / ←).
pane: bool,
```

In `start`, replace `level: ReportLevel::List,` with `pane: false,`.
3. Replace `level`, `go` and `back` with:

```rust
    /// Whether the selected row's pane is open beside the rows.
    #[must_use]
    pub fn has_pane(&self) -> bool {
        self.pane
    }

    /// → (`Move::Pane`) or ← (`Move::NoPane`) on the report; nothing without a selected row.
    pub fn go(&mut self, m: Move, cx: &mut Context<Self>) {
        if self.selected_row(cx).is_some() {
            self.pane = m == Move::Pane;
            cx.notify();
        }
    }
```

4. In `render_report`, replace the `match (self.level, selected) { … }` with:

```rust
match selected.filter(|_| self.pane) {
    Some(row) => h_flex()
        .size_full()
        .child(
            div()
                .h_full()
                .w(relative(d.components.split.list))
                .flex_none()
                .border_r(pt(d.components.border.hairline))
                .border_color(c.edge)
                .child(rows()),
        )
        .child(div().h_full().flex_1().min_w_0().child(report_pane(&row, m, false)))
        .into_any_element(),
    None => v_flex()
        .size_full()
        .child(summary)
        .child(div().flex_1().min_h_0().child(rows()))
        .into_any_element(),
}
```

- [ ] **Step 4: Push and draw the place in `SearchView`.** In `crates/kb-app/src/search_view.rs`:

1. Imports: `use crate::sync_view::{Move, Phase, SyncFinished, SyncJob, SyncView};` (no `ReportLevel`), and add
   `use crate::ui::report_pane::report_pane;` plus `StatefulInteractiveElement as _` to the `gpui_kit::{…}` import.
2. In `run_action`'s report block, replace the `"details"` arm with:

```rust
("details", Some(_)) => return self.push(Place::ReportDetail, window, cx),
```

3. Remove the two "Until Task 3" blocks: the `ReportDetail` override in `key_screen`, and the `SyncView::back` call
   in `back`.
4. Replace `report_level` (the test accessor) with:

```rust
    /// The current place, for tests.
    #[must_use]
    pub fn place(&self) -> Place {
        self.navigator.top()
    }

    /// Whether the report shows its pane beside the rows, for tests.
    #[must_use]
    pub fn report_pane_open(&self, cx: &App) -> bool {
        self.sync.as_ref().is_some_and(|sync| sync.read(cx).has_pane())
    }
```

5. In `render_content`, split the `Place::Refresh | Place::ReportDetail` arm. The report's detail uses the entry
   detail's scroll handle, so the `Detail` context's ↑↓ / ⌘↑↓ scroll it through the same handlers:

```rust
Place::Refresh => self
    .sync
    .clone()
    .map_or_else(|| div().into_any_element(), IntoElement::into_any_element),
Place::ReportDetail => match self.report(cx).and_then(|sync| sync.read(cx).selected_row(cx).cloned()) {
    Some(row) => div()
        .id("report-detail")
        .size_full()
        .overflow_y_scroll()
        .track_scroll(&self.detail_scroll)
        .child(report_pane(&row, &self.catalog.messages, true))
        .into_any_element(),
    None => div().into_any_element(),
},
```

If `report_pane(.., true)` already sets its own `.id(…)` with `overflow_y_scroll`, take the scroll there
instead: give `report_pane` a `scroll: Option<&ScrollHandle>` parameter, call `.track_scroll` on that element, and
pass `Some(&self.detail_scroll)` here and `None` from `sync_view.rs`. One scroller, not two nested ones.

`crates/kb-app/src/search_view/footer.rs:86-87`: the row shows in the footer on the split and the detail:

```rust
let report_row = self
    .report(cx)
    .filter(|sync| sync.read(cx).has_pane() || self.navigator.top() == Place::ReportDetail)
    .and_then(|sync| sync.read(cx).selected_row(cx).cloned());
```

- [ ] **Step 5: Bind the report-only keys in the report's context.** In `crates/kb-app/src/keys.rs`, in
      `key_bindings`, the three report-only actions get `REPORT_CONTEXT` (A12):

```rust
"copy-location" => KeyBinding::new(keys, CopyLocation, Some(REPORT_CONTEXT)),
"copy-error" => KeyBinding::new(keys, CopyError, Some(REPORT_CONTEXT)),
"reveal" => KeyBinding::new(keys, RevealFile, Some(REPORT_CONTEXT)),
```

The row arrows (`NextRow`, `PreviousRow`, `LastRow`, `FirstRow`) stay in `REFRESH_CONTEXT`, which `ReportDetail` does
not have, so on the report's detail ↑↓ scroll instead (K3).

- [ ] **Step 6: Run the report tests.**

Run: `cargo nextest run -p kb-app --test ui navigates_the_report_like_the_list audit_k3_arrows_on_the_report_detail_keep_its_row copies_a_failed_file_s_error_from_the_report ignores_row_actions_on_a_report_without_rows`
Expected: 4 passed.

Run the baseline check from Task 2 Step 6. Expected: no new failure; `audit_k3_…` is gone from the failures.

- [ ] **Step 7: Commit.**

```bash
hk fix --all
git add crates/kb-app/src/sync_view.rs crates/kb-app/src/search_view.rs crates/kb-app/src/search_view/footer.rs crates/kb-app/src/keys.rs crates/kb-app/src/ui/report_pane.rs crates/kb-app/tests/ui.rs
git commit -m "feat(app): Open a report row as a place that scrolls by keyboard" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: One action path: keys act only for the top place

Every keyed command goes through one guard and `run_action`, which acts on the top place. That turns green the audit
tests for A05, A06, A07 and A08, and new tests cover A12 and A13.

**Files:**

- Modify: `crates/kb-app/src/search_view.rs`
- Test: `crates/kb-app/tests/ui.rs`

- [ ] **Step 1: Write the two new failing tests.** Add them to `crates/kb-app/tests/ui.rs`, after
      `audit_copy_path_on_a_stopped_refresh_copies_nothing`:

```rust
/// A12: the report-only keys do nothing on the list.
#[gpui_kit::test]
fn reveal_and_copy_location_do_nothing_on_the_list(cx: &mut TestAppContext) {
    let (_home, window, view) = fresh(cx);
    cx.write_to_clipboard(gpui_kit::ClipboardItem::new_string(String::new()));
    press(cx, window, "cmd-shift-l");
    press(cx, window, "cmd-shift-e");
    let copied = cx.read_from_clipboard().and_then(|item| item.text()).unwrap_or_default();
    assert_eq!(copied, "");
    assert_eq!(cx.read(|cx| view.read(cx).last_notice().map(str::to_owned)), None);
}

/// A13: → on a refresh screen leaves the list's split alone.
#[gpui_kit::test]
fn right_on_a_refresh_screen_leaves_the_list_s_split_alone(cx: &mut TestAppContext) {
    let (_home, window, view) = fresh(cx);
    press(cx, window, "cmd-r");
    press(cx, window, "escape"); // Stop
    cx.run_until_parked();
    press(cx, window, "right");
    press(cx, window, "escape"); // back to the list
    assert!(!cx.read(|cx| view.read(cx).is_showing_details()), "the list's split did not open");
}
```

- [ ] **Step 2: Put the audit tests' doc comments back on their own tests.** Each `audit_*` test from
      `audit_k3_…` onwards carries the doc comment meant for the next test. That's a cut-and-paste slip on the audit
      branch. Move each `///` line so that it sits on the test it describes. For example, "⌘R on a report reruns the
      report's own sources…" belongs on `audit_cmd_r_on_a_skills_report_refreshes_skills_again`. Comments only; no test
      body changes.

- [ ] **Step 3: Run them to see them fail.**

Run: `cargo nextest run -p kb-app --test ui reveal_and_copy_location right_on_a_refresh audit_cmd_p audit_cmd_r audit_cmd_number audit_copy_path`
Expected: `right_on_a_refresh_…`, `audit_cmd_p_…`, `audit_cmd_r_…`, `audit_cmd_number_…` and `audit_copy_path_…` FAIL.
`reveal_and_copy_location_…` already PASSES since Task 3's bindings, so keep it as the regression test for A12.

- [ ] **Step 4: One guarded entry point for keyed actions.** In `crates/kb-app/src/search_view.rs`, add:

```rust
/// A key bound to catalog action `id`: it acts on the top place through `run_action`, and not at all while an
/// overlay owns the keys (D4). The one entry point for every keyed command.
fn key_action(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
    if self.navigator.screen_has_keys() {
        self.run_action(id, window, cx);
    }
}
```

Replace the bodies of these handlers with one `key_action` call each. Nothing else stays in them:

```rust
    fn copy_path(&mut self, _: &CopyPath, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("copy-path", window, cx);
    }

    fn open_source_file(&mut self, _: &OpenSourceFile, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("edit", window, cx);
    }

    fn rebuild(&mut self, _: &Rebuild, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("refresh", window, cx);
    }

    fn copy_location(&mut self, _: &CopyLocation, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("copy-location", window, cx);
    }

    fn copy_error(&mut self, _: &CopyError, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("copy-error", window, cx);
    }

    fn reveal_file(&mut self, _: &RevealFile, window: &mut Window, cx: &mut Context<Self>) {
        self.key_action("reveal", window, cx);
    }
```

- [ ] **Step 5: `run_action` acts on the top place.** Restructure `run_action` so that the top place, not the
      phase, chooses the branch (A08). On `Refresh` and `ReportDetail`, a row action acts on the selected report row,
      and with no row (running, stopped or failed, or an empty report) it does nothing. `refresh` reruns the refresh's
      own sources in every phase (A06). The entry branch serves only `List` and `Detail`. The common actions (`refresh-yaml`,
      `refresh-skills`, `configure`, `resize`, `back`, `stop`, `retry`) are shared by every place.

```rust
/// Runs catalog action `id` on the top place (D4): the selected entry on the list and its detail, the selected
/// report row on a refresh and its detail. A row action with nothing to act on does nothing.
fn run_action(&mut self, action: &str, window: &mut Window, cx: &mut Context<Self>) {
    if matches!(self.navigator.top(), Place::Refresh | Place::ReportDetail) {
        let row = self.report(cx).and_then(|sync| sync.read(cx).selected_row(cx).cloned());
        match (action, row) {
            ("details", Some(_)) => return self.push(Place::ReportDetail, window, cx),
            // … the existing "edit", "copy-location", "copy-error", "copy-path", "reveal" arms, unchanged …
            ("refresh" | "retry", _) => {
                let selection = self.sync.as_ref().map_or(Selection::All, |sync| sync.read(cx).selection());
                return self.refresh(selection, window, cx);
            }
            ("details" | "edit" | "copy-location" | "copy-error" | "copy-path" | "reveal" | "open" | "copy", _) => {
                return;
            }
            _ => {}
        }
    }
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
        ("refresh", _) => self.refresh(Selection::All, window, cx),
        ("refresh-yaml", _) => self.refresh(Selection::Yaml, window, cx),
        ("refresh-skills", _) => self.refresh(Selection::Skills, window, cx),
        ("configure", _) => self.open_preferences(&OpenPreferences, window, cx),
        ("resize", _) => self.toggle_window_size(&ToggleWindowSize, window, cx),
        ("edit", _) => {
            if let Some(path) = self.selected_source_file(cx) {
                self.open_in_editor(std::path::Path::new(&path), None, None, window, cx);
            }
        }
        ("copy-path", _) => {
            if let Some(path) = self.selected_source_file(cx) {
                self.copy(path, window, cx);
            }
        }
        ("stop", _) => {
            if let Some(sync) = &self.sync {
                sync.read(cx).cancel();
            }
        }
        ("back", _) => self.back(&Back, window, cx),
        _ => {}
    }
}
```

The `"edit"` and `"copy-path"` arms now hold the list's own code. Before, `open_source_file` and `copy_path` called
`run_action`, which called them back; that loop is gone because the key handlers only call `key_action`. Delete the
old `("retry", _)` arm from the entry branch: `retry` exists only on refresh screens, which the first branch handles.

- [ ] **Step 6: Guard the remaining screen handlers by the place they belong to.**

```rust
    fn run_row(&mut self, n: usize, window: &mut Window, cx: &mut Context<Self>) {
        if !self.navigator.screen_has_keys() {
            return; // A07: ⌘n under the action panel or the filter
        }
        match self.navigator.top() {
            Place::Detail => {
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
            }
            Place::List => {
                if self.list.read(cx).delegate().entries().len() >= n {
                    self.list.update(cx, |list, cx| {
                        list.set_selected_index(Some(IndexPath::new(n - 1)), window, cx);
                    });
                    self.run_entry(false, window, cx);
                }
            }
            Place::Refresh | Place::ReportDetail => {}
        }
    }

    /// → (`open`) or ← on the place that has a pane: the list's split, or the report's (A13).
    fn set_pane(&mut self, open: bool, cx: &mut Context<Self>) {
        if !self.navigator.screen_has_keys() {
            return;
        }
        match self.navigator.top() {
            Place::List => self.show_details = open,
            Place::Refresh => {
                if let Some(sync) = self.report(cx) {
                    sync.update(cx, |s, cx| s.go(if open { Move::Pane } else { Move::NoPane }, cx));
                }
            }
            Place::Detail | Place::ReportDetail => {}
        }
        cx.notify();
    }

    fn show_details(&mut self, _: &ShowDetails, _: &mut Window, cx: &mut Context<Self>) {
        self.set_pane(true, cx);
    }

    fn hide_details(&mut self, _: &HideDetails, _: &mut Window, cx: &mut Context<Self>) {
        self.set_pane(false, cx);
    }
```

`open_filter`: the filter belongs to the list (A05). Opening is allowed only there, and closing everywhere:

```rust
fn open_filter(&mut self, _: &OpenFilter, window: &mut Window, cx: &mut Context<Self>) {
    let next = if self.is_filter_open() {
        Overlay::None
    } else if self.navigator.top() == Place::List {
        Overlay::Filter
    } else {
        return;
    };
    self.show_overlay(next, window, cx);
}
```

`run_entry` and the three `screen_*` ladder handlers already go through `ladder`, which checks
`screen_has_keys`. Add the same first line to `run_primary` and `run_secondary`:

```rust
if !self.navigator.screen_has_keys() {
    return;
}
```

- [ ] **Step 7: Run the acceptance tests and the suite.**

Run: `cargo nextest run -p kb-app --test ui reveal_and_copy_location right_on_a_refresh audit_cmd_p audit_cmd_r audit_cmd_number audit_copy_path ignores_row_actions copies_a_failed_file`
Expected: 8 passed.

Run the baseline check from Task 2 Step 6. Expected: no new failure. The failures left are exactly
`audit_k1_…`, `audit_a_new_filter_…`, `audit_k2_…`, `audit_reopening_the_filter_…` and `audit_closing_theme_…`
(batches 1, 2 and 4, which are out of scope).

- [ ] **Step 8: Commit.**

```bash
hk fix --all
git add crates/kb-app/src/search_view.rs crates/kb-app/tests/ui.rs
git commit -m "fix(app): Act on keys only for the top place or the open overlay" -m "Fixes audit A05, A06, A07, A08, A12 and A13." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: One crumb: `‹ <where back goes>` in the header and in ⌘K

**Files:**

- Create: `crates/kb-app/src/ui/crumb.rs`
- Modify: `crates/kb-app/src/ui.rs`, `crates/kb-app/src/ui/action_panel.rs:145-164`,
  `crates/kb-app/src/search_view/header.rs`
- Test: `crates/kb-app/tests/ui.rs`

- [ ] **Step 1: Update the header test and add the navigation tests (failing).** In `crates/kb-app/tests/ui.rs`,
      replace `shows_the_magnifier_on_the_list_and_only_the_chevron_on_the_detail_screen` with:

```rust
#[gpui_kit::test]
fn shows_the_query_on_the_list_and_where_back_goes_on_a_pushed_screen(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    let parts = cx.read(|cx| view.read(cx).header_parts());
    assert_eq!((parts.back, parts.query, parts.filter), (None, true, true));
    press(cx, window, "enter");
    let parts = cx.read(|cx| view.read(cx).header_parts());
    assert_eq!((parts.query, parts.filter), (false, false));
    assert_eq!(
        parts.back.map(|b| (b.text, b.mono)),
        Some(("kubectl".to_owned(), true)),
        "‹ names the search it returns to"
    );
}

#[gpui_kit::test]
fn names_search_when_back_goes_to_an_empty_query(cx: &mut TestAppContext) {
    let (_home, window, view) = fresh(cx);
    press(cx, window, "enter");
    let back = cx.read(|cx| view.read(cx).header_parts()).back.map(|b| b.text);
    assert_eq!(back.as_deref(), Some("Search"));
}

#[gpui_kit::test]
fn names_refresh_on_a_report_row_and_returns_to_its_split(cx: &mut TestAppContext) {
    use kb_app::navigator::Place;

    let (_home, window, view) = fresh(cx);
    press(cx, window, "cmd-r");
    cx.run_until_parked();
    press(cx, window, "right");
    press(cx, window, "enter");
    let back = cx.read(|cx| view.read(cx).header_parts()).back.map(|b| b.text);
    assert_eq!(back.as_deref(), Some("Refresh"));
    click(cx, window, "back-button");
    assert_eq!(cx.read(|cx| view.read(cx).place()), Place::Refresh, "‹ behaves as Esc");
    assert!(cx.read(|cx| view.read(cx).report_pane_open(cx)));
}

#[gpui_kit::test]
fn returns_from_the_detail_to_the_same_row_and_split(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    press(cx, window, "right");
    press(cx, window, "down");
    let row = cx.read(|cx| view.read(cx).list().read(cx).selected_index());
    press(cx, window, "enter");
    press(cx, window, "escape");
    assert_eq!(cx.read(|cx| view.read(cx).list().read(cx).selected_index()), row);
    assert!(cx.read(|cx| view.read(cx).is_showing_details()), "the split as it was");
}
```

The ⌘K crumb test already exists (`opens_refresh_more_and_returns_to_more_with_escape` and the `More` page tests);
they must keep passing after Step 4.

Run: `cargo nextest run -p kb-app --test ui shows_the_query_on_the_list names_search names_refresh returns_from_the_detail`
Expected: FAIL to compile (`parts.back` is a `bool`).

- [ ] **Step 2: Write the crumb.** Create `crates/kb-app/src/ui/crumb.rs`:

```rust
//! The one way kb says "back to X" (navigation spec D6): a left chevron and a name. ⌘K's submenus use it in their
//! crumb row (`‹ More`), and every pushed place uses it in the header (`‹ /k8s`); only the size differs.

use gpui_kit::component::h_flex;
use gpui_kit::{Div, ParentElement as _, SharedString, Styled as _, div, svg};

use crate::ui::design::{TextRoleExt as _, design, pt};

/// Where the crumb sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrumbSize {
    /// A submenu's crumb row in the action panel.
    Panel,
    /// The header of a pushed place.
    Header,
}

/// `‹ name`; `mono` for a query.
#[must_use]
pub fn crumb(name: impl Into<SharedString>, mono: bool, size: CrumbSize) -> Div {
    let d = design();
    let (icon, gap, role) = match size {
        CrumbSize::Panel => (
            d.components.panel.crumb_icon,
            d.spacing.panel.crumb_gap,
            &d.typography.roles.label,
        ),
        CrumbSize::Header => (d.components.header.icon, d.spacing.header.gap, &d.typography.roles.search),
    };
    let label = div().min_w_0().truncate().child(name.into());
    let label = if mono {
        label.font_family(d.typography.mono.clone()).text_color(d.colors.muted)
    } else {
        label.text_color(d.colors.faint)
    };
    h_flex()
        .min_w_0()
        .items_center()
        .gap(pt(gap))
        .text_role(role)
        .child(
            svg()
                .path("icons/chevron-left.svg")
                .size(pt(icon))
                .flex_none()
                .text_color(d.colors.faint),
        )
        .child(label)
}
```

Register it in `crates/kb-app/src/ui.rs`, alphabetically: `pub mod crumb;`.

If `d.typography.mono` is not a `SharedString` or `String` that `font_family` accepts, use what `command_card.rs`
passes to `font_family` for JetBrains Mono. Don't add a second way to name the mono font.

- [ ] **Step 3: The action panel uses it.** In `crates/kb-app/src/ui/action_panel.rs`, replace the crumb row's
      contents (the `h_flex()` inside `.when_some(page.crumb.clone(), …)`, lines 145–164) with:

```rust
.when_some(page.crumb.clone(), |panel, name| {
    panel.child(
        div()
            .h(pt(k.crumb))
            .flex()
            .items_center()
            .px(pt(s.crumb_pad_x))
            .border_b(pt(d.components.border.hairline))
            .border_color(c.edge)
            .child(crumb(name, false, CrumbSize::Panel)),
    )
})
```

Add `use crate::ui::crumb::{CrumbSize, crumb};`, and drop the `svg` import if it is unused.

- [ ] **Step 4: The header draws the back name.** In `crates/kb-app/src/search_view/header.rs`:

```rust
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
```

```rust
    /// What the header shows on the current place.
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
                    .accessibility_label("Back")
                    .tooltip("Back (Esc)")
                    .on_click(cx.listener(|this, _, window, cx| this.back(&Back, window, cx))),
            );
        }
        // … the list's header, unchanged …
    }
```

Imports: `use crate::navigator::{BackName, Place};`, `use crate::ui::crumb::{CrumbSize, crumb};`, and drop
`lead_icon` from the `crate::ui::header::{…}` import only if nothing else uses it (the list still uses
`lead_icon("search")`). The tooltip and label words are audit A19, out of scope here: keep them.

- [ ] **Step 5: Run the tests and the suite.**

Run: `cargo nextest run -p kb-app --test ui shows_the_query_on_the_list names_search names_refresh returns_from_the_detail opens_refresh_more`
Expected: 5 passed.

Run the baseline check from Task 2 Step 6. Expected: the same five failures as at the end of Task 4, and nothing new.

- [ ] **Step 6: Commit.**

```bash
hk fix --all
git add crates/kb-app/src/ui/crumb.rs crates/kb-app/src/ui.rs crates/kb-app/src/ui/action_panel.rs crates/kb-app/src/search_view/header.rs crates/kb-app/tests/ui.rs
git commit -m "feat(ui): Name where back goes with one crumb on every place" -m "Fixes audit A22 (option B of 20-navigation.html)." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Docs, shots and close-out

**Files:**

- Modify: `DESIGN.md` §4, `CONTEXT.md`, `docs/superpowers/audits/2026-10-27-mvp-ui-ux-audit.md`

- [ ] **Step 1: DESIGN.md.** In §4's component list, after **Report pane**, add one bullet. Wrap it at 120
      columns like its neighbours:

```markdown
- **Navigation** (`navigator.rs`, `ui/crumb.rs`; 20-navigation): screens are places on one stack, the list at the
  bottom; ⌘K and the filter float over the top place and own the keys while open. Every place but the list shows
  `‹` and where back goes (the query in mono, an entry's title, `Search`, `Refresh`) in the header, drawn by the same
  crumb as ⌘K's submenus. Esc and `‹` go back one place, to it as it was left. A new screen is a new place.
```

In the **Header** bullet, if it says "the back chevron alone elsewhere", change that to "`‹` and where back goes
elsewhere (Navigation)".

- [ ] **Step 2: CONTEXT.md.** Add a row after **Panel**:

```markdown
| Place | A screen on the navigation stack — the list, an entry's detail, a refresh, a report row's detail; its header names where back goes. | `crates/kb-app/src/navigator.rs` |
```

In the **Sync report** row, change "The report navigates like the list: → its pane, ↵ full screen." to "The report
navigates like the list: → its pane, ↵ a row's detail (a place)." Run `hk fix --all`, which realigns the table.

- [ ] **Step 3: The audit.** In `docs/superpowers/audits/2026-10-27-mvp-ui-ux-audit.md`, section 5, batch 3, add at
      the end of the item:

```markdown
**Done** (plan 2026-10-27-navigation-model): A04, A05, A06, A07, A08, A12, A13 and A22.
```

- [ ] **Step 4: Screenshots, by hand.** `mise run design:shots` takes over the screen and the keyboard, so leave
      it to the owner. Do not run it. Write in the hand-back that the detail and report-detail headers need a shot.

- [ ] **Step 5: Final checks.**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: no warnings.

Run: `cargo nextest run --workspace --no-fail-fast 2>&1 | grep -E "^\s+FAIL|Summary"`
Expected: exactly five failures, the out-of-scope audit tests: `audit_k1_cmd_down_and_cmd_up_jump_to_the_list_s_ends`,
`audit_a_new_filter_selects_its_first_result`, `audit_k2_keeps_the_filter_s_selected_row_in_view`,
`audit_reopening_the_filter_starts_at_the_top` and `audit_closing_theme_with_cmd_k_restores_the_theme`.

Run: `grep -rn "Screen::Search\|Screen::Detail\|Screen::Rebuild\|ReportLevel\|self\.overlay\b\|self\.screen\b" crates/kb-app/src`
Expected: no output. The old state is gone everywhere.

- [ ] **Step 6: Commit.**

```bash
hk fix --all
git add DESIGN.md CONTEXT.md docs/superpowers/audits/2026-10-27-mvp-ui-ux-audit.md
git commit -m "docs: Record the navigation model in design, context and the audit" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
