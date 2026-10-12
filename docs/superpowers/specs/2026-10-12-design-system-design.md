# The new design system (Plan 8) — design

- Status: draft, for the owner's review
- Date: 2026-10-12
- Branch: `feat/design-system`
- Plan: [2026-10-12-kb-native-08-design-system.md](../plans/2026-10-12-kb-native-08-design-system.md)

## Goal

The design system in `docs/guides/design/` replaces the current one. At the end:

- the new DESIGN.md is the root `DESIGN.md`, and its YAML front matter is the single source of every design token;
- the design kit (`tools/design`) reads that front matter: the theme, validation and comparison page derive from it;
- the whole app follows the new rules: typography, geometry, rows, popovers, footer, notifications;
- two new features land: a status line in the footer (with a true result count) and copy feedback;
- every document that names the design is updated, and a change map lists every file a future design change touches;
- committed screenshots of kb, taken on the sample corpus, show the result.

## What was found

The new system (`docs/guides/design/`, commit `c5dc878`) is a Google Stitch export: a `DESIGN.md` in Stitch's format
(YAML tokens, then prose), four screens (`01-list`, `02-actions`, `03-empty`, `04-split`, each an HTML file and a PNG),
and copies of kit files.

| Finding | Detail                                                                                                                                                                                                                                                                                                                           | Consequence                                                                                                |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| F1      | The new DESIGN.md's kb roles (`surface-window` `#23273A` … `accent-red` `#E04F50`) are the current palette, hex for hex.                                                                                                                                                                                                         | The validated palette stays; no colour re-validation is needed beyond the gate.                            |
| F2      | Its front matter also carries ~50 Material 3 roles (`surface` `#0e1224`, `primary`, `on-primary-container`…) no kb surface uses.                                                                                                                                                                                                 | They are dropped (D5).                                                                                     |
| F3      | The screens disagree with the text: `01`–`03` are coloured in Catppuccin Macchiato (`#24273A`, mauve `#C6A0F6`, sky, mint, peach) with radii 14 / 6 / 4; `04-split` uses the Tokyo Night Storm hexes; the text says radii 28 / 16 / 12 / 8 / 6 / 4.                                                                              | The text wins (D1); the screens give anatomy only.                                                         |
| F4      | New typography: Inter for the interface, JetBrains Mono for code, a ten-role scale (`headline-lg` 24/600 … `label-xs` 11/600 uppercase, `code-sm` 12), with line heights and letter spacing.                                                                                                                                     | Fonts are bundled (D3); GPUI has no letter spacing (Deviations).                                           |
| F5      | `layout.json`, `color.ts`, `color.test.ts` and `palettes.json` in the folder are byte-identical copies of `tools/design/`'s; `manifest.json` is a web-app manifest (PWA icons, `start_url`); `icon.png` is an app icon candidate.                                                                                                | Copies and the manifest are deleted (D4).                                                                  |
| F6      | The screens add anatomy the text doesn't describe: tinted icon tiles, an accent bar on the selected row, a ↵ keycap on the selected row, icons on action rows, a status line (`● Ready • 12 results`), a copy toast, a rich empty state, ⌘1–⌘9 quick picks, ⌘N Create Entry, a title block and metadata tiles in the split pane. | The owner picked which land (D2).                                                                          |
| F7      | GPUI Kit 0.7.1's `Notification` has placement (`Anchor`), types (info, success, warning, error), an action, an id (a notification with the same id replaces the last), and a fixed 5-second autohide.                                                                                                                            | Copy feedback uses it directly, with no wrapper (D7).                                                      |
| F8      | The engine has no result count: `search` returns one page of 50.                                                                                                                                                                                                                                                                 | A count query joins `kb-domain` (D8).                                                                      |
| F9      | Copying today writes to the clipboard silently and keeps the panel open.                                                                                                                                                                                                                                                         | Copy feedback fills a real gap; the GPUI Kit guides' rule against redundant success toasts does not apply. |
| F10     | The current docs have drifted: DESIGN.md lines 38 and 78 and `search_view/footer.rs`'s header still say "footer pills", removed in `dbf8cf0`; DESIGN.md's appendix row UI‑16 says "search at the bottom" against §4's "on top".                                                                                                  | The rewrite removes them.                                                                                  |

## Decisions

The owner's, 2026-10-12.

**D1. The text is canonical.** Where the new DESIGN.md's text and its screens disagree, the text wins. Screens show
layout and anatomy, read in the Tokyo Night Storm palette. Colours stay those of F1; radii are the text's.

**D2. Product surface that lands.** From F6: the status line, copy feedback, tinted icon tiles, the accent bar on the
selected row, the ↵ keycap on the selected row, icons on action rows. Not landing: the rich empty state, ⌘1–⌘9 quick
picks, ⌘N Create Entry (kb never writes source files), the search magnifier in the header (the back chevron stays), the
split pane's title block, metadata tiles and Copy button (the metadata card stays), monospaced row titles.

**D3. Typography.** Inter (400, 500, 600) and JetBrains Mono (400, 500) are bundled in the app (SIL OFL 1.1, licence
files beside them) and loaded at start. Icons stay GPUI Kit's Lucide set: each Material Symbol the screens use maps to
its nearest Lucide glyph (table in Components).

**D4. Layout of the system.** The new DESIGN.md replaces the root one. Its reference screens move to
`tools/design/reference/`. `docs/guides/design/` is removed, with its copies and `manifest.json`. `icon.png` moves with
the screens as a reference (the app icon is not changed by this plan).

**D5. Tokens.** DESIGN.md's front matter is the source of every token. It keeps only tokens kb uses; the Material 3
roles go. `layout.json` folds into it. `palettes.json` keeps only the alternative palettes the comparison page
renders; it no longer marks a `chosen` one.

**D6. Always dark.** kb draws its dark theme whatever the system appearance. The light theme, the light accents and the
appearance observer go.

**D7. Copy feedback and notices use GPUI Kit's `Notification` directly.** No kb wrapper and no Raycast-shaped API: kb
sets the defaults once (placement bottom-right, above the footer) and every feature calls
`window.push_notification(Notification::…)`. Every existing notice moves to the same placement and gets its type.

**D8. The status line shows a true count.** `kb-domain` gains a count of the entries a query matches, with the same
text and filter as `search`; it runs off the keystroke path.

**D9. The reference changes.** The design system is the reference for how kb looks; Raycast's kb Search stays the
reference for how kb behaves (keys, actions, flows: `docs/guides/parity.md`). PRODUCT.md principle 2 says so.

**D10. Verification by committed screenshots.** A task runs kb on the committed sample corpus, drives it, and commits
its screens beside the reference screens.

**D11. Accent bar colour.** The selected row's bar is purple (`#A362F2`), whatever the entry's type.

**D12. A change map closes the work.** The last task records every file the design system reaches, so a later design
change starts from a checklist instead of a search.

## Scope

In: the design system (DESIGN.md, the kit), every app surface, the status line and count, copy feedback and notice
defaults, `design:shots`, the documents of section 7 and the change map.

Out: the rich empty state, ⌘1–⌘9, ⌘N, a light palette, a new app icon, blur, and re-running the measurement session
(Plan 5 T9, after this plan).

## Design

### 1. DESIGN.md and its tokens

The root `DESIGN.md` becomes the new one, edited to D1–D11:

- **Front matter** (YAML), the only place a token's value is written. The keys follow the new system's names; values
  are the text's:
  - `name: Tokyo Night Storm Flat Separated`
  - `colors`: `surface-window`, `surface-header`, `surface-raised`, `surface-selection`, `border-divider`,
    `text-primary`, `text-muted`, `text-faint`, and the seven `accent-*`. Nothing else.
  - `typography`: the ten roles of F4 (`fontFamily`, `fontSize`, `fontWeight`, `lineHeight`, `letterSpacing` kept for
    Stitch but unused by the app; `label-xs` gains `textTransform: uppercase`).
  - `rounded`: `panel` 28, `popover` 16, `selection` 12, `media` 12, `action-row` 8, `keycap` 6, `tag` 4 (points).
  - `spacing`: the 4-point ramp (`hairline` 1, `xs` 4, `sm` 8, `md` 12, `lg` 16, `xl` 24, `section` 32).
  - `components`: what `layout.json`'s `components` held (header, result row, tag, footer, popover, action item,
    filter control, split pane, detail, metadata, media), updated to the text (action row 40 high, keycap padding 4 × 2,
    keycap gap 4), plus the new parts: `icon-tile` (24, radius 4, fill the accent at 15% over `surface-window`),
    `selection-bar` (2.5 wide, inset 6 top and bottom, `accent-purple`), `notification` (placement `bottom-right`,
    margins above the footer), `window` (compact 860 × 520, expanded 85% of the display).
  - `alignment`: `layout.json`'s `alignmentRules`, as a list.
- **Prose**, in the Stitch order (Brand & Style, Colors, Typography, Layout & Spacing, Elevation & Depth, Shapes,
  Components), plus kb's sections: Tags (the Raycast hash, the pill-lift rule), Deviations (below), Evidence (the
  reference screens) and the change map's pointer. Prose never repeats a value: it names the token (`rounded.popover`)
  where a reader needs the number, except the colour table, which `design:validate` keeps in step (as today).

**Deviations** (replaces Adaptations). What kb does differently from the GPUI Kit Design Guides, and from Raycast where
it matters, each with its reason: Inter instead of the platform font (the system's choice; owner-approved); uppercase
`label-xs` section headers (owner-approved; the guides advise against uppercase); Title Case (kept); no letter spacing
(GPUI has none); always dark (D6); notices autohide after GPUI Kit's fixed 5 seconds; and the rows of today's
Adaptations table that still hold (opaque surfaces, favicons from DuckDuckGo, tag images from a local folder, images
by `curl`, the patched GPUI that keeps the window centred, the two window sizes, → / ← only at the caret's end).

### 2. The design kit

`tools/design/` keeps its role, with the front matter as its input:

- A small module, `tools/design/tokens.ts` (with `tokens.test.ts`), parses DESIGN.md's front matter and returns typed
  tokens; it fails on an unknown or missing key. It imports no npm package (AGENTS.md: tested modules import none), so
  it parses the front matter's YAML subset itself: maps, quoted and bare scalars, lists of scalars. The test covers that
  subset and the real DESIGN.md.
- `design:validate` reads the colours from the front matter (no longer from `palettes.json`'s `chosen`), and checks the
  rest of `palettes.json` as alternatives. The palette-table drift check stays, against the front matter.
- `design:theme` reads the front matter and writes:
  - `crates/kb-app/assets/themes/kb.json`: as today, plus `font.family` `Inter`, `mono_font.family`
    `JetBrains Mono`; dark only.
  - `crates/kb-app/assets/themes/design.json` (replaces `accents.json`): the accents and their pills (dark only), the
    typography roles in rem (px ÷ 16), the radii, the spacing and the component geometry, all in points.
  - `crates/kb-app/tests/fixtures/tag-accents.json`, as today.
- `design:render` (the comparison page) reads the front matter for geometry and the chosen colours, and
  `palettes.json` for the alternatives.
- `design:sample` keeps reading the local Raycast captures; its output, `raycast.json`, stays a record.
- `layout.json` is deleted. `tools/tags`'s single read of `palettes.json` (`tools/tags/tasks.toml`, the window colour)
  moves to the front matter.
- New `design:shots` (section 6).

### 3. The app: theme, fonts, tokens

- **Fonts.** `crates/kb-app/assets/fonts/`: Inter 400/500/600 and JetBrains Mono 400/500 as static TTFs from their
  pinned upstream releases, with `OFL.txt` for each. `KbAssets` serves them; `run` adds them to the text system
  (`cx.text_system().add_fonts`) before the theme loads. A test fails if a font file is missing or does not load.
- **Always dark.** `init_theme` loads `kb Dark` and sets the dark mode; it no longer syncs to the system appearance, and
  `SearchView` drops its appearance observer.
- **Tokens in Rust.** `accents.rs` becomes `design.rs` (one concept: the generated design tokens), loading
  `design.json` once (`LazyLock`, as `accents.rs` does). It exposes typed accessors: `accent(role)`, `pill(role)`,
  `text(TypeRole) -> TextStyle refinement` (size in rem, weight, line height, family), `radius(Radius)`,
  `space(Space)` and the component geometry. Views use these; no view keeps a literal size, weight, radius or colour
  (the compliance rule today is "no colour literals"; it extends to these). Uppercase is applied where `label-xs` is
  drawn.

### 4. Components

Each item cites the DESIGN.md section it implements. Unlisted components keep today's behaviour, restyled through the
tokens.

| Component                 | Change                                                                                                                                                                                                                                                                                     | Code                                    |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------- |
| Typography                | Every text takes its role: search field `headline-md`; row title `body-lg`; description `body-sm`; metadata `body-md` / `label-md`; keycaps and pills `label-sm`; section headers `label-xs` uppercase; notes headings `headline-lg` / `headline-sm`; code `code-sm`.                      | all views; `notes.rs` through the theme |
| Header                    | Geometry from tokens; back chevron, query and filter button as today.                                                                                                                                                                                                                      | `search_view/header.rs`                 |
| Result row                | Type glyph inside the icon tile (accent at 15%, glyph in the accent); a favicon or tag image is drawn bare, without a tile. Selected row: the inset fill as today, plus the purple bar at its left edge and a trailing ↵ keycap (full list only; beside the split pane titles stay alone). | `entry_list.rs`, `presentation.rs`      |
| Tag pill and tag image    | Geometry from tokens; text `label-sm`.                                                                                                                                                                                                                                                     | `entry_list.rs`, `tag_images.rs`        |
| Footer                    | Left: the status line (section 5) instead of the selected entry. Right: unchanged (primary action, Actions, one keycap per key). Keycaps take the `keycap` radius and padding.                                                                                                             | `search_view/footer.rs`                 |
| Actions popover           | Each action row gets a leading Lucide icon; rows 40 high, radius 8; section headers `label-xs`.                                                                                                                                                                                            | `palette.rs`, `search_view/overlays.rs` |
| Filter dropdown           | Section headers `label-xs`; geometry from tokens; ✓ / ○ as today.                                                                                                                                                                                                                          | `filter.rs`, `search_view/overlays.rs`  |
| Split pane, metadata card | Geometry and type from tokens.                                                                                                                                                                                                                                                             | `search_view.rs`, `metadata.rs`         |
| Notes                     | Body and headings through the theme's fonts; code in JetBrains Mono.                                                                                                                                                                                                                       | `notes.rs`                              |
| Empty states              | Restyled through the tokens; content unchanged.                                                                                                                                                                                                                                            | `entry_list.rs`                         |
| Sync screen               | Restyled through the tokens.                                                                                                                                                                                                                                                               | `sync_view.rs`                          |

Action icons (Material Symbol in the screens → Lucide): copy → `Copy`; terminal → `SquareTerminal`; visibility →
`Eye`; description → `FileText`; code → `Code`; tune → `SlidersHorizontal`; open in browser → `Globe`; links → `Link`;
rebuild → `RefreshCw`; preferences → `Settings`; quit → `Power`. Any icon the default bundle lacks is added to
`ExtraIcons`; the existing test catches a missing one.

### 5. New features

**Status line.** The footer's left side shows one of:

- `● Ready • N results` (`N` with a thousands separator; `1 result` singular), the dot `accent-green`;
- `● Rebuilding…`, the dot `accent-yellow`, while a sync runs;
- `● No index`, the dot `text-faint`, when there is no index.

Text `body-md` in `text-muted`. Status is never told by the dot alone: the word is always there.

**Count.** `kb_domain::search::count(connection, query) -> rusqlite::Result<usize>` runs `SELECT COUNT(*)` over the
statement `build_search_sql` builds, without its `LIMIT`/`OFFSET`, with the same parameters. Its test: for every query
of the sample and private goldens, `count` equals the length of the golden's full result list (the goldens are not
paged). `EntryList` asks for the count with each new query on the background executor, after the page; a stale count
(the query changed) is dropped. The footer shows the last count for the current query, and nothing in its place until
it arrives.

**Copy feedback.** Every action that writes to the clipboard (the primary Copy, Copy URL, the Copy… page) pushes
`Notification::success("Copied to Clipboard")`, with an id so a second copy replaces the first. Notices that exist
today take their type: the sync's finished message `success` (or `error` when it failed), "Couldn't create …"
`error`, the startup notices (unreadable preferences, taken hotkey) `warning`. kb sets
`theme.notification.placement` to bottom-right and its margins from the `notification` component token, once, in
`init_theme`.

### 6. Verification: `design:shots`

`mise run design:shots` (a task in `tools/design/tasks.toml`, Bun inline, AGENTS.md):

1. Builds kb (release), and makes a home under `tmp/design-home` whose `config.toml` points `sources_path` at
   `crates/kb-domain/tests/fixtures/sample`; runs `kb --sync` there.
2. Quits a running kb (it owns ⌥K), launches kb under that home, and drives it with the bench kit's `probe`
   (`tools/bench/probes/probe.swift`, CGEvent): opens the panel and visits each screen.
3. Captures the panel's window with `screencapture -l <window id>` (the id from `probe`, or from a small Swift query),
   compact and expanded (⌘⇧F), into `tools/design/shots/<screen>-<size>.png`.

Screens: `list`, `list-selected-tags` (a row with tags and links), `split`, `detail`, `actions`, `actions-copy-page`,
`filter`, `empty`, `copy-notice`, `bookmark-split`. The task needs the Screen Recording and Accessibility permissions;
it says so and stops when it lacks them. Shots are committed; they hold only the sample corpus.

The final review compares each shot with the reference screen of the same anatomy (`tools/design/reference/`), read in
the palette of D1. Differences either get fixed or become a Deviation with its reason.

### 7. Documents

| Document                           | Change                                                                                                                                                                                                     |
| ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `DESIGN.md`                        | Replaced (section 1).                                                                                                                                                                                      |
| `PRODUCT.md`                       | Principle 2 (D9); Brand Personality's "looks and speaks like Raycast" becomes "speaks like Raycast; looks as DESIGN.md says"; accessibility line points at the new validation.                             |
| `AGENTS.md`                        | The `tools/design/` entry (front matter as source, shots, reference screens); "Read DESIGN.md before any UI change" stays.                                                                                 |
| `CONTEXT.md`                       | Colour palette row (lives in DESIGN.md's front matter); new rows **Design token**, **Status line**, **Notice**; Tag image row unchanged.                                                                   |
| `README.md`                        | Project structure (`docs/guides/design` gone; `tools/design/reference`, `shots`); a current screenshot.                                                                                                    |
| `docs/guides/parity.md`            | Footer row (status line instead of the entry: Adapted, with reason); new Better rows: copy feedback, result count; appearance row (always dark: Adapted).                                                  |
| `docs/guides/compliance.md`        | "Follows the system appearance" becomes "always dark", with reason; "theme tokens replace visual literals" extends to type, radii and spacing; fonts bundled.                                              |
| `tools/bench/README.md`            | Tier 2 size note: the bundled fonts add to the binary; the next session measures it.                                                                                                                       |
| `crates/third-party/README.md`     | Unchanged (no new patch).                                                                                                                                                                                  |
| Source comments citing DESIGN.md   | Updated to the new section names and token names (`accents.rs`, `entry_list.rs`, `http.rs`, `lib.rs`, `metadata.rs`, `notes.rs`, `presentation.rs`, `search_view*.rs`, `tag_images.rs`, `window_size.rs`). |
| `TODO.md`                          | Plan 8 tasks, done as they land.                                                                                                                                                                           |
| `tools/design/CHANGE-MAP.md` (new) | Section 8.                                                                                                                                                                                                 |

### 8. The change map

`tools/design/CHANGE-MAP.md`: for a future design change, what to edit and in what order. One table per kind of
change (a colour, a type role, a radius or spacing, a component's anatomy, a new notice, a new icon), each row a file,
what in it depends on the design, and the check that catches a miss (`design:validate`, the icon test, `cargo nextest`,
`design:shots`). It is written last, from this plan's actual diff (`git diff --stat main...feat/design-system`), so it
records what was touched rather than what was expected. DESIGN.md and AGENTS.md link to it.

## Phases

The plan follows approach B: tokens first, then components, then features, then documents. Every phase ends with
`mise run gate` green and, from phase 1 on, `mise run design:shots` committed, so each phase's visual change is
reviewable on its own.

1. **The system lands.** DESIGN.md replaced; `tokens.ts`; the kit reads the front matter; `design.json`; fonts
   bundled and loaded; always dark; `design.rs`; `design:shots`; `docs/guides/design/` removed, screens moved. The app
   looks nearly the same, in Inter.
2. **Components.** Typography roles everywhere; rows (tile, bar, ↵); popovers (icons, rows 40, keycaps); footer
   keycaps; metadata, notes, sync screen through tokens.
3. **Features.** `count` in `kb-domain`; status line; notice defaults and types; copy feedback.
4. **Documents and close.** Section 7; the final shot review; the change map last.

## Acceptance criteria

1. `DESIGN.md` at the root is the new system edited to D1–D12; `docs/guides/design/` does not exist; the reference
   screens are under `tools/design/reference/`.
2. Every design value is written once, in DESIGN.md's front matter; `layout.json` and `accents.json` do not exist;
   `design:validate` passes and fails on a drifted table (checked by editing a hex in a scratch copy).
3. `kb.json` and `design.json` are generated by `design:theme`; regenerating them leaves no diff.
4. kb renders in Inter, notes code in JetBrains Mono; it stays dark in the light appearance.
5. No view in `crates/kb-app/src` has a literal colour, font size, font weight, radius or spacing; a test or an
   ast-grep rule (`sgconfig.yml`) enforces it for `px(` literals outside `design.rs`.
6. Rows show the icon tile, and the selected row its purple bar and ↵ keycap; action rows show icons; section headers
   are uppercase `label-xs`.
7. The footer shows `● Ready • N results` with the true count, `Rebuilding…` and `No index` in their states; `count`
   matches every golden.
8. Each copy shows "Copied to Clipboard" bottom-right; existing notices appear there with their types.
9. `tools/design/shots/` holds every screen of section 6 in both sizes, taken from the final commit.
10. The documents of section 7 are updated; `tools/design/CHANGE-MAP.md` exists and is linked.
11. `mise run gate` passes with the gates and thresholds unchanged.

## Risks

| Risk                                                                                     | Mitigation                                                                                                                      |
| ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| GPUI Kit components (List, Command, Notification, TextView) draw text in their own sizes | Set the theme's base font and `font.family`; where a component ignores them, restyle through its builder or record a Deviation. |
| Inter's metrics change row and header heights                                            | Heights are tokens, not content-driven; shots show any clipping.                                                                |
| `design:shots` needs macOS permissions and a free ⌥K                                     | The task checks both and says what to grant; it never kills an app other than kb.                                               |
| The front-matter parser misreads YAML                                                    | It accepts only the subset DESIGN.md uses and fails loudly on anything else; tested on the real file.                           |
| Bundled fonts grow the binary (tier 2 size)                                              | Static weights only (five files); the next measurement session records it; noted in `tools/bench/README.md`.                    |
| The count query adds work per query                                                      | It runs after the page, off the keystroke path, and is dropped when stale; tier 1's search cases are unaffected.                |

## Doctrine check

`docs/doctrine/` holds one doctrine, `tag-conventions` (`applies_to: tools/tags/**`). This plan touches
`tools/tags/tasks.toml` only to read the window colour from the front matter instead of `palettes.json`; no tag rule is
involved. No other doctrine applies.
