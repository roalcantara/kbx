# One navigation model — design

- Status: approved
- Date: 2026-10-27
- Findings: audit A22, with fix batch 3 (A04, A05, A06, A07, A08, A12, A13), in
  `docs/superpowers/audits/2026-10-27-mvp-ui-ux-audit.md`
- Context: "where the user is" lives in four places: `Screen` and `Overlay` in `search_view.rs`, `ReportLevel` in
  `sync_view.rs`, and `KeyScreen`, derived from the other three by `SearchView::key_screen`. Each screen makes its own
  choices about its header, Esc and keys. As a result the detail screens say nothing about where back goes (A22), and
  keys reach screens they were not meant for (A05–A08, A12, A13). Approved in the brainstorming companion
  (2026-10-27): option **B, back names where it goes**. The page is kept as `docs/guides/design/20-navigation.html`.

## Goal

One rule for every screen, enforced by one model and drawn by one component. A screen opened on top of another (a
**place**) shows `‹` and the name of the place under it. Esc and `‹` return to that place exactly as it was. Only
the place on top, or the overlay over it, acts on a key. A new screen gets all of this by being a place; it cannot
choose otherwise. Consistency over flexibility: no screen draws its own header, decides its own back, or routes its
own keys.

## Decisions

**D1. Places on a stack; overlays float on top.** The stack holds screens: `List` (always at the bottom), `Detail`,
`Refresh` and `ReportDetail`. The action panel, its submenus and the filter are overlays over the top place. While
an overlay is open it owns every key, and the place under it does nothing. A split (→ / ←) is not a place: it is a
pane of the place it belongs to, the search list or the report. This follows the 2YnZ spec, D1.

**D2. One `Navigator`, without GPUI.** `crates/kb-app/src/navigator.rs` holds the stack and the overlay; `Overlay`
moves there from `search_view.rs`.

```rust
pub enum Place { List, Detail { back: String }, Refresh { back: String }, ReportDetail { back: String } }
pub enum PlaceKind { List, Detail, Refresh, ReportDetail }   // `Place` without its data, for matching
pub enum Owner { Overlay(Overlay), Place(PlaceKind) }

impl Navigator {
    pub fn top(&self) -> &Place;
    pub fn push(&mut self, place: Place);
    pub fn pop(&mut self) -> bool;              // never removes List; false when nothing was popped
    pub fn overlay(&self) -> Overlay;
    pub fn set_overlay(&mut self, overlay: Overlay);
    pub fn owner(&self) -> Owner;
    pub fn key_screen(&self, phase: Option<Phase>) -> KeyScreen;
    pub fn key_context(&self) -> String;
    pub fn back_name(&self) -> Option<&str>;     // None on List
}
```

- `Place` carries its back name, captured when the place is pushed: the query, or the catalog's `Search` when the
  query is empty, for a place pushed on `List`; the catalog's `Refresh` for `ReportDetail`. Typing can never change
  it afterwards.
- `SearchView` holds one `navigator` field in place of `screen` and `overlay`. While `Refresh` is on the stack it
  also holds that place's `Entity<SyncView>`.
- `SyncView` keeps the refresh's work, its phase, its report rows and the split flag. `ReportLevel::Detail` and
  `from_split` go: the report's detail is a place now.

**D3. One back function.** `SearchView::back` is the only way back. Esc, a click on `‹`, and the report's back all
call it, in this order:

1. With an overlay open, the overlay's own Esc order, as today: clear its search, then leave the submenu, then close
   it.
2. With a refresh running, Esc stops it.
3. Otherwise `navigator.pop()`. The place underneath shows exactly as it was left: its selection, scroll, query and
   split.

← only closes a split.

**D4. Keys act only for the owner.** Every action handler begins with one shared guard,
`fn owns(&self, kind: PlaceKind) -> bool` or the overlay equivalent. Inline checks such as `self.report(cx).is_some()`
and `matches!(self.screen, …)` are removed. What each owner handles:

| Owner          | Handles                                                                             | No longer reaches it                                                                                                       |
| -------------- | ----------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| An overlay     | its rows, ↵ / ⌘↵, Esc, ⌘K                                                           | ⌘1–⌘9, → / ←, row actions on the place behind (A07)                                                                        |
| `List`         | the list, its split, ⌘P, the entry's actions                                        | —                                                                                                                          |
| `Detail`       | scrolling, links ⌘1–⌘9, the entry's actions                                         | ⌘P (A05)                                                                                                                   |
| `Refresh`      | the report's rows, its split, the row's actions; ⌘R reruns the report's own sources | the list's split (A13), Edit and Copy Path on a running, stopped or failed refresh (A08), ⌘R refreshing every source (A06) |
| `ReportDetail` | scrolling its pane (↑↓, ⌘↑↓), the row's actions                                     | moving the selection (A04)                                                                                                 |

`Navigator::key_context()` builds the one key-context string that `render` sets. Every place contributes its context
(`Search`, then `Detail`, `Refresh` or `ReportDetail`), and an open overlay adds `ActionPanel`. The report-only keys
(⌘⇧L, ⌘⇧E, ⌘⇧R) are bound in the report's contexts only (A12).

**D5. One header for every place (option B).** `List` keeps its header: the magnifier, the query and the filter
pill. Every other place shows the same header:

- `‹` as a GPUI Kit `Button`, ghost, calling `back`;
- then the back name, cut off with an ellipsis when long: a query in muted mono, and `Search` or `Refresh` in plain
  faint text.

The pane keeps its own title, so the approved entry and report panes do not change. The footer does not change
either.

**D6. One crumb component, shared with ⌘K.** The `‹ name` element moves out of the action panel's
`panel()` into `crates/kb-app/src/ui/crumb.rs`. Both the panel's submenu crumbs (`‹ More`) and the place header use
it; only the band it sits in (the panel's crumb row or the 56 pt header) differs. kb then has one way to say "back
to X".

**D7. DRY by construction.** Each concept lives in one place:

| Concept                 | Its one home                                           |
| ----------------------- | ------------------------------------------------------ |
| Which screen is current | `Navigator::top`                                       |
| Who handles a key       | `Navigator::owner` and the `owns` guard                |
| Which keys show         | `Navigator::key_screen`, then the catalog's `keys_for` |
| Which bindings apply    | `Navigator::key_context`                               |
| How back works          | `SearchView::back` and `Navigator::pop`                |
| How back is drawn       | `ui::crumb`                                            |
| What a place is called  | the `Place` value and the catalog's `[messages]`       |

A new screen is one new `Place` variant. The compiler then lists every `match` it must answer: its key screen, its
context and its owner rows. Nothing else is needed, and nothing else is allowed.

## Error handling

| Case                                                | Behaviour                                                                                                                  |
| --------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| Esc on `List` with no overlay                       | As today: clears the query, then hides the panel; `pop` returns false                                                      |
| Empty query when a place is pushed                  | The back name is the catalog's `Search`                                                                                    |
| A long query                                        | Cut off with an ellipsis in the header; the full query is in the list it returns to                                        |
| Panel hidden (Esc, focus loss) and reopened with ⌥K | The stack is kept; the panel reopens on the same place. Task 1 checks this matches today's behaviour for the detail screen |
| A refresh started from `Detail`                     | `Refresh` is pushed on top of `Detail`; its back name is the entry's back name (the query)                                 |

## Testing

Tests first, beside the code (AGENTS.md).

- `navigator.rs`, without a window:
  - push, pop and `pop` on `List`;
  - `owner` with and without an overlay;
  - `key_screen` for each place and refresh phase;
  - `key_context` for each place, with and without an overlay;
  - back names, including an empty query.
- The UI tests that pass today must still pass after `SearchView` switches to the navigator, before any behaviour
  changes. That is the refactor's safety net.
- The audit's reproductions become acceptance tests and must turn green: `audit_k3_arrows_on_the_report_detail_keep_its_row`,
  `audit_cmd_p_on_the_detail_screen_opens_no_hidden_filter`, `audit_cmd_r_on_a_skills_report_refreshes_skills_again`,
  `audit_cmd_number_under_the_action_panel_runs_no_row`, `audit_copy_path_on_a_stopped_refresh_copies_nothing`. New tests
  for A12 (⌘⇧R on the list does nothing) and A13 (→ on a refresh screen leaves the list's split alone).
- New UI tests:
  - list → detail shows `‹ <query>` and Esc returns to the same row and split;
  - report → its detail shows `‹ Refresh`, and Esc returns to the report's split;
  - a click on `‹` behaves as Esc;
  - ⌘K's `More` submenu still shows `‹ More` (shared crumb).
- By hand: `mise run design:shots` for the detail and report-detail headers.

## Docs and design kit (same change)

- `DESIGN.md` §4: a **Navigation** component. It holds the stack rule, the `‹ name` header for every place but the
  list, overlays floating and owning keys, and the one crumb, citing `20-navigation.html`.
- `CONTEXT.md`: a **Place** term (a screen on the navigation stack; `crates/kb-app/src/navigator.rs`). The _Sync
  report_ row mentions that its detail is a place.
- The audit: A22 and batch 3 marked fixed, with the commit.
- `docs/guides/design/`: `20-navigation.html`, the approved companion page.

## Scope

In scope: the `Navigator` and its places (D1, D2), the one back function (D3), key ownership and contexts (D4), the
place header (D5), the shared crumb (D6), the removal of `Screen`, `ReportLevel::Detail` and the scattered checks
(D7), tests and docs.

Out of scope: audit batches 1–2 (selection and list navigation keys: A01–A03, A09), A10 and A11 (overlay footer keys
and the theme preview), any visual change beyond the place header, and the footer's left side, which stays as
approved.
