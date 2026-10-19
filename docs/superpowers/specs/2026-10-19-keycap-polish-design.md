# Keycap polish — design

- Status: approved
- Date: 2026-10-19
- Context: the final UI polish before the first release candidate. Decided with the owner in the brainstorming
  companion (`keycaps-v3.html`, option C with 9 pt glyphs and 2 pt gaps).

## Goal

Every shortcut in the app is drawn the same way and nothing on screen moves when the selection, an entry type or a
count changes. Keys read clearly, including compound ones (`⌘ ↵`), and nothing that is not a key looks like one.

## Problems

1. **Keycaps differ in width.** `keycap_width` buckets caps by glyph count (28 / 36 / 42 / 48, literals in
   `ui/keycap.rs`), so the selected row's `↵` (28) is narrower than `⌘2` (36). Moving the selection drags the tags
   and link chip beside it sideways.
2. **Compound keys are hard to read.** One cap holds every key in JetBrains Mono 11, where ⌘ and ↵ are small and
   low; `⌘↵` reads as one shape.
3. **The link count looks like a key.** `↗ 2` has the keycap's fill, height and corners.
4. **The footer's keys shift.** Its right side shows the selected entry's own action label (Open on a bookmark, Copy
   on a command); the labels differ in width, so the items slide as the selection moves between types.
5. **The result count jitters.** Proportional digits change the status line's width as the user types.
6. **The filter pill resizes.** "All Entries" and "2 filters" differ in width, moving the search field's end.

## Decisions

**D1. One cap per key.** A shortcut is a row of caps, one per key, in macOS order (⌃ ⌥ ⇧ ⌘, then the key): `cmd-enter`
is `⌘` `↵`; `shift-cmd-,` is `⇧` `⌘` `,`. Every cap is 20 × 20; a word key (`esc`) is 28 wide. Caps of one shortcut
are 2 apart; the tight gap groups them (everything else around them is 6 or more apart).

**D2. Glyphs in Inter 9 / 400.** The keycap text role becomes `{ family: ui, size: 9, weight: 400, line_height: 16 }`,
centred, in the state's text colour (live: muted; rest: faint). Inter draws the system's own ⌘ ⇧ ⌥ ↵.

**D3. Rows reserve a two-cap slot.** A list row's trailing keycap sits in a right-aligned slot as wide as two caps and
their gap (42), so `↵` and `⌘ 3` take the same room and the chips beside them never move.

**D4. The link count is an outlined pill.** `link_chip` loses its fill: a round pill (full radius) with a 1 pt outline
in `faint` at 45 % alpha, faint text and icon, the same in the live and rest states. Only keys have filled square caps.

**D5. Footer labels sit in fixed slots.** Each key item's label on the footer's right side is at least
`footer.label_slot` wide and right-aligned against its caps, so the caps stay put when the label changes (Open ↔
Copy). A label longer than the slot (a catalog override) widens its item rather than being cut.

**D6. Counts use tabular digits.** The status line's count, the refresh report's counts and the link pill's number
use the `tnum` font feature.

**D7. The filter pill has a minimum width.** `header.pill_min_width` fits "All Entries" and its `⌘ P`, so the pill
keeps its width when it switches to "n filters".

**D8. Every value is a design token.** No literal sizes in the components (the 8g rule): the caps, gaps, slots and
widths below live in DESIGN.md's front matter and `ui/design.rs`.

**D9. The list footer shows ↵, ⌘↵ and ⌘K only.** A screen key bound to `shift-cmd-enter` is left out of the footer
(owner, 2026-10-19): an entry type with a secondary action (bookmarks) would otherwise add a footer item that a type
without one (commands) removes, moving every item beside it. ⇧⌘↵ still runs the secondary action and is still listed
in the action panel, as Raycast does.

## Scope

In scope:

- `ui/keycap.rs`: `caps` keeps spelling a shortcut as one string (`⌘↵`), which the catalog, the panel and the tests
  compare; `keys` splits that string into keys; `keycap` draws one cap per key; the width buckets go.
- Every caller: header (filter pill), list rows, footer (key items and the entry status's copy-path keys), action
  panel rows, the entry view's link numbers, the notice's `⌘ V`.
- `ui/chip.rs`: the outlined link pill.
- `ui/footer.rs`: fixed label slots; tabular count. `search_view/footer.rs`: no `shift-cmd-enter` item (D9).
- `ui/header.rs`: the pill's minimum width.
- `sync_view.rs`: tabular counts in the report.
- DESIGN.md tokens and component notes, `ui/design.rs`, the generated theme, the design shots, CHANGE-MAP.md.

Out of scope:

- New shortcuts, or changing which keys an action has.
- Tag pills, tag images and type tiles.
- Layout changes beyond the slots and minimum width above.

## Tokens

| Token (DESIGN.md front matter)       | Value                                                                                                                         |
| ------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| `typography.roles.keycap`            | `{ family: ui, size: 9, weight: 400, line_height: 16 }` (was mono 11)                                                         |
| `components.keycap`                  | `{ height: 20, width: 20, word_width: 28 }`                                                                                   |
| `spacing.keycap`                     | `{ gap: 2 }` (replaces `pad_x: 6`)                                                                                            |
| `components.row.key_slot`            | `42` (two caps and their gap)                                                                                                 |
| `components.footer.label_slot`       | `56`: fits the widest shipped label, "Configure" (about 52 in Inter 11); checked on the design shots                          |
| `components.header.pill_min_width`   | `160`: padding 10 + 10, icon 15, gaps 8 + 8, "All Entries" (about 64 in Inter 12 / 500), caps 42; checked on the design shots |
| `components.chip.link_outline_alpha` | `0.45`                                                                                                                        |

The two text-dependent values are estimates from Inter's metrics; the design shots confirm them (no label wider than
its slot, the pill the same width with and without filters) and the plan adjusts them there if not.

## Behaviour

- `caps("cmd-enter") == ["⌘", "↵"]`; `caps("escape") == ["esc"]`; `caps("shift-cmd-,") == ["⇧", "⌘", ","]`;
  `caps("cmd-1") == ["⌘", "1"]`. A key that does not parse is one cap holding its text.
- `keycaps(keys, state)` draws the caps in a row 2 apart; a cap is 20 wide unless its key has more than one character
  (28).
- A list row's slot holds `↵` on the selected row and `⌘ n` on rows 1–9; rows from 10 on have an empty slot of the
  same width.
- The action panel's rows right-align their caps; shortcuts of one, two or three keys differ in width there by
  design (the column's right edge is what aligns).

## Tests and validation

- `caps` splits each shape above; `keycaps` gives every one-key cap the same width and a word key 28.
- The row slot is the same width for `↵` and `⌘ 9`, and for a row past the ninth.
- A footer item's width does not change between labels that fit its slot.
- UI test: moving the selection down the list leaves every other row's trailing width unchanged.
- The ast-grep literal-size rule passes on the changed components.
- `mise run design:validate`, `mise run design:theme`, `mise run design:shots`, then `mise run gate`.
- By hand on the packaged app: the list, the action panel, the footer on a bookmark and on a command, the filter pill
  with and without filters, and 9 pt glyphs on the built-in display and, if available, a non-Retina one.
