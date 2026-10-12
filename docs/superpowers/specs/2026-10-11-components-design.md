# Components from DESIGN.md (Plan 7) — design

- Status: approved
- Date: 2026-10-11

## Goal

kb's panel takes the shape DESIGN.md describes, and behaves as Raycast's kb Search does, keys included. Every finding of
the UI comparison (UI-01…UI-23, `tools/reports/ui.md`, local) is settled, and the parity matrix rows it contradicts
(A01, S12) become true again. Plan 6 gave the palette and base typography; this plan builds the components.

## Decisions

**K1. Raycast's keys.** The split pane opens with → and closes with ←, as in Raycast, caret-aware so editing the query
still works:

| Key | Pane hidden                                                             | Pane shown                                                              |
| --- | ----------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| →   | Show Details when the caret is at the query's end; else moves the caret | moves the caret                                                         |
| ←   | moves the caret                                                         | Hide Details when the caret is at the query's end; else moves the caret |
| ⌘D  | Show Details (unlisted alias)                                           | Hide Details (unlisted alias)                                           |

On the detail screen (no text field), → and ← show and hide the metadata. The rest is Raycast's: Return Show Details,
⌘Return the primary action, ⌘K Actions, ⌘P Filters, ⌘⇧C Copy URL, ⌘O Open Source File, ⌘R Rebuild Index…,
⌘⇧, Open Preferences, Esc closes an overlay, then goes back, then hides the panel. kb adds ⌘Q Quit kb and ⌘⇧F
Toggle Window Size.

**K2. Overlays are GPUI Kit `Popover`s anchored to their trigger, holding kb's own lists.** The Actions popover opens
upwards from the footer's Actions button; the filter dropdown opens under the header's filter button. Rejected:
`PopupMenu`/`DropdownMenu` (no search field, closes on every pick, which breaks UI-16 and UI-18) and hand-made
`anchored()` overlays (re-implement Popover's dismissal and focus).

**K3. Network: thumbnails and favicons.** kb loads, lazily, the image URLs a bookmark's notes already carry (YouTube
thumbnails) and each bookmark's favicon from DuckDuckGo's icon service (`https://icons.duckduckgo.com/ip3/<host>.ico`;
it receives bookmark hostnames). Raycast's `api.ray.so/favicon` serves Raycast, not other apps. Until an image loads, or
offline, the globe icon (favicon) or nothing (thumbnail) shows. GPUI caches the images for the session.

**K4. Tag images from a local folder.** As in the Raycast extension, a tag is drawn as `<tag>.svg` when that file exists,
its name in the tooltip. The folder is `tag_images_path` in `~/.config/kb/config.toml`, default `~/.config/kb/images`,
read once at start. `mise run images` copies the extension's SVGs from `$KB_TS_REPO` (read-only) into the default
folder. No image enters this public repository: their licences are mixed.

**K5. Seven accents for both appearances.** GPUI Kit's theme has no orange or purple, so `tools/design/theme.mjs` also
writes `crates/kb-app/assets/themes/accents.json`: the seven accents for dark (the chosen palette) and light (interim
values, until the light palette, TODO QTjG), with their pill fills and pill text. kb's `Hue` resolves against it by
appearance. A fixture written by `tools/design/color.mjs` (tag names → accent) proves the Rust tag hash equals the
JavaScript one, which equals the Raycast extension's.

## Components

| Area            | Behaviour                                                                                                                                                                                                                                                                                                                                                         | Findings            |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------- |
| Header          | No divider. Leading glyph (search, or back chevron on the detail screen) on the row-icon axis. Filter button: funnel, current filter, chevron that turns up while open. The detail screen shows only the chevron.                                                                                                                                                 | UI-02, UI-11        |
| List row        | Type icon tinted (Raycast's icons: globe, terminal, document, text, check-circle), or the favicon for bookmarks; inset rounded selection; regular title; description same size, muted; ≤3 pills then `+N`, link count.                                                                                                                                            | UI-03…06, UI-21     |
| Split pane      | List ⅓, pane ⅔; one divider between.                                                                                                                                                                                                                                                                                                                              | UI-08               |
| Metadata card   | Zebra rows, label left, value right; Type and Tags as pills; Source with a file icon; URLs with ↗ (opens them).                                                                                                                                                                                                                                                   | UI-09, UI-22        |
| Notes           | Description quote with the red bar; the thumbnail above the metadata (split) or first (detail).                                                                                                                                                                                                                                                                   | UI-10, UI-20, UI-23 |
| Detail screen   | Starts with the notes; no title block (the footer names the entry).                                                                                                                                                                                                                                                                                               | UI-12, UI-23        |
| Footer          | Two floating pills: left the entry (icon, title); right the primary action ↵ and Actions ⌘ K. One keycap per key.                                                                                                                                                                                                                                                 | UI-07               |
| Actions popover | Raycast's order: Show Details ↵, the primary action ⌘↵; Source & Clipboard (Copy URL, Show/Hide Details, Open Source File); Links ›; Copy… › (Title, Description, Notes, Raw YAML); Administration (Rebuild Index…, Open Preferences, Toggle Window Size, Quit kb). Keycaps from the real bindings. Search at the bottom. A › row drills in; ← or Esc comes back. | UI-13…16, A01       |
| Filter dropdown | A Filters section (count, Clear All Filters) while any is active; types and tags with ✓ / ○; stays open while picking; Esc or ⌘P closes it.                                                                                                                                                                                                                       | UI-17…19            |
| Window          | ⌘⇧F: compact 860 × 520 ↔ 85% of the display's visible area, centred; the size holds for the session.                                                                                                                                                                                                                                                              | D5                  |
| Copy            | Title Case labels for commands, buttons, sections; messages stay sentences.                                                                                                                                                                                                                                                                                       | D2                  |
| Surface         | Done in Plan 6 (palette, no blur, radius from layout.json).                                                                                                                                                                                                                                                                                                       | UI-01               |

## Scope

In: every row of the table above, the keys of K1, and the docs below. Out: the "Commands" typo, which is in a source
file (kb never writes source files); the light palette (TODO QTjG); re-running the measurements (the next task).

## Units

- `presentation.rs`: type icons, the seven accents, tag hash (fixture-tested), Title Case labels.
- `accents.rs` (new): loads `accents.json`; `Hue` → `Hsla` by appearance.
- `tag_images.rs` (new): the folder scan and the tag → image lookup.
- `keys.rs` (new, from `palette.rs`): actions and bindings, the caret-aware arrow handling.
- `palette.rs`: the action list (groups, drill-in pages) as data; `actions_popover.rs` (new) renders it.
- `filter.rs`: the dropdown's data; `filter_dropdown.rs` (new) renders it.
- `metadata.rs` (new): the card. `notes.rs`: quote bar, thumbnail.
- `footer.rs`, `header.rs` (new, from `search_view.rs`): the footer pills and the header.
- `window_size.rs` (new): compact and expanded bounds, the toggle.
- `search_view.rs`: composes them; shrinks.
- `config.rs`: `tag_images_path`.

## Tests

- Unit: tag hash equals the fixture; Hue → accent per appearance; action list order, labels and drill-in pages for each
  entry type; filter sections; window bounds maths; tag image lookup (case, missing folder); Title Case labels.
- UI (`crates/kb-app/tests/ui.rs`): → at the query's end shows the pane, → mid-query moves the caret; ← hides it from
  the end; ⌘D still toggles; → / ← on the detail screen; Return opens the detail screen with the chevron-only header;
  the actions popover lists shortcuts and drills into Copy…; the filter stays open across two picks; ⌘⇧F changes the
  window size and back; the existing task tests (T1–T5) still pass.
- By eye: `mise run show` against the reference screenshots, screen by screen.

## Docs

- DESIGN.md: Adaptations lose the ⌘D row and gain the favicon service and the tag-images folder; §4 notes the drill-in.
- `docs/guides/parity.md`: A01 and S12 back to Same; S09 → / ←; any row the new keys change.
- `docs/guides/compliance.md`: the Title Case item checked.
- README: the keyboard table; `tag_images_path`; `mise run images`.
- CONTEXT.md: Tag image; Size toggle.
- TODO.md: Plan 7 (eWka) split per area, each ticked as it lands; the measurement session stays next.

## Delivery

Plan `docs/superpowers/plans/2026-10-11-kb-native-07-components.md`; branch `feat/components`; one commit per task;
`mise run gate` green before each; a pull request; merge on green; the release (0.4.0 or later) checked on
`roalcantara/kbx`.
