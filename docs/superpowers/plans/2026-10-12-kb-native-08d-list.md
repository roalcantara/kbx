# Plan 8d: The list screen — keycaps, chips, tiles, rows, header, footer, notice

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** the list screen matches `01-list.html`: 20 pt keycaps and chips in their live or rest state (D2, D3), rows
with their tinted tiles or favicons, mono command titles, tag icon chips and grey pills, the `↗ n` link chip and a
keycap (`↵` selected, `⌘1`…`⌘9` the first nine rows) (D5); the header with the magnifier and the filter pill (D4); the
footer with the status line and the screen's keys (D14); ⌘1–⌘9 run a row's primary action; every copy shows the
_Copied to clipboard_ notice (D17); the screen ladder runs the catalog's actions (D7, D8).

**Architecture:** five small components in `crates/kb-app/src/ui/` (`keycap`, `chip`, `tile`, `row`, `header`,
`footer`, `notice`) draw from `design()` alone. `EntryList` renders `ui::row::Row` (its own `Selectable` element, so
GPUI Kit's list paints nothing of its own). `Catalog::keys_for` resolves a screen's keys to labels and actions;
`SearchView::ladder` runs them.

**Tech Stack:** Rust, GPUI Kit 0.7.1 (`List`, `Button`, `Notification`, `Tooltip`).

**Expects:** Plans 8a–8c done (`design()`, `pt`, `TextRoleExt`, `no_ligatures`, `Catalog`, `Screen`,
`EntryList::count`, `EntryList::with_types`).

**Index:** [2026-10-12-kb-native-08-design-system.md](2026-10-12-kb-native-08-design-system.md).

---

## File structure

| File                                       | Change | Responsibility                                                    |
| ------------------------------------------ | ------ | ----------------------------------------------------------------- |
| `crates/kb-app/src/ui/design.rs`           | modify | `glow`, `pixels`                                                  |
| `crates/kb-app/src/ui/keycap.rs`           | create | `ChipState`, `caps`, `keycap`                                     |
| `crates/kb-app/src/ui/chip.rs`             | create | `pill`, `link_chip`, `icon_chip`                                  |
| `crates/kb-app/src/ui/tile.rs`             | create | `tile`, `favicon`, `TileSize`                                     |
| `crates/kb-app/src/ui/row.rs`              | create | `Row`                                                             |
| `crates/kb-app/src/ui/header.rs`           | create | `header`, `lead_icon`, `filter_pill`                              |
| `crates/kb-app/src/ui/footer.rs`           | create | `footer`, `status`, `entry_status`, `key_item`, `Dot`             |
| `crates/kb-app/src/ui/notice.rs`           | create | `copied`, `configure`                                             |
| `crates/kb-app/src/ui.rs`                  | modify | the new modules                                                   |
| `crates/kb-app/src/catalog.rs`             | modify | `KeyItem`, `Catalog::keys_for`                                    |
| `crates/kb-app/src/presentation.rs`        | modify | `row_tags(tags, max)`, `row_label(entry, type_label)`             |
| `crates/kb-app/src/entry_list.rs`          | modify | Renders `Row`; holds the catalog                                  |
| `crates/kb-app/src/keys.rs`                | modify | Ladder actions, `RunSecondary`, `RunRow1`…`RunRow9`               |
| `crates/kb-app/src/search_view.rs`         | modify | `ladder`, `run_action`, `copy`, `run_row`; test accessors          |
| `crates/kb-app/src/search_view/header.rs`  | modify | Composes `ui::header`                                             |
| `crates/kb-app/src/search_view/footer.rs`  | modify | Composes `ui::footer`                                             |
| `crates/kb-app/src/lib.rs`                 | modify | Notice placement                                                  |
| `crates/kb-app/tests/ui.rs`                | modify | Ladder, ⌘n, status line, notice, header tests                     |

---

### Task 1: Keycap, chips and tiles

**Files:** create `ui/keycap.rs`, `ui/chip.rs`, `ui/tile.rs`; modify `ui.rs`, `ui/design.rs` (all under
`crates/kb-app/src/`).

- [ ] **Step 1: Write the failing tests.** Create `crates/kb-app/src/ui/keycap.rs`:

```rust
//! D2's keycap (01-chips): one cap per shortcut (`⌘K`, `⇧⌘,`), 20 tall, JetBrains Mono 11.

use gpui_kit::{Div, Hsla, Keystroke, ParentElement as _, SharedString, Styled as _, div};

use crate::ui::design::{TextRoleExt as _, design, pt};

/// A chip's or keycap's state (D3): live on every keycap and on the selected row's chips; rest
/// elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipState {
    /// Mantle fill, muted text.
    Live,
    /// Grey (badge) fill.
    Rest,
}

impl ChipState {
    /// `Live` on the selected row, `Rest` elsewhere.
    #[must_use]
    pub fn of(selected: bool) -> Self {
        if selected { Self::Live } else { Self::Rest }
    }

    /// The fill.
    #[must_use]
    pub fn fill(self) -> Hsla {
        let colors = &design().colors;
        match self {
            Self::Live => colors.mantle,
            Self::Rest => colors.badge,
        }
    }
}

/// GPUI's key spelling as one keycap's text: modifiers in macOS order (⌃ ⌥ ⇧ ⌘), then the key.
#[must_use]
pub fn caps(keys: &str) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_one_cap_per_shortcut_in_macos_order() {
        assert_eq!(caps("cmd-k"), "⌘K");
        assert_eq!(caps("shift-cmd-enter"), "⇧⌘↵");
        assert_eq!(caps("cmd-shift-,"), "⇧⌘,");
        assert_eq!(caps("cmd-alt-c"), "⌥⌘C");
        assert_eq!(caps("escape"), "esc");
        assert_eq!(caps("cmd-1"), "⌘1");
    }

    #[test]
    fn fills_live_with_mantle_and_rest_with_the_badge_grey() {
        let colors = &design().colors;
        assert_eq!(ChipState::of(true).fill(), colors.mantle);
        assert_eq!(ChipState::of(false).fill(), colors.badge);
    }
}
```

Create `crates/kb-app/src/ui/tile.rs`:

```rust
//! The type tile (01-list): the type's accent at 15 % behind its icon in the accent; a bookmark's
//! favicon in the same box, the tile when it fails to load.

use gpui_kit::{Div, Hsla, IntoElement as _, ParentElement as _, Styled as _, StyledImage as _, div, img, svg};

use crate::ui::design::{design, pt};

/// Where a tile sits, which sets its size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileSize {
    /// A list row: 24, icon 16.
    Row,
    /// The footer and the pane header: 20, icon 13.
    Small,
    /// The filter's rows: 18, icon 12.
    Mini,
}

impl TileSize {
    /// The box and the icon, in points.
    #[must_use]
    pub fn points(self) -> (f32, f32) {
        let t = &design().components.tile;
        match self {
            Self::Row => (t.size, t.icon),
            Self::Small => (t.small, t.small_icon),
            Self::Mini => (t.mini, t.mini_icon),
        }
    }
}

/// The tinted tile of `icon` (a Lucide name) in `tint`.
#[must_use]
pub fn tile(icon: &str, tint: Hsla, size: TileSize) -> Div {
    let (box_size, glyph) = size.points();
    div()
        .flex_none()
        .size(pt(box_size))
        .rounded(pt(design().rounded.tile))
        .flex()
        .items_center()
        .justify_center()
        .bg(tint.opacity(design().components.tile.alpha))
        .child(svg().path(format!("icons/{icon}.svg")).size(pt(glyph)).text_color(tint))
}

/// A bookmark's favicon from `url`, centred in the tile's box; the tile of `icon` if it fails.
#[must_use]
pub fn favicon(url: String, icon: String, tint: Hsla, size: TileSize) -> Div {
    let (box_size, _) = size.points();
    let tokens = design();
    div()
        .flex_none()
        .size(pt(box_size))
        .flex()
        .items_center()
        .justify_center()
        .child(
            img(url)
                .size(pt(tokens.components.favicon.size))
                .rounded(pt(tokens.rounded.favicon))
                .with_fallback(move || tile(&icon, tint, size).into_any_element()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_tiles_by_where_they_sit() {
        assert_eq!(TileSize::Row.points(), (24., 16.));
        assert_eq!(TileSize::Small.points(), (20., 13.));
        assert_eq!(TileSize::Mini.points(), (18., 12.));
    }
}
```

Create `crates/kb-app/src/ui/chip.rs`:

```rust
//! D3's chips (01-list, 01-chips), 20 tall with 4 pt corners: tag pills, tag icon chips, the
//! `↗ n` link chip and text chips (a task's status and priority).

use gpui_kit::component::h_flex;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::{
    Div, Hsla, InteractiveElement as _, IntoElement as _, ParentElement as _, SharedString, Stateful,
    StatefulInteractiveElement as _, Styled as _, div, img, svg,
};

use crate::tag_images::TagImage;
use crate::ui::design::{TextRoleExt as _, design, pt};
use crate::ui::keycap::ChipState;

/// A text chip: `#tag`, `+2`, a task's status. Its text is muted in both states.
#[must_use]
pub fn pill(text: impl Into<SharedString>, state: ChipState) -> Div {
    let d = design();
    div()
        .flex_none()
        .h(pt(d.components.chip.height))
        .px(pt(d.spacing.chip.pad_x))
        .flex()
        .items_center()
        .rounded(pt(d.rounded.chip))
        .bg(state.fill())
        .text_color(d.colors.muted)
        .text_role(&d.typography.roles.chip)
        .child(text.into())
}

/// The `↗ n` link chip: its text muted when live, faint at rest.
#[must_use]
pub fn link_chip(count: usize, state: ChipState) -> Div {
    let d = design();
    let color = link_text(state);
    h_flex()
        .flex_none()
        .h(pt(d.components.chip.height))
        .px(pt(d.spacing.chip.link_pad_x))
        .gap(pt(d.spacing.chip.link_gap))
        .items_center()
        .rounded(pt(d.rounded.chip))
        .bg(state.fill())
        .text_color(color)
        .text_role(&d.typography.roles.count)
        .child(svg().path("icons/arrow-up-right.svg").size(pt(d.components.chip.link_icon)).text_color(color))
        .child(count.to_string())
}

/// The link chip's text colour in `state`.
#[must_use]
pub fn link_text(state: ChipState) -> Hsla {
    let colors = &design().colors;
    match state {
        ChipState::Live => colors.muted,
        ChipState::Rest => colors.faint,
    }
}

/// A tag drawn as its image: a shape takes the tag's hue, a brand keeps its colours; its name in
/// a tooltip, since the image carries no text.
#[must_use]
pub fn icon_chip(id: SharedString, image: &TagImage, hue: Hsla, state: ChipState, name: String) -> Stateful<Div> {
    let d = design();
    let glyph = pt(d.components.chip.icon);
    let drawn = if image.coloured {
        img(image.path.clone()).size(glyph).into_any_element()
    } else {
        svg()
            .external_path(image.path.to_string_lossy().into_owned())
            .size(glyph)
            .text_color(hue)
            .into_any_element()
    };
    div()
        .id(id)
        .flex_none()
        .size(pt(d.components.chip.height))
        .flex()
        .items_center()
        .justify_center()
        .rounded(pt(d.rounded.chip))
        .bg(state.fill())
        .child(drawn)
        .tooltip(move |window, cx| Tooltip::new(name.clone()).build(window, cx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dims_the_link_count_at_rest_only() {
        let colors = &design().colors;
        assert_eq!(link_text(ChipState::Live), colors.muted);
        assert_eq!(link_text(ChipState::Rest), colors.faint);
    }
}
```

Change `crates/kb-app/src/ui.rs` to:

```rust
//! The components the approved pages show, one file each, and the design tokens they read.

pub mod chip;
pub mod design;
pub mod keycap;
pub mod tile;
```

Run: `cargo nextest run -p kb-app ui::`
Expected: FAIL: `draws_one_cap_per_shortcut_in_macos_order` panics at `todo!()` (the others pass).

- [ ] **Step 2: Write `caps` and `keycap`.** In `ui/keycap.rs`, replace the `caps` stub with:

```rust
/// GPUI's key spelling as one keycap's text: modifiers in macOS order (⌃ ⌥ ⇧ ⌘), then the key.
#[must_use]
pub fn caps(keys: &str) -> String {
    let Ok(keystroke) = Keystroke::parse(keys) else {
        return keys.to_owned();
    };
    let m = keystroke.modifiers;
    let mut text: String = [(m.control, "⌃"), (m.alt, "⌥"), (m.shift, "⇧"), (m.platform, "⌘")]
        .into_iter()
        .filter_map(|(held, cap)| held.then_some(cap))
        .collect();
    text.push_str(&match keystroke.key.as_str() {
        "enter" => "↵".to_owned(),
        "escape" => "esc".to_owned(),
        "right" => "→".to_owned(),
        "left" => "←".to_owned(),
        "up" => "↑".to_owned(),
        "down" => "↓".to_owned(),
        "backspace" => "⌫".to_owned(),
        "tab" => "⇥".to_owned(),
        "space" => "␣".to_owned(),
        key => key.to_uppercase(),
    });
    text
}

/// A keycap reading `text`: live (mantle, muted text) or, on the popover and notice surfaces,
/// rest (badge, faint text).
#[must_use]
pub fn keycap(text: impl Into<SharedString>, state: ChipState) -> Div {
    let d = design();
    let text_color = match state {
        ChipState::Live => d.colors.muted,
        ChipState::Rest => d.colors.faint,
    };
    div()
        .flex_none()
        .h(pt(d.components.keycap.height))
        .min_w(pt(d.components.keycap.min_width))
        .px(pt(d.spacing.keycap.pad_x))
        .flex()
        .items_center()
        .justify_center()
        .rounded(pt(d.rounded.chip))
        .bg(state.fill())
        .text_color(text_color)
        .text_role(&d.typography.roles.keycap)
        .child(text.into())
}
```

- [ ] **Step 3: Add the glow and pixel helpers.** In `ui/design.rs`, add `BoxShadow, Pixels, point, px` to the
      `use gpui_kit::{…}` list and add after `pt`:

```rust
/// `points` as GPUI pixels, for the few APIs that take pixels (shadows, window frames, notice
/// margins). One point is one pixel at the base font size.
#[must_use]
pub fn pixels(points: f32) -> Pixels {
    px(points)
}

/// A glow around an element: `color` at `alpha`, blurred `blur` points (the selection bar, the
/// ready dot).
#[must_use]
pub fn glow(color: Hsla, blur: f32, alpha: f32) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: color.opacity(alpha),
        offset: point(px(0.), px(0.)),
        blur_radius: px(blur),
        spread_radius: px(0.),
        inset: false,
    }]
}
```

- [ ] **Step 4: Run the tests and see them pass.**

Run: `cargo nextest run -p kb-app ui::`
Expected: PASS.

- [ ] **Step 5: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/src/ui.rs crates/kb-app/src/ui
git commit -m "feat(ui): Add the keycap, chip and tile components" \
  -m "20 pt keycaps and chips in their live or rest state (D2, D3); tinted type tiles and favicons." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: The row, and the list drawn with it

**Files:** create `ui/row.rs`; modify `ui.rs`, `presentation.rs`, `entry_list.rs`, `search_view.rs`.

- [ ] **Step 1: Write the failing test.** Append to `crates/kb-app/tests/ui.rs`:

```rust
#[gpui_kit::test]
fn marks_the_selected_row_with_return_and_the_first_nine_with_their_number(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (_window, view) = open(cx, sample_config(home.path()));
    let keys = cx.read(|cx| view.read(cx).list().read(cx).delegate().row_keycaps());
    assert_eq!(keys, ["↵", "⌘2", "⌘3", "⌘4", "⌘5", "⌘6", "⌘7", "⌘8"]);
}
```

Run: `cargo nextest run -p kb-app --test ui marks_the_selected_row`
Expected: FAIL to compile: no method `row_keycaps`.

- [ ] **Step 2: Write the row.** Create `crates/kb-app/src/ui/row.rs`:

```rust
//! D5's row (01-list): 56 tall, 4 apart, 6 pt corners; its lead (tile or favicon), title over
//! description, then whatever trails it; the selected row on `selected` with the 2.5 pt mauve bar
//! and its glow. The refresh report's rows are the same component (11-14-refresh).

use gpui_kit::base::Selectable;
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, Role, SharedString,
    Styled as _, Window, div,
};

use crate::ui::design::{TextRoleExt as _, design, glow, no_ligatures, pt};

/// A list row.
#[derive(IntoElement)]
pub struct Row {
    id: SharedString,
    label: SharedString,
    lead: Option<AnyElement>,
    title: SharedString,
    mono: bool,
    description: Option<SharedString>,
    trailing: Vec<AnyElement>,
    compact: bool,
    selected: bool,
}

impl Row {
    /// A row `id` reading `title`.
    #[must_use]
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        let title = title.into();
        Self {
            id: id.into(),
            label: title.clone(),
            lead: None,
            title,
            mono: false,
            description: None,
            trailing: Vec::new(),
            compact: false,
            selected: false,
        }
    }

    /// What assistive technology reads for the row.
    #[must_use]
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// The tile or favicon before the title.
    #[must_use]
    pub fn lead(mut self, lead: impl IntoElement) -> Self {
        self.lead = Some(lead.into_any_element());
        self
    }

    /// Draws the title in JetBrains Mono (commands).
    #[must_use]
    pub fn mono(mut self, mono: bool) -> Self {
        self.mono = mono;
        self
    }

    /// The line under the title.
    #[must_use]
    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = description.map(Into::into);
        self
    }

    /// What follows the text, 12 apart: tags, the link chip, the keycap.
    #[must_use]
    pub fn trailing(mut self, trailing: Vec<AnyElement>) -> Self {
        self.trailing = trailing;
        self
    }

    /// Beside the split pane: the narrower inset of `04-entry-template.html`.
    #[must_use]
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
}

impl Selectable for Row {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl RenderOnce for Row {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let d = design();
        let (colors, spacing, size) = (&d.colors, &d.spacing.row, &d.components.row);
        let (pad_x, gap) = if self.compact {
            (spacing.split_pad_x, spacing.split_gap)
        } else {
            (spacing.pad_x, spacing.gap)
        };
        let mono = self.mono;
        div()
            .id(self.id)
            .w_full()
            // Half the list's gap above and below: rows sit 4 apart.
            .py(pt(d.spacing.list.gap / 2.))
            .role(Role::Label)
            .aria_label(self.label)
            .child(
                h_flex()
                    .relative()
                    .w_full()
                    .h(pt(size.height))
                    .items_center()
                    .gap(pt(gap))
                    .px(pt(pad_x))
                    .rounded(pt(d.rounded.row))
                    .map(|row| {
                        if self.selected {
                            row.bg(colors.selected).child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .top(pt(size.bar_inset))
                                    .bottom(pt(size.bar_inset))
                                    .w(pt(size.bar_width))
                                    .rounded_full()
                                    .bg(colors.mauve)
                                    .shadow(glow(colors.mauve, size.glow, size.glow_alpha)),
                            )
                        } else {
                            row.hover(|style| style.bg(colors.hover))
                        }
                    })
                    .children(self.lead)
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(pt(spacing.text_gap))
                            .child(
                                div()
                                    .truncate()
                                    .text_role(&d.typography.roles.title)
                                    .text_color(colors.text)
                                    .when(mono, |title| {
                                        title.font_family(d.typography.mono.clone()).font_features(no_ligatures())
                                    })
                                    .child(self.title),
                            )
                            .when_some(self.description, |text, description| {
                                text.child(
                                    div()
                                        .truncate()
                                        .text_role(&d.typography.roles.description)
                                        .text_color(colors.muted)
                                        .child(description),
                                )
                            }),
                    )
                    .child(
                        h_flex()
                            .flex_none()
                            .items_center()
                            .gap(pt(spacing.trailing_gap))
                            .children(self.trailing),
                    ),
            )
    }
}
```

Add `pub mod row;` to `ui.rs` (sorted: after `pub mod keycap;`).

- [ ] **Step 3: Adapt the presentation helpers.** In `crates/kb-app/src/presentation.rs`:

  - replace `row_tags` with:

```rust
/// The tags a row shows (at most `max`) and how many more there are.
#[must_use]
pub fn row_tags(tags: &[String], max: usize) -> (&[String], usize) {
    let shown = tags.len().min(max);
    (&tags[..shown], tags.len() - shown)
}
```

  - delete `pub const MAX_ROW_TAGS: usize = 3;` and its doc comment;
  - replace `row_label` with:

```rust
/// What assistive technology reads for a list row: its visible text, in reading order, because
/// a row's children do not name it. "docker ps -a, Commands, List every container…, tags docker,
/// containers".
#[must_use]
pub fn row_label(entry: &Entry, type_label: &str) -> String {
    let mut parts = vec![entry.title.clone(), type_label.to_owned()];
    parts.extend(entry.description.clone());
    if !entry.tags.is_empty() {
        parts.push(format!("tags {}", entry.tags.join(", ")));
    }
    if !entry.links.is_empty() {
        parts.push(format!("{} links", entry.links.len()));
    }
    parts.join(", ")
}
```

  - in its tests: `collapses_tags_after_three` calls become `row_tags(&tags, 3)` and `row_tags(&tags[..2], 3)`;
    `names_a_row_by_its_visible_text` calls `row_label(&command, "Commands")`.

- [ ] **Step 4: Draw the list with rows.** In `crates/kb-app/src/entry_list.rs`:

  - replace the imports block (from `use std::time::Duration;` to `use crate::trace;`) with:

```rust
use std::rc::Rc;
use std::time::Duration;

use gpui_kit::component::empty::{Empty, EmptyDescription, EmptyHeader, EmptyTitle};
use gpui_kit::component::list::{ListDelegate, ListState};
use gpui_kit::component::{ActiveTheme as _, IndexPath, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{AnyElement, App, Context, IntoElement as _, ParentElement as _, SharedString, Styled as _, Task, Window};
use kb_domain::Connection;
use kb_domain::entry::Entry;
use kb_domain::row::row_to_entry;
use kb_domain::search::{FilterState, PAGE_SIZE, SearchQuery, count, entry_types, search};

use crate::catalog::{Catalog, Hero};
use crate::presentation::{favicon_url, row_label, row_tags, tag_hue};
use crate::tag_images::TagImages;
use crate::trace;
use crate::ui::chip::{icon_chip, link_chip, pill};
use crate::ui::design::{design, pt};
use crate::ui::keycap::{ChipState, caps, keycap};
use crate::ui::row::Row;
use crate::ui::tile::{TileSize, favicon, tile};
```

  - delete `pub const TAG_RADIUS: f32 = 4.;` and its comment (8f removes `metadata.rs`, its last user; until then
    change `metadata.rs`'s `use crate::entry_list::TAG_RADIUS;` to `const TAG_RADIUS: f32 = 4.;` defined in
    `metadata.rs` itself, with the comment `// Removed with this module in Plan 8f.`);
  - add the field `catalog: Rc<Catalog>,` after `compact: bool,` in `EntryList`, `catalog: Rc::new(Catalog::default()),`
    in `new`'s struct literal, and replace 8c's `with_types` with:

```rust
    /// Shows only the catalog's types (D6), drawn as the catalog says.
    #[must_use]
    pub fn with_catalog(mut self, catalog: Rc<Catalog>) -> Self {
        self.known_types = catalog.types.keys().cloned().collect();
        self.catalog = catalog;
        self.refresh();
        self
    }

    /// The keycap each row shows, in order, for tests: `↵` on the selected row, `⌘n` on the
    /// first nine (R7).
    #[must_use]
    pub fn row_keycaps(&self) -> Vec<String> {
        (0..self.entries.len())
            .filter_map(|ix| self.row_keycap(ix, Some(IndexPath::new(ix)) == self.selected))
            .collect()
    }

    fn row_keycap(&self, ix: usize, selected: bool) -> Option<String> {
        if selected {
            Some(caps("enter"))
        } else if ix < 9 {
            Some(caps(&format!("cmd-{}", ix + 1)))
        } else {
            None
        }
    }
```

  - replace `render_row` and `render_tag` with:

```rust
    /// A row as `01-list.html` draws it.
    fn render_row(&self, entry: &Entry, ix: usize, selected: bool) -> Row {
        let d = design();
        let kind = self.catalog.entry_type(&entry.entry_type);
        let (icon, tint, label, mono, media) = kind.map_or(("file-text", d.colors.faint, "", false, false), |kind| {
            (
                kind.icon.as_str(),
                d.colors.named(&kind.tint).unwrap_or(d.colors.faint),
                kind.label.as_str(),
                kind.mono_title,
                kind.hero == Hero::Media,
            )
        });
        let lead = match media.then(|| favicon_url(&entry.primary_content)).flatten() {
            Some(url) => favicon(url, icon.to_owned(), tint, TileSize::Row).into_any_element(),
            None => tile(icon, tint, TileSize::Row).into_any_element(),
        };
        let state = ChipState::of(selected);
        let mut trailing: Vec<AnyElement> = Vec::new();
        if !self.compact {
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a token of a few tags")]
            let max = d.components.row.max_tags as usize;
            let (tags, more) = row_tags(&entry.tags, max);
            if !tags.is_empty() {
                trailing.push(
                    h_flex()
                        .items_center()
                        .gap(pt(d.spacing.row.tags_gap))
                        .children(tags.iter().map(|tag| self.render_tag(&entry.id, tag, state)))
                        .when(more > 0, |group| group.child(pill(format!("+{more}"), state)))
                        .into_any_element(),
                );
            }
            if !entry.links.is_empty() {
                trailing.push(link_chip(entry.links.len(), state).into_any_element());
            }
        }
        if let Some(key) = self.row_keycap(ix, selected) {
            trailing.push(keycap(key, ChipState::Live).into_any_element());
        }
        Row::new(SharedString::from(entry.id.clone()), entry.title.clone())
            .label(row_label(entry, label))
            .lead(lead)
            .mono(mono)
            .description(entry.description.clone())
            .trailing(trailing)
            .compact(self.compact)
    }

    /// A tag: its image when the tag-images folder has one, else its grey pill.
    fn render_tag(&self, entry_id: &str, tag: &str, state: ChipState) -> AnyElement {
        match self.tag_images.image(tag) {
            Some(image) => icon_chip(
                SharedString::from(format!("tag-{entry_id}-{tag}")),
                image,
                design().colors.hue(tag_hue(tag)),
                state,
                format!("#{tag}"),
            )
            .into_any_element(),
            None => pill(format!("#{tag}"), state).into_any_element(),
        }
    }
```

  - in `impl ListDelegate for EntryList`, change `type Item = ListItem;` to `type Item = Row;` and `render_item` to:

```rust
    fn render_item(&mut self, ix: IndexPath, _: &mut Window, _: &mut Context<ListState<Self>>) -> Option<Row> {
        let entry = self.entries.get(ix.row)?;
        Some(self.render_row(entry, ix.row, Some(ix) == self.selected))
    }
```

- [ ] **Step 5: Give the list the catalog.** In `search_view.rs`, `SearchView::new`: build `let catalog = Rc::new(catalog);`
      right after the placeholder line, change the list's builder to `.with_catalog(catalog.clone())` (instead of
      8c's `.with_types(types)`, and delete the `let types = …` line), and set the field with `catalog,` (it is now
      an `Rc`).

- [ ] **Step 6: Pad the list.** In `search_view.rs`, `render_content`, the two `List::new(&self.list)` calls become
      `List::new(&self.list).px(pt(tokens.spacing.list.pad)).py(pt(tokens.spacing.list.pad - tokens.spacing.list.gap / 2.))`
      (rows carry half the gap themselves). `render_content` receives the tokens: add `let tokens = design();` as its
      first line.

- [ ] **Step 7: Run every kb-app test.**

Run: `cargo nextest run -p kb-app`
Expected: PASS, including `marks_the_selected_row_with_return_and_the_first_nine_with_their_number`.

- [ ] **Step 8: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/src crates/kb-app/tests/ui.rs
git commit -m "feat(ui): Draw the list's rows as the approved page" \
  -m "Tinted tiles or favicons, mono command titles, tag icon chips and grey pills, the link chip, ↵ and ⌘1–⌘9 (D5)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: The header

**Files:** create `ui/header.rs`; modify `ui.rs`, `search_view/header.rs`, `tests/ui.rs`.

- [ ] **Step 1: Write the failing test.** In `crates/kb-app/tests/ui.rs`, replace the test
      `shows_only_the_back_chevron_in_the_detail_screen_header` with:

```rust
#[gpui_kit::test]
fn shows_the_magnifier_on_the_list_and_only_the_chevron_on_the_detail_screen(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    let parts = cx.read(|cx| view.read(cx).header_parts());
    assert_eq!((parts.back, parts.query, parts.filter), (false, true, true));
    press(cx, window, "enter");
    let parts = cx.read(|cx| view.read(cx).header_parts());
    assert_eq!((parts.back, parts.query, parts.filter), (true, false, false));
}
```

Run: `cargo nextest run -p kb-app --test ui shows_the_magnifier`
Expected: FAIL: the list reports `back: true`.

- [ ] **Step 2: Write the component.** Create `crates/kb-app/src/ui/header.rs`:

```rust
//! D4's header (01-list, 07-filter): 56 tall, a hairline under it; the magnifier on the list, the
//! back chevron elsewhere; the query at 15; the filter pill (sliders, the filter's name, `⌘P`).

use gpui_kit::component::h_flex;
use gpui_kit::{Div, ParentElement as _, SharedString, Styled as _, div, svg};

use crate::ui::design::{TextRoleExt as _, design, pt};
use crate::ui::keycap::{ChipState, keycap};

/// The header's row; the caller adds the lead icon, the query and the pill.
#[must_use]
pub fn header() -> Div {
    let d = design();
    h_flex()
        .h(pt(d.components.header.height))
        .w_full()
        .flex_none()
        .items_center()
        .px(pt(d.spacing.header.pad_x))
        .gap(pt(d.spacing.header.gap))
        .border_b(pt(d.components.border.hairline))
        .border_color(d.colors.edge)
}

/// The lead icon (`search` on the list, `chevron-left` elsewhere), 20 faint.
#[must_use]
pub fn lead_icon(icon: &str) -> Div {
    let d = design();
    div().flex_none().child(
        svg()
            .path(format!("icons/{icon}.svg"))
            .size(pt(d.components.header.icon))
            .text_color(d.colors.faint),
    )
}

/// The query field's box: 15 pt text, the rest of the row.
#[must_use]
pub fn query_box() -> Div {
    let d = design();
    div()
        .flex_1()
        .min_w_0()
        .text_role(&d.typography.roles.search)
        .text_color(d.colors.text)
}

/// The filter pill: sliders, `label` (truncated), and the filter's keycap.
#[must_use]
pub fn filter_pill(label: impl Into<SharedString>, keys_caps: impl Into<SharedString>) -> Div {
    let d = design();
    h_flex()
        .flex_none()
        .items_center()
        .gap(pt(d.spacing.pill.gap))
        .px(pt(d.spacing.pill.pad_x))
        .py(pt(d.spacing.pill.pad_y))
        .rounded(pt(d.rounded.pill))
        .bg(d.colors.badge)
        .text_color(d.colors.muted)
        .text_role(&d.typography.roles.label)
        .child(
            svg()
                .path("icons/sliders-horizontal.svg")
                .size(pt(d.components.header.pill_icon))
                .text_color(d.colors.faint),
        )
        .child(div().max_w(pt(d.components.header.pill_max_width)).truncate().child(label.into()))
        .child(keycap(keys_caps, ChipState::Live))
}
```

Add `pub mod header;` to `ui.rs` (sorted, after `pub mod design;`).

- [ ] **Step 3: Compose it.** Replace `crates/kb-app/src/search_view/header.rs` entirely with:

```rust
//! The header on each screen (D4): the magnifier, the query and the filter pill on the list; the
//! back chevron alone elsewhere.

use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants as _};
use gpui_kit::component::input::{Input, MoveDown, MoveLeft, MoveRight, MoveUp};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{App, Context, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _};

use super::SearchView;
use crate::filter::filter_summary;
use crate::keys::{Back, HideDetails, ShowDetails};
use crate::ui::header::{filter_pill, header, lead_icon, query_box};
use crate::ui::keycap::caps;

/// What the header shows on the current screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderParts {
    /// The back chevron: every screen but the list, which shows the magnifier.
    pub back: bool,
    /// The query field: the list only.
    pub query: bool,
    /// The filter pill: the list only.
    pub filter: bool,
}

/// A button drawn as its children alone: no fill, even on hover (the header, the footer).
pub(super) fn plain_button(cx: &App) -> ButtonCustomVariant {
    ButtonCustomVariant::new(cx)
        .color(gpui_kit::transparent_black())
        .hover(gpui_kit::transparent_black())
        .active(gpui_kit::transparent_black())
        .shadow(false)
}

impl SearchView {
    /// What the header shows on the current screen.
    #[must_use]
    pub fn header_parts(&self) -> HeaderParts {
        let list = !self.is_showing_detail() && !self.is_rebuilding();
        HeaderParts {
            back: !list,
            query: list,
            filter: list,
        }
    }

    pub(super) fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let parts = self.header_parts();
        let row = header();
        if parts.back {
            return row.child(
                Button::new("back-button")
                    .custom(plain_button(cx))
                    .child(lead_icon("chevron-left"))
                    .accessibility_label("Back")
                    .tooltip("Back (Esc)")
                    .on_click(cx.listener(|this, _, window, cx| this.back(&Back, window, cx))),
            );
        }
        let filter = self.list.read(cx).delegate().filter();
        let label = filter_summary(filter, &self.catalog);
        let filter_caps = self.catalog.actions["filter"].keys.as_deref().map(caps).unwrap_or_default();
        row.child(lead_icon("search"))
            .child(
                // A single-line input consumes ↑ and ↓ without propagating them, so they are
                // captured here, before it, to move the list selection.
                query_box()
                    .capture_action(cx.listener(|this, _: &MoveUp, window, cx| this.move_selection(-1, window, cx)))
                    .capture_action(cx.listener(|this, _: &MoveDown, window, cx| this.move_selection(1, window, cx)))
                    // → and ← toggle the split pane, as in Raycast, but only with the caret at the
                    // query's end, so they still move it while editing.
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
                    .child(Input::new(&self.query_input).cleanable(true).appearance(false).p_0()),
            )
            .when(parts.filter, |row| {
                row.child(
                    self.filter_popover(
                        Button::new("filter-button")
                            .custom(plain_button(cx))
                            .child(filter_pill(label, filter_caps)),
                        cx,
                    ),
                )
            })
    }
}
```

- [ ] **Step 4: The filter's name from the catalog.** In `crates/kb-app/src/filter.rs`, replace `filter_summary` and
      its test `summarizes_the_filter` with:

```rust
/// The filter in a few words, for the pill: "All Entries", "Commands", "Commands, #k8s".
#[must_use]
pub fn filter_summary(filter: &FilterState, catalog: &Catalog) -> String {
    let parts: Vec<String> = filter
        .types
        .iter()
        .map(|t| catalog.entry_type(t).map_or_else(|| type_label(t), |kind| kind.label.clone()))
        .chain(filter.tags.iter().map(|t| format!("#{t}")))
        .collect();
    if parts.is_empty() {
        catalog.messages.all_entries.clone()
    } else {
        parts.join(", ")
    }
}
```

```rust
    #[test]
    fn summarizes_the_filter() {
        let catalog = Catalog::default();
        assert_eq!(filter_summary(&FilterState::default(), &catalog), "All Entries");
        assert_eq!(
            filter_summary(
                &FilterState {
                    types: strings(&["commands"]),
                    tags: strings(&["git"])
                },
                &catalog
            ),
            "Commands, #git"
        );
    }
```

and add `use crate::catalog::Catalog;` at the top of `filter.rs`.

In `search_view/footer.rs`, `plain_button` moved to `header.rs`: replace its definition there with
`use super::header::plain_button;` (keep its call sites).

- [ ] **Step 5: Run every kb-app test.**

Run: `cargo nextest run -p kb-app`
Expected: PASS. (`opens_the_actions_and_the_filter_with_a_click` still clicks `filter-button`.)

- [ ] **Step 6: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/src crates/kb-app/tests/ui.rs
git commit -m "feat(ui): Draw the header with the magnifier and the filter pill" \
  -m "The back chevron replaces the magnifier on the other screens (D4)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: The screen ladder, the footer and the status line

**Files:** create `ui/footer.rs`; modify `ui.rs`, `catalog.rs`, `keys.rs`, `search_view.rs`,
`search_view/footer.rs`, `tests/ui.rs`.

- [ ] **Step 1: Write the failing tests.** Append to `catalog.rs`'s `tests` module:

```rust
    #[test]
    fn resolves_a_screen_s_keys_for_the_selected_entry() {
        let catalog = Catalog::default();
        let labels = |screen, kind: &str| {
            catalog
                .keys_for(screen, catalog.types.get(kind))
                .into_iter()
                .map(|item| (item.label, item.caps, item.action))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            labels(Screen::List, "bookmarks"),
            [
                ("Details".into(), "↵".into(), "details".into()),
                ("Open".into(), "⌘↵".into(), "open".into()),
                ("Copy".into(), "⇧⌘↵".into(), "copy".into())
            ]
        );
        assert_eq!(
            labels(Screen::Detail, "commands"),
            [("Copy".into(), "↵".into(), "copy".into())],
            "a command has no secondary"
        );
        assert_eq!(catalog.keys_for(Screen::Refreshing, None)[0].caps, "esc");
    }
```

Append to `crates/kb-app/tests/ui.rs`:

```rust
#[gpui_kit::test]
fn shows_the_status_line_and_the_screen_keys_in_the_footer(cx: &mut TestAppContext) {
    let (_home, _window, view) = open_with_query(cx, "conference");
    let (status, keys) = cx.read(|cx| (view.read(cx).status_line(cx), view.read(cx).footer_keys(cx)));
    assert_eq!(status, "Ready • 1 result");
    assert_eq!(keys, ["Details ↵", "Open ⌘↵", "Copy ⇧⌘↵", "Actions ⌘K"]);
}

#[gpui_kit::test]
fn copies_a_bookmark_s_url_with_its_secondary_action(cx: &mut TestAppContext) {
    let (_home, window, _view) = open_with_query(cx, "conference");
    press(cx, window, "shift-cmd-enter");
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("https://www.youtube.com/watch?v=aqz-KE-bpKQ"));
}
```

Run: `cargo nextest run -p kb-app catalog && cargo nextest run -p kb-app --test ui shows_the_status_line copies_a_bookmark`
Expected: FAIL to compile: no method `keys_for`, `status_line`, `footer_keys`.

- [ ] **Step 2: Resolve a screen's keys.** In `catalog.rs`, add after the `Screen` enum:

```rust
/// One of a screen's keys, resolved for the selected entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyItem {
    /// The key in GPUI's spelling (`cmd-enter`).
    pub keys: &'static str,
    /// Its keycap text (`⌘↵`).
    pub caps: String,
    /// The action's label.
    pub label: String,
    /// The action's id.
    pub action: String,
}
```

and to `impl Catalog`:

```rust
    /// `screen`'s keys in footer order, each resolved to an action: `entry.primary` and
    /// `entry.secondary` become `kind`'s actions; a key whose action the entry lacks is left out.
    #[must_use]
    pub fn keys_for(&self, screen: Screen, kind: Option<&EntryType>) -> Vec<KeyItem> {
        let keys = self.screen(screen);
        SCREEN_KEYS
            .iter()
            .filter_map(|key| {
                let value = keys.get(*key)?;
                let action = match value.as_str() {
                    ENTRY_PRIMARY => kind?.primary.clone(),
                    ENTRY_SECONDARY => kind?.secondary.clone()?,
                    id => id.to_owned(),
                };
                Some(KeyItem {
                    keys: key,
                    caps: crate::ui::keycap::caps(key),
                    label: self.label(&action).to_owned(),
                    action,
                })
            })
            .collect()
    }
```

Run: `cargo nextest run -p kb-app catalog`
Expected: PASS.

- [ ] **Step 3: The ladder actions.** In `keys.rs`:
  - in `actions!`, add after `RunPrimary`:

```rust
        /// Run the selected entry's secondary action.
        RunSecondary,
        /// The screen's ↵ (its catalog key `enter`).
        ScreenEnter,
        /// The screen's ⌘↵.
        ScreenCmdEnter,
        /// The screen's ⇧⌘↵.
        ScreenShiftCmdEnter,
```

  - in `key_bindings`, replace `KeyBinding::new("enter", RunPrimary, Some(DETAIL_CONTEXT)),` and its comment with:

```rust
        // The list's ↵ and ⌘↵ come from the query field (it reports Enter, then propagates it), so
        // the ladder is bound here only where no text field has the keyboard: the detail screen.
        KeyBinding::new("enter", ScreenEnter, Some(DETAIL_CONTEXT)),
        KeyBinding::new("cmd-enter", ScreenCmdEnter, Some(DETAIL_CONTEXT)),
        // The query field does not bind ⇧⌘↵, so it reaches the panel everywhere.
        KeyBinding::new("shift-cmd-enter", ScreenShiftCmdEnter, context),
```

  - update the count test: `assert_eq!(bindings.len(), 18);` (7 catalog + 11 fixed) and its comment to
    `// 7 keyed catalog actions + enter, cmd-enter (detail), shift-cmd-enter, escape, cmd-q, right, left, down, up, cmd-down, cmd-up.`

- [ ] **Step 4: Run the ladder.** In `search_view.rs`:
  - add `RunSecondary, ScreenCmdEnter, ScreenEnter, ScreenShiftCmdEnter` to the `use crate::keys::{…}` list and
    `use crate::catalog::{KeyItem, Screen as KeyScreen};` and `use crate::ui::notice;` (the notice module comes in
    Task 6; until then leave the `copy` method's notice line out — Step 4 of Task 6 adds it);
  - replace `run_primary` with:

```rust
    /// The screen whose keys apply now (8g adds the empty and refresh screens).
    fn key_screen(&self) -> KeyScreen {
        match self.screen {
            Screen::Detail => KeyScreen::Detail,
            _ => KeyScreen::List,
        }
    }

    /// The current screen's keys, resolved for the selected entry.
    fn screen_keys(&self, cx: &App) -> Vec<KeyItem> {
        let selected = self.list.read(cx).delegate().selected_entry();
        let kind = selected.and_then(|entry| self.catalog.entry_type(&entry.entry_type));
        self.catalog.keys_for(self.key_screen(), kind)
    }

    /// Runs what the current screen binds to `keys` (`enter`, `cmd-enter`, `shift-cmd-enter`).
    fn ladder(&mut self, keys: &str, window: &mut Window, cx: &mut Context<Self>) {
        let action = self.screen_keys(cx).into_iter().find(|item| item.keys == keys).map(|item| item.action);
        if let Some(action) = action {
            self.run_action(&action, window, cx);
        }
    }

    /// Runs a catalog action by id on the selected entry.
    fn run_action(&mut self, action: &str, window: &mut Window, cx: &mut Context<Self>) {
        let content = self
            .list
            .read(cx)
            .delegate()
            .selected_entry()
            .map(|entry| entry.primary_content.clone());
        match (action, content) {
            ("details", _) => self.show_detail_screen(window, cx),
            ("open", Some(url)) => cx.open_url(&url),
            ("copy", Some(text)) => self.copy(text, window, cx),
            ("refresh", _) => self.rebuild(&Rebuild, window, cx),
            ("configure", _) => self.open_preferences(&OpenPreferences, window, cx),
            ("resize", _) => self.toggle_window_size(&ToggleWindowSize, window, cx),
            ("edit", _) => self.open_source_file(&OpenSourceFile, window, cx),
            ("copy-path", _) => self.copy_path(&CopyPath, window, cx),
            _ => {}
        }
    }

    /// Copies `text` and says so (D17).
    fn copy(&mut self, text: String, _window: &mut Window, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(text));
    }

    /// The selected entry's primary (`secondary` false) or secondary action.
    fn run_entry(&mut self, secondary: bool, window: &mut Window, cx: &mut Context<Self>) {
        let action = self.list.read(cx).delegate().selected_entry().and_then(|entry| {
            let kind = self.catalog.entry_type(&entry.entry_type)?;
            if secondary { kind.secondary.clone() } else { Some(kind.primary.clone()) }
        });
        if let Some(action) = action {
            self.run_action(&action, window, cx);
        }
    }

    fn run_primary(&mut self, _: &RunPrimary, window: &mut Window, cx: &mut Context<Self>) {
        self.run_entry(false, window, cx);
    }

    fn run_secondary(&mut self, _: &RunSecondary, window: &mut Window, cx: &mut Context<Self>) {
        self.run_entry(true, window, cx);
    }

    fn screen_enter(&mut self, _: &ScreenEnter, window: &mut Window, cx: &mut Context<Self>) {
        self.ladder("enter", window, cx);
    }

    fn screen_cmd_enter(&mut self, _: &ScreenCmdEnter, window: &mut Window, cx: &mut Context<Self>) {
        self.ladder("cmd-enter", window, cx);
    }

    fn screen_shift_cmd_enter(&mut self, _: &ScreenShiftCmdEnter, window: &mut Window, cx: &mut Context<Self>) {
        self.ladder("shift-cmd-enter", window, cx);
    }
```

  - in `copy_url` and `copy_path`, replace `cx.write_to_clipboard(ClipboardItem::new_string(url));` (and `path`)
    with `self.copy(url, window, cx);` (and `path`); rename their `_: &mut Window` parameter to `window`;
  - in `on_query_input_event`, replace the `InputEvent::PressEnter { secondary, .. }` arm's body with
    `self.ladder(if *secondary { "cmd-enter" } else { "enter" }, window, cx);`;
  - in `on_list_event`, replace the `ListEvent::Confirm(_)` arm's body with:

```rust
                let keys = if self.list.read(cx).delegate().is_secondary_confirm() { "cmd-enter" } else { "enter" };
                self.ladder(keys, window, cx);
```

  - in `render`, add after `.on_action(cx.listener(Self::run_primary))`:

```rust
            .on_action(cx.listener(Self::run_secondary))
            .on_action(cx.listener(Self::screen_enter))
            .on_action(cx.listener(Self::screen_cmd_enter))
            .on_action(cx.listener(Self::screen_shift_cmd_enter))
```

  - add the imports `Rebuild`, `OpenPreferences`, `ToggleWindowSize`, `OpenSourceFile`, `CopyPath` if missing;
    remove `PrimaryKind`, `primary_action` from the `presentation` import if now unused.

- [ ] **Step 5: Write the footer component.** Create `crates/kb-app/src/ui/footer.rs`:

```rust
//! D14's footer (01-list, 04-entry-template): 40 tall, a hairline over it; left the status line
//! or the entry's tile, file and `⌥⌘C`; right the screen's keys and _Actions ⌘K_.

use gpui_kit::component::h_flex;
use gpui_kit::{Div, IntoElement, ParentElement as _, SharedString, Styled as _, div};

use crate::ui::design::{TextRoleExt as _, design, glow, pt};
use crate::ui::keycap::{ChipState, keycap};

/// The status dot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dot {
    /// Green with its glow: the index is ready.
    Ready,
    /// Yellow: a refresh runs.
    Busy,
    /// Faint, no glow: no index.
    Off,
}

/// The footer's row; `entry` (a pane or the detail screen is open) narrows its right inset and
/// key gap, as the template page does.
#[must_use]
pub fn footer(left: impl IntoElement, right: Vec<gpui_kit::AnyElement>, entry: bool) -> Div {
    let d = design();
    let s = &d.spacing.footer;
    h_flex()
        .h(pt(d.components.footer.height))
        .w_full()
        .flex_none()
        .items_center()
        .justify_between()
        .pl(pt(s.pad_left))
        .pr(pt(if entry { s.entry_pad_right } else { s.pad_right }))
        .border_t(pt(d.components.border.hairline))
        .border_color(d.colors.edge)
        .text_role(&d.typography.roles.footer)
        .text_color(d.colors.faint)
        .child(div().min_w_0().child(left))
        .child(
            h_flex()
                .flex_none()
                .items_center()
                .gap(pt(if entry { s.entry_gap } else { s.gap }))
                .children(right),
        )
}

/// `● Ready • 12 results`: the dot, the word in muted text, then the count.
#[must_use]
pub fn status(dot: Dot, word: impl Into<SharedString>, count: Option<String>) -> Div {
    let d = design();
    let size = pt(d.components.footer.dot);
    let dot = match dot {
        Dot::Ready => div().size(size).rounded_full().bg(d.colors.green).shadow(glow(
            d.colors.green,
            d.components.footer.dot_glow,
            0.7,
        )),
        Dot::Busy => div().size(size).rounded_full().bg(d.colors.yellow).shadow(glow(
            d.colors.yellow,
            d.components.footer.dot_glow,
            0.7,
        )),
        Dot::Off => div().size(size).rounded_full().bg(d.colors.faint),
    };
    h_flex()
        .items_center()
        .gap(pt(d.spacing.footer.left_gap))
        .child(dot)
        .child(div().text_color(d.colors.muted).child(word.into()))
        .when_some_count(count)
}

/// Adds `• count` to the status line when there is a count.
trait WhenSomeCount {
    fn when_some_count(self, count: Option<String>) -> Self;
}

impl WhenSomeCount for Div {
    fn when_some_count(self, count: Option<String>) -> Self {
        match count {
            Some(count) => self.child("•").child(count),
            None => self,
        }
    }
}

/// The entry's tile, its file in mono and the copy-path keycap.
#[must_use]
pub fn entry_status(tile: impl IntoElement, file: impl Into<SharedString>, copy_caps: impl Into<SharedString>) -> Div {
    let d = design();
    h_flex()
        .min_w_0()
        .items_center()
        .gap(pt(d.spacing.footer.left_gap))
        .child(tile)
        .child(
            div()
                .truncate()
                .font_family(d.typography.mono.clone())
                .text_color(d.colors.muted)
                .child(file.into()),
        )
        .child(keycap(copy_caps, ChipState::Live))
}

/// A key on the right: its label in muted text and its keycap.
#[must_use]
pub fn key_item(label: impl Into<SharedString>, keys_caps: impl Into<SharedString>) -> Div {
    let d = design();
    h_flex()
        .items_center()
        .gap(pt(d.spacing.footer.item_gap))
        .text_color(d.colors.muted)
        .child(label.into())
        .child(keycap(keys_caps, ChipState::Live))
}
```

The glow alpha `0.7` is the dot's, as `01-list.html` draws it; move it into the tokens: add `dot_glow_alpha: 0.7` to
`components.footer` in DESIGN.md's front matter, add `dot_glow_alpha` to `FooterSize`'s field list in
`ui/design.rs`, and use `d.components.footer.dot_glow_alpha` in both `glow(…)` calls above instead of `0.7`.

Add `pub mod footer;` to `ui.rs` (sorted, after `pub mod design;`).

- [ ] **Step 6: Compose the footer.** Replace `crates/kb-app/src/search_view/footer.rs` entirely with:

```rust
//! The footer on each screen (D14): the status line on the list, the entry's file while a pane or
//! the detail screen is open; the screen's keys and Actions.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{AnyElement, App, Context, IntoElement, ParentElement as _, SharedString, Styled as _, Window};
use kb_domain::entry::Entry;

use super::header::plain_button;
use super::{Screen, SearchView};
use crate::catalog::fill;
use crate::ui::design::design;
use crate::ui::footer::{Dot, entry_status, footer, key_item, status};
use crate::ui::keycap::caps;
use crate::ui::tile::{TileSize, tile};

impl SearchView {
    /// Whether the footer names the entry rather than the index: a pane or the detail screen.
    fn footer_names_entry(&self, selected: Option<&Entry>) -> bool {
        selected.is_some() && (self.is_showing_detail() || (matches!(self.screen, Screen::Search) && self.show_details))
    }

    /// The status line's text, for tests: "Ready • 8 results".
    #[must_use]
    pub fn status_line(&self, cx: &App) -> String {
        let (word, count) = self.status_parts(cx);
        count.map_or_else(|| word.clone(), |count| format!("{word} • {count}"))
    }

    fn status_parts(&self, cx: &App) -> (String, Option<String>) {
        let messages = &self.catalog.messages;
        let list = self.list.read(cx).delegate();
        if !list.has_index() {
            return (messages.no_index_status.clone(), None);
        }
        let count = list.count();
        let count = if count == 1 {
            messages.one_result.clone()
        } else {
            fill(&messages.results, &[("count", &count.to_string())])
        };
        (messages.ready.clone(), Some(count))
    }

    /// The footer's right side as text, for tests: "Details ↵", …, "Actions ⌘K".
    #[must_use]
    pub fn footer_keys(&self, cx: &App) -> Vec<String> {
        let mut keys: Vec<String> = self
            .screen_keys(cx)
            .into_iter()
            .map(|item| format!("{} {}", item.label, item.caps))
            .collect();
        keys.push(format!("{} {}", self.catalog.label("actions"), self.actions_caps()));
        keys
    }

    fn actions_caps(&self) -> String {
        self.catalog.actions["actions"].keys.as_deref().map(caps).unwrap_or_default()
    }

    pub(super) fn render_footer(&self, selected: Option<&Entry>, _: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let names_entry = self.footer_names_entry(selected);
        let left: AnyElement = match selected.filter(|_| names_entry) {
            Some(entry) => {
                let colors = &design().colors;
                let kind = self.catalog.entry_type(&entry.entry_type);
                let (icon, tint) = kind.map_or(("file-text", colors.faint), |kind| {
                    (kind.icon.as_str(), colors.named(&kind.tint).unwrap_or(colors.faint))
                });
                let copy_caps = self.catalog.actions["copy-path"].keys.as_deref().map(caps).unwrap_or_default();
                entry_status(tile(icon, tint, TileSize::Small), entry.source_rel_path.clone(), copy_caps)
                    .into_any_element()
            }
            None => {
                let (word, count) = self.status_parts(cx);
                let dot = if self.list.read(cx).delegate().has_index() { Dot::Ready } else { Dot::Off };
                status(dot, word, count).into_any_element()
            }
        };
        let plain = plain_button(cx);
        let mut right: Vec<AnyElement> = self
            .screen_keys(cx)
            .into_iter()
            .enumerate()
            .map(|(ix, item)| {
                let id = if ix == 0 {
                    SharedString::from("footer-primary-action")
                } else {
                    SharedString::from(format!("footer-key-{ix}"))
                };
                let keys = item.keys;
                Button::new(id)
                    .custom(plain)
                    .child(key_item(item.label, item.caps))
                    .on_click(cx.listener(move |this, _, window, cx| this.ladder(keys, window, cx)))
                    .into_any_element()
            })
            .collect();
        right.push(
            self.actions_popover(
                Button::new("footer-open-actions")
                    .custom(plain)
                    // The popover opens on its trigger's click; a handler here would close it again.
                    .child(key_item(self.catalog.label("actions").to_owned(), self.actions_caps())),
                cx,
            )
            .into_any_element(),
        );
        footer(left, right, names_entry)
    }
}
```

(The old `keycaps`, `action_keycaps`, `keycap_row` and `pill` functions go; `overlays.rs` imports
`action_keycaps` and `keycap_row`: in `overlays.rs`, replace `use super::footer::{action_keycaps, keycap_row};` with
the two functions below, kept there until 8e rewrites the file.)

```rust
/// The keycap text of `action`'s binding in the panel, if it has one.
fn action_keycaps(action: &dyn gpui_kit::Action, window: &Window) -> Vec<String> {
    gpui_kit::KeyContext::parse(crate::keys::CONTEXT)
        .ok()
        .and_then(|context| window.highest_precedence_binding_for_action_in_context(action, context))
        .and_then(|binding| {
            binding
                .keystrokes()
                .first()
                .map(|key| vec![crate::ui::keycap::caps(&key.as_keystroke().unparse())])
        })
        .unwrap_or_default()
}

/// The keys as one live keycap.
fn keycap_row(caps: Vec<String>, _: &App) -> gpui_kit::Div {
    gpui_kit::div().child(crate::ui::keycap::keycap(caps.concat(), crate::ui::keycap::ChipState::Live))
}
```

- [ ] **Step 7: Run every kb-app test.**

Run: `cargo nextest run -p kb-app`
Expected: PASS, including `shows_the_status_line_and_the_screen_keys_in_the_footer`,
`copies_a_bookmark_s_url_with_its_secondary_action`, `the_action_bar_opens_details_and_runs_the_primary_action`,
`runs_the_primary_action_with_enter_on_the_detail_screen`, `task_t1_…` and `task_t2_…`.

- [ ] **Step 8: Gate and commit.**

```bash
mise run gate
git add DESIGN.md crates/kb-app/src crates/kb-app/tests/ui.rs
git commit -m "feat(ui): Run the screen ladder and show it in the footer" \
  -m "↵ / ⌘↵ / ⇧⌘↵ come from the catalog (D7, D8); the status line shows the true count (D14)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: ⌘1–⌘9

**Files:** modify `keys.rs`, `search_view.rs`, `tests/ui.rs`.

- [ ] **Step 1: Write the failing test.** Append to `crates/kb-app/tests/ui.rs`:

```rust
#[gpui_kit::test]
fn runs_a_row_s_primary_action_with_its_number(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, _view) = open(cx, sample_config(home.path()));
    // The sample's rows by title: A conference talk (a bookmark), All pods (a command), …
    press(cx, window, "cmd-2");
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("kubectl get pods -A"));
    press(cx, window, "cmd-1");
    assert_eq!(
        cx.opened_url().as_deref(),
        Some("https://www.youtube.com/watch?v=aqz-KE-bpKQ")
    );
}
```

Run: `cargo nextest run -p kb-app --test ui runs_a_row_s_primary_action_with_its_number`
Expected: FAIL: nothing is copied.

- [ ] **Step 2: Bind ⌘1–⌘9.** In `keys.rs`:
  - add to `actions!`, after `OpenDetail`:

```rust
        /// Run row 1's primary action (list) or open link 1 (detail screen).
        RunRow1,
        /// Row or link 2.
        RunRow2,
        /// Row or link 3.
        RunRow3,
        /// Row or link 4.
        RunRow4,
        /// Row or link 5.
        RunRow5,
        /// Row or link 6.
        RunRow6,
        /// Row or link 7.
        RunRow7,
        /// Row or link 8.
        RunRow8,
        /// Row or link 9.
        RunRow9,
```

  - in `key_bindings`'s `bindings.extend([…])`, append:

```rust
        // ⌘1–⌘9: row n's primary action in the list; link n on the detail screen (R7).
        KeyBinding::new("cmd-1", RunRow1, context),
        KeyBinding::new("cmd-2", RunRow2, context),
        KeyBinding::new("cmd-3", RunRow3, context),
        KeyBinding::new("cmd-4", RunRow4, context),
        KeyBinding::new("cmd-5", RunRow5, context),
        KeyBinding::new("cmd-6", RunRow6, context),
        KeyBinding::new("cmd-7", RunRow7, context),
        KeyBinding::new("cmd-8", RunRow8, context),
        KeyBinding::new("cmd-9", RunRow9, context),
```

  - the count test becomes `assert_eq!(bindings.len(), 27);` with `+ cmd-1…cmd-9` added to its comment.

- [ ] **Step 3: Run row n.** In `search_view.rs`, add `RunRow1, …, RunRow9` to the keys import and these methods:

```rust
    /// ⌘n: selects row n − 1 and runs its primary action (R7); the detail screen's links come in 8f.
    fn run_row(&mut self, n: usize, window: &mut Window, cx: &mut Context<Self>) {
        if !matches!(self.screen, Screen::Search) {
            return;
        }
        let exists = self.list.read(cx).delegate().entries().len() >= n;
        if !exists {
            return;
        }
        self.list.update(cx, |list, cx| list.set_selected_index(Some(IndexPath::new(n - 1)), window, cx));
        self.run_entry(false, window, cx);
    }
```

and in `render`, after the ladder `on_action` lines:

```rust
            .on_action(cx.listener(|this, _: &RunRow1, window, cx| this.run_row(1, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow2, window, cx| this.run_row(2, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow3, window, cx| this.run_row(3, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow4, window, cx| this.run_row(4, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow5, window, cx| this.run_row(5, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow6, window, cx| this.run_row(6, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow7, window, cx| this.run_row(7, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow8, window, cx| this.run_row(8, window, cx)))
            .on_action(cx.listener(|this, _: &RunRow9, window, cx| this.run_row(9, window, cx)))
```

- [ ] **Step 4: Run every kb-app test.**

Run: `cargo nextest run -p kb-app`
Expected: PASS.

- [ ] **Step 5: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/src crates/kb-app/tests/ui.rs
git commit -m "feat(app): Run the first nine rows' primary actions with ⌘1–⌘9" \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: The copy notice

**Files:** create `ui/notice.rs`; modify `ui.rs`, `lib.rs`, `search_view.rs`, `search_view/overlays.rs`,
`tests/ui.rs`.

- [ ] **Step 1: Write the failing test.** In `crates/kb-app/tests/ui.rs`, extend
      `copies_a_command_with_its_primary_action`: after its `assert_eq!(copied…)` add

```rust
    assert_eq!(
        cx.read(|cx| _view.read(cx).last_notice().map(str::to_owned)).as_deref(),
        Some("Copied to clipboard")
    );
```

(rename `_view` to `view` in that test's `let` and in the new lines).

Run: `cargo nextest run -p kb-app --test ui copies_a_command_with_its_primary_action`
Expected: FAIL to compile: no method `last_notice`.

- [ ] **Step 2: Write the notice.** Create `crates/kb-app/src/ui/notice.rs`:

```rust
//! D17's notice (01-list): bottom right above the footer, through GPUI Kit's Notification. The
//! copy notice reads _Copied to clipboard_ with a `⌘V` hint. Its restyle is parked (TODO MOB4).

use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Theme, h_flex};
use gpui_kit::{Anchor, Edges, IntoElement as _, ParentElement as _, Styled as _, div, svg};

use crate::catalog::Messages;
use crate::ui::design::{TextRoleExt as _, design, pixels, pt};
use crate::ui::keycap::{ChipState, keycap};

/// The copy notice's id: a new copy replaces the one on screen.
#[derive(Debug)]
pub struct CopyNotice;

/// Puts every notice bottom right, above the footer (D17).
pub fn configure(theme: &mut Theme) {
    let d = design();
    let s = &d.spacing.notice;
    theme.notification.placement = Anchor::BottomRight;
    theme.notification.margins = Edges {
        top: pixels(s.right),
        right: pixels(s.right),
        bottom: pixels(s.bottom),
        left: pixels(s.right),
    };
    theme.notification.width = pixels(d.components.notice.width);
}

/// _Copied to clipboard_, a check in a green ring, and the `⌘V to paste` hint on a rest keycap.
#[must_use]
pub fn copied(messages: &Messages) -> Notification {
    let (message, hint) = (messages.copied.clone(), messages.paste_hint.clone());
    Notification::new().id::<CopyNotice>().content(move |_, _, _| {
        let d = design();
        let (c, n) = (&d.colors, &d.components.notice);
        h_flex()
            .items_center()
            .gap(pt(d.spacing.notice.gap))
            .text_role(&d.typography.roles.notice)
            .text_color(c.text)
            .child(
                div()
                    .size(pt(n.icon))
                    .flex_none()
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(c.green.opacity(0.2))
                    .border_1()
                    .border_color(c.green.opacity(0.4))
                    .child(svg().path("icons/check.svg").size(pt(n.check)).text_color(c.green)),
            )
            .child(message.clone())
            .child(
                h_flex()
                    .items_center()
                    .gap(pt(d.spacing.notice.hint_gap))
                    .text_role(&d.typography.roles.description)
                    .text_color(c.muted)
                    .child(keycap("⌘V", ChipState::Rest))
                    .child(hint.clone()),
            )
            .into_any_element()
    })
}
```

The ring's `0.2` and `0.4` are `01-list.html`'s; move them into the tokens as the dot's glow was: add
`ring_fill: 0.2, ring_border: 0.4` to `components.notice` in DESIGN.md's front matter, add them to `NoticeSize` in
`ui/design.rs`, and use `n.ring_fill` and `n.ring_border` above.

Add `pub mod notice;` to `ui.rs` (sorted, after `pub mod keycap;`).

- [ ] **Step 3: Place the notices.** In `lib.rs`, `init_theme`, add as the last line:
      `ui::notice::configure(Theme::global_mut(cx));` (after the `theme.font_size = …` line, the `theme` borrow has
      ended; if the borrow checker objects, call it after a new `let theme = Theme::global_mut(cx);`).

- [ ] **Step 4: Show it on every copy.** In `search_view.rs`:
  - add the field `last_notice: Option<String>,` to `SearchView` (after `overlay`), `last_notice: None,` in `new`;
  - replace `copy` with:

```rust
    /// Copies `text` and says so (D17).
    fn copy(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.last_notice = Some(self.catalog.messages.copied.clone());
        window.push_notification(notice::copied(&self.catalog.messages), cx);
    }

    /// The last notice the view showed, for tests.
    #[must_use]
    pub fn last_notice(&self) -> Option<&str> {
        self.last_notice.as_deref()
    }
```

  In `search_view/overlays.rs`, `run`, replace
  `PaletteCommand::Copy(text) => cx.write_to_clipboard(ClipboardItem::new_string(text.clone())),` with:

```rust
        PaletteCommand::Copy(text) => {
            let text = text.clone();
            let _ = view.update(cx, |this, cx| this.copy(text, window, cx));
        }
```

  and change `fn copy` in `search_view.rs` to `pub(super) fn copy`. Remove `ClipboardItem` from `overlays.rs`'s
  imports if now unused.

- [ ] **Step 5: Run every kb-app test.**

Run: `cargo nextest run -p kb-app`
Expected: PASS.

- [ ] **Step 6: Gate and commit.**

```bash
mise run gate
git add DESIGN.md crates/kb-app/src crates/kb-app/tests/ui.rs
git commit -m "feat(ui): Say Copied to clipboard on every copy" \
  -m "Every notice moves bottom right above the footer (D17)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Done when

- `cargo nextest run -p kb-app` and `mise run gate` pass.
- `cargo run -p kb-app` over the showcase (8a Task 7, Step 6's scratch home) shows the list as `01-list.html`:
  compare side by side and note any difference for 8h's checklist.
