# Spacing on the ramp — design

- Status: draft for owner review
- Date: 2026-10-19
- Research: [2026-10-19-spacing-system.md](../research/2026-10-19-spacing-system.md)
- Decided with the owner in the companion (`spacing.html`, option B): one shared edge at 20; the keys' 2 pt gap kept.

## Goal

Every gap, padding and inset in kb comes from the GPUI Kit Design Guides' spacing ramp, and every region of the
window shares one leading and one trailing edge, so the filter pill, the rows' keys and the footer's keys end on one
line and nothing needs hand-tuning per screen.

## Decisions

**D1. The ramp.** DESIGN.md gains `spacing.scale: { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16, xl: 24, xxl: 32 }`, the
GPUI Kit Design Guides' scale. Every other spacing token takes one of these values. A gap is chosen by what it
separates (the guide's spatial grammar): parts of one control `xs`, related controls `sm`, one content group `md`,
separate groups `lg`, sections `xl`, region boundaries `xxl`.

**D2. One content inset, derived.** The window's content edge is `list.pad + row.pad_x` (8 + 12 = 20), computed in
`ui/design.rs` as `content_inset()`, never written as a number. The header, the footer, the side pane, the detail
screen and the notice's right edge use it, so their leading and trailing edges coincide with the rows' content.

**D3. The footer is one bar.** Items are `lg` (16) apart; a label and its keys `sm` (8) apart; the footer has no
separate entry-screen inset or gap. The fixed label slots (`components.footer.label_slot`, 2026-10-19) go: since
D9 of the keycap spec the footer neither adds nor drops items, and the one label that changes on the list (the
primary action, Open or Copy) differs by about a point.

**D4. The action panel keeps its own edge.** As a separate surface it uses `pad` 4 + `row_pad_x` 8 = 12, which its
search field, crumb and section labels share.

**D5. Rows use one padding in both layouts.** The split pane's list rows take the full rows' `pad_x` and `gap`
(12, 12); `row.split_pad_x` and `row.split_gap` go.

**D6. Two documented exceptions.** `card.pad_right` (40: room for the copy button) and `notice.bottom` (48: the
footer's 40 plus `sm`) are composite clearances, not gaps; they stay as written and are the only values a test
allows off the ramp. The keys' 2 pt gap is `xxs`, on the ramp.

**D7. A test keeps it so.** A `ui::design` unit test fails when a spacing value is off the ramp (except D6's two),
and asserts `content_inset()` is the rows' content edge.

## Scope

In scope: DESIGN.md's `spacing` section and the components that read it (`header.rs`, `footer.rs`, `row.rs`,
`entry_list.rs`, `search_view.rs`, `entry_view.rs`, `action_panel.rs`, `empty_view.rs`, `sync_view.rs`, `notice.rs`,
`command_card.rs`), `ui/design.rs`, DESIGN.md's prose, the shots.

Out of scope: colours, type, radii, component heights and sizes (`components.*` other than `footer.label_slot`),
keycaps and the link pill (2026-10-19 keycap spec), and the refresh report's behaviour (a later pass).

## Mapping

Every spacing value, today → on the ramp. Unchanged values are listed so the whole section is accounted for.

| Token                                                         | Today      | New                    | Why                                                      |
| ------------------------------------------------------------- | ---------- | ---------------------- | -------------------------------------------------------- |
| `header.pad_x`                                                | 24         | `content_inset()` (20) | D2: the header's edges are the rows' edges               |
| `header.gap`                                                  | 12         | 12                     |                                                          |
| `list.pad`, `list.gap`                                        | 8, 4       | 8, 4                   |                                                          |
| `row.pad_x`                                                   | 14         | 12 (`md`)              | the row's content group; with `list.pad` makes the inset |
| `row.gap`, `trailing_gap`                                     | 12         | 12                     |                                                          |
| `row.text_gap`                                                | 2          | 2                      |                                                          |
| `row.tags_gap`                                                | 6          | 8 (`sm`)               | related chips                                            |
| `row.split_pad_x`, `split_gap`                                | 10         | removed (D5)           | rows use `pad_x` and `gap`                               |
| `pane.split_pad_x`                                            | 24         | `content_inset()` (20) | D2                                                       |
| `pane.split_pad_y`                                            | 18         | 16 (`lg`)              |                                                          |
| `pane.detail_pad_x`                                           | 32         | `content_inset()` (20) | D2: the detail screen's text meets the header's chevron  |
| `pane.detail_pad_y`                                           | 22         | 24 (`xl`)              |                                                          |
| `pane.gap`                                                    | 12         | 12                     |                                                          |
| `pane.header_gap`                                             | 10         | 8 (`sm`)               |                                                          |
| `footer.pad_left`, `pad_right`                                | 24         | `content_inset()` (20) | D2                                                       |
| `footer.entry_pad_right`                                      | 16         | removed (D3)           | one bar                                                  |
| `footer.gap`                                                  | 20         | 16 (`lg`)              | separate items                                           |
| `footer.entry_gap`                                            | 16         | removed (D3)           | one bar                                                  |
| `footer.item_gap`                                             | 6          | 8 (`sm`)               | a label and its keys                                     |
| `footer.left_gap`                                             | 8          | 8                      |                                                          |
| `components.footer.label_slot`                                | 56         | removed (D3)           |                                                          |
| `panel.pad`                                                   | 6          | 4 (`xs`)               | D4                                                       |
| `panel.row_pad_x`                                             | 10         | 8 (`sm`)               | D4                                                       |
| `panel.row_gap`                                               | 10         | 8 (`sm`)               | icon, label, keys                                        |
| `panel.section_pad_top`                                       | 6          | 8 (`sm`)               |                                                          |
| `panel.section_pad_x`                                         | 10         | 8 (`sm`)               | D4: 4 + 8 = 12                                           |
| `panel.section_pad_bottom`, `rule_margin`                     | 4          | 4                      |                                                          |
| `panel.search_pad_x`, `crumb_pad_x`                           | 12         | 12                     | D4                                                       |
| `panel.search_gap`                                            | 8          | 8                      |                                                          |
| `panel.crumb_gap`                                             | 6          | 4 (`xs`)               | icon and label                                           |
| `pill.pad_x`                                                  | 10         | 8 (`sm`)               |                                                          |
| `pill.pad_y`, `pill.gap`                                      | 4, 8       | 4, 8                   |                                                          |
| `chip.*`                                                      | 8, 8, 4    | unchanged              |                                                          |
| `keycap.gap`                                                  | 2          | 2 (`xxs`)              | owner: keep                                              |
| `empty.pad_x`, `icon_gap`, `title_gap`                        | 16, 12, 4  | unchanged              |                                                          |
| `empty.bar_gap`                                               | 10         | 8 (`sm`)               |                                                          |
| `empty.summary_pad_x`                                         | 16         | 12 (`md`)              | the report's summary meets the rows' content edge        |
| `empty.summary_pad_top`                                       | 10         | 8 (`sm`)               |                                                          |
| `empty.summary_pad_bottom`                                    | 4          | 4                      |                                                          |
| `quote.*`                                                     | 12, 2      | unchanged              |                                                          |
| `card.pad_x`                                                  | 14         | 12 (`md`)              |                                                          |
| `card.pad_y`, `copy_inset`                                    | 12, 8      | unchanged              |                                                          |
| `card.pad_right`                                              | 40         | 40                     | D6 exception                                             |
| `link_hero.pad_x`, `gap`                                      | 12, 8      | unchanged              |                                                          |
| `link_hero.pad_y`                                             | 10         | 8 (`sm`)               |                                                          |
| `links.gap`, `label_pad_top`, `label_pad_bottom`, `row_pad_x` | 4, 2, 4, 8 | unchanged              |                                                          |
| `links.label_gap`                                             | 6          | 4 (`xs`)               |                                                          |
| `links.row_gap`                                               | 10         | 8 (`sm`)               |                                                          |
| `notice.gap`                                                  | 12         | 12                     |                                                          |
| `notice.hint_gap`                                             | 6          | 8 (`sm`)               | a label and its keys, as in the footer                   |
| `notice.right`                                                | 16         | `content_inset()` (20) | D2: the notice's edge is the footer's                    |
| `notice.bottom`                                               | 48         | 48                     | D6 exception                                             |

## Tests and validation

- `ui::design`: every value under `spacing` (except `scale` itself and D6's two) is a value of `spacing.scale`;
  `content_inset()` equals `list.pad + row.pad_x`.
- The existing unit and UI tests pass; tests that name removed tokens are updated.
- `mise run design:validate`, `mise run design:shots` (with a Skills fixture so the shots show only the showcase:
  the shots task's config sets `skills_command` to a script printing `[]`), the owner's look at the shots, then
  `mise run gate`.
