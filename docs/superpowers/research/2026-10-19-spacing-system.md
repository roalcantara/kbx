# Which design system should own kb's spacing and alignment?

- Date: 2026-10-19
- Question: adopt an existing, documented design system for spacing, padding and alignment, so kb stops hand-tuning
  them screen by screen.
- Trigger: the owner's screenshot of the list (2026-10-19): the filter pill, the rows' keys and the footer's keys end
  on three different right edges, and the footer's gaps look uneven.

## Recommendation

**Adopt the GPUI Kit Design Guides' spacing system** (`~/.agents/skills/gpui-kit-design-guides`, a verbatim copy of
[gpui-kit.com/docs/design-guides.md](https://gpui-kit.com/docs/design-guides.md)): its seven-step ramp, its spatial
grammar and its alignment spines. Keep kb's colours, type and component sizes; change only spacing and alignment.

Why this one and not a better-known system:

1. **It is already binding.** AGENTS.md requires the GPUI Kit Design Guides for UI work; the guide calls itself "a
   requirement, not inspiration". kb follows it for components but never adopted its spacing; this closes that gap
   instead of adding a second authority.
2. **It answers exactly the failures we see.** It has explicit rules for a shared content inset across header, list
   and footer, fixed trailing lanes in rows, bar spacing, and "a one-rendered-pixel difference is a defect"
   (section Alignment details). The other systems give a ramp but little or no rule for these.
3. **It is the toolkit we render with.** Its ramp is GPUI Kit's own default spacing scale; kb components built from
   GPUI Kit parts (popover, list, input) already use it, so adopting it makes our custom parts match the stock ones.
4. **It agrees with the big systems.** Its ramp (2, 4, 8, 12, 16, 24, 32) is a subset of Fluent 2's ramp, on the same
   4 px base Apple and Microsoft use. Nothing we adopt conflicts with them.

## What the guide prescribes

**Ramp** (named by relationship, not by size):

| Token | Size | Use                                                              |
| ----- | ---- | ---------------------------------------------------------------- |
| `xxs` | 2    | optical correction; e.g. the gap between one shortcut's keycaps  |
| `xs`  | 4    | parts of one control: icon/label, title/description              |
| `sm`  | 8    | closely related controls: a label and its keycap, dialog actions |
| `md`  | 12   | one content group: a row's columns, notification columns         |
| `lg`  | 16   | separate groups in one section: panel padding, form groups       |
| `xl`  | 24   | separate sections                                                |
| `xxl` | 32   | major region boundaries: empty-state breathing room              |

**Rules** (quoted or closely paraphrased from the guide):

- Inside before outside: a component's padding belongs to it; the gap between components belongs to the parent.
- Do not double padding: a row in an already padded list does not add a full second inset.
- Give sibling regions a shared content inset: heading, toolbar, list row, empty state and footer at the same level
  share their leading and trailing edges.
- Repeated rows keep icons, labels, badges and trailing actions on stable columns; trailing actions sit in
  fixed-width lanes.
- Keep horizontal bands coherent: items in a toolbar or status bar share one centre line and one gap.
- `justify_between` only when two sides truly own opposite edges.
- Density: medium by default; compact for menus, toolbars and repeated professional data. A launcher's list and
  popovers are compact.
- Verify by measured bounds, not screenshots: equal edges and gaps must resolve from the same token.

## The candidates

| System                                                  | Spacing ramp                                                                             | Alignment / layout rules                                                                                                  | Fit for kb                                                                                                                 |
| ------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| **GPUI Kit Design Guides**                              | 2 · 4 · 8 · 12 · 16 · 24 · 32, semantic names                                            | Detailed: shared insets, lanes, bands, measurement as a quality invariant                                                 | **Best**: already required, same toolkit, rules match our defects                                                          |
| Fluent 2 (Microsoft; PowerToys Command Palette, WinUI)  | 4 px base; 0 · 2 · 4 · 6 · 8 · 10 · 12 · 16 · 20 · 24 · 28 · 32 · 36 · 40 · 48 · 52 · 56 | Generic (edges align, gutters multiple of the base); WinUI adds 8 between buttons, 12 control/label, 16 surface/edge text | Good numbers, too many steps (6, 10, 20 invite the drift we have); Windows idiom                                           |
| Apple HIG (macOS)                                       | No published numeric ramp; layout guides and margins                                     | Layout guides, safe areas, readable widths                                                                                | Right platform, but nothing to adopt as tokens (8 pt grid / 20 pt margins come only from unverified third-party summaries) |
| GNOME HIG                                               | 6 px base: 6 · 12 · 18 (GNOME 2 era)                                                     | Group spacing, indentation                                                                                                | Old; 6 px base conflicts with GPUI Kit's 4 px base                                                                         |
| GitHub Primer                                           | Not verified in this pass                                                                | Not verified                                                                                                              | Web product system; no gain over the guide we already follow                                                               |
| Launchers (PowerToys Command Palette, Vicinae, Raycast) | Not published (code only, or closed)                                                     | Not published                                                                                                             | Useful as visual references (owner's screenshots), not as a spec                                                           |

## How far kb is from it today

DESIGN.md has 68 spacing values; **43 are on the ramp, 25 are not**: 6 (×7), 10 (×11), 14 (×2), 18, 20, 22, 40, 48.
The visible defects follow from three of them:

| Region    | Trailing edge today                       | Why                                                              |
| --------- | ----------------------------------------- | ---------------------------------------------------------------- |
| Header    | 24 from the window edge                   | `header.pad_x: 24`                                               |
| List rows | 22 (8 + 14)                               | `list.pad: 8` + `row.pad_x: 14`: two owners, off-ramp 14         |
| Footer    | 24, plus 56 pt label slots and gaps of 20 | `footer.pad_right: 24`, `label_slot: 56` (2026-10-19), `gap: 20` |

Under the guide, these become one shared inset and one bar gap from the ramp; for example, list 8 + row 12 = **20**
for every region, and footer items **16** apart (`lg`) with label and keycap **8** apart (`sm`), so the keycaps of
the header, rows and footer end on one line. The exact choice belongs to the spec, shown side by side in the
companion before anything changes.

## What adopting it means

1. **DESIGN.md gains the ramp** (`spacing.scale: { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16, xl: 24, xxl: 32 }`); every
   other spacing token takes one of its values; `design:validate` fails an off-ramp spacing value, with a short,
   documented list of optical exceptions (e.g. the 2.5 pt selection bar).
2. **One content inset** shared by header, list rows, empty view, report and footer; the action panel and filter
   keep their own (compact) inset as separate surfaces.
3. **The footer becomes a bar**: one gap between items, one gap between a label and its caps. The fixed label slots
   added on 2026-10-19 go if, once the footer no longer adds or drops items (D9), only one label still changes (the
   primary action's Open/Copy); the spec decides whether that one change deserves a fixed lane.
4. **A spine check**: a test that measures the trailing edges of the header pill, a row's key slot and the footer's
   last keycap and asserts they are equal (GPUI's laid-out bounds), so "aligned" stops depending on screenshots.

Changed decisions, on the owner's terms ("not written in stone, open to improvement upon proven benefits"): the 25
off-ramp values, the footer's 56 pt label slots and 20 pt gaps, and the per-surface insets. Colours, type, radii,
heights, keycaps and the link pill stay.

## Not done in this pass

- Primer's ramp and Flow Launcher, Ulauncher and Sol were not checked (the search found nothing usable); they would
  not change the recommendation, since none is the toolkit kb renders with.
- No values were measured from PowerToys Command Palette's source.

## Sources

- GPUI Kit Design Guides, local copy `~/.agents/skills/gpui-kit-design-guides/references/design-guides.md`
  (sections "Radius, spacing, and density", "Spatial grammar", "Alignment details", "Density tiers");
  [gpui-kit.com/docs/design-guides.md](https://gpui-kit.com/docs/design-guides.md)
- [Fluent 2 — Layout](https://fluent2.microsoft.design/layout) (spacing ramp, 4 px base, alignment)
- [Microsoft Learn — Content layout and spacing (Windows apps)](https://learn.microsoft.com/windows/apps/design/style/spacing)
- [PowerToys 0.93 release notes (Command Palette)](https://devblogs.microsoft.com/commandline/powertoys-0-93-is-here-faster-command-palette-new-dashboard-ux-and-more/)
- [Apple HIG — Layout](https://developer-rno.apple.com/design/human-interface-guidelines/foundations/layout)
- [GNOME HIG — Layout (wiki)](https://wiki.gnome.org/Design/HIG/Planning/Layout)
- [Vicinae docs](https://docs.vicinae.com/)
