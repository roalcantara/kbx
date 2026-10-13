# Design system brainstorm, 2026-10-12

The approved pages of the Plan 8 design sessions with the owner, kept as the visual record behind the decisions.
Each decision is written out, with its reasons, in the comments of its task in [TODO.md](../../../../TODO.md) (Plan 8
tasks). Open a page in a browser; fonts and icons load from Google Fonts and unpkg.

The owner's Stitch screens are the reference; where a page deviates, it says why. Pages after 01 load `01-list.html`
unchanged and add only their new part with a script, so the list they show is the approved one. 08–09 (submenus) are in
02-actions.html; 15–16 (bookmark media) in 04-entry-template.html; 10 (no index) follows 03.

| Page                                             | Screen              | What it settled                                                                                                                             |
| ------------------------------------------------ | ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| [01-list.html](01-list.html)                     | 01 List             | The list in the owner's design: Catppuccin Macchiato, 14 pt window, hairlines, small type, tiles, the mauve bar, ⌘1–⌘9 row shortcuts        |
| [01-chips.html](01-chips.html)                   | 01 List             | Every chip and keycap 20 pt; fills as drawn (option A)                                                                                      |
| [02-actions.html](02-actions.html)               | 02 Actions          | One action component (Raycast layout); primary/secondary per entry type, mapped by each screen onto ↵ / ⌘↵ / ⇧⌘↵; Links… and More… submenus |
| [03-empty.html](03-empty.html)                   | 03 Empty            | Raycast's empty view as is; Refresh ↵ (also covers 10 No index)                                                                             |
| [04-entry-template.html](04-entry-template.html) | 04–06 Split, Detail | Template v3: Header (row icon + title), Hero by type, Quote (description), Body (notes); bookmarks with and without a hero                  |
| [04-command-cards.html](04-command-cards.html)   | 04–06               | The command card on real commands: fmt-rs breaks at operators, wrap at spaces with a hanging indent, operators glued, ligatures off         |
| [04-spike-screens.html](04-spike-screens.html)   | 04–06               | Real GPUI renders of the panel (a throwaway spike on kb's GPUI Kit 0.7.1): split and detail, command, bookmark, cheat sheet                 |
| [07-filter.html](07-filter.html)                 | 07 Filter           | The action component reused, its search at the bottom (Raycast); ✓ on chosen rows only; the pill names the filters; Toggle ↵, Only ⌘↵       |
| [11-14-refresh.html](11-14-refresh.html)         | 11–14 Refresh       | Running, stopped, failed as 03's empty view; the report as 01's rows; Stop esc, Back ↵, Retry ↵, Configure ⌘↵                               |

`requested-*.png` are the owner's Stitch screens the proposals were compared against; `spike-*.png` are the spike's
screenshots.
