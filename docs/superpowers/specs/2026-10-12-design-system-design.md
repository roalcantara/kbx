# The new design system (Plan 8) — design

- Status: draft, for the owner's review
- Date: 2026-10-12 (rewritten after the design sessions; supersedes the draft of `27918b2`)
- Branch: `feat/design-system`
- Visual contract: [docs/superpowers/brainstorm/2026-10-12-design-system/](../brainstorm/2026-10-12-design-system/README.md)
- Decisions, with their reasons: the Plan 8 tasks' comments in [TODO.md](../../../TODO.md)
- Plan: [2026-10-12-kb-native-08-design-system.md](../plans/2026-10-12-kb-native-08-design-system.md)

## Goal

kb takes the design the owner brought (the Stitch screens in `docs/guides/design/`), as settled screen by screen with
the owner. At the end, the app matches the approved pages on every screen, every design value and every action lives in one
place (DESIGN.md's front matter for how it looks, the catalog for what it offers), the documents say so, and committed
screenshots of the real app prove it.

## How this was decided

- **The owner's Stitch screens are the reference**, not the DESIGN.md text Stitch generated with them; where kb departs
  from a screen, the approved page says why.
- **Buy over build.** Behaviour copies Raycast wherever Raycast has an answer (action panel, primary and secondary
  actions, empty view, submenus); components and crates are adopted before anything is written (fmt-rs, GPUI Kit's
  highlighter, Clipboard and Notification; a terminal view later).
- Each screen was approved in the brainstorming companion against the owner's screen, and every page after the first is
  the approved 01 list's own file plus only its new part, so every page shows the same list. A throwaway GPUI spike
  rendered the panel on kb's own GPUI Kit to check the HTML against the real thing (`04-spike-screens.html`).
- Before this spec, the owner weighed adopting massCode or porting it to GPUI; kb was kept (TODO.md, `KhLt`).

## Decisions

| #   | Decision                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        | Page                               |
| --- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| D1  | **Look:** Catppuccin Macchiato; window corners 14; hairlines under the header and above the footer; Inter for the interface, JetBrains Mono for commands and code (both bundled); small type (titles 13, descriptions 12, footer 11); rows 56, 4 apart, corners 6; always dark.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | 01-list                            |
| D2  | **Chips and keycaps:** every chip and keycap is 20 high. One keycap component: one cap per shortcut (`⌘K`, `⇧⌘,`), JetBrains Mono 11, muted text on mantle; on the darker popover and notice surfaces, resting keycaps take the grey fill.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | 01-list, 01-chips                  |
| D3  | **Chip state** decides fill and text: _live_ (mantle, muted text) on every keycap and on the selected row's chips; _rest_ (grey `#2E3248`) elsewhere, tags muted, the link count faint.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | 01-list                            |
| D4  | **Header:** the magnifier on the list, the back chevron on the detail and refresh screens; the filter pill (sliders icon, the filter's name, `⌘P`).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | 01-list, 07-filter                 |
| D5  | **Rows:** tinted type tile (or the bookmark's favicon); commands' titles in JetBrains Mono; the description under the title; then tag icon chips and grey `#tag` pills, the `↗ n` link chip, and a keycap: `↵` on the selected row, `⌘1`…`⌘9` on the others (⌘n runs row n's primary action).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | 01-list                            |
| D6  | **Entry types:** Bookmarks, Cheat Sheets, Commands, Tasks. Any other type is ignored by the app (list, filter, counts); kb-domain and the index stay unchanged, so the TypeScript goldens hold.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | 07-filter                          |
| D7  | **Entry actions** (catalog): Bookmark — primary _Open_ (default browser), secondary _Copy_ (its URL); Command, Cheat Sheet, Task — primary _Copy_ (their main content), no secondary. When Paste lands (its own task), it becomes Commands' and Cheat Sheets' primary and Copy their secondary.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | 02-actions                         |
| D8  | **Screen keys** (catalog): each screen maps the ladder ↵ / ⌘↵ / ⇧⌘↵. List: _Details_ ↵, entry primary ⌘↵, entry secondary ⇧⌘↵. Detail: entry primary ↵, entry secondary ⌘↵. The footer shows the screen's keys and _Actions ⌘K_.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | 02-actions                         |
| D9  | **Action panel** (⌘K, Raycast layout): rows 28, search at the bottom, sections by a hairline, at most two levels. Root: the screen's keyed actions, _Resize_ ⇧⌘F, _Links… ›_ (only when the entry has links; each link row opens with ↵ and copies with ⌘↵), _More… ›_. More…: COPY _Description_, _Notes_, _Path_ (absolute, ⌥⌘C) │ ADMINISTRATION _Edit_ ⌘O, _Refresh_ ⌘R, _Configure_ ⇧⌘,. Submenus have a `‹` header and no search; the root's search finds their rows.                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | 02-actions                         |
| D10 | **Empty and no index:** Raycast's empty view: one large icon, a title, a sentence (the query in mauve mono); footer primary _Refresh ↵_; Esc clears the query.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 03-empty                           |
| D11 | **Entry template**, the same component in the split (34 % list) and on the detail screen: 1 _Header_ — the row's icon and the title (title field, else the key), one line, tags right; 2 _Hero_ by type; 3 _Quote_ — the description, whole, with the red bar; 4 _Body_ — the notes, then a _Links_ section when the entry has links (D20). Empty slots collapse. No type badge, path or metadata in the pane.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 04-entry-template                  |
| D12 | **Heroes:** Command → the command card; Cheat Sheet → the command card for its snippet when it has one; Bookmark → its thumbnail (YouTube; capped in the split), else its URL as a link line (`↗ host/path`); Task → its status and priority chips.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | 04-entry-template                  |
| D13 | **Command card:** fmt-rs breaks the command at `&&`, `\|\|`, `\|` to the card's real column count; kb wraps what remains at spaces with a hanging indent, never splitting a token and keeping operators with the word before them; colours from GPUI Kit's bash highlighter with an extended highlight query (argument values coloured); ligatures off; 12 pt in the split, 13 in detail; GPUI Kit's Clipboard button in the corner; no line numbers.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | 04-command-cards, 04-spike-screens |
| D14 | **Footer:** left, the status line on the list (`● Ready • N results`, `● Refreshing…`, `● No index`; N is a true count), or, while a pane or the detail screen is open, the entry's type tile, its file and `⌥⌘C`; right, the screen's keys and _Actions ⌘K_.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | 01-list, 04-spike-screens          |
| D15 | **Filter** (⌘P): the action panel's component under the pill, search at the bottom; TYPES with their tile, TAGS with their icon or `#`; only chosen rows show ✓; picking keeps it open; with filters set, a _n filters · Clear All_ row leads and the pill names the filters; footer _Toggle ↵ · Only ⌘↵_.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | 07-filter                          |
| D16 | **Refresh** (the sync, ⌘R): header chevron only; running, stopped and failed use the empty view (running adds a thin progress bar and _n of m files · file_); the report uses the list's rows, one per file worth a look, failed first; keys _Stop esc_, _Back ↵_, _Retry ↵_, _Retry ↵ · Configure ⌘↵_.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | 11-14-refresh                      |
| D17 | **Copy feedback:** a notice bottom right above the footer, _Copied to clipboard_ with a `⌘V` hint, through GPUI Kit's Notification; every existing notice moves there. Its restyle is parked (TODO `MOB4`).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | 01-list                            |
| D18 | **One place for each thing:** DESIGN.md's front matter holds every design token (colours, type, radii, spacing, component sizes); `crates/kb-app/assets/catalog.toml` holds what kb offers (types, actions, screens' keys, labels, messages), deep-merged under `~/.config/kb/config.toml` (same keys win; validated at start with a notice and a per-key fallback; colours and sizes are not overridable).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | —                                  |
| D19 | **Accent separation becomes advisory** (owner-approved gate change, 2026-10-12): `design:validate` warns instead of failing when Catppuccin's accents fall under the colour-blind separation thresholds; colour is never the only cue.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | —                                  |
| D20 | **Links and no metadata column** (owner, 2026-10-13): every attribute Raycast's metadata column listed already has a home (type: tile; description: quote; URL, status, priority: hero; tags: header; source: footer), so there is no metadata column and → / ← do nothing on the detail screen. Links close the Body as a compact list (Notion's mention, Obsidian's links section): a `LINKS · n` label, then one 28 pt row per link — favicon (tile fallback), the title on one line trimmed with an ellipsis, then the URL in faint mono without its scheme, `www.`, query or fragment, trimmed in the middle so the host and the last path segment stay (`github.com/…/mini-redis`); a tooltip shows the full title and URL; `⌘1`…`⌘9` open the first nine links (copying a link stays in the Links… submenu, ⌘↵). A link without a title shows its trimmed URL as the title. Header tags are clickable: they filter the list by that tag. | detail-links                       |

## Scope

In: DESIGN.md and the design kit, the catalog and its overrides, every screen of the table above, the four entry types,
the status line and its count, copy feedback, ⌘1–⌘9, the command card, the showcase corpus, screenshots of the app,
the documents and the change map.

Out: Paste, Execute/Run Command (with a terminal view), the two link shapes the engines drop (a bare URL, a top-level
`title: url` map: its own task after the experiment's report, since it changes the engine's output), Editor-based code blocks in notes, og:image previews, the
notice's restyle, a footer hint for ← →, the massCode source idea, re-running the measurement session (Plan 5 T9). Each
is its own TODO task.

## Design

### 1. DESIGN.md and the design kit

DESIGN.md is rewritten around the approved pages. Its YAML front matter is the only place a design value is written:

- `colors`: the Catppuccin Macchiato roles kb uses (window, header, selection, mantle, badge, divider, text, muted, faint,
  the accents) and the derived fills (tile at 15 %, chip rest and live, code block).
- `typography`: the pages' roles (search 15, title 13 medium, description 12, labels 12 and 11, section labels 10
  semibold uppercase, footer 11, card 12/13 mono, keycap 11 mono, notes body and headings).
- `rounded`, `spacing` and `components` (window 860 × 520 and expanded, header 56, row 56 / gap 4 / radius 6, chip 20,
  keycap 20, tile 24 and 20, popover 220 wide / rows 28 / radius 14, footer 40, split 34 %, selection bar 2.5).

The prose describes the look and links each section to its approved page. `tools/design` reads the front matter:
`design:validate` (contrast; accent separation as warnings, D19), `design:theme` (GPUI Kit's theme and a `design.json`
the app loads), `design:prototypes` and `design:shots` (section 8). `layout.json` and `accents.json` go; the Tokyo Night
palette stays only as a comparison alternative.

### 2. The catalog

`crates/kb-app/assets/catalog.toml`, compiled in and deep-merged under the user's `~/.config/kb/config.toml`:

```toml
[types.bookmark]
label = "Bookmarks"
icon = "bookmark"
tint = "blue"
hero = "media"
primary = "open"
secondary = "copy-url"

[actions.open]
label = "Open"

[screens.list]
enter = "details"
cmd-enter = "entry.primary"
shift-cmd-enter = "entry.secondary"
```

It holds the four types (label, icon, tint, hero, primary, secondary), every action (label, keys), each screen's keys
(list, detail, filter, empty, no index, the refresh states), the action panel's and More…'s rows, and the messages. A
test validates it: every icon exists, every key parses, every reference resolves, no screen binds a key twice. Configure
(⇧⌘,) creates `config.toml` with the defaults as comments.

### 3. Components (`crates/kb-app/src/ui/`)

One component per concept, each matching a part of the pages, so a redesign touches one file:

| Component      | Draws                                                                      |
| -------------- | -------------------------------------------------------------------------- |
| `keycap`       | D2's keycap                                                                |
| `chip`         | tag pills, tag icon chips, the link chip, task chips; live or rest (D3)    |
| `tile`         | the tinted type tile, and the favicon with its tile fallback               |
| `row`          | the list row (D5) and the refresh report's row                             |
| `header`       | the magnifier or chevron, the query, the filter pill                       |
| `footer`       | the status line or the entry's statusline, the screen's keys               |
| `action_panel` | the root and its submenus, search at the bottom (D9); reused by the filter |
| `empty_view`   | D10 and the refresh states                                                 |
| `entry_view`   | the four-slot template (D11), split and detail                             |
| `command_card` | D13, with no kb types, ready to extract into its own crate                 |
| `notice`       | D17, through GPUI Kit's Notification                                       |

Views keep no literal colour, size, radius or spacing: they come from `design.json` and the catalog. An ast-grep rule
fails a `px(` literal outside the design module.

### 4. Window and theme

GPUI keeps its windows square, so the panel is drawn on a transparent window background with a rounded root (14). kb
always uses its dark theme. Inter (400, 500, 600 and the italic the quote uses) and JetBrains Mono (400, 500) are bundled
with their OFL licences and loaded at start; ligatures are off wherever code or commands are drawn.

### 5. Engine additions (kb-domain)

- `search::count(query)`: how many entries a query matches, run off the keystroke path; tested against the goldens'
  full result sets.
- A body-only notes Markdown (the notes without the description quote, the command block, links or source) for the
  template's Body; the stored `notes_markdown` and the goldens stay as they are.
- Type narrowing (D6) happens in the app, not in sync.

### 6. Notes body

GPUI Kit's Markdown view, styled from the type tokens (headings stop being oversized). Before the plan commits to it,
a spike renders the same notes with `zorite-markdown` (MIT, on kb's `gpui-pre`; `MarkdownStyle`, `on_highlight`,
`on_mermaid`); the plan adopts it if it renders the body better.

### 7. The showcase corpus

`tools/design/showcase/`: committed, realistic YAML for the four types, holding the entries the approved pages show
(`/k8s-rollout-restart`, the Kubernetes cheat sheet, the Tokio bookmark…), so the app's screenshots and the pages show
the same data. The sample corpus stays the engine's golden fixture.

### 8. Verification

`mise run design:shots` runs kb on the showcase corpus, drives each screen with the bench kit's `probe`, and captures
the window by its id (the spike's method) into `tools/design/shots/`, compact and expanded. The final review compares
each shot with its approved page through the checklists below; a difference is fixed or recorded as a deviation.

### 9. Per-screen checklists

Each item is checked on the shot against its approved page.

**List (01):** magnifier, query in 15 pt, the filter pill with `⌘P`; a hairline under the header; rows 56 tall, 4 apart,
corners 6; the selected row's fill `#363A4F` with a 2.5 mauve bar; tiles 24 tinted by type (Commands green, Cheat Sheets
mauve, Bookmarks blue, Tasks pink); command titles in mono; chips and keycaps exactly 20 tall; the selected row's chips
on mantle with muted text, other rows' chips grey; `↵` on the selected row, `⌘2`…`⌘9` on the next eight; footer status
`● Ready • N results` with the true count, then _Details ↵ · Copy ⌘↵ · Actions ⌘K_ (a bookmark selected: _Open ⌘↵ ·
Copy ⇧⌘↵_).

**Action panel (02):** opens above _Actions_, bottom right, 220 wide, rows 28, search at the bottom; the root lists the
screen's actions in D9's order with their keycaps; _Links… ›_ only for entries with links; More… shows COPY and
ADMINISTRATION under a `‹ More` header with no search; typing in the root finds More…'s rows, marked _More_.

**Empty and no index (03, 10):** one large icon, the title, the sentence with the query in mauve mono; status
`0 results` or `● No index`; footer _Refresh ↵_.

**Split and detail (04–06):** the list at 34 %; the pane's header is the row's icon and title on one line with tags
right (clickable: they filter); the hero by type; the description as a red-bar quote; the notes; then, with links,
the _Links_ section (D20): one row each, titles and URLs on one line, trimmed as D20 says, `⌘1`…`⌘9`; nothing else (no
badge, path or metadata column). The
command card: breaks at `&&` and `|`, a hanging indent, no split tokens, no ligatures, the copy button, 12 pt in the
split, 13 in detail. Footer left: the type tile, the file, `⌥⌘C`. Detail: the chevron only; → and ← do nothing.

**Filter (07):** under the pill, search at the bottom; the four types with their tiles, tags with their icons or `#`;
✓ only on chosen rows; stays open while picking; _n filters · Clear All_ first when filters are set; the pill names
them; footer _Toggle ↵ · Only ⌘↵_.

**Refresh (11–14):** the chevron only; running shows its icon, _Refreshing the index…_, the progress bar and _n of m
files · file_, footer _Stop esc_; the report lists files, failed first, under a summary line, footer _Back ↵_; stopped
and failed show their icon, title and sentence, footers _Retry ↵_ and _Retry ↵ · Configure ⌘↵_.

**Notice (17):** bottom right above the footer, _Copied to clipboard_, a `⌘V` hint on a grey keycap.

### 10. Documents

DESIGN.md (rewritten); PRODUCT.md (the design system is the visual reference, Raycast the behavioural one); CONTEXT.md
(the entry types narrowed; new terms Catalog, Design token, Status line, Command card, Notice; Refresh as the action's
name); AGENTS.md (the design kit and the catalog); the README; `docs/guides/parity.md` and `compliance.md` (keys,
labels, always dark, fonts); `docs/guides/design/` replaced by the approved pages; TODO.md.

### 11. The change map

`tools/design/CHANGE-MAP.md`, written last from the plan's actual diff: for each kind of change (a colour, a type role,
a size, an action, a screen's keys, a component), the files it reaches and the check that catches a miss.

## Acceptance criteria

1. Every screen's checklist (section 9) passes on `design:shots`, compact and expanded.
2. Every design value is in DESIGN.md's front matter, and every action, label and key in the catalog; no view keeps a
   literal colour, size, radius or spacing (ast-grep rule).
3. `design:validate` passes, with accent separation reported as warnings (D19).
4. `config.toml` overrides a label, a key and a type's primary action; an invalid value shows a notice and falls back.
5. Only the four entry types appear in the list, the filter and the counts; the goldens still pass.
6. The status line shows the true count; ⌘1–⌘9 run the rows' primary actions; copy feedback appears on every copy.
7. The documents of section 10 are updated, and `tools/design/CHANGE-MAP.md` exists.
8. `mise run gate` passes; the only threshold change is D19.

## Risks

| Risk                                                                 | Mitigation                                                                               |
| -------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| GPUI renders differently from the HTML pages                         | The spike already checked the panel; shots are compared screen by screen.                |
| A transparent rounded window misbehaves with the patched GPUI        | Fall back to a square window and record it as a deviation.                               |
| fmt-rs (v0.1.0, one author) stalls                                   | Pinned; pure Rust with no dependencies, so it can be vendored into `crates/third-party`. |
| `design:shots` needs Screen Recording and Accessibility, and ⌥K free | The task checks and says what to grant; it quits only kb.                                |
| Catalog overrides make a screen bind a key twice                     | The validation rejects it and falls back to the default for that key.                    |

## Doctrine check

`docs/doctrine/` holds one doctrine, `tag-conventions` (`applies_to: tools/tags/**`); this work does not touch the tag
tools or their rules.
