---
# The design tokens: the only place a design value is written (spec D18). The app reads this front matter at start
# (crates/kb-app/src/ui/design.rs). Sizes are points.
# The theme kb uses when config.toml names none (spec 2026-10-19-themes). Colours live in theme files:
# crates/kb-app/assets/themes/<id>.yaml, seeded into ~/.config/kb/themes/.
theme: kb-native
typography:
  ui: Inter
  mono: JetBrains Mono
  roles:
    search: { family: ui, size: 15, weight: 400 }
    title: { family: ui, size: 13, weight: 500 }
    description: { family: ui, size: 12, weight: 400 }
    label: { family: ui, size: 12, weight: 500 }
    section: { family: ui, size: 10, weight: 600 }
    footer: { family: ui, size: 11, weight: 400 }
    chip: { family: ui, size: 11, weight: 600 }
    count: { family: mono, size: 11, weight: 400 }
    keycap: { family: ui, size: 10, weight: 400, line_height: 16 }
    heading: { family: ui, size: 15, weight: 600, line_height: 24 }
    quote: { family: ui, size: 14, weight: 400, line_height: 21, italic: true }
    body: { family: ui, size: 13, weight: 400, line_height: 20 }
    notes_h1: { family: ui, size: 15, weight: 600 }
    notes_h2: { family: ui, size: 14, weight: 600 }
    notes_h3: { family: ui, size: 13, weight: 600 }
    card_split: { family: mono, size: 12, weight: 400, line_height: 20 }
    card_detail: { family: mono, size: 13, weight: 400, line_height: 22 }
    link_title: { family: ui, size: 13, weight: 500 }
    link_url: { family: mono, size: 12, weight: 400 }
    link_hero: { family: mono, size: 13, weight: 500 }
    empty_title: { family: ui, size: 15, weight: 500 }
    empty_text: { family: ui, size: 13, weight: 400 }
    notice: { family: ui, size: 13, weight: 500 }
rounded:
  window: 14
  row: 6
  chip: 4
  tile: 4
  pill: 6
  popover: 14
  card: 10
  hero: 12
  favicon: 3
spacing:
  scale: { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16, xl: 24, xxl: 32 }
  header: { gap: 12 }
  list: { pad: 8, gap: 4 }
  row: { pad_x: 12, gap: 12, text_gap: 2, trailing_gap: 12, tags_gap: 8 }
  pane: { split_pad_y: 16, detail_pad_y: 24, gap: 12, header_gap: 8 }
  footer: { gap: 16, item_gap: 8, left_gap: 8 }
  panel: { pad: 4, row_pad_x: 8, row_gap: 8, section_pad_top: 8, section_pad_x: 8, section_pad_bottom: 4, rule_margin: 4, search_pad_x: 12, search_gap: 8, crumb_pad_x: 12, crumb_gap: 4, swatch_gap: 2 }
  pill: { pad_x: 8, pad_y: 4, gap: 8 }
  chip: { pad_x: 8, link_pad_x: 8, link_gap: 4 }
  keycap: { gap: 2 }
  empty: { pad_x: 16, icon_gap: 12, title_gap: 4, bar_gap: 8, summary_pad_x: 12, summary_pad_top: 8, summary_pad_bottom: 4 }
  quote: { pad_left: 12, pad_y: 2 }
  card: { pad_x: 12, pad_y: 12, pad_right: 40, copy_inset: 8 }
  link_hero: { pad_x: 12, pad_y: 8, gap: 8 }
  links: { gap: 4, label_gap: 4, label_pad_top: 2, label_pad_bottom: 4, row_pad_x: 8, row_gap: 8 }
  notice: { gap: 12, hint_gap: 8, bottom: 48 }
components:
  window: { width: 860, height: 520, expanded_share: 0.85 }
  header: { height: 56, icon: 20, pill_icon: 15, pill_max_width: 220, pill_min_width: 160 }
  row: { height: 56, bar_width: 2.5, bar_inset: 6, glow: 8, glow_alpha: 0.6, max_tags: 3, key_slot: 42 }
  chip: { height: 20, icon: 14, link_icon: 12, link_outline_alpha: 0.45 }
  keycap: { height: 20, width: 20, word_width: 28 }
  tile: { size: 24, icon: 16, small: 20, small_icon: 13, mini: 18, mini_icon: 12, alpha: 0.15 }
  favicon: { size: 16 }
  panel: { width: 220, filter_width: 230, row: 28, search: 36, crumb: 26, max_height: 300, chevron: 14, search_icon: 16, crumb_icon: 12, check: 14, swatch: 8 }
  footer: { height: 40, dot: 6, dot_glow: 6, dot_glow_alpha: 0.7 }
  split: { list: 0.34 }
  empty: { icon: 56, text_max_width: 420, bar_width: 260, bar_height: 4 }
  detail: { scroll_step: 60 }
  hero: { thumb_split: 150, thumb_detail: 250, link_icon: 14 }
  quote: { bar: 3 }
  links: { row: 28, url_chars: 32, title_share: 0.5 }
  notice: { width: 340, icon: 20, check: 13, ring_fill: 0.2, ring_border: 0.4 }
  pane: { fact_label: 72, tint_alpha: 0.12 }
  border: { hairline: 1 }
---

# Design system

How kb looks. The owner's Stitch screens are the reference, settled screen by screen into the **approved pages**
([docs/guides/design/](docs/guides/design/README.md)), which are the visual contract: where this text and a page
differ, the page wins and this text is fixed. [PRODUCT.md](PRODUCT.md) says what kb is for; Raycast is the reference
for behaviour. Read this file before any UI change.

Every value lives in one place:

| What                                   | Where                                                                                                           | Read by                                                               |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| Type, radii, spacing, sizes            | This file's front matter (the design tokens)                                                                    | the app (`ui/design.rs`)                                              |
| Colours                                | Theme files: [`crates/kb-app/assets/themes/`](crates/kb-app/assets/themes/), seeded into `~/.config/kb/themes/` | the app (`theme.rs`); the front matter names the default, `kb-native` |
| Entry types, actions, keys, words      | [`crates/kb-app/assets/catalog.toml`](crates/kb-app/assets/catalog.toml)                                        | the app (`catalog.rs`); `~/.config/kb/config.toml` overrides it       |
| GPUI Kit's theme                       | Built from the active theme by `theme::gpui`                                                                    | GPUI Kit's own components                                             |
| What reaches what when a value changes | [`tools/design/CHANGE-MAP.md`](tools/design/CHANGE-MAP.md)                                                      | whoever changes a value                                               |

`theme::legibility` checks every shipped theme's contrast in a test; `mise run design:validate` scores the comparison
palettes; `mise run design:shots` captures the app on the showcase corpus for comparison with the approved pages.

## 1. Visual Theme & Atmosphere

A calm, dark launcher in Catppuccin Macchiato: a navy panel with 14 pt corners floating over the desktop, flat
surfaces close in tone, hairlines only under the header and above the footer. Hierarchy comes from type weight, a
selected row's fill and its mauve bar, never from boxes. Colour is reserved for meaning: an entry's type (its tile),
a tag's identity (its icon), and state (the status dot). Always dark: kb has one approved palette.

## 2. Color Palette & Roles

The roles, by name. Their values come from the active theme (below); kb-native's are kb's own:

| Role       | Job                                                                                                                                                                             |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `window`   | The panel                                                                                                                                                                       |
| `mantle`   | Live chips and keycaps; popovers; command cards and the link hero                                                                                                               |
| `crust`    | The notice                                                                                                                                                                      |
| `selected` | The selected row and panel row; popover and card borders                                                                                                                        |
| `hover`    | A hovered row                                                                                                                                                                   |
| `badge`    | Rest chips; the filter pill; keycaps on the darker popover and notice surfaces                                                                                                  |
| `code`     | Code blocks in the notes                                                                                                                                                        |
| `edge`     | Hairlines (faint at 10 %)                                                                                                                                                       |
| `ring`     | The panel's 1 pt outline (text at 8 %)                                                                                                                                          |
| `text`     | Titles, the query, values                                                                                                                                                       |
| `muted`    | Descriptions, labels, live chip text, keycap text                                                                                                                               |
| `faint`    | Tertiary text: placeholders, counts, section labels, URLs, rest link chips; icons in the header                                                                                 |
| accents    | `mauve` (selection bar, caret, Cheat Sheets), `blue` (Bookmarks, links), `green` (Commands, ready), `pink` (Tasks), `peach`, `yellow` (Skills), `red` (the quote bar, failures) |

- **Type tiles:** a type's accent at 15 % behind its icon in the accent (24 pt in rows, 20 in the footer and the pane
  header, 18 in the filter). The type → accent pairs are the catalog's (`tint`).
- **Tags:** a tag with an image in the tag-images folder is an icon chip, the image tinted in the tag's accent (the
  Raycast hash of its lowercased name over blue, peach, green, mauve, pink, yellow, red); any other tag is a grey
  `#tag` pill. Colour is never the only cue: pills carry the name, icon chips a tooltip.
- **Chip state** (D3): _live_ (mantle, muted text) on every keycap and on the selected row's chips; _rest_ (badge)
  elsewhere, tag text muted, the link count faint.

**Themes** (spec 2026-10-19-themes): the colours come from a theme file, a base24 (or base16) scheme in
`~/.config/kb/themes/`, where kb seeds its eight (kb-native, Neon Dusk, Archwave, Azure Glow, Aetheria, Monokai,
Sapphire, Nagai Poolside; page 19) and never overwrites one. kb maps a scheme onto the roles by each colour's base24
meaning (base00 window, base01 badge, base02 selected, base03 faint, base04 muted, base05 text, base10 and base11 the
darker layers, base08–base0E the accents, base0E the main accent `mauve`); a file's `kb:` section pins any role
exactly. config.toml names the theme (`theme = "neon-dusk"`, kb-native without one) and may override single roles in
`[colors]`. More… → Theme… lists the files and previews each as the selection moves; ↵ keeps and saves one, Esc
restores the one in use. A theme below the legibility rules still loads, with a notice saying why.

## 3. Typography Rules

Inter for the interface and keycaps, JetBrains Mono for commands, code, counts and URLs; both bundled
(`crates/kb-app/assets/fonts`), ligatures off wherever mono text is drawn, so what you read is what you copy. Small,
dense type: titles 13 medium, descriptions 12, labels 12 medium, the footer 11, section labels 10 semibold uppercase;
the query 15; the pane's heading 15 semibold; the quote 14 italic. Command titles are mono (the catalog's
`mono_title`). The type roles are the front matter's `typography.roles`.

**Spacing** follows the GPUI Kit Design Guides' ramp, `spacing.scale`: 2 · 4 · 8 · 12 · 16 · 24 · 32. A gap is chosen
by what it separates: parts of one control 4, related controls 8 (a label and its keys), one content group 12,
separate groups 16 (footer items), sections 24, region boundaries 32; the keys of one shortcut sit 2 apart. The window
has one content edge, `content_inset()` = the list's padding 8 + a row's 12 = 20: the header, rows, footer, panes and
notice all start and end on it, so the filter pill, the rows' keys and the footer's keys line up. A control with its own fill (a
selected row, the filter pill) lets the fill reach into the margin by its padding; its contents stay on the edge. The action panel,
a separate surface, has its own 12 (4 + 8). A test fails on any spacing value off the ramp, except two composite
clearances (the command card's room for its copy button, 40; the notice above the footer, 48).

## 4. Component Stylings

Each component is one file in `crates/kb-app/src/ui/`, and matches a part of an approved page:

- **Keycap** (`keycap.rs`; spec 2026-10-19 D1): one 20 × 20 cap per key, 2 apart (`⌘` `K`, `⇧` `⌘` `,`); a word key
  (`esc`) 28 wide; Inter 10 / 400; live or rest.
- **Chip** (`chip.rs`; 01-list, 01-chips): 20 tall, 4 pt corners: tag pills, tag icon chips, task chips; live or rest. The `↗ n`
  link count is a round pill with a faint outline and no fill, never a key's shape.
- **Tile** (`tile.rs`; 01-list): the tinted type tile, or a bookmark's favicon with the tile as its fallback.
- **Row** (`row.rs`; 01-list): 56 tall, 4 apart, 6 pt corners; tile, title over description, tags, link pill, then a
  fixed key slot (`↵` selected, `⌘ 1`…`⌘ 9` the first nine rows, empty after); the selected row on `selected` with a 2.5 pt mauve bar and its glow.
- **Header** (`header.rs`; 01-list, 07-filter): 56 tall, a hairline under it; the magnifier on the list, the back
  chevron elsewhere; the query at 15; the filter pill (sliders, the filter's name, `⌘ P`),
  never narrower than `pill_min_width`.
- **Footer** (`footer.rs`; 01-list, 04-entry-template): 40 tall, a hairline over it; left the status line on the
  list (`● Ready • N results`) or, with a pane or the detail screen, the entry's tile, file and `⌥⌘C`; right the
  screen's ↵ and ⌘↵ keys and _Actions ⌘K_ (never ⇧⌘↵, which the action panel lists), 16 apart, each label 8 from its
  keys; counts in tabular digits.
- **Action panel** (`action_panel.rs`; 02-actions, 07-filter): mantle, 14 pt corners, 220 wide (the filter 230),
  28 pt rows, search at the bottom, sections by a hairline, a `‹` header on submenus.
- **Empty view** (`empty_view.rs`; 03-empty, 11-14-refresh): one 56 pt icon, a 15 pt title, a sentence; the query in
  mauve mono.
- **Report pane** (`report_pane.rs`; 12-report-pane): the row's tile, path in mono and status chip; the error as the
  red-bar quote, skipped entries and warnings in a yellow one; the lines around an error on mantle, the failing line
  with a red bar and tint; skipped keys as a list; facts as label and value (`Location`, `File`, `Entries`, `Exit`,
  `Stderr`, `Index`). The split and the detail screen draw it, as the entry view.
- **Entry view** (`entry_view.rs`; 04-entry-template, 05-detail-links): Header (icon, title, tags), Hero by type,
  Quote (the description, red bar), Body (the notes, then the Links list); the same component in the split (34 %
  list) and on the detail screen.
- **Command card** (`command_card.rs`; 04-command-cards): mantle, a `selected` border, 10 pt corners; fmt-rs breaks
  at operators, kb wraps at spaces with a hanging indent; GPUI Kit's bash colours; a copy button in the corner.
- **Notice** (`notice.rs`; 01-list): bottom right above the footer, _Copied to clipboard_ with a `⌘V` hint.

## 5. Layout Principles

Compact 860 × 520, expanded to 85 % of the display (⇧⌘F), keeping its centre. The header and footer insets are
24; the list is inset 8; rows inset their content 14 (10 in the split). The split gives the list 34 %. The pane
pads 18 × 24 in the split and 22 × 32 on the detail screen, 12 between its slots. Search fields sit at the bottom of
every panel (Raycast).

## 6. Depth & Elevation

Flat. Only three things float: the panel itself (the system's window shadow and a 1 pt ring), popovers (GPUI Kit's
popover shadow) and the notice. The selected row's bar glows mauve; nothing else has a shadow.

## Adaptations

Where kb departs from the owner's Stitch screens or from Raycast, and why:

- **Back chevron instead of the magnifier** on the detail and refresh screens: kb needs a way back (01-list, note 1).
- **No `deploy/api` chip**: kb entries have no field it could come from (01-list, note 4).
- **Lucide icons** for Material ones: the set GPUI Kit ships.
- **Type colours as the owner drew them**: Commands green, Cheat Sheets mauve, Bookmarks blue, Tasks pink.
- **Four entry types** (D6): any other type is ignored by the app; the engine still indexes it.
- **No metadata column** (D20): every attribute has a home (tile, quote, hero, header tags, footer file).
- **Always dark** (D1).

## Evidence

`tools/design/shots/` holds the app captured on the showcase corpus by `mise run design:shots`, one PNG per screen,
compact and expanded; `tools/design/CHECKLISTS.md` records each screen's checklist against its approved page.
