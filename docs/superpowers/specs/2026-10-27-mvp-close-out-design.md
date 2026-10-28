# MVP close-out — design

- Status: approved
- Date: 2026-10-27
- Findings: every finding left in `docs/superpowers/audits/2026-10-27-mvp-ui-ux-audit.md` after the navigation model:
  A01–A03, A09–A11, A14–A21, plus a real guard for A12 and the keyboard into the split's pane (D3, new).
- Context: the navigation model (spec `2026-10-27-navigation-model-design.md`) gave "where the user is" one home. What
  is left repeats one pattern: a concept handled in several places that drift apart. A selection changes without
  scrolling. Each list answers its own keys. An overlay changes in more than one place. Some words bypass the
  catalog. Approved in the brainstorming companion (2026-10-27): the pane's focus mark, **A with a thin bar**, kept
  as `docs/guides/design/21-pane-focus.html`.

## Goal

Close the MVP. Every audit finding is fixed with one home for its concept, the audit's five failing tests pass, and
`mise run gate` is green again. Consistency over flexibility, as for the navigation model: each rule below is written
once and used by every surface it applies to.

## Decisions

**D1. One "select and show" function per kind of list.** A selection never changes without being scrolled into view.

- **The list:** `SearchView::select_entry(row)` sets the selection (`ListState::set_selected_index`) and scrolls to it
  (`scroll_to_selected_item`).
  - Every filter change (the filter's Toggle and Only, a tag click) selects row 0, as a new query does.
  - After a refresh (`set_connection`), the delegate looks up the selected entry by id. If the new results still
    contain it, that row stays selected; otherwise row 0 is selected (A01).
- **⌘K, the filter and Theme…:** `select_panel_row(ix)` is the only writer of `panel_selected`. Opening, typing in the
  search, clearing it, Toggle, Theme…'s opening row and the arrows all call it (A09). It scrolls to the row's
  **child index**, not its row index.
  - `ui/action_panel.rs` gives that index through `child_index(page, visible, row)`, which counts the section labels
    and hairlines above the row. It walks the same loop that draws them, so the two cannot drift (A02).
- **The panes:** every pane and detail scrolls through the one `detail_scroll` handle (D3), reset whenever the
  selection changes, so each row's pane starts at its top (A17).

**D2. One navigation-key model for every list (A03).**

| Key                 | List, filter, ⌘K and its submenus, Theme…, report | Entry detail, report detail, a focused pane (D3) |
| ------------------- | ------------------------------------------------- | ------------------------------------------------ |
| ↑ / ↓               | one row                                           | scroll a line                                    |
| ⌘↑ / ⌘↓             | first / last row                                  | top / bottom                                     |
| Page Up / Page Down | one visible page                                  | one page                                         |

- **One function routes every navigation key.** Six actions, `PreviousRow`, `NextRow`, `FirstRow`, `LastRow`,
  `PageUp` and `PageDown`, are bound once in `Search` (up, down, cmd-up, cmd-down, pageup, pagedown). They replace
  `ScrollNotes*`, `PageNotes*` and the report's `Refresh`-only bindings. Each calls `SearchView::nav(Nav::…)`, which
  sends the key to its owner:
  - an open overlay: `select_panel_row`;
  - the list: `select_entry`;
  - the report: its rows;
  - a scrolling target (an entry's or a row's detail, or a split pane with the keys, D3): the shared scroll
    functions.

  `Nav` is a plain enum in `navigator.rs`.
- **The text fields capture the same keys.** The query field and ⌘K's search field bind ↑↓, ⌘↑↓ (`MoveToStart` /
  `MoveToEnd`) and Page Up/Down (`MovePageUp` / `MovePageDown`) to their caret. `search_view/header.rs` and the
  panel's search capture all six and call `nav`, as ↑↓ are captured today.
- **A page** is the number of whole rows that fit the visible rows: the panel's `max_height / row` for the panel; the
  window's height less the header and footer, divided by `row.height`, for the list and the report. One
  `page_rows` computes it.
- The detail screens' ⌘↑ / ⌘↓ change from paging to top / bottom, as macOS text views do. Page Up / Down page.
- Home and End stay with the search field's caret.

**D3. The keyboard into the split's pane (new; approved page `21-pane-focus.html`, option A, thin bar).**

- On a split (the list's or the report's), **⇥** moves the keys into the pane. The query field binds ⇥ / ⇧⇥ to
  indenting (`IndentInline` / `OutdentInline`), so it captures them as it does the arrows, and `Search` binds them
  for the report, which has no field.
- The pane shows a **1 pt mauve bar** on its left edge, with no glow. The list's selected row keeps its fill, and its
  bar goes faint (`c.faint`).
- ↑↓, ⌘↑↓ and Page Up / Down then scroll the pane through `nav`'s scrolling target. ↵ and ⌘↵ still act on the
  selected entry or row, so the footer keeps the list's or the report's keys. The bar shows where the keys go.
- **⇥, ⇧⇥ or Esc** return the keys to the list. Esc goes back one level: pane → list → (clear the query) → hide.
- **The pane's focus belongs to the navigator**, which already owns where the keys go:
  - `Navigator::pane_has_keys` / `set_pane_has_keys`;
  - it is cleared by `push`, `pop` and `pop_to_list`, and by `SearchView` when the selection changes or the split
    closes;
  - while it is set, `key_context` is the detail screens' (`Search Detail`, `Search Detail Report`).
- Every pane and detail scrolls through the one `detail_scroll` handle: the entry split, the entry detail, the report
  split and the report detail. Only one of them shows at a time. The handle is reset whenever the selection changes
  (A17).

**D4. `show_overlay` is the only place an overlay changes.** Besides resetting the search and the focus, as today, it:

- cancels a Theme… preview whenever the new overlay is not Theme…. Leaving with ⌘K, ⌘P or a click outside then
  restores the theme in use, as Esc does (A11). `keep_theme` clears the preview before closing, so ↵ still keeps the
  chosen theme;
- resets the panel's selection and scroll through `select_panel_row(0)` (D1).

**D5. The action panel's own footer keys (A10).** A new catalog `[screens.actions]` (`enter = "run"`,
`escape = "back"`) gives _Run ↵ · Back esc_. While an overlay is open, the footer's trailing item reads _Close ⌘K_
(the new `close` action's label) instead of _Actions ⌘K_. New actions `run` and `close` carry those labels.
`Navigator::key_screen` returns it for `Overlay::Actions` on the root and on every submenu except Links…, which keeps
`[screens.links]`. ⌘K always closes the whole panel, from the root or from a submenu.

**D6. Theme… does not depend on Refresh More… (A15).** `more_page` adds Theme… unconditionally, after the admin rows,
as the catalog's `[panel]` comment describes.

**D7. Report and render details.**

- **A14:** `ReportRow::configures()` is also true for `Subject::SourcesDir`, so Edit (⌘↵) opens config.toml there and
  ⌘K offers it. The row's message ends "Check it in Configure.", as `failed` does.
- **A16:** `render` no longer mutates state. One `apply_compact` sets the list's compact flag. It is called from
  `set_pane`, and from `push` / `settle` when the place changes.
- **A18:** an excerpt line that does not fit is cut off with an ellipsis. Copy Location and Copy Error carry the full
  text.

**D8. Words and polish.**

- **A19:** the hard-coded notices move to `[messages]`, with `{file}` / `{error}` / `{key}` placeholders. They are:
  config.toml could not be created, the theme could not be saved, ⌥K could not be registered, the themes could not
  be seeded, and the three config notices.
  - The notices raised before the catalog loads (the config notices and the themes seed) use the shipped catalog's
    words (`Catalog::default()`): config.toml cannot be trusted to word its own errors.
  - The back button's label is the catalog's `back` label, and its tooltip is that label with Esc's keycap.
- **A20:** the empty-sources screen gets its own title, `no_entries_title = "No entries yet"`. Its sentence is built
  from the catalog's type labels (`no_entries = "Your sources have no {types}."`), so an overridden `[types]` stays
  right.
- **A21:** the action panel's and the filter's rows get the list rows' hover fill (`c.hover`), except the selected row.

## Error handling

| Case                                                      | Behaviour                                                                |
| --------------------------------------------------------- | ------------------------------------------------------------------------ |
| A refresh drops the selected entry                        | Row 0 is selected; with no results, nothing is, and the empty view shows |
| ⇥ with the split closed, or on a place with no split      | Nothing: ⇥ moves the keys only between a split's list and its pane       |
| ⇥ while an overlay is open                                | The overlay's own focus order, as today; the pane does not take the keys |
| Page Up / Down on a list shorter than a page              | First / last row, as ⌘↑ / ⌘↓                                             |
| A catalog override removes `[screens.actions]` or `.pane` | The footer shows _Actions ⌘K_ alone, as for any screen without keys      |

## Testing

Tests first, beside the code (AGENTS.md).

- The five failing audit tests turn green:
  - `audit_a_new_filter_selects_its_first_result`;
  - `audit_k2_keeps_the_filter_s_selected_row_in_view`;
  - `audit_k1_cmd_down_and_cmd_up_jump_to_the_list_s_ends`;
  - `audit_reopening_the_filter_starts_at_the_top`;
  - `audit_closing_theme_with_cmd_k_restores_the_theme`.
- New UI tests:
  - **A01:** a refresh keeps the selected entry, and a filter change selects row 0;
  - **D2:** ⌘↑↓ and Page Up / Down on the list, the filter, a submenu and the report;
  - **D3:** ⇥ into the pane, ↓ scrolls it, ⇥ returns, Esc returns, then a second Esc on an empty query hides the panel;
  - **D5:** the panel's footer keys on the root and on a submenu;
  - **A14:** Edit on the sources-directory row opens Configure;
  - **A15:** Theme… is in More… with `more_refresh = []`;
  - **A20:** the empty-sources title.
- Unit tests:
  - `child_index` with labels and hairlines;
  - the entry kept by id after `set_connection`;
  - **A12:** `keys.rs` asserts that the report-only actions are bound in `Report` alone.

## Final verification (written by the plan, run after it lands)

The plan's last task writes `docs/superpowers/handoffs/2026-10-27-mvp-final-verification.md` and does not run it. It
holds the long or screen-taking procedures:

- `mise run gate`;
- `mise run design:shots`, compact and expanded, including the `‹` headers, the focused pane and the panel's footer,
  compared with the approved pages;
- by-hand checks the audit could not make:
  - long titles cut off;
  - tag images at size;
  - hover and focus marks;
  - hiding with focus loss and reopening with ⌥K;
  - VoiceOver labels;
  - the eight themes;
- bookkeeping:
  - every audit finding marked fixed with its commit;
  - TODO 2YnZ and the report-scroll TODO closed;
  - `docs/guides/parity.md` and `docs/guides/compliance.md` updated.

## Docs (same change)

- `DESIGN.md` §4: D2's key model under **Navigation**; the focused pane (D3, citing `21-pane-focus.html`); panel rows'
  hover.
- `CONTEXT.md`: _Screen keys_ mentions `[screens.actions]`.
- The audit: each finding marked fixed as its batch lands.

## Scope

In scope: A01–A03, A09–A11, A14–A21, A12's guard, D3's pane focus, their tests and docs, and writing the final
verification handoff.

Out of scope: new features beyond D3, Raycast parity, and running the final verification. The owner runs it after
the plan lands.
