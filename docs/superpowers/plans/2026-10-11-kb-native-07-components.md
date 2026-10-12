# Plan 7: Components Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** kb's panel takes the shape DESIGN.md describes and behaves as Raycast's kb Search does, keys included,
settling UI-01…UI-23 and the parity rows A01 and S12 (spec:
[2026-10-11-components-design.md](../specs/2026-10-11-components-design.md)).

**Architecture:** `search_view.rs` keeps the state and the handlers; each component moves to its own module (header,
footer, metadata card, actions popover, filter dropdown, window size) that renders from plain data the tests assert
directly. Colours come from the theme (Plan 6) plus a generated `accents.json` for the seven accents. Overlays are GPUI
Kit `Popover`s anchored to their trigger. The window keeps its centre when resized through a patched `gpui-pre-macos`.

**Tech Stack:** Rust 2024, GPUI Kit 0.7.1 (`gpui-component`, `gpui-base`), Node for `tools/design`, mise, hk.

**Rules for every task:** write the failing test first; run `cargo nextest run -p kb-app` after each step that changes
Rust; `mise run gate` before each commit; Conventional Commits, sentence-case subject ≤ 72 characters, body lines ≤ 72;
end each message with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Never write to `$KB_TS_REPO`.

## File map

| File                                            | Change | Responsibility                                                         |
| ----------------------------------------------- | ------ | ---------------------------------------------------------------------- |
| `tools/design/theme.mjs`                        | modify | also writes `accents.json` and the tag-hash fixture                    |
| `crates/kb-app/assets/themes/accents.json`      | create | the seven accents, pill fill and pill text, per appearance (generated) |
| `crates/kb-app/tests/fixtures/tag-accents.json` | create | tag → accent, written by `color.mjs` (generated)                       |
| `crates/kb-app/src/accents.rs`                  | create | loads `accents.json`; `Hue` → colours by appearance                    |
| `crates/kb-app/src/presentation.rs`             | modify | seven hues, Raycast tag hash and type tints/icons, Title Case labels   |
| `crates/kb-app/src/keys.rs`                     | create | actions and key bindings (moved from `palette.rs`), caret rule         |
| `crates/kb-app/src/palette.rs`                  | modify | the action list as pages (main, Links, Copy…), Raycast order           |
| `crates/kb-app/src/actions_popover.rs`          | create | renders the action list in a Popover anchored to the footer            |
| `crates/kb-app/src/filter.rs`                   | modify | Title Case; query clears after a pick                                  |
| `crates/kb-app/src/filter_dropdown.rs`          | create | renders the filter in a Popover anchored to the header                 |
| `crates/kb-app/src/header.rs`                   | create | header: glyph on the icon axis, query field, filter button             |
| `crates/kb-app/src/footer.rs`                   | create | the two floating pills with one keycap per key                         |
| `crates/kb-app/src/metadata.rs`                 | create | the metadata card                                                      |
| `crates/kb-app/src/notes.rs`                    | modify | thumbnail, quote bar, card below (split) or beside (detail)            |
| `crates/kb-app/src/entry_list.rs`               | modify | row: inset selection, icon or favicon, pills or tag images, link count |
| `crates/kb-app/src/tag_images.rs`               | create | scans the tag-images folder once; tag → SVG path                       |
| `crates/kb-app/src/window_size.rs`              | create | compact/expanded sizes and the toggle                                  |
| `crates/kb-app/src/config.rs`                   | modify | `tag_images_path`                                                      |
| `crates/kb-app/src/search_view.rs`              | modify | composes the components; handlers for the new actions                  |
| `crates/third-party/gpui-pre-macos/`            | create | `gpui-pre-macos 0.3.8` with `resize` keeping the window centre         |
| `Cargo.toml`                                    | modify | `[patch.crates-io] gpui-pre-macos`                                     |
| `mise.toml`                                     | modify | `images` task                                                          |
| `DESIGN.md`, `README.md`, `CONTEXT.md`, docs    | modify | Task 12                                                                |

---

### Task 1: Seven accents and Raycast's tag hash

**Files:** modify `tools/design/theme.mjs`, `tools/design/color.mjs`; create `crates/kb-app/src/accents.rs`,
`crates/kb-app/assets/themes/accents.json` (generated), `crates/kb-app/tests/fixtures/tag-accents.json` (generated);
modify `crates/kb-app/src/presentation.rs`, `crates/kb-app/src/entry_list.rs`, `crates/kb-app/src/lib.rs`.

- [ ] **Step 1: Generate the accents and the fixture.** In `color.mjs` export the light interim accents and append to
      `theme.mjs`:

```js
// color.mjs — interim light accents (Tailwind 600), until the light palette (TODO QTjG).
export const LIGHT_ACCENTS = {
  window: '#FFFFFF',
  text: '#0A0A0A',
  blue: '#2563EB',
  green: '#16A34A',
  orange: '#EA580C',
  purple: '#9333EA',
  magenta: '#DB2777',
  yellow: '#CA8A04',
  red: '#DC2626'
}
```

```js
// theme.mjs — after kb.json: the seven accents GPUI Kit's theme has no keys for, and the tag-hash fixture.
const accentsOut = resolve(root, 'crates/kb-app/assets/themes/accents.json')
const accentSet = (roles) => Object.fromEntries(ACCENTS.map((a) => [a, { accent: roles[a], ...pill(roles, a) }]))
writeFileSync(accentsOut, `${JSON.stringify({ dark: accentSet(r), light: accentSet(LIGHT_ACCENTS) }, null, 2)}\n`)
const fixtureOut = resolve(root, 'crates/kb-app/tests/fixtures/tag-accents.json')
const sampleTags = [
  'git',
  'docker',
  'kube',
  'rust',
  'kb',
  'video',
  'talks',
  'docs',
  'containers',
  'mixed-case',
  'aws',
  'nu',
  'linux',
  'gradle',
  'grails',
  'schedule',
  'ai',
  'react',
  'café'
]
writeFileSync(fixtureOut, `${JSON.stringify(Object.fromEntries(sampleTags.map((t) => [t, tagAccent(t)])), null, 2)}\n`)
```

(import `ACCENTS`, `LIGHT_ACCENTS`, `tagAccent` from `./color.mjs`; `mkdirSync` the fixture folder.) Run
`node tools/design/theme.mjs && dprint fmt crates/kb-app/assets/themes/*.json crates/kb-app/tests/fixtures/*.json`.

- [ ] **Step 2: Failing tests in `presentation.rs`.** Replace `gives_each_type_its_badge` and
      `hashes_tags_to_a_stable_hue_whatever_their_case` with:

```rust
#[test]
fn hashes_tags_as_the_raycast_extension_does() {
    let fixture: std::collections::BTreeMap<String, String> =
        serde_json::from_str(include_str!("../tests/fixtures/tag-accents.json")).unwrap();
    for (tag, accent) in fixture {
        assert_eq!(format!("{:?}", tag_hue(&tag)).to_lowercase(), accent, "#{tag}");
    }
    assert_eq!(tag_hue("Git"), tag_hue("git"));
}

#[test]
fn tints_each_type_as_raycast() {
    assert_eq!(type_badge("bookmarks"), TypeBadge { icon: IconName::Globe, hue: Hue::Purple });
    assert_eq!(type_badge("Commands"), TypeBadge { icon: IconName::SquareTerminal, hue: Hue::Green });
    assert_eq!(type_badge("cheats"), TypeBadge { icon: IconName::FileText, hue: Hue::Blue });
    assert_eq!(type_badge("notes"), TypeBadge { icon: IconName::NotebookText, hue: Hue::Orange });
    assert_eq!(type_badge("tasks"), TypeBadge { icon: IconName::CircleCheck, hue: Hue::Magenta });
    assert_eq!(type_badge("snippets").hue, tag_hue("snippet"));
}
```

`serde_json` is already a dev-dependency (Plan 6). Run `cargo nextest run -p kb-app presentation` — FAIL (no
`Hue::Purple`, glyph field still present).

- [ ] **Step 3: Implement in `presentation.rs`.** `Hue` becomes the seven accents (no `Neutral`, no `Cyan`); `TypeBadge`
      loses `glyph`; the Raycast hash:

```rust
/// A colour role among the seven accents of DESIGN.md (`accents.json`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hue { Blue, Green, Orange, Purple, Magenta, Yellow, Red }

/// The order tags and unknown types hash into, as the Raycast extension's `accessories.ts`.
const TAG_ORDER: [Hue; 7] = [Hue::Blue, Hue::Orange, Hue::Green, Hue::Purple, Hue::Magenta, Hue::Yellow, Hue::Red];

/// The Raycast extension's hash: `acc × 31 + code point`, modulo 100 003, over the characters.
fn hash(value: &str) -> usize {
    value.chars().fold(0_usize, |acc, c| (acc * 31 + c as usize) % 100_003)
}

/// A tag's hue: its lowercased name hashed, so a tag keeps its colour everywhere.
#[must_use]
pub fn tag_hue(tag: &str) -> Hue {
    TAG_ORDER[hash(&tag.to_lowercase()) % TAG_ORDER.len()]
}

/// The badge for an entry type: Raycast's icon and tint; an unknown type hashes its singular name.
#[must_use]
pub fn type_badge(entry_type: &str) -> TypeBadge {
    let (icon, hue) = match type_key(entry_type) {
        TypeKey::Bookmark => (IconName::Globe, Hue::Purple),
        TypeKey::Command => (IconName::SquareTerminal, Hue::Green),
        TypeKey::Cheat => (IconName::FileText, Hue::Blue),
        TypeKey::Note => (IconName::NotebookText, Hue::Orange),
        TypeKey::Task => (IconName::CircleCheck, Hue::Magenta),
        TypeKey::Other => {
            let name = entry_type.trim().to_lowercase();
            (IconName::FileText, tag_hue(name.strip_suffix('s').unwrap_or(&name)))
        }
    };
    TypeBadge { icon, hue }
}
```

- [ ] **Step 4: `accents.rs`.** Failing test first, then the module:

```rust
//! The seven accents of DESIGN.md, which GPUI Kit's theme has no keys for: each with its tag-pill fill and text, for
//! the dark and light appearances. Generated by `tools/design/theme.mjs` into `assets/themes/accents.json`.

use std::collections::HashMap;
use std::sync::LazyLock;

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{App, Hsla, Rgba};
use serde::Deserialize;

use crate::presentation::Hue;

/// An accent, and its tag pill.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Accent {
    /// Icons, bars, the type tint.
    pub color: Hsla,
    /// The tag pill's fill.
    pub pill: Hsla,
    /// The tag pill's text.
    pub pill_text: Hsla,
}

#[derive(Deserialize)]
struct Raw { accent: String, background: String, text: String }

#[derive(Deserialize)]
struct Sets { dark: HashMap<String, Raw>, light: HashMap<String, Raw> }

static SETS: LazyLock<Sets> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../assets/themes/accents.json")).unwrap_or(Sets {
        dark: HashMap::new(),
        light: HashMap::new(),
    })
});

fn hsla(hex: &str) -> Hsla {
    u32::from_str_radix(hex.trim_start_matches('#'), 16).map_or_else(|_| Hsla::default(), |n| Rgba::from(gpui_kit::rgb(n)).into())
}

fn key(hue: Hue) -> &'static str {
    match hue {
        Hue::Blue => "blue", Hue::Green => "green", Hue::Orange => "orange", Hue::Purple => "purple",
        Hue::Magenta => "magenta", Hue::Yellow => "yellow", Hue::Red => "red",
    }
}

/// The accent for `hue` in the current appearance.
#[must_use]
pub fn accent(hue: Hue, cx: &App) -> Accent {
    let set = if cx.theme().is_dark() { &SETS.dark } else { &SETS.light };
    set.get(key(hue)).map_or(Accent { color: cx.theme().foreground, pill: cx.theme().muted, pill_text: cx.theme().foreground }, |raw| Accent {
        color: hsla(&raw.accent),
        pill: hsla(&raw.background),
        pill_text: hsla(&raw.text),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_every_accent_for_both_appearances() {
        for hue in [Hue::Blue, Hue::Green, Hue::Orange, Hue::Purple, Hue::Magenta, Hue::Yellow, Hue::Red] {
            assert!(SETS.dark.contains_key(key(hue)) && SETS.light.contains_key(key(hue)), "{hue:?}");
        }
    }
}
```

`serde_json` moves from dev-dependencies to dependencies in `crates/kb-app/Cargo.toml` (it is in the workspace).
Register `mod accents;` in `lib.rs`. In `entry_list.rs`, delete `hue_color` and use `accent(hue, cx).color` for
icons and `Tag::custom(a.pill, a.pill_text, a.pill)` for pills; the type badge becomes the tinted icon alone
(`Icon::new(badge.icon).small().text_color(accent(badge.hue, cx).color)`), no bordered square.

- [ ] **Step 5:** `cargo nextest run -p kb-app` — PASS. `mise run gate`. Commit
      `feat(app): Tint types and tags with DESIGN.md's seven accents` (add `tools/design/*.mjs`, the two generated
      JSON files, `crates/kb-app`). Extend `tools/design/tasks.toml`'s `design:palettes` dprint step to the new files.

### Task 2: Raycast's labels in Title Case

**Files:** modify `presentation.rs`, `palette.rs` (labels only here), `filter.rs`, `search_view.rs`, tests.

- [ ] **Step 1: Failing tests.** In `presentation.rs` `matches_the_raycast_primary_action_matrix`, the titles become
      `"Open in Browser"`, `"Copy Command Snippet"`, `"Copy Cheat Sheet"`, `"Copy Notes"`, `"Copy Task"`,
      `"Copy Primary Content"` (Raycast's `typePrimaryActionOf`). In `filter.rs` tests, "Clear all filters" becomes
      "Clear All Filters" and "All entries" becomes "All Entries". Run: FAIL.
- [ ] **Step 2: Implement.** Change those strings; the query placeholder becomes `"Search Knowledge Base…"`; the
      footer's "Show details" becomes "Show Details"; sync messages stay sentences.
- [ ] **Step 3:** tests PASS; gate; commit `feat(app): Use Raycast's Title Case labels`.

### Task 3: Raycast's keys: → / ←, caret-aware

**Files:** create `crates/kb-app/src/keys.rs`; modify `palette.rs`, `search_view.rs`, `lib.rs`, `tests/ui.rs`.

- [ ] **Step 1: Failing UI tests** (`tests/ui.rs`):

```rust
#[gpui_kit::test]
fn shows_details_with_right_at_the_end_of_the_query_and_hides_them_with_left(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    press(cx, window, "right");
    assert!(cx.read(|cx| view.read(cx).is_showing_details()));
    press(cx, window, "left");
    assert!(!cx.read(|cx| view.read(cx).is_showing_details()));
}

#[gpui_kit::test]
fn moves_the_caret_with_right_and_left_inside_the_query(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    press(cx, window, "left");
    assert!(!cx.read(|cx| view.read(cx).is_showing_details()), "left at the end, pane hidden: caret moves");
    press(cx, window, "right");
    assert!(!cx.read(|cx| view.read(cx).is_showing_details()), "right mid-query: caret moves");
    press(cx, window, "right");
    assert!(cx.read(|cx| view.read(cx).is_showing_details()), "right at the end: pane shows");
}

#[gpui_kit::test]
fn shows_and_hides_the_metadata_with_right_and_left_on_the_detail_screen(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    press(cx, window, "enter");
    press(cx, window, "right");
    assert!(cx.read(|cx| view.read(cx).is_showing_metadata()));
    press(cx, window, "left");
    assert!(!cx.read(|cx| view.read(cx).is_showing_metadata()));
}
```

Keep `opens_on_the_list_alone_and_toggles_the_split_pane_with_cmd_d` (the alias). Run: FAIL.

- [ ] **Step 2: `keys.rs`.** Move `actions!`, `CONTEXT`, `DETAIL_CONTEXT` and `key_bindings` out of `palette.rs`
      (re-export nothing; update imports). Add the actions and bindings:

```rust
actions!(kb, [
    // …the existing ten…
    /// Show the split pane (list) or the metadata (detail screen).
    ShowDetails,
    /// Hide the split pane (list) or the metadata (detail screen).
    HideDetails,
    /// Switch the window between its compact and expanded sizes.
    ToggleWindowSize,
]);

// in key_bindings(), besides the existing ones:
// → and ← reach these only where no text field takes them first: the detail screen. In the list
// the query field binds them deeper; `SearchView` captures them there (`caret_at_end`).
KeyBinding::new("right", ShowDetails, context),
KeyBinding::new("left", HideDetails, context),
KeyBinding::new("cmd-shift-f", ToggleWindowSize, context),

/// Whether → and ← act on the pane rather than the caret: no selection, and the caret after the last character.
#[must_use]
pub fn caret_at_end(cursor: usize, selection: std::ops::Range<usize>, len: usize) -> bool {
    selection.is_empty() && cursor == len
}
```

Unit test in `keys.rs`: `caret_at_end(7, 7..7, 7)` true; `(3, 3..3, 7)` false; `(7, 2..7, 7)` false; `(0, 0..0, 0)`
true. Update `binds_every_shortcut_in_the_search_context` to 13.

- [ ] **Step 3: Capture in `search_view.rs`.** In the header wrapper that already captures `MoveUp`/`MoveDown`:

```rust
.capture_action(cx.listener(|this, _: &MoveRight, window, cx| {
    if !this.show_details && this.caret_at_end(cx) {
        this.show_details(&ShowDetails, window, cx);
        cx.stop_propagation();
    }
}))
.capture_action(cx.listener(|this, _: &MoveLeft, window, cx| {
    if this.show_details && this.caret_at_end(cx) {
        this.hide_details(&HideDetails, window, cx);
        cx.stop_propagation();
    }
}))
```

```rust
fn caret_at_end(&self, cx: &App) -> bool {
    let input = self.query_input.read(cx);
    caret_at_end(input.cursor(), input.selected_range(), input.value().len())
}

fn show_details(&mut self, _: &ShowDetails, _: &mut Window, cx: &mut Context<Self>) {
    match self.screen { Screen::Detail => self.show_metadata = true, _ => self.show_details = true }
    cx.notify();
}

fn hide_details(&mut self, _: &HideDetails, _: &mut Window, cx: &mut Context<Self>) {
    match self.screen { Screen::Detail => self.show_metadata = false, _ => self.show_details = false }
    cx.notify();
}

/// Whether the detail screen shows its metadata.
#[must_use]
pub fn is_showing_metadata(&self) -> bool { self.show_metadata }
```

Register both with `.on_action` in `render`. `MoveLeft`/`MoveRight` come from `gpui_kit::component::input`.

- [ ] **Step 4:** tests PASS; gate; commit `feat(app): Toggle the split pane with → and ← as Raycast`.

### Task 4: ⌘⇧F toggles the window size, centred

**Files:** create `crates/third-party/gpui-pre-macos/` (copy of the 0.3.8 crate), `crates/kb-app/src/window_size.rs`;
modify `Cargo.toml`, `crates/third-party/README.md`, `search_view.rs`, `panel.rs`, `tests/ui.rs`.

- [ ] **Step 1: Failing tests.** `window_size.rs` unit tests:

```rust
#[test]
fn expands_to_85_percent_of_the_display_and_back() {
    let display = size(px(1800.), px(1100.));
    assert_eq!(WindowSize::Compact.toggled().size(display), size(px(1530.), px(935.)));
    assert_eq!(WindowSize::Expanded.toggled().size(display), COMPACT);
}

#[test]
fn never_expands_below_the_compact_size() {
    assert_eq!(WindowSize::Expanded.size(size(px(900.), px(560.))), COMPACT);
}
```

UI test: `cmd-shift-f` then `cx.read(|cx| view.read(cx).window_size())` is `Expanded`; again → `Compact`. Run: FAIL.

- [ ] **Step 2: `window_size.rs`:**

```rust
//! The panel's two sizes (DESIGN.md, Layout): compact, and expanded to most of the display. ⌘⇧F switches them.

use gpui_kit::{Pixels, Size, px, size};

/// The compact size, in points: Raycast's 750 × 475, enlarged so the split pane leaves the notes room.
pub const COMPACT: Size<Pixels> = Size { width: px(860.), height: px(520.) };
/// The expanded share of the display's visible area.
const EXPANDED_SHARE: f32 = 0.85;

/// The panel's size.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WindowSize {
    /// 860 × 520.
    #[default]
    Compact,
    /// 85% of the display, never smaller than compact.
    Expanded,
}

impl WindowSize {
    /// The other size.
    #[must_use]
    pub fn toggled(self) -> Self {
        match self { Self::Compact => Self::Expanded, Self::Expanded => Self::Compact }
    }

    /// The window size on a display of `display`.
    #[must_use]
    pub fn size(self, display: Size<Pixels>) -> Size<Pixels> {
        match self {
            Self::Compact => COMPACT,
            Self::Expanded => size(
                (display.width * EXPANDED_SHARE).max(COMPACT.width),
                (display.height * EXPANDED_SHARE).max(COMPACT.height),
            ),
        }
    }
}
```

`panel::window_options` uses `COMPACT`. In `SearchView`: a `window_size: WindowSize` field, a `window_size()`
getter, and

```rust
fn toggle_window_size(&mut self, _: &ToggleWindowSize, window: &mut Window, cx: &mut Context<Self>) {
    self.window_size = self.window_size.toggled();
    let display = window.display(cx).map_or(COMPACT, |d| d.visible_bounds().size);
    window.resize(self.window_size.size(display));
    cx.notify();
}
```

(registered with `.on_action`). If `visible_bounds` is not on `PlatformDisplay` in 0.3.8, use `bounds()`.

- [ ] **Step 3: Patch `resize` to keep the centre.** `cp -R ~/.local/share/cargo/registry/src/index.crates.io-*/gpui-pre-macos-0.3.8 crates/third-party/gpui-pre-macos`,
      remove its `.cargo_vcs_info.json` and `Cargo.toml.orig`, add to the root `Cargo.toml`:

```toml
[patch.crates-io]
block = { path = "crates/third-party/block" }
gpui-pre-macos = { path = "crates/third-party/gpui-pre-macos" }
```

In `crates/third-party/gpui-pre-macos/src/window.rs` `fn resize`, replace the `setContentSize_` call:

```rust
if_window_not_closed(closed, || unsafe {
    // kb: keep the window's centre (crates/third-party/README.md). setContentSize keeps the
    // bottom-left corner, so a panel growing in place would leave the screen's centre.
    let frame = NSWindow::frame(window);
    let (width, height) = (size.width.as_f32() as f64, size.height.as_f32() as f64);
    let origin = NSPoint::new(
        frame.origin.x + (frame.size.width - width) / 2.,
        frame.origin.y + (frame.size.height - height) / 2.,
    );
    window.setFrame_display_(NSRect::new(origin, NSSize::new(width, height)), YES);
})
```

(the panel has no title bar, so frame and content sizes match). Add the row to `crates/third-party/README.md`:
crate `gpui-pre-macos 0.3.8`; why: "GPUI resizes a window from its bottom-left corner and has no API to move one;
kb's ⌘⇧F needs the panel to stay centred"; change: "`src/window.rs` `resize`: sets the frame around the old centre";
drop when: "GPUI can position or centre a window". Apache-2.0. Check the `domain-purity`, `deny` and `machete` steps
still pass (`mise run gate`).

- [ ] **Step 4:** tests PASS; `mise run show`, press ⌘⇧F twice: the panel grows and shrinks around its centre. Commit
      `feat(app): Toggle the window between compact and expanded with ⌘⇧F`.

### Task 5: Header

**Files:** create `crates/kb-app/src/header.rs`; modify `search_view.rs`, `tests/ui.rs`.

- [ ] **Step 1: Failing UI test:** after `enter`, the element `query-field` is absent and `back-button` is present;
      in the list, `query-field` and `filter-button` are present. Use `window.click` failing as absence? Instead expose
      `SearchView::header_parts(&self) -> HeaderParts { query: bool, filter: bool }` computed from the screen and assert
      on it (`HeaderParts { query: false, filter: false }` on the detail screen). Run: FAIL.
- [ ] **Step 2: `header.rs`.** `pub struct HeaderParts { pub query: bool, pub filter: bool }` and
      `pub fn header_parts(detail: bool) -> HeaderParts` (both false on the detail screen). `render_header` moves here
      as `pub fn header(view: &SearchView, …)`, with: no `border_b_1`; height `h(px(56.))`; horizontal padding
      `px(px(24.))` so the leading glyph sits on the row-icon axis (`layout.json`: header inset 24 = selection inset 8 +
      row inset 16); a leading `Icon::new(IconName::Search)` (list) or the `back-button` chevron (detail screen);
      `gap(px(12.))` before the field; the field `.text_lg()`; the filter button `.icon(IconName::Funnel)`, its label,
      and a trailing `ChevronUp` while the dropdown is open, `ChevronDown` otherwise (no `dropdown_caret`).
- [ ] **Step 3:** tests PASS; gate; commit `feat(app): Draw the header as Raycast's`.

### Task 6: List rows, favicons and tag images

**Files:** create `crates/kb-app/src/tag_images.rs`; modify `config.rs`, `entry_list.rs`, `presentation.rs`,
`mise.toml`, `tests/ui.rs`.

- [ ] **Step 1: Failing tests.** `config.rs`: `tag_images_path = "~/art"` resolves under home; default is
      `~/.config/kb/images`. `tag_images.rs`:

```rust
#[test]
fn finds_a_tag_image_whatever_the_case_and_ignores_other_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("linux.svg"), "<svg/>").unwrap();
    std::fs::write(dir.path().join("notes.txt"), "").unwrap();
    let images = TagImages::scan(dir.path());
    assert_eq!(images.get("Linux"), Some(dir.path().join("linux.svg").as_path()));
    assert_eq!(images.get("notes"), None);
}

#[test]
fn a_missing_folder_means_no_images() {
    assert_eq!(TagImages::scan(std::path::Path::new("/nonexistent")).get("linux"), None);
}
```

`presentation.rs`: `favicon_url("https://www.youtube.com/watch?v=x")` is
`Some("https://icons.duckduckgo.com/ip3/www.youtube.com.ico")`; `favicon_url("not a url")` is `None`. Run: FAIL.

- [ ] **Step 2: Implement.**

```rust
//! Tag images (DESIGN.md, Tag pill): a tag is drawn as `<tag>.svg` when the tag-images folder has it, as in the
//! Raycast extension. The folder is read once, at start.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The SVGs of the tag-images folder, by lowercased tag.
#[derive(Debug, Clone, Default)]
pub struct TagImages(HashMap<String, PathBuf>);

impl TagImages {
    /// Reads `dir`; a missing or unreadable folder gives no images.
    #[must_use]
    pub fn scan(dir: &Path) -> Self {
        let entries = std::fs::read_dir(dir).into_iter().flatten().flatten();
        Self(entries.filter_map(|e| {
            let path = e.path();
            (path.extension()? == "svg").then(|| (path.file_stem()?.to_str()?.to_lowercase(), path.clone()))?
        }).collect())
    }

    /// The image for `tag`, if any.
    #[must_use]
    pub fn get(&self, tag: &str) -> Option<&Path> {
        self.0.get(&tag.to_lowercase()).map(PathBuf::as_path)
    }
}
```

```rust
/// The bookmark's favicon from DuckDuckGo's icon service (DESIGN.md, Adaptations); `None` without a host.
#[must_use]
pub fn favicon_url(url: &str) -> Option<String> {
    let rest = url.split_once("://")?.1;
    let host = rest.split(['/', '?', '#']).next().filter(|h| !h.is_empty())?;
    Some(format!("https://icons.duckduckgo.com/ip3/{host}.ico"))
}
```

`Config` gains `tag_images: PathBuf` (`tag_images_path` in `Preferences`, `~` expanded as `sources_path` is;
default `home.join(".config/kb/images")`) and `tag_images()`. `EntryList` holds a `TagImages` (scanned in
`SearchView::new` from `config.tag_images()`). `render_row`:

- the row: `ListItem` with `.rounded(px(12.))`, horizontal margin 8 and padding 16 (`layout.json` resultRow), height 50;
- the icon column (24): for a bookmark with a favicon, `img(url).size_4().with_fallback(move || Icon::new(IconName::Globe)…)`,
  else the tinted type icon;
- the title `.text_sm()` regular; the description `.text_sm()` muted (same size as the title, DESIGN.md §3);
- per shown tag: `img(path).size_4()` with `.tooltip(format!("#{tag}"))` when `TagImages::get` has it, else the pill
  `Tag::custom(a.pill, a.pill_text, a.pill).xsmall().rounded(px(10.))`;
- `+N` muted; then, with links, `Icon::new(IconName::Link).xsmall()` and the count, muted.

`mise.toml`:

```toml
[tasks.images]
description = "Copy the Raycast extension's tag images into ~/.config/kb/images (read-only on $KB_TS_REPO)"
run = '''
  #!/usr/bin/env bash
  set -euo pipefail
  src="${KB_TS_REPO:?set KB_TS_REPO}/apps/kb/assets/images"
  mkdir -p ~/.config/kb/images
  cp -n "$src"/*.svg ~/.config/kb/images/
  ls ~/.config/kb/images | wc -l
'''
```

- [ ] **Step 3:** tests PASS; gate; `mise run images && mise run show`: `#linux` rows show the penguin, YouTube rows
      their favicon. Commit `feat(app): Draw rows as Raycast's, with favicons and tag images`.

### Task 7: Split pane, metadata card, notes and detail screen

**Files:** create `crates/kb-app/src/metadata.rs`; modify `notes.rs`, `presentation.rs`, `search_view.rs`,
`tests/ui.rs`.

- [ ] **Step 1: Failing tests.** `presentation.rs`: `thumbnail(&youtube_bookmark)` is
      `Some("https://img.youtube.com/vi/aqz-KE-bpKQ/hqdefault.jpg")` (from `kb_domain::markdown::youtube_video_id` on the
      primary content); `None` for a command. `metadata.rs`: `metadata_rows(&task)` keeps `presentation::metadata`'s
      order and marks `Type` and `Tags` as pills and `URL`/`Source` with icons:

```rust
#[test]
fn marks_pills_and_links_in_the_card() {
    let bookmark = entry("bookmarks", "https://b.example", "tags: [git]\n");
    let rows = metadata_rows(&bookmark);
    assert_eq!(rows[0], MetaRow { label: "URL", value: MetaValue::Link("https://b.example".into()) });
    assert!(matches!(rows[1].value, MetaValue::Pills(ref p) if p == &[("Bookmarks".to_owned(), Hue::Purple)]));
    assert!(matches!(rows.last().unwrap().value, MetaValue::File(_)));
}
```

UI test: on the detail screen `view.read(cx).detail_shows_title_block()` is false. Run: FAIL.

- [ ] **Step 2: `metadata.rs`.**

```rust
//! The metadata card (DESIGN.md, Metadata card): zebra rows, label left, value right; Type and Tags as pills, the URL
//! with ↗, the source with a file icon.

/// A value in the card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaValue { Text(String), Pills(Vec<(String, Hue)>), Link(String), File(String) }

/// One card row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaRow { pub label: &'static str, pub value: MetaValue }

/// The card's rows, in `presentation::metadata`'s order.
#[must_use]
pub fn metadata_rows(entry: &Entry) -> Vec<MetaRow> {
    metadata(entry).into_iter().map(|(label, value)| MetaRow { label, value: match label {
        "URL" => MetaValue::Link(value),
        "Type" => MetaValue::Pills(vec![(value, type_badge(&entry.entry_type).hue)]),
        "Tags" => MetaValue::Pills(entry.tags.iter().map(|t| (format!("#{t}"), tag_hue(t))).collect()),
        "Source" => MetaValue::File(value),
        _ => MetaValue::Text(value),
    }}).collect()
}

/// Renders the card.
pub fn metadata_card(entry: &Entry, cx: &App) -> impl IntoElement { /* v_flex of rows: odd rows .bg(muted at 40%),
   h_flex justify_between, px 12 py 4, label .text_color(muted_foreground) .text_xs(), value per MetaValue: Text →
   .text_xs(); Pills → Tag::custom per accent; Link → Button::new(..).link().label(url).icon(IconName::ArrowUpRight)
   .on_click(open_url); File → h_flex(Icon File, text). Card: .border_1().border_color(theme.border).rounded(px(10.)).overflow_hidden() */ }
```

(write the render body exactly as the comment says; the card is the only renderer of `MetaValue`).

`notes.rs`: `notes_pane(entry, show_metadata, layout, cx)` where `layout` is `PaneLayout::Split` (card below the
notes) or `PaneLayout::Detail` (card in a right column ⅓ wide); the thumbnail (`img(url).w_full().rounded(px(12.))`)
first in the pane when `thumbnail(entry)` is `Some`. The description quote's red bar: TextView renders blockquotes
with `theme.border`; wrap the pane in a `TextViewStyle` override if 0.7.1 offers one (`TextView::style`), else draw
the description as its own `div().border_l_3().border_color(accent(Hue::Red, cx).color).pl_2()` above the TextView
and drop the `> description` line from the Markdown passed to it (`notes_markdown_without_description`, tested).

`search_view.rs`: the split's list panel width becomes ⅓ of the window (`window.bounds().size.width / 3.`) instead
of `LIST_WIDTH_REMS`; the detail screen drops the title and type lines (`detail_shows_title_block` returns false and
documents why: the footer names the entry).

- [ ] **Step 3:** tests PASS; gate; `mise run show`: compare screens 2, 3, 4, 7, 8. Commit
      `feat(app): Show the metadata card, thumbnails and Raycast's split`.

### Task 8: Footer pills

**Files:** create `crates/kb-app/src/footer.rs`; modify `search_view.rs`.

- [ ] **Step 1: Failing test** in `footer.rs`: `keycaps("cmd-k")` is `["⌘", "K"]`, `keycaps("cmd-enter")` is
      `["⌘", "↵"]`, `keycaps("cmd-shift-,")` is `["⌘", "⇧", ","]`. Run: FAIL.
- [ ] **Step 2: Implement** `pub fn keycaps(binding: &str) -> Vec<String>` (split on `-`; map `cmd`→⌘, `shift`→⇧,
      `alt`→⌥, `ctrl`→⌃, `enter`→↵, `escape`→⎋, `right`→→, `left`→←, else uppercase) and `footer(...)`: no border, two
      groups floating `px(8.)` from the panel's edges, each `rounded_full()`, `.bg(theme.popover)`,
      `.border_1().border_color(theme.border)`, `.shadow_md()`; left the entry's tinted icon and title (truncate);
      right `footer-primary-action` (label, `Kbd` ↵) and `footer-open-actions` ("Actions", one `Kbd` per keycap of
      the ⌘K binding). The ids stay, so the existing tests keep passing.
- [ ] **Step 3:** tests PASS; gate; commit `feat(app): Float the footer as Raycast's pills`.

### Task 9: Actions popover

**Files:** create `crates/kb-app/src/actions_popover.rs`; modify `palette.rs`, `search_view.rs`, `tests/ui.rs`.

- [ ] **Step 1: Failing tests.** `palette.rs`:

```rust
#[test]
fn offers_the_raycast_actions_for_a_bookmark_with_links() {
    let bookmark = entry("bookmarks", "https://b.example", "desc: D\nlinks: [{Guide: https://g.example}]\n");
    assert_eq!(labels(&entry_palette(Some(&bookmark), false)), [
        ("", vec!["Show Details", "Open in Browser"]),
        ("Source & Clipboard", vec!["Copy URL", "Show Details", "Open Source File"]),
        ("", vec!["Links", "Copy…"]),
        ("Administration", vec!["Rebuild Index…", "Open Preferences", "Toggle Window Size", "Quit kb"]),
    ]);
}

#[test]
fn opens_copy_and_links_as_pages() {
    let bookmark = entry("bookmarks", "https://b.example", "desc: D\nlinks: [{Guide: https://g.example}]\n");
    assert_eq!(labels(&page(&bookmark, Page::Copy)), [("Copy…", vec!["Copy Title", "Copy Description", "Copy Notes", "Copy Raw YAML"])]);
    assert_eq!(labels(&page(&bookmark, Page::Links)), [("Guide", vec!["Open Guide", "Copy URL"])]);
}
```

(the first row "Show Details" is the detail screen; the one in Source & Clipboard is the split pane, shown as
"Hide Details" while it shows, bound to → / ←). UI test: ⌘K, type "copy", Enter on "Copy…", type "title", Enter
copies the title; ⌘K, the popover row for Open Source File shows the ⌘ O keycaps
(`view.read(cx).actions_shortcut("Open Source File") == Some(vec!["⌘","O"])`). Run: FAIL.

- [ ] **Step 2: Data in `palette.rs`.** `PaletteCommand` gains `OpenPage(Page)` with
      `pub enum Page { Main, Links, Copy }`; `entry_palette` returns the main page in Raycast's order (above);
      `page(entry, Page::Copy | Page::Links)` returns those pages; rows keep their Action (`Run`) so keycaps resolve.
      The first row runs a new `OpenDetail` Action (Return in the list); add it to `keys.rs` without a binding.
- [ ] **Step 3: `actions_popover.rs`.** The footer's Actions button becomes the trigger of
      `Popover::new("actions").anchor(Anchor::BottomRight).open(self.actions_open).on_open_change(...)`, whose content
      is the existing `Command` built from the current page, `.max_h(px(360.))`, `.bordered(false)`, and the search
      field moved under the list: first try `.flex_col_reverse()` on the `Command`; if the field stays on top, use
      `.searchable(false)` and `.footer(...)` with kb's own `Input` bound to `CommandState::set_query`. A row with
      `OpenPage(p)` sets `self.actions_page = p` and keeps the popover open; ← with an empty query or Esc on a page
      returns to `Page::Main`; Esc on the main page closes it. ⌘K toggles `actions_open`. Delete `open_palette` and
      the dialog. The Command rows show their Action's binding (they now render inside the panel's key context).
- [ ] **Step 4:** tests PASS (update `copies_the_title_from_the_action_palette` to the drill-in); gate; commit
      `feat(app): Anchor the actions to the footer, with keycaps and pages`.

### Task 10: Filter dropdown

**Files:** create `crates/kb-app/src/filter_dropdown.rs`; modify `filter.rs`, `search_view.rs`, `tests/ui.rs`,
`docs/guides/parity.md` (T3).

- [ ] **Step 1: Failing tests.** `task_t3_filters_by_a_type_and_a_tag_with_two_picks` becomes one visit:

```rust
press(cx, window, "cmd-p");
cx.run_until_parked();
for choice in ["Commands", "#docker"] {
    type_text(cx, window, choice);
    cx.run_until_parked();
    press(cx, window, "enter");
    settle(cx);
}
assert!(cx.read(|cx| view.read(cx).is_filter_open()), "stays open while picking");
press(cx, window, "escape");
assert!(!cx.read(|cx| view.read(cx).is_filter_open()));
```

`filter.rs`: the Filters section reads "2 Filters" / "Clear All Filters" while a filter is active (Raycast UI-19).
Run: FAIL.

- [ ] **Step 2: Implement.** The filter button is the trigger of
      `Popover::new("filter").anchor(Anchor::TopRight).open(self.filter_open)`; content is a `Command` of the
      groups with `.checked(choice.checked)` rows and a leading ✓ / ○ (`Icon::new(IconName::Check)` /
      `IconName::Circle`); confirming applies the filter, clears the dropdown's query and keeps it open; Esc or ⌘P
      closes it. Delete the dialog path. `is_filter_open()` getter.
- [ ] **Step 3:** `parity.md` T3: kb sequence "⌘P, type, ↩, tag, ↩, Esc", kb count 4. Tests PASS; gate; commit
      `feat(app): Keep the filter open under its button while picking`.

### Task 11: Visual pass against the reference

- [ ] `mise run show`; for each of the eight screens of `tools/reports/` (local), compare by eye and fix spacing or
      colour drift in the component that owns it (DESIGN.md §5 values from `layout.json`). Re-run the gate. Commit
      any fix as `fix(app): <what>`.

### Task 12: Docs, TODO, PR, merge, release

- [ ] `DESIGN.md`: Adaptations — remove the ⌘D row; add "Favicons from DuckDuckGo's icon service | Raycast's own
      service | `api.ray.so` serves Raycast", "Tag images from `tag_images_path` | bundled in the extension | mixed
      licences stay out of the public repo", "Window keeps its centre through a patched GPUI | — | GPUI cannot move a
      window". §4 Actions popover: pages for Links and Copy….
- [ ] `docs/guides/parity.md`: A01 Same (keycaps); S12 Same (thumbnail); S09 and A06 Same (→ / ←); T3 updated.
- [ ] `docs/guides/compliance.md`: tick the Title Case item.
- [ ] `README.md`: a "Keys" table (↩, ⌘↩, → / ←, ⌘K, ⌘P, ⌘⇧C, ⌘O, ⌘R, ⌘⇧,, ⌘⇧F, ⌘Q, Esc); `tag_images_path` in
      the preferences example; `mise run images` in the Development Workflow table.
- [ ] `CONTEXT.md`: **Tag image** (`tag_images.rs`) and **Size toggle** (`window_size.rs`).
- [ ] `TODO.md` (todo CLI): add sub-tasks of eWka per task above as they land, tick them, then tick eWka with the merge
      commit; keep "measurement session after the polish" pending.
- [ ] `git ls-files tools/reports` empty; `mise run gate`; push `feat/components`; `gh pr create -R roalcantara/kbx`
      (body: what changed, findings settled, deviations, test plan, ending "🤖 Generated with [Claude Code]…");
      wait for CI green; merge (merge commit); watch the release: v0.4.0 tag, GitHub Release with the zip.

## Self-review

- Spec coverage: K1 → Task 3; K2 → Tasks 9, 10; K3 → Tasks 6, 7; K4 → Task 6; K5 → Task 1; header/row/split/card/
  notes/detail/footer/window/copy → Tasks 5, 6, 7, 8, 4, 2; docs → Task 12; release → Task 12.
- Known unknowns, each with its fallback in the step: the Command search field's position (Task 9), TextView's
  blockquote colour (Task 7), `PlatformDisplay::visible_bounds` (Task 4).
