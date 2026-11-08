# Design System Checklists

Per-screen review of the app against its approved pages, from the screenshots `mise run design:shots` takes
of the real app on the showcase corpus (`tools/design/shots/`, compact and expanded). Every row was checked by opening
the screenshots named in its section (2026-10-14).

Legend: ✓ seen in the screenshot · ✗ differs (see Note) · — not visible in any screenshot (a UI test or a hand check
covers it, as the Note says).

## List (01) — `compact-01-list.png`, `expanded-01-list.png` vs `docs/guides/design/01-list.html`

| Item                                                                                     | Compact | Expanded | Note                                                            |
| ---------------------------------------------------------------------------------------- | ------- | -------- | --------------------------------------------------------------- |
| Magnifier, query in 15 pt, the filter pill with `⌘P`                                     | ✓       | ✓        |                                                                 |
| Hairline under the header                                                                | ✓       | ✓        |                                                                 |
| Rows 56 tall, 4 apart, corners 6                                                         | ✓       | ✓        | row pitch measured at 60 pt                                     |
| Selected row fill `#363A4F` with a 2.5 mauve bar                                         | ✓       | ✓        |                                                                 |
| Tiles 24 tinted by type (Commands green, Cheat Sheets mauve, Bookmarks blue, Tasks pink) | ✓       | ✓        | bookmarks show their favicon (YouTube, Tokio), as planned       |
| Command titles in mono                                                                   | ✓       | ✓        |                                                                 |
| Chips and keycaps exactly 20 tall                                                        | ✓       | ✓        |                                                                 |
| Selected row's chips on mantle with muted text, other rows' chips grey                   | ✓       | ✓        |                                                                 |
| `↵` on the selected row, `⌘2`…`⌘9` on the next eight                                     | ✓       | ✓        | compact shows `⌘2`–`⌘7`; the rest scroll                        |
| Footer status `● Ready • N results` with the true count                                  | ✓       | ✓        | 9 results: the tenth entry is a note, not one of the four types |
| Footer keys: _Details ↵ · Copy ⌘↵ · Actions ⌘K_ (bookmark: _Open ⌘↵ · Copy ⇧⌘↵_)         | ✓       | ✓        | bookmark keys seen in `04-split-bookmark-hero`                  |

## Action panel (02) — `*-02-actions.png`, `*-02-more.png` vs `docs/guides/design/02-actions.html`

| Item                                                                         | Compact | Expanded | Note                                                                               |
| ---------------------------------------------------------------------------- | ------- | -------- | ---------------------------------------------------------------------------------- |
| Opens above _Actions_, bottom right, 220 wide, rows 28, search at the bottom | ✓       | ✓        |                                                                                    |
| Root lists the screen's actions in D9's order with their keycaps             | ✓       | ✓        | Details, Copy, Resize, Links…, More…                                               |
| _Links… ›_ only for entries with links                                       | —       | —        | only an entry with links is shot; UI tests cover it                                |
| More… shows COPY and ADMINISTRATION under a `‹ More` header with no search   | ✓       | ✓        | `compact-02-more.png`, `expanded-02-more.png`; reached with ↓×4 ↵                  |
| Typing in the root finds More…'s rows, marked _More_                         | —       | —        | not shot; seen by hand (“res” finds Resize and Refresh, marked More) and UI-tested |

## Empty and no index (03, 10) — `compact-03-empty.png`, `expanded-03-empty.png`, `compact-10-no-index.png` vs `docs/guides/design/03-empty.html`

No approved page shows the no-index screen; it was checked against the catalog's `no_index_title` and `no_index`.

| Item                                                                 | Compact | Expanded | Note                                  |
| -------------------------------------------------------------------- | ------- | -------- | ------------------------------------- |
| One large icon, the title, the sentence with the query in mauve mono | ✓       | ✓        | no index: database icon, compact only |
| Status `0 results` or `● No index`                                   | ✓       | ✓        | `● No index` compact only             |
| Footer _Refresh ↵_                                                   | ✓       | ✓        |                                       |

## Split and detail (04–06) — `*-04-split-command.png`, `*-04-split-bookmark-hero.png`, `*-05-detail-command.png`, `*-05-detail-bookmark-links.png`, `*-06-detail-cheat.png` vs `docs/guides/design/04-entry-template.html`, `04-command-cards.html`, `05-detail-links.html`

| Item                                                                                                                            | Compact | Expanded | Note                                                                                                                            |
| ------------------------------------------------------------------------------------------------------------------------------- | ------- | -------- | ------------------------------------------------------------------------------------------------------------------------------- |
| The list at 34 %                                                                                                                | ✓       | ✓        |                                                                                                                                 |
| Pane header is row's icon and title on one line with tags right (clickable: they filter)                                        | ✓       | ✓        | clicking is not shot; UI test `filters_the_list_by_a_tag_from_the_detail_screen` covers it                                      |
| Hero by type                                                                                                                    | ✓       | ✓        | command card, YouTube thumbnail and link line seen; task chips not shot                                                         |
| Description as a red-bar quote                                                                                                  | ✓       | ✓        |                                                                                                                                 |
| Notes rendered in Markdown                                                                                                      | ✗       | ✗        | lists, inline code and the YAML block render; a `###` heading draws at body size and `**bold**` draws regular (TODO 8qUa, 4hQn) |
| Links section (D20): one row each, titles and URLs on one line, trimmed, `⌘1`…`⌘9`                                              | ✓       | ✓        | in split the keycaps are hidden, as planned                                                                                     |
| Nothing else in pane (no badge, path or metadata column)                                                                        | ✓       | ✓        |                                                                                                                                 |
| Command card: breaks at `&&` and `\|`, hanging indent, no split tokens, no ligatures, copy button, 12 pt in split, 13 in detail | ✓       | ✓        | point sizes not measured; the expanded detail fits on one line, so it has no break                                              |
| Footer left: type tile, file, `⌥⌘C`                                                                                             | ✓       | ✓        |                                                                                                                                 |
| Detail: chevron only; → and ← do nothing                                                                                        | ✓       | ✓        | → and ← are not shot; UI test `ignores_right_and_left_on_the_detail_screen` covers them                                         |

## Filter (07) — `compact-07-filter.png`, `expanded-07-filter.png` vs `docs/guides/design/07-filter.html`

| Item                                                      | Compact | Expanded | Note                                            |
| --------------------------------------------------------- | ------- | -------- | ----------------------------------------------- |
| Under the pill, search at the bottom                      | ✓       | ✓        |                                                 |
| Four types with their tiles, tags with their icons or `#` | ✓       | ✓        | the visible tags have no image, so all show `#` |
| ✓ only on chosen rows                                     | —       | —        | nothing chosen in the shot; UI tests cover it   |
| Stays open while picking                                  | —       | —        | UI tests cover it                               |
| _n filters · Clear All_ first when filters are set        | —       | —        | UI tests cover it                               |
| Pill names the filters                                    | —       | —        | UI tests cover it                               |
| Footer _Toggle ↵ · Only ⌘↵_                               | ✓       | ✓        |                                                 |

## Refresh (11–14) — `compact-12-report.png`, `expanded-12-report.png` vs `docs/guides/design/11-14-refresh.html`

| Item                                                                                                           | Compact | Expanded | Note                                                                   |
| -------------------------------------------------------------------------------------------------------------- | ------- | -------- | ---------------------------------------------------------------------- |
| Chevron only in header                                                                                         | ✓       | ✓        |                                                                        |
| Running shows its icon, _Refreshing the index…_, the progress bar and _n of m files · file_, footer _Stop esc_ | —       | —        | not captured: ten entries refresh before a capture; not checked by eye |
| Report lists files, failed first, under a summary line, footer _Back ↵_                                        | ✓       | ✓        | the refresh toast repeats the summary line over the lower rows         |
| Stopped and failed show their icon, title and sentence, footers _Retry ↵_ and _Retry ↵ · Configure ⌘↵_         | —       | —        | not captured; unit and UI tests cover them                             |

## Notice (17) — `compact-17-notice.png`, `expanded-17-notice.png`

No approved page shows the notice; it was checked against DESIGN.md (D17).

| Item                                                                               | Compact | Expanded | Note |
| ---------------------------------------------------------------------------------- | ------- | -------- | ---- |
| Bottom right above the footer, _Copied to clipboard_, a `⌘V` hint on a grey keycap | ✓       | ✓        |      |

## Sign-off

Not signed off. The owner writes this line after reviewing every screenshot.
