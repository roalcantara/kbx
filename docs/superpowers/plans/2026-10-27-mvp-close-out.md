# MVP close-out Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix every finding left in the MVP audit, each with one home for its concept. Add the keyboard into the
split's pane. Make `mise run gate` green again. Write, but do not run, the final verification handoff.

**Architecture:** Each batch of findings shares one cause, so each gets one function:

- `select_entry` and `select_panel_row`: a selection is always scrolled into view;
- `SearchView::nav(Nav)`: every navigation key, on every surface;
- `Navigator::pane_has_keys`: the split pane's focus;
- `show_overlay`: the only place an overlay changes;
- `apply_list_state`: the list's derived display state, out of `render`;
- `[messages]`: every word.

**Tech Stack:** Rust 2024, GPUI Kit 0.7.1 (`gpui_kit`), `cargo nextest`, hk, mise.

**Spec:** `docs/superpowers/specs/2026-10-27-mvp-close-out-design.md`. Read D1–D8 first. The approved page for D3 is
`docs/guides/design/21-pane-focus.html` (option A, the thin bar). The audit's findings, with file and line evidence,
are in `docs/superpowers/audits/2026-10-27-mvp-ui-ux-audit.md`.

**Consistency over flexibility (owner, 2026-10-27).** Every concept has one home. If a task tempts you to special-case
one screen, put the answer in the shared function instead. Never add a second way to select a row, route a key,
change an overlay or word a message.

---

## Before you start

- Read `AGENTS.md` (commit rules: `git add <paths>`, then `git commit`; Conventional Commits; sentence-case subject of
  at most 72 characters; end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`).
- Read the GPUI Kit Coding Guides, "Events, actions, and focus" and "Lists, tables, and large data", at
  `~/.claude/skills/gpui-kit/references/coding-guides.md`.
- GPUI Kit's source is at `~/.local/share/cargo/registry/src/index.crates.io-*/gpui-base-0.7.1/`. Its input binds
  `cmd-up` / `cmd-down` to `MoveToStart` / `MoveToEnd`, `pageup` / `pagedown` to `MovePageUp` / `MovePageDown`, and
  `tab` / `shift-tab` to `IndentInline` / `OutdentInline` (`src/input/base/state.rs:184-187,276-278`). Check what
  `gpui_kit::component::input` re-exports before you import any of them.
- **The baseline: five failing tests**, all in `crates/kb-app/tests/ui.rs`:
  - `audit_k1_cmd_down_and_cmd_up_jump_to_the_list_s_ends`;
  - `audit_a_new_filter_selects_its_first_result`;
  - `audit_k2_keeps_the_filter_s_selected_row_in_view`;
  - `audit_reopening_the_filter_starts_at_the_top`;
  - `audit_closing_theme_with_cmd_k_restores_the_theme`.

  Record them:

  ```bash
  cargo nextest run --workspace --no-fail-fast 2>&1 | grep -E "^\s+FAIL" | sed 's/.*kb-app::ui //' | sort > /tmp/kb-close-baseline.txt
  wc -l /tmp/kb-close-baseline.txt   # expected: 5
  ```

  "No regressions" in every task means: run the same command into `/tmp/kb-close-now.txt`, then
  `comm -13 /tmp/kb-close-baseline.txt /tmp/kb-close-now.txt` prints nothing.
- Run `hk fix --all` before each commit. Do **not** run `mise run design:shots`, the probes or `mise run show`: they
  take over the owner's screen. Task 9 writes them into the verification handoff.

## File map

| File                                                  | Change                                                                                               |
| ----------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `crates/kb-app/src/navigator.rs`                      | `Nav` enum; `pane_has_keys`; the key screen for the action panel                                     |
| `crates/kb-app/src/keys.rs`                           | the six `Nav` actions and `TogglePane`, bound once; `ScrollNotes*` / `PageNotes*` removed; A12 test  |
| `crates/kb-app/src/entry_list.rs`                     | `row_of(id)`; an `idle` selection; the empty-sources title and types                                 |
| `crates/kb-app/src/search_view.rs`                    | `select_entry`, `nav`, `page_rows`, `toggle_pane`, `apply_list_state`; `apply_filter` takes a window |
| `crates/kb-app/src/search_view/panel.rs`              | `select_panel_row`; `show_overlay` cancels a theme preview; the search field's captures              |
| `crates/kb-app/src/search_view/header.rs`             | the query field's captures call `nav` and `toggle_pane`; the back button's words                     |
| `crates/kb-app/src/search_view/footer.rs`             | _Close ⌘K_ while an overlay is open                                                                  |
| `crates/kb-app/src/search_view/theme.rs`              | `begin_theme_preview` selects through `select_panel_row`; the save notice's words                    |
| `crates/kb-app/src/ui/action_panel.rs`                | `children` / `child_index` from one walk; hover on rows                                              |
| `crates/kb-app/src/ui/row.rs`                         | `idle`: a faint bar, no glow                                                                         |
| `crates/kb-app/src/ui/report_pane.rs`                 | excerpt lines truncate                                                                               |
| `crates/kb-app/src/ui/entry_view.rs`                  | the split pane tracks the shared scroll handle and shows the focus bar                               |
| `crates/kb-app/src/sync_view.rs`, `report_list.rs`    | the split pane uses the shared scroll handle; `idle`; the sources-directory row opens Configure      |
| `crates/kb-app/src/actions.rs`                        | `more_page` adds Theme… unconditionally                                                              |
| `crates/kb-app/src/catalog.rs`, `assets/catalog.toml` | `Screen::Actions`, `[screens.actions]`, the `run` and `close` actions, the new messages              |
| `crates/kb-app/src/config.rs`, `panel.rs`, `lib.rs`   | notices from `[messages]`                                                                            |
| `crates/kb-app/tests/ui.rs`                           | the new UI tests                                                                                     |
| `DESIGN.md`, `CONTEXT.md`, the audit, the handoff     | docs                                                                                                 |

---

### Task 1: One "select and show" function for the list (A01)

**Files:**

- Modify: `crates/kb-app/src/entry_list.rs` (add `row_of`)
- Modify: `crates/kb-app/src/search_view.rs` (`select_entry`; `apply_filter`; the refresh subscription)
- Modify: `crates/kb-app/src/search_view/panel.rs` (`apply_filter`'s two calls)
- Test: `crates/kb-app/tests/ui.rs`, `crates/kb-app/src/entry_list.rs`

- [ ] **Step 1: The failing UI test for a refresh.** Add it to `crates/kb-app/tests/ui.rs`, after
      `audit_a_new_filter_selects_its_first_result`:

```rust
/// A01: a refresh keeps the selected entry when it is still listed.
#[gpui_kit::test]
fn a_refresh_keeps_the_selected_entry(cx: &mut TestAppContext) {
    let (_home, window, view) = fresh(cx);
    press(cx, window, "down");
    press(cx, window, "down");
    let before = selected_title(&view, cx);
    press(cx, window, "cmd-r");
    cx.run_until_parked();
    press(cx, window, "escape"); // back to the list
    assert_eq!(selected_title(&view, cx), before, "the same entry, not the same index");
}
```

- [ ] **Step 2: The failing unit test for `row_of`.** In `crates/kb-app/src/entry_list.rs`'s `mod tests`, using the
      module's existing helpers to build an `EntryList` with entries:

```rust
#[test]
fn finds_an_entry_s_row_by_id() {
    let list = list_with_entries(&["a#commands#x", "b#commands#y"]);
    assert_eq!(list.row_of("b#commands#y"), Some(1));
    assert_eq!(list.row_of("gone"), None);
}
```

If the module has no helper that builds an `EntryList` with given entries, add `fn list_with_entries(ids: &[&str]) ->
EntryList` to `mod tests`. It builds `EntryList::new(None)` and sets `entries` to entries made with
`kb_domain::entry::Entry { id: (*id).to_owned(), ..Entry::default() }`. If `Entry` has no `Default`, use the fixture
constructor the other tests use.

Run: `cargo nextest run -p kb-app finds_an_entry_s_row_by_id a_refresh_keeps_the_selected_entry audit_a_new_filter_selects_its_first_result`
Expected: compile error (`row_of`), then, once it is added, the two UI tests FAIL.

- [ ] **Step 3: `row_of`** in `impl EntryList`, after `selected_entry`:

```rust
/// The row of the entry with `id`, if the results list it.
#[must_use]
pub fn row_of(&self, id: &str) -> Option<usize> {
    self.entries.iter().position(|entry| entry.id == id)
}
```

- [ ] **Step 4: `select_entry`, the list's one way to select.** In `crates/kb-app/src/search_view.rs`, replace
      `move_selection` with:

```rust
    /// Selects row `row` of the list (clamped; nothing with no results) and scrolls to it (D1). The list's one way to
    /// select: every move, filter change and refresh comes through here.
    fn select_entry(&mut self, row: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.list.update(cx, |list, cx| {
            let count = list.delegate().entries().len();
            let ix = (count > 0).then(|| IndexPath::new(row.min(count - 1)));
            list.set_selected_index(ix, window, cx);
            if ix.is_some() {
                list.scroll_to_selected_item(window, cx);
            }
        });
        cx.notify();
    }

    /// The list's selected row, if any.
    fn selected_row(&self, cx: &App) -> Option<usize> {
        self.list.read(cx).selected_index().map(|ix| ix.row)
    }
```

Task 3 replaces the old `move_selection(delta)` callers with `nav`. Until then, keep them compiling with this wrapper,
which Task 3 deletes:

```rust
fn move_selection(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
    let row = self.selected_row(cx).unwrap_or(0).saturating_add_signed(delta);
    self.select_entry(row, window, cx);
}
```

- [ ] **Step 5: A filter change selects row 0.** `apply_filter` gains a window, and its two callers in
      `search_view/panel.rs` (`self.apply_filter(only, cx)` and `self.apply_filter(toggle, cx)`) pass theirs:

```rust
fn apply_filter(&mut self, filter: FilterState, window: &mut Window, cx: &mut Context<Self>) {
    self.list.update(cx, |list, cx| {
        list.delegate_mut().set_filter(filter);
        cx.notify();
    });
    self.select_entry(0, window, cx);
}
```

`filter_by_tag` passes its `window`.

- [ ] **Step 6: A refresh keeps the entry.** In `refresh`'s `SyncFinished` subscription, replace the
      `if finished.changed { … }` block with:

```rust
if finished.changed {
    let kept = this.list.read(cx).delegate().selected_entry().map(|e| e.id.clone());
    let connection = open_readonly(this.config.database()).ok();
    this.list.update(cx, |list, cx| {
        list.delegate_mut().set_connection(connection);
        cx.notify();
    });
    let row = kept.and_then(|id| this.list.read(cx).delegate().row_of(&id)).unwrap_or(0);
    this.select_entry(row, window, cx);
}
```

- [ ] **Step 7: Run.**

Run: `cargo nextest run -p kb-app finds_an_entry_s_row_by_id a_refresh_keeps_the_selected_entry audit_a_new_filter_selects_its_first_result`
Expected: 3 passed. Then run the regression check from "Before you start": `audit_a_new_filter_…` leaves the list.

- [ ] **Step 8: Commit.**

```bash
hk fix --all
git add crates/kb-app/src/entry_list.rs crates/kb-app/src/search_view.rs crates/kb-app/src/search_view/panel.rs crates/kb-app/tests/ui.rs
git commit -m "fix(app): Select through one function that keeps the row in view" -m "Fixes audit A01." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: One "select and show" function for the panel (A02, A09)

**Files:**

- Modify: `crates/kb-app/src/ui/action_panel.rs` (`PanelChild`, `children`, `child_index`; `panel()` draws from it)
- Modify: `crates/kb-app/src/search_view/panel.rs`, `crates/kb-app/src/search_view/theme.rs`
- Test: `crates/kb-app/src/ui/action_panel.rs`

- [ ] **Step 1: The failing unit test.** In `crates/kb-app/src/ui/action_panel.rs`, add a `#[cfg(test)] mod tests` (or
      extend the existing one):

```rust
#[test]
fn counts_the_labels_and_hairlines_above_a_row() {
    let mut page = PanelPage::default();
    page.sections = vec![
        PanelSection { label: Some("Types".into()), rows: vec![PanelRow::default(); 2] },
        PanelSection { label: Some("Tags".into()), rows: vec![PanelRow::default(); 2] },
    ];
    let visible = [(0, 0), (0, 1), (1, 0), (1, 1)];
    // Types label, row 0, row 1, hairline, Tags label, row 2, row 3
    assert_eq!(child_index(&page, &visible, 0), 1);
    assert_eq!(child_index(&page, &visible, 1), 2);
    assert_eq!(child_index(&page, &visible, 2), 5);
    assert_eq!(child_index(&page, &visible, 3), 6);
}
```

If `PanelPage` / `PanelSection` have no public constructor or `Default`, build them the way `actions.rs` tests do. The
field names are `sections`, `label` and `rows` (`action_panel.rs:121-140`).

Run: `cargo nextest run -p kb-app counts_the_labels_and_hairlines_above_a_row`
Expected: compile error, `child_index` not found.

- [ ] **Step 2: One walk, used to draw and to count.** In `crates/kb-app/src/ui/action_panel.rs`, add above `panel`:

```rust
/// One child of the panel's scroll container, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PanelChild {
    /// The rule between two sections.
    Hairline,
    /// A section's label.
    Label(usize),
    /// Visible row `ix`, of section `section`, row `row`.
    Row { ix: usize, section: usize, row: usize },
}

/// The scroll container's children for `visible`: the one walk `panel()` draws and `child_index` counts, so the two
/// cannot drift (close-out D1).
fn children(page: &PanelPage, visible: &[(usize, usize)]) -> Vec<PanelChild> {
    let mut out = Vec::new();
    let mut shown_section = None;
    for (ix, &(section, row)) in visible.iter().enumerate() {
        if shown_section != Some(section) {
            if shown_section.is_some() {
                out.push(PanelChild::Hairline);
            }
            if page.sections[section].label.is_some() {
                out.push(PanelChild::Label(section));
            }
            shown_section = Some(section);
        }
        out.push(PanelChild::Row { ix, section, row });
    }
    out
}

/// The scroll container's child index of visible row `ix`, for `ScrollHandle::scroll_to_item`.
#[must_use]
pub fn child_index(page: &PanelPage, visible: &[(usize, usize)], ix: usize) -> usize {
    children(page, visible)
        .iter()
        .position(|child| matches!(child, PanelChild::Row { ix: i, .. } if *i == ix))
        .unwrap_or(0)
}
```

Replace `panel()`'s loop (the `let mut shown_section = None; for … { … }` block) with:

```rust
for child in children(page, visible) {
    list = match child {
        PanelChild::Hairline => list.child(hairline().mx(pt(s.rule_margin)).my(pt(s.rule_margin))),
        PanelChild::Label(section) => list.child(
            div()
                .pt(pt(s.section_pad_top))
                .px(pt(s.section_pad_x))
                .pb(pt(s.section_pad_bottom))
                .text_role(&d.typography.roles.section)
                .text_color(c.faint)
                .child(page.sections[section].label.clone().unwrap_or_default().to_uppercase()),
        ),
        PanelChild::Row { ix, section, row: r } => {
            list.child(row(&page.sections[section].rows[r], ix, ix == selected, on_row))
        }
    };
}
```

- [ ] **Step 3: `select_panel_row`, the only writer of `panel_selected`.** In
      `crates/kb-app/src/search_view/panel.rs`, add:

```rust
/// Selects visible row `ix` of the open page and scrolls it into view (close-out D1). Row 0 scrolls to the very
/// top, so its section label shows too. The one writer of `panel_selected`.
pub(super) fn select_panel_row(&mut self, ix: usize, cx: &mut Context<Self>) {
    let page = self.current_page(cx);
    let visible = matches(&page.page, &self.panel_query.read(cx).value());
    self.panel_selected = ix.min(visible.len().saturating_sub(1));
    if self.panel_selected == 0 {
        self.panel_scroll.set_offset(gpui_kit::Point::default());
    } else {
        self.panel_scroll
            .scroll_to_item(crate::ui::action_panel::child_index(&page.page, &visible, self.panel_selected));
    }
    self.preview_selected_theme(cx);
    cx.notify();
}
```

Replace every other write of `panel_selected` with a call to it:

- `show_overlay`: `self.panel_selected = 0;` becomes `self.select_panel_row(0, cx);`. Move the call **after**
  `self.navigator.set_overlay(overlay)`, the `begin_theme_preview` call and the search reset, so it counts the new
  page's rows.
- `set_panel_query`: `self.panel_selected = 0;` becomes `self.select_panel_row(0, cx);`, after the `set_value`.
- `on_panel_query_event`: `self.panel_selected = 0;` becomes `self.select_panel_row(0, cx);`.
- `move_panel`:

```rust
fn move_panel(&mut self, delta: isize, cx: &mut Context<Self>) {
    self.select_panel_row(self.panel_selected.saturating_add_signed(delta), cx);
}
```

- `confirm`'s Toggle branch: replace `self.panel_selected = matches(…).position(…).unwrap_or(0);` with
  `let ix = matches(…).position(…).unwrap_or(0); self.select_panel_row(ix, cx);`.
- `search_view/theme.rs` `begin_theme_preview`: it can no longer set the row itself, because the page is not open yet.
  Make it return the row, and let `show_overlay` select it:

```rust
/// Opening Theme…: remembers the theme in use, to restore it; the row of that theme, to select.
pub(super) fn begin_theme_preview(&mut self) -> usize {
    self.theme_preview = Some(crate::theme::active_palette());
    self.themes.iter().position(|t| t.current).unwrap_or(0)
}
```

In `show_overlay`: `let first = if overlay == Overlay::Actions(Some(Submenu::Theme)) { self.begin_theme_preview() }
else { 0 };`, then `self.select_panel_row(first, cx);` at the end, as above.

Check: `grep -n "panel_selected =" crates/kb-app/src` prints only the line in `select_panel_row`.

- [ ] **Step 4: Run.**

Run: `cargo nextest run -p kb-app counts_the_labels audit_k2_keeps_the_filter_s_selected_row_in_view audit_reopening_the_filter_starts_at_the_top scrolls_selected_panel_row moves_one_row_per_arrow`
Expected: all pass. Then the regression check; `audit_k2_…` and `audit_reopening_…` leave the list.

- [ ] **Step 5: Commit.**

```bash
hk fix --all
git add crates/kb-app/src/ui/action_panel.rs crates/kb-app/src/search_view/panel.rs crates/kb-app/src/search_view/theme.rs
git commit -m "fix(ui): Scroll the panel to its selected row's own child" -m "Fixes audit A02 and A09." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: One navigation-key model (A03)

**Files:**

- Modify: `crates/kb-app/src/navigator.rs` (`Nav`)
- Modify: `crates/kb-app/src/keys.rs` (actions and bindings)
- Modify: `crates/kb-app/src/search_view.rs` (`nav`, `page_rows`; the old handlers go)
- Modify: `crates/kb-app/src/search_view/header.rs`, `crates/kb-app/src/search_view/panel.rs` (captures)
- Test: `crates/kb-app/tests/ui.rs`

- [ ] **Step 1: The failing UI tests.** Add them to `crates/kb-app/tests/ui.rs`:

```rust
/// Close-out D2: ⌘↑↓ and Page Up/Down on the filter.
#[gpui_kit::test]
fn cmd_arrows_reach_the_filter_s_ends(cx: &mut TestAppContext) {
    let (_home, window, view) = fresh(cx);
    open_filter(cx, window);
    let rows: usize = cx.read(|cx| view.read(cx).panel_labels(cx)).iter().map(Vec::len).sum();
    press(cx, window, "cmd-down");
    assert_eq!(cx.read(|cx| view.read(cx).panel_selected()), rows - 1);
    press(cx, window, "cmd-up");
    assert_eq!(cx.read(|cx| view.read(cx).panel_selected()), 0);
    press(cx, window, "pagedown");
    assert!(cx.read(|cx| view.read(cx).panel_selected()) > 1, "a page is more than one row");
}

/// Close-out D2: ⌘↑↓ on a submenu, which has no search field.
#[gpui_kit::test]
fn cmd_arrows_reach_a_submenu_s_ends(cx: &mut TestAppContext) {
    let (_home, window, view) = fresh(cx);
    run_from_panel(cx, window, "More");
    press(cx, window, "cmd-down");
    let last = cx.read(|cx| view.read(cx).panel_labels(cx)).iter().map(Vec::len).sum::<usize>() - 1;
    assert_eq!(cx.read(|cx| view.read(cx).panel_selected()), last);
}

/// Close-out D2: ⌘↓ on the report selects its last row.
#[gpui_kit::test]
fn cmd_down_reaches_the_report_s_last_row(cx: &mut TestAppContext) {
    let (_home, window, view) = fresh(cx);
    press(cx, window, "cmd-r");
    cx.run_until_parked();
    let first = cx.read(|cx| view.read(cx).report_row_path(cx));
    press(cx, window, "cmd-down");
    assert_ne!(cx.read(|cx| view.read(cx).report_row_path(cx)), first);
}
```

Add the accessor they need to `search_view/panel.rs`:

```rust
/// The panel's selected visible row, for tests.
#[must_use]
pub fn panel_selected(&self) -> usize {
    self.panel_selected
}
```

Run: `cargo nextest run -p kb-app --test ui cmd_arrows_reach cmd_down_reaches audit_k1`
Expected: `cmd_arrows_reach_the_filter_s_ends`, `cmd_arrows_reach_a_submenu_s_ends` and `audit_k1_…` FAIL.
`cmd_down_reaches_the_report_s_last_row` already passes: keep it as a regression test.

- [ ] **Step 2: `Nav`.** In `crates/kb-app/src/navigator.rs`:

```rust
/// A navigation key, the same on every surface (close-out D2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nav {
    /// ↑: a row up, or a line up.
    Up,
    /// ↓.
    Down,
    /// ⌘↑: the first row, or the top.
    First,
    /// ⌘↓: the last row, or the bottom.
    Last,
    /// Page Up.
    PageUp,
    /// Page Down.
    PageDown,
}
```

- [ ] **Step 3: Bind the six keys once.** In `crates/kb-app/src/keys.rs`:

1. In `actions!`, remove `ScrollNotesDown`, `ScrollNotesUp`, `PageNotesDown` and `PageNotesUp`. Keep `NextRow`,
   `PreviousRow`, `LastRow` and `FirstRow`, and add `PageUp` and `PageDown` (each with a `///` line, as their
   neighbours have).
2. In `key_bindings`, delete the four `REFRESH_CONTEXT` row bindings and the four `DETAIL_CONTEXT` scroll bindings, and
   the `up` / `down` → `MoveUp` / `MoveDown` bindings in `PANEL_CONTEXT`. Add:

```rust
// One navigation-key model (close-out D2): the same six keys on every surface; `SearchView::nav` sends each
// to its owner. The text fields capture them first (`search_view/header.rs`, `panel.rs`).
KeyBinding::new("up", PreviousRow, context),
KeyBinding::new("down", NextRow, context),
KeyBinding::new("cmd-up", FirstRow, context),
KeyBinding::new("cmd-down", LastRow, context),
KeyBinding::new("pageup", PageUp, context),
KeyBinding::new("pagedown", PageDown, context),
```

3. Update `binds_the_catalog_keys_and_the_fixed_ones`' count: 42 − 4 − 4 − 2 + 6 = **38**.

- [ ] **Step 4: `nav` and `page_rows`.** In `crates/kb-app/src/search_view.rs`, add:

```rust
    /// Whole rows that fit `visible` points of rows `row` points tall; at least one (close-out D2).
    fn page_rows(visible: f32, row: f32) -> usize {
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a few dozen rows")]
        let rows = (visible / row).floor() as usize;
        rows.max(1)
    }

    /// The step `key` makes over `count` rows from `current`, `page` rows to a page.
    fn step(key: Nav, current: usize, count: usize, page: usize) -> usize {
        let last = count.saturating_sub(1);
        match key {
            Nav::Up => current.saturating_sub(1),
            Nav::Down => (current + 1).min(last),
            Nav::First => 0,
            Nav::Last => last,
            Nav::PageUp => current.saturating_sub(page),
            Nav::PageDown => (current + page).min(last),
        }
    }

    /// Every navigation key, on every surface (close-out D2): the open overlay's rows, a scrolling detail or pane,
    /// the report's rows, or the list. The one place a navigation key is routed.
    fn nav(&mut self, key: Nav, window: &mut Window, cx: &mut Context<Self>) {
        let d = design();
        if !self.navigator.screen_has_keys() {
            let page = Self::page_rows(d.components.panel.max_height, d.components.panel.row);
            let count = matches(&self.current_page(cx).page, &self.panel_query.read(cx).value()).len();
            return self.select_panel_row(Self::step(key, self.panel_selected, count, page), cx);
        }
        let scrolls = matches!(self.navigator.top(), Place::Detail | Place::ReportDetail) || self.navigator.pane_has_keys();
        if scrolls {
            return self.scroll(key, cx);
        }
        let rows_height = f32::from(window.viewport_size().height) - d.components.header.height - d.components.footer.height;
        let page = Self::page_rows(rows_height, d.components.row.height);
        match self.navigator.top() {
            Place::Refresh => {
                if let Some(sync) = self.report(cx) {
                    sync.update(cx, |s, cx| s.step_selection(key, page, window, cx));
                    self.detail_scroll.set_offset(gpui_kit::Point::default());
                }
            }
            Place::List => {
                let count = self.list.read(cx).delegate().entries().len();
                let row = Self::step(key, self.selected_row(cx).unwrap_or(0), count, page);
                self.select_entry(row, window, cx);
            }
            Place::Detail | Place::ReportDetail => {}
        }
    }

    /// A scrolling detail or pane: ↑↓ a line, ⌘↑↓ its ends, Page Up/Down a page.
    fn scroll(&mut self, key: Nav, cx: &mut Context<Self>) {
        let max = self.detail_scroll.max_offset().y;
        let offset = self.detail_scroll.offset();
        let y = match key {
            Nav::Up => next_offset(offset.y, -notes_line_step(), max),
            Nav::Down => next_offset(offset.y, notes_line_step(), max),
            Nav::First => pixels(0.),
            Nav::Last => -max,
            Nav::PageUp => next_offset(offset.y, -self.notes_page(), max),
            Nav::PageDown => next_offset(offset.y, self.notes_page(), max),
        };
        self.detail_scroll.set_offset(point(offset.x, y));
        cx.notify();
    }
```

`navigator.pane_has_keys()` comes in Task 4. Until then, add it to `Navigator` returning `false`:
`pub fn pane_has_keys(&self) -> bool { false }`. Task 4 replaces it.

Delete `move_selection` (Task 1's wrapper), `report_step`, `scroll_notes`, `scroll_notes_down`, `scroll_notes_up`,
`page_notes_down` and `page_notes_up`. Keep `notes_page` and `next_offset`. In `render`, replace their `.on_action`
lines and the four `report_step` lines with:

```rust
.on_action(cx.listener(|this, _: &PreviousRow, w, cx| this.nav(Nav::Up, w, cx)))
.on_action(cx.listener(|this, _: &NextRow, w, cx| this.nav(Nav::Down, w, cx)))
.on_action(cx.listener(|this, _: &FirstRow, w, cx| this.nav(Nav::First, w, cx)))
.on_action(cx.listener(|this, _: &LastRow, w, cx| this.nav(Nav::Last, w, cx)))
.on_action(cx.listener(|this, _: &PageUp, w, cx| this.nav(Nav::PageUp, w, cx)))
.on_action(cx.listener(|this, _: &PageDown, w, cx| this.nav(Nav::PageDown, w, cx)))
```

In `crates/kb-app/src/sync_view.rs`, replace `move_selection(delta, …)` with:

```rust
/// The report's rows under `key` (close-out D2), `page` rows to a page; scrolls to the new row.
pub fn step_selection(&mut self, key: crate::navigator::Nav, page: usize, window: &mut Window, cx: &mut Context<Self>) {
    use crate::navigator::Nav;
    let Some(list) = &self.report else { return };
    list.update(cx, |list, cx| {
        let Some(from) = list.delegate().selected() else { return };
        let counts = list.delegate().counts();
        let delta = match key {
            Nav::Up => -1,
            Nav::Down => 1,
            Nav::First => isize::MIN,
            Nav::Last => isize::MAX,
            Nav::PageUp => -isize::try_from(page).unwrap_or(1),
            Nav::PageDown => isize::try_from(page).unwrap_or(1),
        };
        let to = step(from, delta, &counts);
        list.set_selected_index(Some(to), window, cx);
        list.scroll_to_selected_item(window, cx);
    });
    cx.notify();
}
```

`report_list::step` already clamps `isize::MIN` / `MAX` to the ends and skips the section labels.

- [ ] **Step 5: The text fields capture the same keys, through one helper.** The query field and ⌘K's search field
      bind the six keys to their caret, so both capture them first and call `nav`. Write the captures once, in
      `crates/kb-app/src/search_view/header.rs`:

```rust
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
    }
}
```

In the header, replace `query_box()` and its two `MoveUp` / `MoveDown` captures with
`Self::nav_captures(query_box(), cx)`. Keep the `MoveRight` / `MoveLeft` captures after it.

In `search_view/panel.rs` `render_panel`, replace the search field's `MoveUp` / `MoveDown` captures with
`Self::nav_captures(<the search field's div>, cx)`. Delete the panel's own `.on_action(… MoveUp …)` and
`.on_action(… MoveDown …)`: the `Search` bindings reach a submenu through `nav`.

Import `MoveToStart, MoveToEnd, MovePageUp, MovePageDown` from `gpui_kit::component::input` (check its re-exports
first), plus `crate::navigator::Nav`. In `search_view.rs`, `nav` needs `use crate::ui::action_panel::matches;`.

- [ ] **Step 6: Run.**

Run: `cargo nextest run -p kb-app --test ui cmd_arrows_reach cmd_down_reaches audit_k1 audit_k3 moves_one_row_per_arrow scrolls`
Expected: all pass. Then `cargo nextest run -p kb-app --lib` and the regression check: `audit_k1_…` leaves the list.

- [ ] **Step 7: Commit.**

```bash
hk fix --all
git add crates/kb-app/src/navigator.rs crates/kb-app/src/keys.rs crates/kb-app/src/search_view.rs crates/kb-app/src/search_view crates/kb-app/src/sync_view.rs crates/kb-app/tests/ui.rs
git commit -m "feat(app): Route every navigation key through one function" -m "Fixes audit A03." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: The keyboard into the split's pane (D3, A17)

**Files:**

- Modify: `crates/kb-app/src/navigator.rs` (`pane_has_keys`, cleared by push and pop; the context)
- Modify: `crates/kb-app/src/keys.rs` (`TogglePane`, `tab` / `shift-tab`)
- Modify: `crates/kb-app/src/search_view.rs` (`toggle_pane`; Esc; the split pane's scroll handle; selection resets)
- Modify: `crates/kb-app/src/search_view/header.rs` (capture ⇥ / ⇧⇥)
- Modify: `crates/kb-app/src/ui/entry_view.rs`, `crates/kb-app/src/ui/report_pane.rs`, `crates/kb-app/src/sync_view.rs`
  (the shared handle and the bar)
- Modify: `crates/kb-app/src/ui/row.rs`, `crates/kb-app/src/entry_list.rs`, `crates/kb-app/src/report_list.rs`
  (`idle`)
- Test: `crates/kb-app/src/navigator.rs`, `crates/kb-app/tests/ui.rs`

- [ ] **Step 1: The failing tests.** In `navigator.rs` tests:

```rust
#[test]
fn gives_the_pane_the_keys_until_the_place_changes() {
    let mut nav = Navigator::default();
    nav.set_pane_has_keys(true);
    assert!(nav.pane_has_keys());
    assert_eq!(nav.key_context(), "Search Detail", "the pane scrolls as a detail does");
    nav.push(Place::Detail, name("/k8s"));
    assert!(!nav.pane_has_keys(), "a push takes the keys back");
    nav.pop();
    nav.push(Place::Refresh, name("Search"));
    nav.set_pane_has_keys(true);
    assert_eq!(nav.key_context(), "Search Detail Report");
    nav.pop();
    assert!(!nav.pane_has_keys(), "so does a pop");
}
```

In `tests/ui.rs`:

```rust
/// Close-out D3: ⇥ moves the keys into the split's pane; Esc goes back one level at a time.
#[gpui_kit::test]
fn tab_moves_the_keys_into_the_pane_and_esc_brings_them_back(cx: &mut TestAppContext) {
    let (_home, window, view) = fresh(cx);
    press(cx, window, "right");
    let row = selected_title(&view, cx);
    press(cx, window, "tab");
    assert!(cx.read(|cx| view.read(cx).pane_has_keys()));
    press(cx, window, "down");
    assert_eq!(selected_title(&view, cx), row, "↓ scrolls the pane, not the list");
    press(cx, window, "tab");
    assert!(!cx.read(|cx| view.read(cx).pane_has_keys()), "⇥ again: back to the list");
    press(cx, window, "tab");
    press(cx, window, "escape");
    assert!(!cx.read(|cx| view.read(cx).pane_has_keys()), "Esc: back to the list");
    assert!(cx.read(|cx| view.read(cx).is_showing_details()), "the split stays open");
}

/// Close-out D3: ⇥ with the split closed does nothing.
#[gpui_kit::test]
fn tab_without_a_split_does_nothing(cx: &mut TestAppContext) {
    let (_home, window, view) = fresh(cx);
    press(cx, window, "tab");
    assert!(!cx.read(|cx| view.read(cx).pane_has_keys()));
}
```

Add `pub fn pane_has_keys(&self) -> bool { self.navigator.pane_has_keys() }` to `SearchView` for tests.

Run: `cargo nextest run -p kb-app gives_the_pane_the_keys tab_moves tab_without`
Expected: FAIL / compile error.

- [ ] **Step 2: The navigator owns the pane's focus.** In `navigator.rs`, add a `pane: bool` field (default `false`)
      and replace Task 3's stub:

```rust
    /// Whether the keys are in the split's pane (close-out D3).
    #[must_use]
    pub fn pane_has_keys(&self) -> bool {
        self.pane
    }

    /// Moves the keys into the split's pane, or back to its list.
    pub fn set_pane_has_keys(&mut self, pane: bool) {
        self.pane = pane;
    }
```

In `push`, `pop` and `pop_to_list`, add `self.pane = false;` as the first line. In `key_context`, before the `match`:

```rust
// A pane with the keys scrolls as the detail screens do, with their bindings.
if self.pane {
    return match self.top() {
        Place::Refresh => [CONTEXT, DETAIL_CONTEXT, REPORT_CONTEXT].join(" "),
        _ => [CONTEXT, DETAIL_CONTEXT].join(" "),
    };
}
```

- [ ] **Step 3: ⇥ and ⇧⇥.** In `keys.rs`, add `TogglePane` to `actions!` (`/// Move the keys between a split's list
      and its pane.`), and bind it:

```rust
// ⇥ / ⇧⇥: the keys between a split's list and its pane (close-out D3); the query field captures them first.
KeyBinding::new("tab", TogglePane, context),
KeyBinding::new("shift-tab", TogglePane, context),
```

Add 2 to the binding count (40).

In `search_view.rs`:

```rust
/// ⇥ / ⇧⇥ on a split: the keys to its pane, or back to its list (close-out D3). Nothing without a split.
fn toggle_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) {
    if !self.navigator.screen_has_keys() {
        return;
    }
    let split = match self.navigator.top() {
        Place::List => self.show_details && self.list.read(cx).delegate().selected_entry().is_some(),
        Place::Refresh => self.report(cx).is_some_and(|sync| sync.read(cx).has_pane()),
        Place::Detail | Place::ReportDetail => false,
    };
    if !split {
        return;
    }
    let pane = !self.navigator.pane_has_keys();
    self.navigator.set_pane_has_keys(pane);
    self.apply_list_state(cx);
    self.focus_screen(window, cx);
    cx.notify();
}
```

`focus_screen`: give the keys to the view while the pane has them, so typing does not reach the query field:

```rust
fn focus_screen(&self, window: &mut Window, cx: &mut Context<Self>) {
    if self.navigator.top() == Place::List && !self.navigator.pane_has_keys() {
        self.query_input.update(cx, |input, cx| input.focus(window, cx));
    } else {
        self.focus.focus(window, cx);
    }
}
```

In `render`, add `.on_action(cx.listener(|this, _: &TogglePane, w, cx| this.toggle_pane(w, cx)))`. In
`search_view/header.rs`'s `nav_captures`, add the ⇥ capture (it belongs with the caret keys, since the query field
binds ⇥ to indent):

```rust
.capture_action(cx.listener(|this, _: &IndentInline, w, cx| { this.toggle_pane(w, cx); cx.stop_propagation(); }))
.capture_action(cx.listener(|this, _: &OutdentInline, w, cx| { this.toggle_pane(w, cx); cx.stop_propagation(); }))
```

If `gpui_kit::component::input` does not re-export `IndentInline` / `OutdentInline`, check whether the single-line
query field binds ⇥ at all. Press it in the test. If ⇥ already reaches `TogglePane`, drop these two captures.

- [ ] **Step 4: Esc, and the place the pane's focus ends.** In `back`, after the overlay step, add:

```rust
// The pane: Esc gives the keys back to its list (close-out D3).
if self.navigator.pane_has_keys() {
    self.navigator.set_pane_has_keys(false);
    self.apply_list_state(cx);
    self.focus_screen(window, cx);
    cx.notify();
    return;
}
```

The focus also ends when the selection changes or the split closes. In `select_entry` (Task 1), and in `set_pane` when
`open` is `false`, add:

```rust
self.navigator.set_pane_has_keys(false);
self.detail_scroll.set_offset(gpui_kit::Point::default());
```

`nav`'s `Place::Refresh` branch already resets `detail_scroll` after moving a row.

- [ ] **Step 5: One scroll handle for every pane (A17).** The split's pane draws with `detail_scroll`, like the
      detail screens, and shows the thin bar while it has the keys:

- `render_content`'s `Place::List` split arm: `self.entry_view(entry, Layout::Split, window, cx)` gains
  `.track_scroll(self.detail_scroll.clone()).focused(self.navigator.pane_has_keys())`.
- `ui/entry_view.rs`: add a `focused: bool` field (default `false`) with a builder `pub fn focused(mut self, focused:
  bool) -> Self`. Where the view's root is built, add the bar:

```rust
// Close-out D3: the pane has the keys — a 1 pt mauve bar on its left edge, no glow (21-pane-focus, A).
.when(self.focused, |el| el.border_l(pt(d.components.border.hairline)).border_color(d.colors.mauve))
```

- The report's split: `SyncView::start` takes the shared handle, `scroll: ScrollHandle`, and stores it. `refresh`
  passes `self.detail_scroll.clone()`. Its split arm draws
  `report_pane(&row, m, false, Some(&self.scroll))`. `SyncView` gets `pub fn set_focused(&mut self, focused: bool)`,
  which stores the flag and notifies. The split arm wraps the pane in
  `div().when(self.focused, |el| el.border_l(pt(d.components.border.hairline)).border_color(c.mauve))`.
- The bar is the same two lines in two places. Put them in one helper, `crate::ui::design::focus_bar(el: Div, focused:
  bool) -> Div`, and call it from both.

- [ ] **Step 6: The list's row goes faint while the pane has the keys.** In `ui/row.rs`, add `idle: bool` to `Row`
      (default `false`) with `pub fn idle(mut self, idle: bool) -> Self`. In `render`'s selected branch, the bar becomes:

```rust
.bg(if self.idle { colors.faint } else { colors.mauve })
.when(!self.idle, |bar| bar.shadow(glow(colors.mauve, size.glow, size.glow_alpha))),
```

`EntryList` and `ReportList` each get an `idle: bool` with `set_idle(bool)`. Their row builders call
`.idle(self.idle)` on the selected row.

- [ ] **Step 7: `apply_list_state`, the list's derived state out of `render` (A16).** In `search_view.rs`:

```rust
/// The list's display state, derived from the place and the split: compact rows beside an open pane, and a faint
/// selection while the pane has the keys. Called where they change (close-out D7, A16), never from `render`.
fn apply_list_state(&mut self, cx: &mut Context<Self>) {
    let on_list = self.navigator.top() == Place::List;
    let compact = on_list && self.show_details && self.list.read(cx).delegate().selected_entry().is_some();
    let idle = self.navigator.pane_has_keys();
    self.list.update(cx, |list, cx| {
        list.delegate_mut().set_compact(compact);
        list.delegate_mut().set_idle(idle && on_list);
        cx.notify();
    });
    if let Some(sync) = self.sync.clone() {
        sync.update(cx, |s, cx| s.set_idle_and_focus(idle, cx));
    }
}
```

`SyncView::set_idle_and_focus(idle, cx)` sets its `ReportList`'s idle and its own `focused`, both to `idle`. Call
`apply_list_state` from `set_pane`, `push`, `settle`, `toggle_pane`, `back`'s pane step and `select_entry`. In
`render`, delete the `let compact = …; if … set_compact …` lines.

- [ ] **Step 8: Run.**

Run: `cargo nextest run -p kb-app gives_the_pane_the_keys tab_moves tab_without navigates_the_report opens_on_the_list_alone`
Expected: all pass. Then the whole of `-p kb-app` and the regression check.

- [ ] **Step 9: Commit.**

```bash
hk fix --all
git add crates/kb-app/src crates/kb-app/tests/ui.rs
git commit -m "feat(app): Move the keys into a split's pane with tab" -m "Close-out D3 (21-pane-focus, thin bar); fixes audit A16 and A17." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: The overlay lifecycle (A10, A11, A15)

**Files:**

- Modify: `crates/kb-app/src/search_view/panel.rs` (`show_overlay`)
- Modify: `crates/kb-app/src/catalog.rs`, `assets/catalog.toml` (`Screen::Actions`, `[screens.actions]`, `run`, `close`)
- Modify: `crates/kb-app/src/navigator.rs` (`key_screen`), `crates/kb-app/src/search_view/footer.rs`
- Modify: `crates/kb-app/src/actions.rs` (`more_page`)
- Test: `crates/kb-app/tests/ui.rs`, `crates/kb-app/src/actions.rs`, `crates/kb-app/src/navigator.rs`

- [ ] **Step 1: The failing tests.** In `tests/ui.rs`:

```rust
/// A10: under ⌘K, the footer shows the panel's own keys, and ⌘K closes it.
#[gpui_kit::test]
fn shows_the_panel_s_keys_in_the_footer(cx: &mut TestAppContext) {
    let (_home, window, view) = fresh(cx);
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    assert_eq!(cx.read(|cx| view.read(cx).footer_keys(cx)), ["Run ↵", "Back esc", "Close ⌘K"]);
    type_text(cx, window, "More");
    cx.run_until_parked();
    press(cx, window, "enter");
    cx.run_until_parked();
    assert_eq!(cx.read(|cx| view.read(cx).footer_keys(cx)), ["Run ↵", "Back esc", "Close ⌘K"], "a submenu too");
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    assert!(!cx.read(|cx| view.read(cx).is_actions_open()), "⌘K closes the panel from a submenu");
}
```

In `actions.rs` tests:

```rust
#[test]
fn offers_theme_without_refresh_more() {
    let (mut catalog, _) = Catalog::load(&toml::Table::new());
    catalog.panel.more_refresh.clear();
    let labels: Vec<String> = more_page(&catalog, None).page.sections.iter().flat_map(|s| s.rows.iter().map(|r| r.label.clone())).collect();
    assert!(labels.contains(&catalog.panel.theme), "{labels:?}");
}
```

In `navigator.rs` tests, extend `maps_each_place_and_overlay_to_its_key_screen`:

```rust
nav.set_overlay(Overlay::Actions(None));
assert_eq!(nav.key_screen(KeyScreen::List, None), KeyScreen::Actions);
nav.set_overlay(Overlay::Actions(Some(Submenu::More)));
assert_eq!(nav.key_screen(KeyScreen::List, None), KeyScreen::Actions);
```

Run: `cargo nextest run -p kb-app shows_the_panel_s_keys offers_theme_without maps_each_place audit_closing_theme`
Expected: FAIL or compile error.

- [ ] **Step 2: The catalog's panel screen and its two words.** In `assets/catalog.toml`, add to `[actions]`:

```toml
[actions.run]
label = "Run"

[actions.close]
label = "Close"
```

and, after `[screens.links]`:

```toml
# While ⌘K is open (close-out D5): ↵ runs the selected row; ⌘K, shown as Close, shuts the panel.
[screens.actions]
enter = "run"
escape = "back"
```

In `catalog.rs`:

- add `"run"` and `"close"` to `ACTION_IDS` (27 entries);
- add `/// ⌘K is open (the root or a submenu other than Links…). Actions,` to `pub enum Screen`;
- add `pub actions: ScreenKeys,` to `Screens`;
- add `Screen::Actions => &s.actions,` to `Catalog::screen`.

Follow the compiler for any other exhaustive `match` on `Screen`.

- [ ] **Step 3: The navigator maps it.** In `Navigator::key_screen`, extend the overlay match:

```rust
match self.overlay {
    Overlay::Filter => return KeyScreen::Filter,
    Overlay::Actions(Some(Submenu::Links)) => return KeyScreen::Links,
    Overlay::Actions(_) => return KeyScreen::Actions,
    Overlay::None => {}
}
```

- [ ] **Step 4: The footer says Close.** In `search_view/footer.rs`, both the drawn footer and `footer_keys` take the
      trailing item's label from one function:

```rust
/// The trailing ⌘K item's label: Actions, or Close while an overlay is open (close-out D5).
fn actions_label(&self) -> &str {
    if self.navigator.screen_has_keys() {
        self.catalog.label("actions")
    } else {
        self.catalog.label("close")
    }
}
```

Replace each `self.catalog.label("actions")` in `footer.rs` with `self.actions_label()`.

- [ ] **Step 5: `show_overlay` cancels a preview (A11).** In `search_view/panel.rs`'s `show_overlay`, first line:

```rust
// Leaving Theme… any way but ↵ restores the theme in use (close-out D4); `keep_theme` clears the preview first.
if overlay != Overlay::Actions(Some(Submenu::Theme)) {
    self.cancel_theme_preview(cx);
}
```

Then remove the now-redundant `self.cancel_theme_preview(cx)` from `back`'s overlay step in `search_view.rs`, because
`back` reaches `show_overlay`.

- [ ] **Step 6: Theme… stands on its own (A15).** In `actions.rs` `more_page`, move the Theme… `admin.push((…
      Submenu::Theme …))` out of the `if id == "refresh" && …` block, to after the `for` loop:

```rust
admin.push((
    PanelRow {
        label: catalog.panel.theme.clone(),
        submenu: true,
        ..PanelRow::default()
    },
    PanelCommand::Open(Submenu::Theme),
));
```

Then fix the expected row order in any `actions.rs` or `ui.rs` test that lists More…'s rows: Theme… now comes after
Configure, at the end. For example, `clears_the_search_then_leaves_more_then_closes_with_escape` and
`opens_refresh_more_and_returns_to_more_with_escape` expect `["Edit", "Refresh", "Refresh More…", "Theme…",
"Configure"]`; that becomes `["Edit", "Refresh", "Refresh More…", "Configure", "Theme…"]`. Check the catalog's
`[panel]` comment (`catalog.toml`) and update it to say Theme… ends the admin rows.

- [ ] **Step 7: Run.**

Run: `cargo nextest run -p kb-app shows_the_panel_s_keys offers_theme_without maps_each_place audit_closing_theme opens_refresh_more clears_the_search`
Expected: all pass. Then the regression check: the baseline is now empty.

- [ ] **Step 8: Commit.**

```bash
hk fix --all
git add crates/kb-app/src crates/kb-app/assets/catalog.toml crates/kb-app/tests/ui.rs
git commit -m "fix(app): Change overlays in one place, with their own footer keys" -m "Fixes audit A10, A11 and A15." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Report details (A14, A18)

**Files:**

- Modify: `crates/kb-app/src/report_list.rs` (`configures`), `assets/catalog.toml` (`report_source_failed`)
- Modify: `crates/kb-app/src/ui/report_pane.rs` (excerpt lines)
- Test: `crates/kb-app/src/report_list.rs`, `crates/kb-app/src/actions.rs`

- [ ] **Step 1: The failing test.** In `report_list.rs` tests:

```rust
#[test]
fn opens_configure_from_an_unreadable_sources_directory() {
    let row = ReportRow {
        path: "/s".into(),
        kind: ReportKind::Failed,
        entries: 0,
        detail: "No such directory.".into(),
        subject: Subject::SourcesDir("/s".into()),
    };
    assert!(row.configures(), "A14: Edit opens config.toml");
}
```

Run: `cargo nextest run -p kb-app opens_configure_from_an_unreadable`
Expected: FAIL.

- [ ] **Step 2: Fix it.** In `ReportRow::configures`:

```rust
matches!(
    self.subject,
    Subject::SourcesDir(_) | Subject::Skills(SkillsOutcome::Failed { .. } | SkillsOutcome::NotConfigured)
)
```

Update its doc comment to name the sources directory too. In `catalog.toml`, change `report_source_failed` to
`"{error} The previous entries are unchanged. Check it in Configure."`. If a test asserts the old sentence, update it.

- [ ] **Step 3: Excerpt lines truncate (A18).** In `ui/report_pane.rs`, the excerpt line's text cell becomes:

```rust
.child(div().flex_1().min_w_0().truncate().child(line.text.clone()))
```

and its `h_flex()` row gains `.min_w_0()`.

- [ ] **Step 4: Run, then commit.**

Run: `cargo nextest run -p kb-app report_list report_pane actions`
Expected: all pass.

```bash
hk fix --all
git add crates/kb-app/src/report_list.rs crates/kb-app/src/ui/report_pane.rs crates/kb-app/assets/catalog.toml crates/kb-app/src
git commit -m "fix(app): Open Configure from an unreadable sources directory" -m "Fixes audit A14 and A18." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Words, the empty title, hover, and A12's guard (A19, A20, A21, A12)

**Files:**

- Modify: `crates/kb-app/assets/catalog.toml`, `crates/kb-app/src/catalog.rs` (`Messages`)
- Modify: `crates/kb-app/src/search_view.rs`, `search_view/theme.rs`, `search_view/header.rs`, `panel.rs`, `lib.rs`,
  `config.rs`
- Modify: `crates/kb-app/src/entry_list.rs` (`render_empty`), `crates/kb-app/src/ui/action_panel.rs` (`row`)
- Test: `crates/kb-app/src/keys.rs`, `crates/kb-app/src/entry_list.rs`

- [ ] **Step 1: A12's real guard (failing first, if the bindings regress).** In `keys.rs` tests:

```rust
#[test]
fn binds_the_report_only_actions_in_the_report_alone() {
    let catalog = Catalog::default();
    for (id, action) in [("copy-location", "CopyLocation"), ("copy-error", "CopyError"), ("reveal", "RevealFile")] {
        let keys = catalog.actions[id].keys.clone().unwrap();
        let binding = key_bindings(&catalog)
            .into_iter()
            .find(|b| b.action().name().ends_with(action))
            .unwrap_or_else(|| panic!("{id} is bound"));
        assert_eq!(
            binding.predicate().map(|p| p.to_string()).as_deref(),
            Some(REPORT_CONTEXT),
            "{id} ({keys}) acts only on the report"
        );
    }
}
```

`KeyBinding::predicate()` and `Action::name()` are GPUI's. Check their exact names in the `gpui` crate under the cargo
registry: `grep -n "pub fn predicate\|pub fn action(" …/gpui*/src/keymap/binding.rs`. If the predicate does not print
as `"Report"`, compare with the predicate parsed from `REPORT_CONTEXT` instead.

Run: `cargo nextest run -p kb-app binds_the_report_only_actions` — Expected: PASS (it guards Task 3 of the navigation
plan). Break it once by moving `"reveal"` back to `context`, see it fail, then restore it.

- [ ] **Step 2: The words move to `[messages]` (A19).** Add to `catalog.toml`'s `[messages]`, and as `String` fields
      of `Messages`, each with a `///` line:

```toml
config_create_failed = "Couldn’t create {file}: {error}"
theme_save_failed = "Couldn’t save the theme in {file}: {error}"
hotkey_failed = "Couldn’t register ⌥K: {error}. Another app may be using it."
themes_seed_failed = "Couldn’t write kb’s themes to {dir}: {error}"
config_colors_ignored = "Ignored `colors` in {file}: not a table"
config_unknown_ignored = "Ignored the unknown setting `{key}` in {file}"
config_editor_ignored = "Ignored `editor` in {file}: {error}"
```

Replace each `format!(…)` at these call sites with `fill(&m.<name>, &[("file", …), ("error", …)])`, using the
same placeholders:

- `search_view.rs` `configure` (`Couldn’t create`): use `self.catalog.messages`;
- `search_view/theme.rs` `keep_theme` (`Couldn’t save the theme`): use `self.catalog.messages`;
- `panel.rs:85` (`Couldn’t register ⌥K`): use the `catalog` loaded on line 69;
- `lib.rs:89` (themes seed) and `config.rs:118,123,165` (the three config notices): these run before the catalog
  loads, so they use the shipped catalog's words: `let m = crate::catalog::Catalog::default().messages;`. Add that
  rule as a comment where `m` is built.

In `search_view/header.rs`, the back button: `.accessibility_label(self.catalog.label("back").to_owned())`, and
`.tooltip(format!("{} ({})", self.catalog.label("back"), caps("escape")))`.

Check: `grep -rn 'format!("Couldn\|format!("Ignored\|"Back' crates/kb-app/src` prints nothing.

- [ ] **Step 3: The empty-sources title (A20).** Add to `[messages]`:

```toml
no_entries_title = "No entries yet"
no_entries = "Your sources have no {types}."
```

(Replace the old `no_entries` sentence.) Add `no_entries_title` to `Messages`. In `entry_list.rs` `render_empty`, the
"no query, no filter" branch becomes:

```rust
EmptyView::new(
    "search",
    d.colors.faint,
    m.no_entries_title.clone(),
    Sentence::of(&fill(&m.no_entries, &[("types", &self.type_list())]), ""),
)
```

with, in `impl EntryList`:

```rust
/// The catalog's type labels, lowercased, as a sentence's list: "bookmarks, cheat sheets, commands, tasks or skills".
fn type_list(&self) -> String {
    let labels: Vec<String> = self.catalog.types.values().map(|t| t.label.to_lowercase()).collect();
    match labels.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} {} {last}", rest.join(", "), self.catalog.messages.list_or),
        Some((last, _)) => last.clone(),
        None => String::new(),
    }
}
```

Use the catalog's actual field names for the type table and its label (`catalog.rs`, `EntryType`), and keep the
catalog's own order. The joining word "or" is a word kb shows, so it goes in `[messages]` too
(`list_or = "or"`), and `type_list` reads it from there. Add a unit test that `type_list()` on the default catalog gives
"bookmarks, cheat sheets, commands, tasks or skills".

- [ ] **Step 4: Hover on panel rows (A21).** In `ui/action_panel.rs` `row()`, on the row's element, where it applies
      the selected fill:

```rust
.when(!selected, |el| el.hover(|style| style.bg(c.hover)))
```

matching `ui/row.rs:140`.

- [ ] **Step 5: Run, then commit.**

Run: `cargo nextest run --workspace`
Expected: all pass.

```bash
hk fix --all
git add crates/kb-app/src crates/kb-app/assets/catalog.toml
git commit -m "fix(app): Take every notice's words from the catalog" -m "Fixes audit A19, A20 and A21; guards A12 at the binding." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Docs

**Files:**

- Modify: `DESIGN.md` §4, `CONTEXT.md`, `docs/superpowers/audits/2026-10-27-mvp-ui-ux-audit.md`

- [ ] **Step 1: DESIGN.md.** In the **Navigation** bullet, add after its first sentence (wrap at 120 columns):

```markdown
Every list answers the same keys — ↑↓ a row, ⌘↑↓ the first and last, Page Up/Down a page — and the detail screens
scroll with them. On a split, ⇥ moves the keys into the pane, which shows a 1 pt mauve bar on its left edge while the
list's selection bar goes faint (21-pane-focus); ⇥, ⇧⇥ or Esc give them back.
```

In the **Action panel** bullet, add: "rows take the hover fill; the footer shows _Run ↵ · Back esc · Close ⌘K_ while it
is open."

- [ ] **Step 2: CONTEXT.md.** In the **Screen keys** row, after `the catalog's [screens.*]`, add "; `[screens.actions]`
      while ⌘K is open". Run `hk fix --all`, which realigns the table.

- [ ] **Step 3: The audit.** In section 5, end each of batches 1, 2, 4, 5 and 6 with
      `**Done** (plan 2026-10-27-mvp-close-out).`, and update section 1's verdict to say every finding is fixed.

- [ ] **Step 4: Commit.**

```bash
hk fix --all
git add DESIGN.md CONTEXT.md docs/superpowers/audits/2026-10-27-mvp-ui-ux-audit.md
git commit -m "docs: Record the close-out's keys, pane focus and fixes" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: Write the final verification handoff (do not run it)

**Files:**

- Create: `docs/superpowers/handoffs/2026-10-27-mvp-final-verification.md`

- [ ] **Step 1: Write the handoff** with exactly this content:

```markdown
# Handoff: the MVP's final verification

Run this after plan `docs/superpowers/plans/2026-10-27-mvp-close-out.md` lands. It holds what that plan left out on
purpose: the long checks and the ones that take over the screen. Report each result. Fix nothing here: a failure
becomes a TODO with its evidence.

## 1. Gates

- `mise run gate`: it must pass. The audit's reproductions are green now.
- `mise run size`: within `tools/size/budget`.

## 2. Screenshots (takes over the screen and the keyboard; tell the owner before starting)

`mise run design:shots`, then compare each shot in `tools/design/shots/`, compact and expanded, with its approved page
in `docs/guides/design/` (render them with `mise run design:prototypes`). New since the last shots:

- the `‹ <query>` and `‹ Refresh` headers (20-navigation);
- the report's split and detail (12-report-*);
- a split with the keys in the pane (21-pane-focus: the 1 pt bar, the list's faint bar);
- the footer under ⌘K (_Run ↵ · Back esc · Close ⌘K_).

Add any of these that `tools/design/tasks.toml` does not capture yet.

## 3. By hand (`mise run app`, against the sample or showcase corpus, never the private one)

- [ ] Long titles and descriptions are cut off with an ellipsis, in compact and expanded.
- [ ] Tag images draw at their size beside `#tag` pills.
- [ ] Hover: list rows, ⌘K rows and filter rows; the selected row shows no hover.
- [ ] Focus: the pane's bar after ⇥; Esc's levels: pane → list → clear the query → hide.
- [ ] Hide by focus loss (click another app), reopen with ⌥K: the same place, query and selection.
- [ ] Every navigation key on the list, filter, ⌘K and a submenu, Theme…, the report, and both detail screens.
- [ ] Each of the eight themes: More… → Theme…, preview with ↑↓, Esc restores, ↵ keeps; text is legible.
- [ ] VoiceOver (⌘F5): rows read their labels; the back button reads its label.

## 4. Bookkeeping

- [ ] Every finding in `docs/superpowers/audits/2026-10-27-mvp-ui-ux-audit.md` is marked fixed with its commit.
- [ ] TODO.md: 2YnZ and the report-detail scrolling task are closed (`todo status`), each with a comment naming
      its plan.
- [ ] `docs/guides/parity.md` and `docs/guides/compliance.md` reflect the navigation model and the close-out.
- [ ] The verdict: the MVP is closed, or the list of what still blocks it.
```

- [ ] **Step 2: Commit.**

```bash
hk fix --all
git add docs/superpowers/handoffs/2026-10-27-mvp-final-verification.md
git commit -m "docs(handoff): Write the MVP's final verification pass" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

- [ ] **Step 3: Final checks.**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: no warnings.

Run: `cargo nextest run --workspace --no-fail-fast 2>&1 | grep -E "FAIL|Summary"`
Expected: no failures.

Hand back with:

- the commits;
- any deviation from this plan, and why;
- the note that the handoff in Task 9 is written and not run.
