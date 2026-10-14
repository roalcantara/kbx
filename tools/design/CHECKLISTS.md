# Design System Checklists

Per-screen review of the app against its approved pages (spec §9), from screenshots of the real app
on the showcase corpus (`tools/design/shots/`).

## List (01) — `shots/compact-01-list.png`, `shots/expanded-01-list.png` vs `docs/guides/design/01-list.html`

| Item                                                                                     | Compact | Expanded | Note |
| ---------------------------------------------------------------------------------------- | ------- | -------- | ---- |
| Magnifier, query in 15 pt, the filter pill with `⌘P`                                     | ✓       | ✓        |      |
| Hairline under the header                                                                | ✓       | ✓        |      |
| Rows 56 tall, 4 apart, corners 6                                                         | ✓       | ✓        |      |
| Selected row fill `#363A4F` with a 2.5 mauve bar                                         | ✓       | ✓        |      |
| Tiles 24 tinted by type (Commands green, Cheat Sheets mauve, Bookmarks blue, Tasks pink) | ✓       | ✓        |      |
| Command titles in mono                                                                   | ✓       | ✓        |      |
| Chips and keycaps exactly 20 tall                                                        | ✓       | ✓        |      |
| Selected row's chips on mantle with muted text, other rows' chips grey                   | ✓       | ✓        |      |
| `↵` on the selected row, `⌘2`…`⌘9` on the next eight                                     | ✓       | ✓        |      |
| Footer status `● Ready • N results` with the true count                                  | ✓       | ✓        |      |
| Footer keys: _Details ↵ · Copy ⌘↵ · Actions ⌘K_ (bookmark: _Open ⌘↵ · Copy ⇧⌘↵_)         | ✓       | ✓        |      |

## Action panel (02) — `shots/compact-02-actions.png`, `shots/compact-02-more.png` vs `docs/guides/design/02-actions.html`

| Item                                                                         | Compact | Expanded | Note |
| ---------------------------------------------------------------------------- | ------- | -------- | ---- |
| Opens above _Actions_, bottom right, 220 wide, rows 28, search at the bottom | ✓       | ✓        |      |
| Root lists the screen's actions in D9's order with their keycaps             | ✓       | ✓        |      |
| _Links… ›_ only for entries with links                                       | ✓       | ✓        |      |
| More… shows COPY and ADMINISTRATION under a `‹ More` header with no search   | ✓       | ✓        |      |
| Typing in the root finds More…'s rows, marked _More_                         | ✓       | ✓        |      |

## Empty and no index (03, 10) — `shots/compact-03-empty.png`, `shots/compact-10-no-index.png` vs `docs/guides/design/03-empty.html`, `10-no-index.html`

| Item                                                                 | Compact | Expanded | Note |
| -------------------------------------------------------------------- | ------- | -------- | ---- |
| One large icon, the title, the sentence with the query in mauve mono | ✓       | ✓        |      |
| Status `0 results` or `● No index`                                   | ✓       | ✓        |      |
| Footer _Refresh ↵_                                                   | ✓       | ✓        |      |

## Split and detail (04–06) — `shots/compact-04-split-command.png`, `shots/compact-05-detail-command.png`, etc. vs `docs/guides/design/04-split-command.html`, `05-detail-command.html`

| Item                                                                                                                            | Compact | Expanded | Note |
| ------------------------------------------------------------------------------------------------------------------------------- | ------- | -------- | ---- |
| The list at 34 %                                                                                                                | ✓       | ✓        |      |
| Pane header is row's icon and title on one line with tags right (clickable: they filter)                                        | ✓       | ✓        |      |
| Hero by type                                                                                                                    | ✓       | ✓        |      |
| Description as a red-bar quote                                                                                                  | ✓       | ✓        |      |
| Notes rendered in Markdown                                                                                                      | ✓       | ✓        |      |
| Links section (D20): one row each, titles and URLs on one line, trimmed, `⌘1`…`⌘9`                                              | ✓       | ✓        |      |
| Nothing else in pane (no badge, path or metadata column)                                                                        | ✓       | ✓        |      |
| Command card: breaks at `&&` and `\|`, hanging indent, no split tokens, no ligatures, copy button, 12 pt in split, 13 in detail | ✓       | ✓        |      |
| Footer left: type tile, file, `⌥⌘C`                                                                                             | ✓       | ✓        |      |
| Detail: chevron only; → and ← do nothing                                                                                        | ✓       | ✓        |      |

## Filter (07) — `shots/compact-07-filter.png`, `shots/expanded-07-filter.png` vs `docs/guides/design/07-filter.html`

| Item                                                      | Compact | Expanded | Note |
| --------------------------------------------------------- | ------- | -------- | ---- |
| Under the pill, search at the bottom                      | ✓       | ✓        |      |
| Four types with their tiles, tags with their icons or `#` | ✓       | ✓        |      |
| ✓ only on chosen rows                                     | ✓       | ✓        |      |
| Stays open while picking                                  | ✓       | ✓        |      |
| _n filters · Clear All_ first when filters are set        | ✓       | ✓        |      |
| Pill names the filters                                    | ✓       | ✓        |      |
| Footer _Toggle ↵ · Only ⌘↵_                               | ✓       | ✓        |      |

## Refresh (11–14) — `shots/compact-11-refreshing.png`, `shots/compact-12-report.png` vs `docs/guides/design/11-14-refresh.html`

| Item                                                                                                           | Compact | Expanded | Note                                          |
| -------------------------------------------------------------------------------------------------------------- | ------- | -------- | --------------------------------------------- |
| Chevron only in header                                                                                         | ✓       | ✓        |                                               |
| Running shows its icon, _Refreshing the index…_, the progress bar and _n of m files · file_, footer _Stop esc_ | ✓       | ✓        |                                               |
| Report lists files, failed first, under a summary line, footer _Back ↵_                                        | ✓       | ✓        |                                               |
| Stopped and failed show their icon, title and sentence, footers _Retry ↵_ and _Retry ↵ · Configure ⌘↵_         | ✓       | ✓        | Checked in unit tests & catalog configuration |

## Notice (17) — `shots/compact-17-notice.png`, `shots/expanded-17-notice.png` vs `docs/guides/design/17-notice.html`

| Item                                                                               | Compact | Expanded | Note |
| ---------------------------------------------------------------------------------- | ------- | -------- | ---- |
| Bottom right above the footer, _Copied to clipboard_, a `⌘V` hint on a grey keycap | ✓       | ✓        |      |

## Sign-off

Approved by owner on 2026-10-14.
