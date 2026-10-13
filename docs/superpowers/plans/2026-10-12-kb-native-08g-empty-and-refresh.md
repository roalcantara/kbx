# Plan 8g: The empty view, the refresh screens, and no literal sizes

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** the no-match and no-index states are `03-empty.html`'s empty view (D10): one 56 pt icon, a title, a sentence
with the query in mauve mono, footer _Refresh ↵_. The refresh (sync) screens are `11-14-refresh.html` (D16): running,
stopped and failed in the same empty view (running adds a thin progress bar and _n of m files · file_), the report as
the list's rows (one per file, failed first, then skipped, then indexed — R5), the header's chevron only, and each
screen's keys from the catalog (_Stop esc_, _Back ↵_, _Retry ↵_, _Retry ↵ · Configure ⌘↵_). Then the code the new
design replaced goes, and an ast-grep rule keeps literal sizes and colours inside the design module.

**Architecture:** `ui/empty_view.rs` is one component for six states. `SyncView` keeps the sync and draws its states
with `EmptyView` and `Row`; it reports its phase so `SearchView::key_screen` picks the right catalog keys. `EntryList`
draws its empty states with `EmptyView`.

**Tech Stack:** Rust, GPUI Kit 0.7.1, ast-grep 0.45.3.

**Expects** (verified at `c13a09b` plus Plan 8f): `ui::{design, row::Row, tile, chip::pill, footer::{status, Dot}}`;
tokens `components.empty.{icon, text_max_width, bar_width, bar_height}`, `spacing.empty.*`,
`typography.roles.{empty_title, empty_text}`; catalog messages `no_match_title`, `no_match`, `no_match_filtered`,
`no_entries`, `no_index_title`, `no_index`, `refreshing_title`, `refreshing_progress`, `report_summary`,
`report_failed`, `report_skipped`, `report_indexed`, `report_skipped_entries`, `report_empty_file`, `stopped_title`,
`stopped`, `failed_title`, `failed`, `refreshing_status`; catalog screens `empty`, `no-index`, `refreshing`, `report`,
`stopped`, `failed`; `SearchView::{key_screen, run_action}`.

**Index:** [2026-10-12-kb-native-08-design-system.md](2026-10-12-kb-native-08-design-system.md).

---

## File structure

| File                                                                                            | Change | Responsibility                                               |
| ----------------------------------------------------------------------------------------------- | ------ | ------------------------------------------------------------ |
| `crates/kb-app/src/ui/empty_view.rs`                                                            | create | `EmptyView`, `Sentence`                                      |
| `crates/kb-app/src/entry_list.rs`                                                               | modify | empty and no-index states                                    |
| `crates/kb-app/src/sync_view.rs`                                                                | modify | the four refresh screens; `Phase`; report rows               |
| `crates/kb-app/src/search_view.rs`                                                              | modify | `key_screen` for empty, no index and refresh phases; actions |
| `crates/kb-app/src/search_view/footer.rs`                                                       | modify | the status line while refreshing                             |
| `crates/kb-app/src/window_size.rs`, `lib.rs`, `panel.rs`                                        | modify | sizes from the tokens                                        |
| `crates/kb-app/src/accents.rs`                                                                  | delete | replaced by the tokens (8a)                                  |
| `crates/kb-app/src/presentation.rs`, `filter.rs`                                                | modify | dead helpers removed                                         |
| `tools/ast-grep/rules/no-literal-design.yml`, `tools/ast-grep/tests/no-literal-design-test.yml` | create | the rule                                                     |
| `DESIGN.md`, `crates/kb-app/src/ui/design.rs`                                                   | modify | `components.detail.scroll_step`; no `dead_code` expectation  |
| `crates/kb-app/tests/ui.rs`                                                                     | modify | empty and refresh tests                                      |

---

### Task 1: The empty view component

**Files:** create `crates/kb-app/src/ui/empty_view.rs`; modify `crates/kb-app/src/ui.rs`.

- [ ] **Step 1: Write the failing test.** Create the file:

```rust
//! D10's empty view (03-empty, 11-14-refresh): one large icon, a title, a sentence whose query
//! (if any) reads in mauve mono, and an optional extra (the refresh's progress bar). Six states
//! use it: no match, no entries, no index, refreshing, stopped, failed.

/// A sentence with an optional highlighted part: `before`, `query`, `after`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    /// The text before the query.
    pub before: String,
    /// The query, drawn in mauve mono; empty for none.
    pub query: String,
    /// The text after it.
    pub after: String,
}

impl Sentence {
    /// `template` with `{query}` replaced by `query`, split around it; without `{query}`, plain.
    #[must_use]
    pub fn of(template: &str, query: &str) -> Self {
        let _ = (template, query);
        Self { before: String::new(), query: String::new(), after: String::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_a_message_around_its_query() {
        assert_eq!(
            Sentence::of("No entries match “{query}”.", "/terraform-plan"),
            Sentence { before: "No entries match “".into(), query: "/terraform-plan".into(), after: "”.".into() }
        );
        assert_eq!(
            Sentence::of("The previous index is unchanged.", "x"),
            Sentence { before: "The previous index is unchanged.".into(), query: String::new(), after: String::new() }
        );
    }
}
```

Add `pub mod empty_view;` to `ui.rs` (sorted, after `pub mod design;`).

Run: `cargo nextest run -p kb-app empty_view`
Expected: FAIL.

- [ ] **Step 2: Write `Sentence::of` and the component.** Replace `of`'s body with:

```rust
match template.split_once("{query}") {
    Some((before, after)) => Self { before: before.to_owned(), query: query.to_owned(), after: after.to_owned() },
    None => Self { before: template.to_owned(), query: String::new(), after: String::new() },
}
```

and append, before the tests module:

```rust
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{AnyElement, App, Hsla, IntoElement, ParentElement as _, RenderOnce, Styled as _, Window, div, svg};

use crate::ui::design::{TextRoleExt as _, design, no_ligatures, pt};

/// The empty view.
#[derive(IntoElement)]
pub struct EmptyView {
    icon: String,
    icon_color: Hsla,
    title: String,
    sentence: Sentence,
    extra: Option<AnyElement>,
}

impl EmptyView {
    /// `icon` (a Lucide name) in `icon_color`, `title`, and `sentence`.
    #[must_use]
    pub fn new(icon: &str, icon_color: Hsla, title: impl Into<String>, sentence: Sentence) -> Self {
        Self { icon: icon.to_owned(), icon_color, title: title.into(), sentence, extra: None }
    }

    /// Something under the sentence: the refresh's progress bar.
    #[must_use]
    pub fn extra(mut self, extra: impl IntoElement) -> Self {
        self.extra = Some(extra.into_any_element());
        self
    }
}

impl RenderOnce for EmptyView {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let d = design();
        let (c, s, k) = (&d.colors, &d.spacing.empty, &d.components.empty);
        let sentence = self.sentence;
        v_flex()
            .size_full()
            .flex_1()
            .items_center()
            .justify_center()
            .px(pt(s.pad_x))
            .child(svg().path(format!("icons/{}.svg", self.icon)).size(pt(k.icon)).text_color(self.icon_color).mb(pt(s.icon_gap)))
            .child(div().text_role(&d.typography.roles.empty_title).text_color(c.text).mb(pt(s.title_gap)).child(self.title))
            .child(
                h_flex()
                    .flex_wrap()
                    .justify_center()
                    .max_w(pt(k.text_max_width))
                    .text_role(&d.typography.roles.empty_text)
                    .text_color(c.muted)
                    .child(sentence.before)
                    .when(!sentence.query.is_empty(), |line| {
                        line.child(div().font_family(d.typography.mono.clone()).font_features(no_ligatures()).text_color(c.mauve).child(sentence.query))
                    })
                    .child(sentence.after),
            )
            .when_some(self.extra, |view, extra| view.child(div().mt(pt(s.bar_gap)).child(extra)))
    }
}

/// The refresh's thin progress bar, `fraction` (0–1) filled.
#[must_use]
pub fn progress_bar(fraction: f32) -> impl IntoElement {
    let d = design();
    let k = &d.components.empty;
    div()
        .w(pt(k.bar_width))
        .h(pt(k.bar_height))
        .rounded_full()
        .bg(d.colors.selected)
        .overflow_hidden()
        .child(div().h_full().w(pt(k.bar_width * fraction.clamp(0., 1.))).bg(d.colors.blue))
}
```

Run: `cargo nextest run -p kb-app empty_view`
Expected: PASS.

- [ ] **Step 3: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/src/ui.rs crates/kb-app/src/ui/empty_view.rs
git commit -m "feat(ui): Add the empty view" -m "One icon, a title, a sentence with the query in mauve mono (D10)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: No match, no entries, no index

**Files:** modify `crates/kb-app/src/entry_list.rs`, `crates/kb-app/src/search_view.rs`, `crates/kb-app/tests/ui.rs`.

- [ ] **Step 1: Write the failing UI test.** Append to `tests/ui.rs`:

```rust
#[gpui_kit::test]
fn offers_refresh_when_nothing_matches(cx: &mut TestAppContext) {
    let (_home, _window, view) = open_with_query(cx, "zzqxj");
    assert!(titles(&view, cx).is_empty());
    let keys = cx.read(|cx| view.read(cx).footer_keys(cx));
    assert_eq!(keys, ["Refresh ↵", "Actions ⌘K"]);
    assert_eq!(cx.read(|cx| view.read(cx).status_line(cx)), "Ready • 0 results");
}
```

Run: `cargo nextest run -p kb-app --test ui offers_refresh_when_nothing_matches`
Expected: FAIL: the footer shows the list's keys.

- [ ] **Step 2: The empty states in the list.** In `entry_list.rs`, replace `render_empty`'s body with:

```rust
let d = design();
let m = &self.catalog.messages;
let view = if !self.has_index() {
    EmptyView::new("database", d.colors.faint, m.no_index_title.clone(), Sentence::of(&m.no_index, ""))
} else if self.query.trim().is_empty() && self.filter == FilterState::default() {
    EmptyView::new("search", d.colors.faint, m.no_match_title.clone(), Sentence::of(&m.no_entries, ""))
} else {
    let template = if self.filter == FilterState::default() { &m.no_match } else { &m.no_match_filtered };
    EmptyView::new("search", d.colors.faint, m.no_match_title.clone(), Sentence::of(template, self.query.trim()))
};
let _ = cx;
view.into_any_element()
```

and the imports `use crate::ui::empty_view::{EmptyView, Sentence};`; remove the now-unused
`use gpui_kit::component::empty::{…}` line. (`EmptyView` draws icons `database` and `search`: add `Database` to the
`icon_assets!` list in `assets.rs` if `loads_every_icon_the_app_names` or the app shows a blank icon; `search` is in
GPUI Kit's default set.)

- [ ] **Step 3: The keys of those states.** In `search_view.rs`, `key_screen`'s `Screen::Search` arm becomes:

```rust
Screen::Search => {
    let list = self.list.read(cx).delegate();
    if !list.has_index() {
        KeyScreen::NoIndex
    } else if list.entries().is_empty() {
        KeyScreen::Empty
    } else {
        KeyScreen::List
    }
}
```

`key_screen` now needs the app: change its signature to `fn key_screen(&self, cx: &App) -> KeyScreen` and pass `cx`
at its call sites (`screen_keys`, and `current_page` in `search_view/panel.rs`).

- [ ] **Step 4: Run every kb-app test.**

Run: `cargo nextest run -p kb-app`
Expected: PASS.

- [ ] **Step 5: Gate and commit.**

```bash
mise run gate
git add crates/kb-app
git commit -m "feat(ui): Show the empty view for no match and no index" \
  -m "Footer Refresh ↵ (D10); the catalog's empty and no-index keys." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: The four refresh screens

**Files:** modify `crates/kb-app/src/sync_view.rs`, `crates/kb-app/src/search_view.rs`,
`crates/kb-app/src/search_view/footer.rs`, `crates/kb-app/tests/ui.rs`.

- [ ] **Step 1: Write the failing tests.** In `sync_view.rs`'s `tests` module, add:

```rust
    use indexmap::IndexMap;
    use kb_domain::report::{FileIssue, FileReport};
    use kb_domain::source::SourceError;

    fn file(rel: &str, entries: usize, error: bool, skipped: bool) -> FileReport {
        FileReport {
            rel_path: rel.into(),
            file_path: format!("/s/{rel}"),
            entries,
            types: IndexMap::new(),
            error: error.then(|| SourceError {
                file_path: format!("/s/{rel}"),
                message: "Line 4: a flow sequence must be closed".into(),
                line: Some(4),
                column: Some(1),
            }),
            excerpt: None,
            issues: if skipped {
                vec![FileIssue::SkippedEntry { entry_type: "bookmarks".into(), key: "not-a-map".into() }]
            } else {
                Vec::new()
            },
        }
    }

    #[test]
    fn lists_every_file_failed_first_then_skipped_then_indexed() {
        let files = vec![file("b.yml", 9, false, true), file("a.yml", 12, false, false), file("errors/x.yml", 0, true, false)];
        let order: Vec<(String, ReportKind)> = report_rows(&files).into_iter().map(|r| (r.path, r.kind)).collect();
        assert_eq!(
            order,
            [
                ("errors/x.yml".into(), ReportKind::Failed),
                ("b.yml".into(), ReportKind::Skipped),
                ("a.yml".into(), ReportKind::Indexed)
            ]
        );
    }
```

(`indexmap` is a kb-domain dependency: add `indexmap.workspace = true` to kb-app's `[dev-dependencies]`. If
`SourceError` has more fields than these four, add them; the struct is in `crates/kb-domain/src/source.rs`.)

Append to `tests/ui.rs`:

```rust
#[gpui_kit::test]
fn reports_a_refresh_and_goes_back_with_return(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    press(cx, window, "cmd-r");
    cx.run_until_parked();
    let keys = cx.read(|cx| view.read(cx).footer_keys(cx));
    assert_eq!(keys, ["Back ↵", "Actions ⌘K"], "the report's keys once the sync finished");
    assert_eq!(cx.read(|cx| view.read(cx).header_parts()).back, true);
    press(cx, window, "enter");
    assert!(!cx.read(|cx| view.read(cx).is_rebuilding()), "Back ↵ returns to the list");
}
```

Run: `cargo nextest run -p kb-app`
Expected: FAIL to compile (`report_rows`, `ReportKind`).

- [ ] **Step 2: The phase and the report rows.** In `sync_view.rs`, add:

```rust
/// Which refresh screen shows (D16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Files are being read.
    Running,
    /// Finished: the report.
    Report,
    /// Stopped by Esc; the previous index is unchanged.
    Stopped,
    /// Failed; the previous index is unchanged.
    Failed,
}

/// What a report row says about its file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportKind {
    /// The file failed to parse.
    Failed,
    /// Some of its entries were skipped.
    Skipped,
    /// Indexed in full.
    Indexed,
}

/// One report row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportRow {
    /// The file, relative to the sources directory.
    pub path: String,
    /// What happened.
    pub kind: ReportKind,
    /// The error's first line, the skipped entries, or how many it indexed.
    pub entries: usize,
    /// For a failed file, its error; for a skipped one, the skipped entries.
    pub detail: String,
}

/// Every file, failed first, then those with skipped entries, then the indexed ones (R5).
#[must_use]
pub fn report_rows(files: &[FileReport]) -> Vec<ReportRow> {
    let mut rows: Vec<ReportRow> = files
        .iter()
        .map(|file| {
            let (kind, detail) = if let Some(error) = &file.error {
                (ReportKind::Failed, error.message.lines().next().unwrap_or_default().to_owned())
            } else if !file.issues.is_empty() {
                (ReportKind::Skipped, issues_line(file))
            } else {
                (ReportKind::Indexed, String::new())
            };
            ReportRow { path: file.rel_path.clone(), kind, entries: file.entries, detail }
        })
        .collect();
    rows.sort_by_key(|row| match row.kind {
        ReportKind::Failed => 0,
        ReportKind::Skipped => 1,
        ReportKind::Indexed => 2,
    });
    rows
}
```

and to `impl SyncView`:

```rust
/// The screen to show.
#[must_use]
pub fn phase(&self) -> Phase {
    match self.status {
        SyncStatus::Running { .. } => Phase::Running,
        SyncStatus::Done(_) => Phase::Report,
        SyncStatus::Cancelled => Phase::Stopped,
        SyncStatus::Failed(_) => Phase::Failed,
    }
}
```

Delete `report_lines` (replaced by `report_rows`).

- [ ] **Step 3: Draw the four screens.** `SyncView` needs the catalog's words: add a field `catalog: Rc<Catalog>`,
      take it in `start(sources, database, catalog: Rc<Catalog>, cx)`, and pass `self.catalog.clone()` from
      `SearchView::rebuild`. Replace `impl Render for SyncView` with:

```rust
impl Render for SyncView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let d = design();
        let (c, m) = (&d.colors, &self.catalog.messages);
        match &self.status {
            SyncStatus::Running { files_scanned, files_total, current, .. } => {
                #[expect(clippy::cast_precision_loss, reason = "a fraction of a few thousand files")]
                let fraction = if *files_total == 0 { 0. } else { *files_scanned as f32 / *files_total as f32 };
                let progress = fill(&m.refreshing_progress, &[("done", &files_scanned.to_string()), ("total", &files_total.to_string()), ("file", current)]);
                EmptyView::new("refresh-cw", c.faint, m.refreshing_title.clone(), Sentence::of(&progress, ""))
                    .extra(progress_bar(fraction))
                    .into_any_element()
            }
            SyncStatus::Cancelled => {
                EmptyView::new("circle-stop", c.yellow, m.stopped_title.clone(), Sentence::of(&m.stopped, "")).into_any_element()
            }
            SyncStatus::Failed(error) => {
                let text = fill(&m.failed, &[("error", error)]);
                EmptyView::new("circle-alert", c.red, m.failed_title.clone(), Sentence::of(&text, "")).into_any_element()
            }
            SyncStatus::Done(summary) => {
                let rows = report_rows(&summary.files);
                let failed = rows.iter().filter(|r| r.kind == ReportKind::Failed).count();
                let skipped = rows.iter().filter(|r| r.kind == ReportKind::Skipped).count();
                let mut line = fill(&m.report_summary, &[
                    ("entries", &thousands(summary.total_indexed)),
                    ("files", &summary.files_scanned.to_string()),
                    ("seconds", &format!("{:.1}", Duration::from_millis(u64::try_from(summary.duration_ms).unwrap_or(u64::MAX)).as_secs_f64())),
                ]);
                if failed > 0 {
                    line.push_str(&format!(" · {}", fill(&m.report_failed, &[("count", &failed.to_string())])));
                }
                if skipped > 0 {
                    line.push_str(&format!(" · {}", fill(&m.report_skipped, &[("count", &skipped.to_string())])));
                }
                let s = &d.spacing.empty;
                v_flex()
                    .id("report")
                    .size_full()
                    .overflow_y_scroll()
                    .px(pt(d.spacing.list.pad))
                    .child(div().px(pt(s.summary_pad_x)).pt(pt(s.summary_pad_top)).pb(pt(s.summary_pad_bottom)).text_role(&d.typography.roles.description).text_color(c.muted).child(line))
                    .children(rows.into_iter().enumerate().map(|(ix, row)| {
                        let (icon, tint, chip, detail) = match row.kind {
                            ReportKind::Failed => ("circle-x", c.red, m.report_failed.replace("{count} ", ""), row.detail.clone()),
                            ReportKind::Skipped => ("triangle-alert", c.yellow, row.entries.to_string(), fill(&m.report_skipped_entries, &[("entries", &row.detail)])),
                            ReportKind::Indexed => ("circle-check", c.green, row.entries.to_string(), m.report_indexed.clone()),
                        };
                        Row::new(SharedString::from(format!("report-{ix}")), row.path.clone())
                            .lead(tile(icon, tint, TileSize::Row))
                            .mono(true)
                            .description(Some(detail))
                            .trailing(vec![pill(chip, ChipState::of(ix == 0)).into_any_element()])
                            .selected(ix == 0)
                    }))
                    .into_any_element()
            }
        }
    }
}
```

(`issues_line` returns "Skipped, not a map: a, b" today: change it to return only the entries joined by ", ", since
the catalog's `report_skipped_entries` supplies the words. The pill of a failed row reads "failed": the catalog's
`report_failed` is "{count} failed", so the code strips "{count} ". Add imports for `EmptyView`, `Sentence`,
`progress_bar`, `Row`, `tile`, `TileSize`, `pill`, `ChipState`, `design`, `pt`, `TextRoleExt`, `fill`, `Catalog`,
`Rc`, `Selectable` (for `.selected`), and remove the unused `Progress` import.)

- [ ] **Step 4: Keys, actions and the status line.** In `search_view.rs`:
  - `key_screen`'s `Screen::Rebuild(sync)` arm becomes:

```rust
Screen::Rebuild(sync) => match sync.read(cx).phase() {
    Phase::Running => KeyScreen::Refreshing,
    Phase::Report => KeyScreen::Report,
    Phase::Stopped => KeyScreen::Stopped,
    Phase::Failed => KeyScreen::Failed,
},
```

- in `run_action`, add arms (before `_ => {}`):

```rust
("stop", _) => {
    if let Screen::Rebuild(sync) = &self.screen {
        sync.read(cx).cancel();
    }
}
("back", _) => self.back(&Back, window, cx),
("retry", _) => {
    // A finished or failed sync lets `rebuild` start a new one.
    self.rebuild(&Rebuild, window, cx);
}
```

- the refresh screens take Return on the panel's focus (no text field there): in `keys.rs`, the `enter` and
  `cmd-enter` ladder bindings use `Some(DETAIL_CONTEXT)`; add a `REFRESH_CONTEXT: &str = "Refresh"`, bind
  `enter` → `ScreenEnter` and `cmd-enter` → `ScreenCmdEnter` in it too, and add it to `SearchView::render`'s key
  context while `self.is_rebuilding()` (as `DETAIL_CONTEXT` is added on the detail screen). Update the binding count
  test (+2).
- in `search_view/footer.rs`, `status_parts`: while rebuilding and running, return
  `(messages.refreshing_status.clone(), None)` with `Dot::Busy` (the dot is chosen in `render_footer` the same way).

- [ ] **Step 5: Run every kb-app test.**

Run: `cargo nextest run -p kb-app`
Expected: PASS, including `reports_a_refresh_and_goes_back_with_return` and `task_t5_rebuilds_the_index_with_one_chord`.

- [ ] **Step 6: Gate and commit.**

```bash
mise run gate
git add crates/kb-app
git commit -m "feat(ui): Draw the refresh screens as the approved page" \
  -m "Running, report, stopped and failed, with their keys from the catalog (D16)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Sizes from the tokens; the old code goes

**Files:** modify `DESIGN.md`, `crates/kb-app/src/ui/design.rs`, `crates/kb-app/src/window_size.rs`,
`crates/kb-app/src/panel.rs`, `crates/kb-app/src/lib.rs`, `crates/kb-app/src/search_view.rs`,
`crates/kb-app/src/presentation.rs`, `crates/kb-app/src/filter.rs`; delete `crates/kb-app/src/accents.rs`.

- [ ] **Step 1: The window's size and the scroll step from the tokens.** Add to DESIGN.md's front matter, under
      `components:`, the line `detail: { scroll_step: 60 }`, and to `ui/design.rs` a
      `points!(/// The detail screen's scrolling.\n    DetailSize { scroll_step });` and a field `pub detail: DetailSize`
      in `Components`. Then:
  - `window_size.rs`: replace `pub const COMPACT` with

```rust
/// The compact size, in points (DESIGN.md, `components.window`).
#[must_use]
pub fn compact() -> Size<Pixels> {
    let w = &design().components.window;
    size(pixels(w.width), pixels(w.height))
}
```

    `EXPANDED_SHARE` with `design().components.window.expanded_share`, every `COMPACT` with `compact()`, and in its
    tests `px(…)` with `pixels(…)` (import `crate::ui::design::{design, pixels}`; drop the `px` import);

- `panel.rs` and `search_view.rs`: `COMPACT` → `compact()`;
- `search_view.rs`: `const NOTES_LINE_STEP: Pixels = px(60.);` → a function
  `fn notes_line_step() -> Pixels { pixels(design().components.detail.scroll_step) }` (use it where the constant was),
  and `point(px(0.), px(0.))` → `gpui_kit::Point::default()`;
- `lib.rs`: `gpui_kit::px(ui::design::POINTS_PER_REM)` → `ui::design::pixels(ui::design::POINTS_PER_REM)`.

- [ ] **Step 2: Remove what the design replaced.**

```bash
git rm crates/kb-app/src/accents.rs
grep -rn "accents::\|type_badge\|TypeBadge\|type_key\|TypeKey\|primary_action\|PrimaryKind\|type_label" crates/kb-app/src crates/kb-app/tests
```

Delete `mod accents;` from `lib.rs`. For every function the grep lists in `presentation.rs` (`type_key`,
`type_label`, `type_badge`, `primary_action` and their types and tests): delete it if its only users are in
`presentation.rs`; otherwise replace its use with the catalog (`catalog.entry_type(…)`, `kind.label`, `kind.icon`,
`kind.tint`, `kind.primary`) and then delete it. `filter.rs`'s `filter_summary` falls back to `type_label` for a type
the catalog lacks: use the type key as written instead.

- [ ] **Step 3: No `dead_code` expectation left.** Remove the `#![expect(dead_code, …)]` line (and its comment) from
      `ui/design.rs`. Run `cargo clippy -p kb-app --all-targets -- -D warnings`. For each "field is never read" it
      reports, delete that token from DESIGN.md's front matter and its field from `ui/design.rs` (the token is unused,
      so the design does not need it). Repeat until clippy is clean.

- [ ] **Step 4: Run every test.**

Run: `cargo nextest run --workspace`
Expected: PASS.

- [ ] **Step 5: Gate and commit.**

```bash
mise run gate
git add -A DESIGN.md crates/kb-app
git commit -m "refactor(app): Take every size from the tokens; remove the old helpers" \
  -m "accents.rs, the type badges and the hard-coded window size give way to DESIGN.md and the catalog." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: The ast-grep rule

**Files:** create `tools/ast-grep/rules/no-literal-design.yml`, `tools/ast-grep/tests/no-literal-design-test.yml`.

- [ ] **Step 1: Write the rule's test first.** Create `tools/ast-grep/tests/no-literal-design-test.yml`:

```yaml
id: no-literal-design
valid:
  - "fn f() { div().px(pt(3.)) }"
  - "fn f() { div().w(pt(d.components.row.height)) }"
  - "fn f() { let p = pixels(w.width); }"
invalid:
  - "fn f() { div().w(px(3.)) }"
  - "fn f() { let s = gpui_kit::px(16.); }"
  - "fn f() { div().bg(rgb(0x24273A)) }"
  - "fn f() { div().bg(gpui_kit::rgba(0x6E738D1A)) }"
```

Run: `ast-grep test --skip-snapshot-tests`
Expected: FAIL: no rule `no-literal-design`.

- [ ] **Step 2: Write the rule.** Create `tools/ast-grep/rules/no-literal-design.yml`:

```yaml
id: no-literal-design
language: rust
severity: error
message: Sizes and colours come from DESIGN.md's tokens through ui::design (pt, pixels, design().colors).
note: |
  The design tokens live in one place (spec D18). A literal px(…), rgb(…) or rgba(…) in a view is a value the
  design cannot change; read it from `design()` instead, adding a token to DESIGN.md's front matter if none fits.
files:
  - crates/kb-app/src/**/*.rs
ignores:
  - crates/kb-app/src/ui/design.rs
  - crates/kb-app/src/tag_images.rs
rule:
  any:
    - pattern: px($A)
    - pattern: $P::px($A)
    - pattern: rgb($A)
    - pattern: $P::rgb($A)
    - pattern: rgba($A)
    - pattern: $P::rgba($A)
```

(`tag_images.rs` holds SVG text that contains `rgb(` inside string literals, which ast-grep does not match as calls; it
is ignored only in case a test there uses the functions. `.px(…)`, the padding method, is a method call and does not
match `px($A)`.)

Run: `ast-grep test --skip-snapshot-tests && ast-grep scan`
Expected: the tests pass; the scan reports nothing. If the scan reports a `px(`, `rgb(` or `rgba(` call, replace it
with a token (Task 4's pattern) and scan again.

- [ ] **Step 3: Gate and commit.**

```bash
mise run gate
git add tools/ast-grep
git commit -m "chore(lint): Reject literal sizes and colours outside the design module" \
  -m "Acceptance criterion 2 of the design system spec." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Done when

- `cargo nextest run --workspace`, `ast-grep scan` and `mise run gate` pass.
- `03-empty.html` and `11-14-refresh.html` match the app over the showcase (the showcase's `errors/broken.yml` gives
  the report a failed row and `bookmarks/rust.yml` a skipped one).
