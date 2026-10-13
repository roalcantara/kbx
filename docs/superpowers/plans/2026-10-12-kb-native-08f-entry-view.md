# Plan 8f: The entry view, the command card and the Links section

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** the split pane and the detail screen show the entry template of `04-entry-template.html` (D11): a header
(the row's icon, the title on one line, the tags on the right, clickable to filter), a hero by type (D12: the command
card for commands and for cheat sheets with a snippet; a bookmark's YouTube thumbnail or its URL as a link line; a
task's status and priority chips), the description as a red-bar quote, the notes body, and — when the entry has links —
the Links section of `05-detail-links.html` (D20, option A1). The command card follows `04-command-cards.html` (D13).
There is no metadata column; → and ← do nothing on the detail screen.

**Architecture:** two new components. `ui/command_card.rs` takes plain GPUI values only (no kb types), so it can
become a crate later: fmt-rs breaks the command at operators, a pure `wrap` function wraps the rest at spaces with a
hanging indent, GPUI Kit's bash highlighter colours it, GPUI Kit's `Clipboard` copies it. `ui/entry_view.rs` draws the
four slots from the design tokens and the catalog. `search_view.rs` replaces `notes.rs` and `metadata.rs` with it.

**Tech Stack:** Rust, GPUI Kit 0.7.1 (`StyledText`, `SyntaxHighlighter`, `Clipboard`, `TextView`), `fmt-rs =0.1.0`.

**Expects** (verified on `feat/design-system` at `c13a09b`): `ui::design::{design, pt, pixels, no_ligatures,
TextRoleExt}`; tokens `typography.roles.{heading, quote, body, notes_h1, notes_h2, notes_h3, card_split, card_detail,
link_title, link_url, link_hero, section}`, `spacing.{pane, quote, card, link_hero, links}`, `components.{hero, quote,
links, split, tile, favicon, chip}`, `rounded.{card, hero, favicon}`; `ui::tile::{tile, favicon, TileSize}`;
`ui::chip::{pill, icon_chip}`; `ui::keycap::{keycap, caps, ChipState}`; `catalog::{Catalog, Hero}`;
`presentation::{favicon_url, tag_hue, trim_url}`; `kb_domain::markdown::{notes_body, youtube_video_id}`;
`EntryList::tag_images()`; `SearchView::{copy, apply_filter, run_row, render_content}`.

**Index:** [2026-10-12-kb-native-08-design-system.md](2026-10-12-kb-native-08-design-system.md) — its "Rules for
every task" apply, including the rule for a GPUI Kit signature that differs.

---

## File structure

| File                                        | Change | Responsibility                                                          |
| ------------------------------------------- | ------ | ----------------------------------------------------------------------- |
| `Cargo.toml`, `crates/kb-app/Cargo.toml`    | modify | `fmt-rs = "=0.1.0"`                                                     |
| `crates/kb-app/src/ui/command_card.rs`      | create | `wrap`, `CardStyle`, `SyntaxColors`, `CommandCard`                      |
| `crates/kb-app/src/ui/entry_view.rs`        | create | `EntryView` (the four slots and Links), `Layout`, `columns_for`         |
| `crates/kb-app/src/ui.rs`                   | modify | the two modules                                                         |
| `crates/kb-app/src/search_view.rs`          | modify | renders `EntryView`; ⌘n opens links on detail; tags filter; no metadata |
| `crates/kb-app/src/notes.rs`, `metadata.rs` | delete | replaced (keep `next_offset`, moved to `search_view.rs`)                |
| `crates/kb-app/src/lib.rs`                  | modify | module list                                                             |
| `crates/kb-app/tests/ui.rs`                 | modify | entry view tests                                                        |

---

### Task 1: The command card's wrapping, a pure function

**Files:** modify `Cargo.toml`, `crates/kb-app/Cargo.toml`; create `crates/kb-app/src/ui/command_card.rs`; modify
`crates/kb-app/src/ui.rs`.

- [ ] **Step 1: Add fmt-rs.** In the workspace `Cargo.toml`, `[workspace.dependencies]`, add (sorted, after
      `dirs`): `fmt-rs = "=0.1.0"`. In `crates/kb-app/Cargo.toml`, `[dependencies]`, add `fmt-rs.workspace = true`
      (sorted, after `dirs.workspace = true`). Run `cargo deny check licenses` and expect it to pass (fmt-rs is MIT).

- [ ] **Step 2: Write the failing tests.** Create `crates/kb-app/src/ui/command_card.rs`:

```rust
//! D13's command card (04-command-cards): a shell command in a monospace box, broken at its
//! operators by fmt-rs, the rest wrapped at spaces with a hanging indent, coloured by GPUI Kit's
//! bash highlighter, with a copy button. It takes plain GPUI values only, no kb types, so it can
//! become a crate of its own.

/// A wrapped command: `text` with the line breaks inserted, and `breaks`, the byte offsets in the
/// unwrapped text where a space became a break, with the text that replaced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrapped {
    /// The text to draw.
    pub text: String,
    /// (offset of the replaced space in the input, its replacement).
    pub breaks: Vec<(usize, String)>,
}

/// Wraps `formatted` (fmt-rs's output) to `columns`: a word moves to the next line, indented four
/// more than its line, when it would pass the column; a token is never split, and `&&`, `||`, `|`,
/// `;` and `\` stay with the word before them.
#[must_use]
pub fn wrap(formatted: &str, columns: usize) -> Wrapped {
    let _ = columns;
    Wrapped {
        text: formatted.to_owned(),
        breaks: Vec::new(),
    }
}

/// Moves `offset`, a byte offset in the unwrapped text, to the wrapped text.
#[must_use]
pub fn shift(offset: usize, breaks: &[(usize, String)]) -> usize {
    offset
        + breaks
            .iter()
            .filter(|(at, _)| *at < offset)
            .map(|(_, replacement)| replacement.len() - 1)
            .sum::<usize>()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LONG: &str = "kubectl rollout restart deployment/api --namespace production --timeout=120s --wait";

    #[test]
    fn wraps_at_spaces_with_a_hanging_indent_and_never_splits_a_token() {
        let wrapped = wrap(LONG, 40);
        let lines: Vec<&str> = wrapped.text.lines().collect();
        assert!(lines.len() > 1, "{lines:?}");
        for line in &lines[1..] {
            assert!(line.starts_with("    "), "continuations hang by four: {line:?}");
        }
        for line in &lines {
            let trimmed = line.trim_start();
            assert!(trimmed.chars().count() <= 40 || !trimmed.contains(' '), "{line:?} passes 40");
        }
        let tokens: Vec<&str> = LONG.split(' ').collect();
        let rebuilt: Vec<&str> = wrapped.text.split_whitespace().collect();
        assert_eq!(rebuilt, tokens, "every token intact, in order");
    }

    #[test]
    fn keeps_operators_with_the_word_before_them() {
        let wrapped = wrap("cd /a/very/long/path/to/a/project && make build-everything", 30);
        for line in wrapped.text.lines() {
            assert!(!line.trim_start().starts_with("&&"), "{:?}", wrapped.text);
        }
    }

    #[test]
    fn leaves_a_short_command_alone() {
        assert_eq!(wrap("git status", 40).text, "git status");
    }

    #[test]
    fn moves_offsets_past_inserted_breaks() {
        let breaks = vec![(5, "\n    ".to_owned())];
        assert_eq!(shift(3, &breaks), 3);
        assert_eq!(shift(6, &breaks), 6 + 4);
    }
}
```

Add `pub mod command_card;` to `crates/kb-app/src/ui.rs` (sorted, after `pub mod chip;`).

Run: `cargo nextest run -p kb-app command_card`
Expected: FAIL: the first two tests (no wrapping yet).

- [ ] **Step 3: Write `wrap`** (the spike's algorithm, `04-spike-screens.html`). Replace the `wrap` stub with:

```rust
pub fn wrap(formatted: &str, columns: usize) -> Wrapped {
    let mut breaks: Vec<(usize, String)> = Vec::new();
    let mut line_start = 0;
    for line in formatted.split('\n') {
        let indent = line.len() - line.trim_start().len();
        let hang = " ".repeat(indent + 4);
        // Words as (offset, byte length), an operator glued to the word before it.
        let mut words: Vec<(usize, usize)> = Vec::new();
        let mut offset = line_start;
        for (ix, part) in line.split(' ').enumerate() {
            if ix > 0 {
                offset += 1;
            }
            if !part.is_empty() {
                let glued = matches!(part, "&&" | "||" | "|" | ";" | "\\");
                match words.last_mut() {
                    Some(last) if glued => last.1 = offset + part.len() - last.0,
                    _ => words.push((offset, part.len())),
                }
            }
            offset += part.len();
        }
        let mut column = 0;
        for (ix, (start, len)) in words.iter().enumerate() {
            let width = formatted[*start..start + len].chars().count();
            if ix == 0 {
                column = (start - line_start) + width;
            } else if column + 1 + width > columns && column > indent {
                breaks.push((start - 1, format!("\n{hang}")));
                column = hang.len() + width;
            } else {
                column += 1 + width;
            }
        }
        line_start += line.len() + 1;
    }
    let mut text = String::with_capacity(formatted.len() + breaks.len() * 8);
    let mut last = 0;
    for (at, replacement) in &breaks {
        text.push_str(&formatted[last..*at]);
        text.push_str(replacement);
        last = at + 1;
    }
    text.push_str(&formatted[last..]);
    Wrapped { text, breaks }
}
```

Run: `cargo nextest run -p kb-app command_card`
Expected: PASS, 4 tests.

- [ ] **Step 4: Gate and commit.**

```bash
mise run gate
git add Cargo.toml Cargo.lock crates/kb-app/Cargo.toml crates/kb-app/src/ui.rs crates/kb-app/src/ui/command_card.rs
git commit -m "feat(ui): Wrap commands at spaces with a hanging indent" \
  -m "The command card's layout (D13), a pure function; fmt-rs added." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: The command card component

**Files:** modify `crates/kb-app/src/ui/command_card.rs`.

- [ ] **Step 1: Write the failing test.** Append to the `tests` module:

```rust
#[test]
fn breaks_a_chain_at_its_operators_before_wrapping() {
    let text = layout("make build && make test && make deploy-to-production-now", 30).text;
    assert!(text.lines().count() >= 2, "{text}");
    assert!(text.lines().all(|l| !l.trim_start().starts_with("&&")), "{text}");
}
```

Run: `cargo nextest run -p kb-app command_card`
Expected: FAIL to compile: no function `layout`.

- [ ] **Step 2: Write the component.** Append to `command_card.rs`, before the tests module:

```rust
use std::ops::Range;

use gpui_kit::base::input::{HighlightStyleResolver, Rope};
use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::highlighter::SyntaxHighlighter;
use gpui_kit::{
    App, ElementId, FontFeatures, FontWeight, HighlightStyle, Hsla, IntoElement, ParentElement as _, RenderOnce, Rems,
    SharedString, Styled as _, StyledText, Window, div,
};

/// fmt-rs's formatting of `command` at `columns`, then [`wrap`]: what the card draws.
#[must_use]
pub fn layout(command: &str, columns: usize) -> Wrapped {
    let formatted = fmt_rs::format::format_command(command, isize::try_from(columns).unwrap_or(isize::MAX));
    wrap(formatted.trim_end(), columns)
}

/// The colour of each bash highlight the card shows; plain words keep the text colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyntaxColors {
    /// Commands (`function`), drawn medium.
    pub function: Hsla,
    /// Flags and numbers (`constant`, `number`).
    pub constant: Hsla,
    /// Strings.
    pub string: Hsla,
    /// Variables and substitutions (`property`, `embedded`, `variable`).
    pub variable: Hsla,
    /// Keywords.
    pub keyword: Hsla,
    /// Operators, punctuation and comments.
    pub punctuation: Hsla,
}

impl HighlightStyleResolver for SyntaxColors {
    fn style(&self, name: &str) -> Option<HighlightStyle> {
        let (color, medium) = match name.split('.').next().unwrap_or(name) {
            "function" => (self.function, true),
            "constant" | "number" => (self.constant, false),
            "string" => (self.string, false),
            "property" | "embedded" | "variable" => (self.variable, false),
            "keyword" => (self.keyword, false),
            "operator" | "punctuation" | "comment" => (self.punctuation, false),
            _ => return None,
        };
        Some(HighlightStyle {
            color: Some(color),
            font_weight: medium.then_some(FontWeight::MEDIUM),
            ..HighlightStyle::default()
        })
    }
}

/// How a card looks: plain GPUI values, so the card knows nothing of kb.
#[derive(Debug, Clone)]
pub struct CardStyle {
    /// The box.
    pub background: Hsla,
    /// Its 1 pt border.
    pub border: Hsla,
    /// Plain words.
    pub text: Hsla,
    /// Highlight colours.
    pub syntax: SyntaxColors,
    /// The monospace family.
    pub family: SharedString,
    /// Ligatures off.
    pub features: FontFeatures,
    /// Text size and line height.
    pub size: Rems,
    /// Line height.
    pub line_height: Rems,
    /// Corner radius.
    pub radius: Rems,
    /// Padding: left, top and bottom, right (room for the copy button).
    pub pad_x: Rems,
    /// Top and bottom padding.
    pub pad_y: Rems,
    /// Right padding.
    pub pad_right: Rems,
    /// The copy button's inset from the top-right corner.
    pub copy_inset: Rems,
    /// The copy button's tooltip.
    pub copy_label: SharedString,
}

/// A command card.
#[derive(IntoElement)]
pub struct CommandCard {
    id: ElementId,
    command: SharedString,
    columns: usize,
    style: CardStyle,
    on_copied: Option<std::rc::Rc<dyn Fn(SharedString, &mut Window, &mut App)>>,
}

impl CommandCard {
    /// A card `id` showing `command` wrapped to `columns`.
    #[must_use]
    pub fn new(id: impl Into<ElementId>, command: impl Into<SharedString>, columns: usize, style: CardStyle) -> Self {
        Self {
            id: id.into(),
            command: command.into(),
            columns,
            style,
            on_copied: None,
        }
    }

    /// Runs `handler` after the copy button copies the command (the copy notice, D17).
    #[must_use]
    pub fn on_copied(mut self, handler: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_copied = Some(std::rc::Rc::new(handler));
        self
    }
}

impl RenderOnce for CommandCard {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let s = self.style;
        let wrapped = layout(&self.command, self.columns);
        let formatted_len = wrapped.text.len();
        let rope = Rope::from_str(&wrapped.text);
        let mut highlighter = SyntaxHighlighter::new("bash");
        highlighter.update(None, &rope, None);
        let styles: Vec<(Range<usize>, HighlightStyle)> = highlighter.styles(&(0..formatted_len), &s.syntax);
        let mut copy = Clipboard::new(ElementId::Name(format!("{:?}-copy", self.id).into()))
            .value(self.command.clone())
            .tooltip(s.copy_label.clone());
        if let Some(handler) = self.on_copied {
            copy = copy.on_copied(move |value, window, cx| handler(value, window, cx));
        }
        div()
            .id(self.id)
            .relative()
            .w_full()
            .pl(s.pad_x)
            .py(s.pad_y)
            .pr(s.pad_right)
            .rounded(s.radius)
            .bg(s.background)
            .border_1()
            .border_color(s.border)
            .font_family(s.family)
            .font_features(s.features)
            .text_size(s.size)
            .line_height(s.line_height)
            .text_color(s.text)
            .child(StyledText::new(wrapped.text).with_highlights(styles))
            .child(div().absolute().top(s.copy_inset).right(s.copy_inset).child(copy))
    }
}
```

(The highlighter runs on the wrapped text itself, so no offsets need shifting; `shift` stays for callers that colour
the unwrapped text. If `SyntaxHighlighter::styles` takes its resolver by another type, apply the index's deviation
rule.)

Run: `cargo nextest run -p kb-app command_card`
Expected: PASS, 5 tests.

- [ ] **Step 3: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/src/ui/command_card.rs
git commit -m "feat(ui): Add the command card" \
  -m "fmt-rs breaks, a hanging indent, GPUI Kit's bash colours, a copy button (D13)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: The entry view

**Files:** create `crates/kb-app/src/ui/entry_view.rs`; modify `crates/kb-app/src/ui.rs`.

- [ ] **Step 1: Write the failing tests.** Create `crates/kb-app/src/ui/entry_view.rs` with its pure parts first:

```rust
//! D11's entry template (04-entry-template, 05-detail-links): the same component in the split
//! (beside a 34 % list) and on the detail screen. Four slots — Header (the row's icon, the title on
//! one line, the tags on the right), Hero by type (D12), Quote (the description), Body (the notes,
//! then the Links section, D20) — and an empty slot collapses. No type badge, path or metadata.

use kb_domain::entry::Entry;

use crate::catalog::{EntryType, Hero};

/// Where the view sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Beside the list.
    Split,
    /// The detail screen.
    Detail,
}

/// What a hero shows, decided from the entry and its type (D12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeroKind {
    /// A command card of this command.
    Card(String),
    /// A YouTube thumbnail.
    Thumbnail(String),
    /// The URL as a link line (`↗ host/path`).
    LinkLine(String),
    /// A task's status and priority chips.
    TaskChips(Vec<String>),
    /// Nothing: the slot collapses.
    None,
}

/// The hero for `entry` of `kind`.
#[must_use]
pub fn hero_of(entry: &Entry, kind: Option<&EntryType>) -> HeroKind {
    let _ = (entry, kind);
    HeroKind::None
}

/// The text columns a command card of `width` points holds, for a monospace `advance` (the width of
/// one character, in points) — fmt-rs breaks to this, and so does [`crate::ui::command_card::wrap`].
#[must_use]
pub fn columns_for(width: f32, advance: f32) -> usize {
    let _ = (width, advance);
    80
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::fixture::entry;

    fn hero(entry_type: &str, key: &str, yaml: &str) -> HeroKind {
        let catalog = Catalog::default();
        hero_of(&entry(entry_type, key, yaml), catalog.entry_type(entry_type))
    }

    #[test]
    fn gives_each_type_its_hero() {
        assert_eq!(hero("commands", "ls -la", "desc: d\n"), HeroKind::Card("ls -la".into()));
        assert_eq!(
            hero("cheats", "k", "cheat: git rebase -i HEAD~3\n"),
            HeroKind::Card("git rebase -i HEAD~3".into())
        );
        assert_eq!(hero("cheats", "k", "notes: only notes\n"), HeroKind::None, "no snippet, no card");
        assert_eq!(
            hero("bookmarks", "https://www.youtube.com/watch?v=aqz-KE-bpKQ", "title: t\n"),
            HeroKind::Thumbnail("https://img.youtube.com/vi/aqz-KE-bpKQ/maxresdefault.jpg".into())
        );
        assert_eq!(
            hero("bookmarks", "https://tokio.rs/tokio/tutorial", "title: t\n"),
            HeroKind::LinkLine("https://tokio.rs/tokio/tutorial".into())
        );
        assert_eq!(
            hero("tasks", "t", "status: in-progress\npriority: high\n"),
            HeroKind::TaskChips(vec!["in-progress".into(), "high".into()])
        );
        assert_eq!(hero("tasks", "t", "title: bare\n"), HeroKind::None);
    }

    #[test]
    fn counts_the_columns_a_card_holds() {
        assert_eq!(columns_for(660., 7.8), 84);
        assert_eq!(columns_for(10., 7.8), 20, "never below 20: a card that narrow still breaks somewhere");
    }
}
```

Add `pub mod entry_view;` to `crates/kb-app/src/ui.rs` (sorted, after `pub mod design;`).

Run: `cargo nextest run -p kb-app entry_view`
Expected: FAIL: both tests.

- [ ] **Step 2: Write `hero_of` and `columns_for`.** Replace the two stubs:

```rust
pub fn hero_of(entry: &Entry, kind: Option<&EntryType>) -> HeroKind {
    match kind.map(|k| k.hero) {
        Some(Hero::Command) => HeroKind::Card(entry.primary_content.clone()),
        Some(Hero::Snippet) => snippet(entry).map_or(HeroKind::None, HeroKind::Card),
        Some(Hero::Media) => match kb_domain::markdown::youtube_video_id(&entry.primary_content) {
            Some(id) => HeroKind::Thumbnail(format!("https://img.youtube.com/vi/{id}/maxresdefault.jpg")),
            None => HeroKind::LinkLine(entry.primary_content.clone()),
        },
        Some(Hero::Task) => {
            let chips: Vec<String> = entry.task_status.iter().chain(&entry.task_priority).cloned().collect();
            if chips.is_empty() { HeroKind::None } else { HeroKind::TaskChips(chips) }
        }
        None => HeroKind::None,
    }
}

/// A cheat sheet's `cheat:` field, when it has one: its snippet.
fn snippet(entry: &Entry) -> Option<String> {
    let raw = kb_domain::yaml::load(&entry.raw_yaml).ok().flatten()?;
    kb_domain::entry::raw_string(&raw, "cheat")
}

pub fn columns_for(width: f32, advance: f32) -> usize {
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a positive count of columns")]
    let columns = (width / advance.max(1.)).floor() as usize;
    columns.max(20)
}
```

Run: `cargo nextest run -p kb-app entry_view`
Expected: PASS, 2 tests.

- [ ] **Step 3: Write the view.** Append to `entry_view.rs`, before the tests module:

```rust
use std::rc::Rc;

use gpui_kit::component::text::{TextView, TextViewStyle};
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled as _, StyledImage as _, Window, div, img, relative, svg,
};

use crate::catalog::Catalog;
use crate::presentation::{favicon_url, tag_hue, trim_url};
use crate::tag_images::TagImages;
use crate::ui::chip::{icon_chip, pill};
use crate::ui::command_card::{CardStyle, CommandCard, SyntaxColors};
use crate::ui::design::{TextRoleExt as _, design, no_ligatures, pixels, pt};
use crate::ui::keycap::{ChipState, caps, keycap};
use crate::ui::tile::{TileSize, favicon, tile};

/// What the view does when a tag, a link or the card's copy is used.
pub type OnTag = Rc<dyn Fn(&str, &mut Window, &mut App)>;
/// Opens a link.
pub type OnLink = Rc<dyn Fn(&str, &mut Window, &mut App)>;
/// After the card copied its command.
pub type OnCopied = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// The entry template.
#[derive(IntoElement)]
pub struct EntryView {
    entry: Entry,
    catalog: Rc<Catalog>,
    images: TagImages,
    layout: Layout,
    columns: usize,
    scroll: Option<ScrollHandle>,
    on_tag: OnTag,
    on_link: OnLink,
    on_copied: OnCopied,
}

impl EntryView {
    /// The view of `entry`, `columns` wide for its command card.
    #[must_use]
    #[expect(clippy::too_many_arguments, reason = "the view's inputs, each used once")]
    pub fn new(
        entry: Entry,
        catalog: Rc<Catalog>,
        images: TagImages,
        layout: Layout,
        columns: usize,
        on_tag: OnTag,
        on_link: OnLink,
        on_copied: OnCopied,
    ) -> Self {
        Self {
            entry,
            catalog,
            images,
            layout,
            columns,
            scroll: None,
            on_tag,
            on_link,
            on_copied,
        }
    }

    /// Scrolls with `handle` (the detail screen's keyboard scrolling).
    #[must_use]
    pub fn track_scroll(mut self, handle: ScrollHandle) -> Self {
        self.scroll = Some(handle);
        self
    }
}

impl RenderOnce for EntryView {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let d = design();
        let (c, p) = (&d.colors, &d.spacing.pane);
        let detail = self.layout == Layout::Detail;
        let kind = self.catalog.entry_type(&self.entry.entry_type).cloned();
        let tint = kind.as_ref().and_then(|k| c.named(&k.tint)).unwrap_or(c.faint);
        let icon = kind.as_ref().map_or("file-text", |k| k.icon.as_str()).to_owned();
        let id = |part: &str| SharedString::from(format!("entry-{part}-{}", self.entry.id));

        // 1 Header: the row's icon, the title on one line, the tags (clickable: they filter).
        let lead = match kind.as_ref().filter(|k| k.hero == Hero::Media).and_then(|_| favicon_url(&self.entry.primary_content)) {
            Some(url) => favicon(url, icon.clone(), tint, TileSize::Small).into_any_element(),
            None => tile(&icon, tint, TileSize::Small).into_any_element(),
        };
        let tags = self.entry.tags.iter().map(|tag| {
            let chip: AnyElement = match self.images.image(tag) {
                Some(image) => icon_chip(id(&format!("tagimg-{tag}")), image, c.hue(tag_hue(tag)), ChipState::Rest, format!("#{tag}")).into_any_element(),
                None => pill(format!("#{tag}"), ChipState::Rest).into_any_element(),
            };
            let (on_tag, tag) = (self.on_tag.clone(), tag.clone());
            div().id(id(&format!("tag-{tag}"))).cursor_pointer().child(chip).on_click(move |_, window, cx| on_tag(&tag, window, cx))
        });
        let header = h_flex()
            .items_center()
            .gap(pt(p.header_gap))
            .child(lead)
            .child(div().flex_1().min_w_0().truncate().text_role(&d.typography.roles.heading).text_color(c.text).child(self.entry.title.clone()))
            .child(h_flex().flex_none().gap(pt(d.spacing.row.tags_gap)).children(tags));

        // 2 Hero by type.
        let hero: Option<AnyElement> = match hero_of(&self.entry, kind.as_ref()) {
            HeroKind::Card(command) => {
                let role = if detail { &d.typography.roles.card_detail } else { &d.typography.roles.card_split };
                let style = CardStyle {
                    background: c.mantle,
                    border: c.selected,
                    text: c.text,
                    syntax: SyntaxColors { function: c.blue, constant: c.peach, string: c.yellow, variable: c.pink, keyword: c.mauve, punctuation: c.faint },
                    family: d.typography.mono.clone().into(),
                    features: no_ligatures(),
                    size: pt(role.size),
                    line_height: pt(role.line_height.unwrap_or(role.size + 8.)),
                    radius: pt(d.rounded.card),
                    pad_x: pt(d.spacing.card.pad_x),
                    pad_y: pt(d.spacing.card.pad_y),
                    pad_right: pt(d.spacing.card.pad_right),
                    copy_inset: pt(d.spacing.card.copy_inset),
                    copy_label: self.catalog.label("copy").to_owned().into(),
                };
                let on_copied = self.on_copied.clone();
                Some(CommandCard::new(id("card"), command, self.columns, style).on_copied(move |value, window, cx| on_copied(value, window, cx)).into_any_element())
            }
            HeroKind::Thumbnail(url) => {
                let height = if detail { d.components.hero.thumb_detail } else { d.components.hero.thumb_split };
                Some(div().w_full().h(pt(height)).flex_none().rounded(pt(d.rounded.hero)).overflow_hidden().child(img(url).size_full().object_fit(gpui_kit::ObjectFit::Cover)).into_any_element())
            }
            HeroKind::LinkLine(url) => {
                let shown = trim_url(&url, usize::MAX);
                let (host, path) = shown.split_once('/').map_or((shown.as_str(), String::new()), |(h, rest)| (h, format!("/{rest}")));
                let (host, on_link) = (host.to_owned(), self.on_link.clone());
                Some(h_flex().id(id("linkline")).cursor_pointer().items_center().gap(pt(d.spacing.link_hero.gap)).px(pt(d.spacing.link_hero.pad_x)).py(pt(d.spacing.link_hero.pad_y))
                    .rounded(pt(d.rounded.card)).bg(c.mantle).border_1().border_color(c.selected).text_role(&d.typography.roles.link_hero).text_color(c.blue)
                    .child(svg().path("icons/arrow-up-right.svg").size(pt(d.components.hero.link_icon)).text_color(c.blue))
                    .child(div().text_color(c.text).child(host))
                    .child(div().min_w_0().truncate().child(path))
                    .on_click(move |_, window, cx| on_link(&url, window, cx))
                    .into_any_element())
            }
            HeroKind::TaskChips(chips) => Some(h_flex().gap(pt(d.spacing.row.tags_gap)).children(chips.into_iter().map(|chip| pill(chip, ChipState::Rest))).into_any_element()),
            HeroKind::None => None,
        };

        // 3 Quote: the description, whole, with the red bar.
        let quote = self.entry.description.clone().map(|text| {
            div().border_l(pt(d.components.quote.bar)).border_color(c.red).pl(pt(d.spacing.quote.pad_left)).py(pt(d.spacing.quote.pad_y))
                .text_role(&d.typography.roles.quote).text_color(c.text).child(text)
        });

        // 4 Body: the notes, headings at the type tokens.
        let body_text = kb_domain::markdown::notes_body(&self.entry).to_owned();
        let roles = &d.typography.roles;
        let (h1, h2, h3) = (pixels(roles.notes_h1.size), pixels(roles.notes_h2.size), pixels(roles.notes_h3.size));
        let body = (!body_text.trim().is_empty()).then(|| {
            div().text_role(&roles.body).text_color(c.text).child(
                TextView::markdown(id("body"), body_text).style(TextViewStyle::default().heading_font_size(move |level, _| match level {
                    1 => h1,
                    2 => h2,
                    _ => h3,
                })),
            )
        });

        // Links (D20, A1): `LINKS · n`, one 28 pt row each; ⌘1–⌘9 on the detail screen only (R7).
        let links = (!self.entry.links.is_empty()).then(|| {
            let l = &d.spacing.links;
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a token of a few dozen characters")]
            let chars = d.components.links.url_chars as usize;
            v_flex()
                .gap(pt(l.gap))
                .child(h_flex().gap(pt(l.label_gap)).pt(pt(l.label_pad_top)).pb(pt(l.label_pad_bottom)).text_role(&roles.section).text_color(c.faint)
                    .child(self.catalog.messages.links_label.to_uppercase()).child(format!("· {}", self.entry.links.len())))
                .children(self.entry.links.iter().enumerate().map(|(ix, link)| {
                    let url_text = trim_url(&link.url, chars);
                    let title = link.title.clone().filter(|t| !t.is_empty()).unwrap_or_else(|| url_text.clone());
                    let full = format!("{} — {}", title, link.url);
                    let (on_link, url) = (self.on_link.clone(), link.url.clone());
                    let fav = favicon_url(&link.url).map_or_else(|| tile(&icon, tint, TileSize::Mini).into_any_element(), |u| favicon(u, icon.clone(), tint, TileSize::Mini).into_any_element());
                    h_flex().id(id(&format!("link-{ix}"))).h(pt(d.components.links.row)).items_center().gap(pt(l.row_gap)).px(pt(l.row_pad_x)).rounded(pt(d.rounded.row))
                        .cursor_pointer().hover(|s| s.bg(c.selected))
                        .tooltip(move |window, cx| gpui_kit::component::tooltip::Tooltip::new(full.clone()).build(window, cx))
                        .on_click(move |_, window, cx| on_link(&url, window, cx))
                        .child(fav)
                        .child(div().max_w(relative(d.components.links.title_share)).truncate().text_role(&roles.link_title).text_color(c.text).child(title))
                        .child(div().flex_1().min_w_0().truncate().text_role(&roles.link_url).text_color(c.faint).child(url_text))
                        .when(detail && ix < 9, |row| row.child(keycap(caps(&format!("cmd-{}", ix + 1)), ChipState::Live)))
                }))
        });

        let (pad_x, pad_y) = if detail { (p.detail_pad_x, p.detail_pad_y) } else { (p.split_pad_x, p.split_pad_y) };
        v_flex()
            .id(id("scroll"))
            .size_full()
            .min_w_0()
            .overflow_y_scroll()
            .when_some(self.scroll, |view, handle| view.track_scroll(&handle))
            .px(pt(pad_x))
            .py(pt(pad_y))
            .gap(pt(p.gap))
            .child(header)
            .children(hero)
            .children(quote)
            .children(body)
            .children(links)
    }
}
```

Run: `cargo nextest run -p kb-app entry_view`
Expected: PASS (the component compiles; the two tests still pass). Formatting is rustfmt's job: run
`cargo fmt -p kb-app`.

- [ ] **Step 4: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/src/ui.rs crates/kb-app/src/ui/entry_view.rs
git commit -m "feat(ui): Add the entry view with heroes and the Links section" \
  -m "Header, hero by type, quote, body and links, as the approved pages (D11, D12, D20)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: The entry view in the panel

**Files:** modify `crates/kb-app/src/search_view.rs`, `crates/kb-app/src/lib.rs`, `crates/kb-app/tests/ui.rs`; delete
`crates/kb-app/src/notes.rs`, `crates/kb-app/src/metadata.rs`.

- [ ] **Step 1: Write the failing UI tests.** In `crates/kb-app/tests/ui.rs`, delete
      `shows_and_hides_the_metadata_with_right_and_left_on_the_detail_screen` and append:

```rust
#[gpui_kit::test]
fn ignores_right_and_left_on_the_detail_screen(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    press(cx, window, "enter");
    press(cx, window, "right");
    press(cx, window, "left");
    assert!(cx.read(|cx| view.read(cx).is_showing_detail()), "still on the detail screen");
}

#[gpui_kit::test]
fn opens_a_link_with_its_number_on_the_detail_screen(cx: &mut TestAppContext) {
    let (_home, window, _view) = open_with_query(cx, "example");
    press(cx, window, "enter");
    press(cx, window, "cmd-2");
    assert_eq!(cx.opened_url().as_deref(), Some("https://example.com/guide"));
}

#[gpui_kit::test]
fn filters_the_list_by_a_tag_from_the_detail_screen(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    press(cx, window, "enter");
    click(cx, window, "entry-tag-kube-commands.yml#commands#kubectl get pods -A");
    settle(cx);
    assert!(!cx.read(|cx| view.read(cx).is_showing_detail()), "back on the list");
    let filter = cx.read(|cx| view.read(cx).list().read(cx).delegate().filter().clone());
    assert_eq!(filter.tags, ["kube"]);
}
```

(If `click` panics with "missing ElementId", the test harness only sees observed elements: in Task 3's tag and link
rows write `div().test_support().id(…)` instead of `div().id(…)`, importing `gpui_kit::TestSupportExt as _`.)

(The sample's "Example docs" bookmark has the links `https://example.com/changelog`, `Guide`, `API`, so ⌘2 is the
guide. A tag's element id is `entry-tag-<tag>-<entry id>`, as Task 3 builds it.)

Run: `cargo nextest run -p kb-app --test ui`
Expected: FAIL: ⌘2 does nothing on the detail screen; the tag id is missing.

- [ ] **Step 2: Draw the entry view.** In `search_view.rs`:
  - replace `use crate::notes::{PaneLayout, next_offset, notes_pane};` with
    `use crate::ui::entry_view::{EntryView, Layout, columns_for};`;
  - add this method to `impl SearchView`:

```rust
    /// The entry view of `entry` in `layout`, its command card as wide as the pane allows.
    fn entry_view(&self, entry: &Entry, layout: Layout, window: &Window, cx: &mut Context<Self>) -> EntryView {
        let d = design();
        let viewport = f32::from(window.viewport_size().width);
        let (share, pad, role) = match layout {
            Layout::Split => (1. - d.components.split.list, d.spacing.pane.split_pad_x, &d.typography.roles.card_split),
            Layout::Detail => (1., d.spacing.pane.detail_pad_x, &d.typography.roles.card_detail),
        };
        let width = viewport * share - 2. * pad - d.spacing.card.pad_x - d.spacing.card.pad_right - 2. * d.components.border.hairline;
        let font = gpui_kit::font(d.typography.mono.clone());
        let advance = window
            .text_system()
            .em_advance(window.text_system().resolve_font(&font), pixels(role.size))
            .map_or(role.size * 0.6, f32::from);
        let view = cx.entity().downgrade();
        let (on_tag_view, on_copy_view) = (view.clone(), view.clone());
        let images = self.list.read(cx).delegate().tag_images().clone();
        EntryView::new(
            entry.clone(),
            self.catalog.clone(),
            images,
            layout,
            columns_for(width, advance),
            Rc::new(move |tag, window, cx| {
                let tag = tag.to_owned();
                let _ = on_tag_view.update(cx, |this, cx| this.filter_by_tag(&tag, window, cx));
            }),
            Rc::new(|url, _, cx| cx.open_url(url)),
            Rc::new(move |_, window, cx| {
                let _ = on_copy_view.update(cx, |this, cx| this.notice_copied(window, cx));
            }),
        )
    }

    /// A header tag clicked: the list, filtered by that tag alone (D20).
    fn filter_by_tag(&mut self, tag: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_filter(FilterState { types: Vec::new(), tags: vec![tag.to_owned()] }, cx);
        self.screen = Screen::Search;
        self.query_input.update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }

    /// The copy notice, after the command card's own button copied (D17).
    fn notice_copied(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.last_notice = Some(self.catalog.messages.copied.clone());
        window.push_notification(notice::copied(&self.catalog.messages), cx);
    }
```

- in `render_content`, give it a `window: &Window` parameter (update its one call in `render` to pass `window`),
  and replace the `Screen::Detail` arm and the split's `notes_pane(…)` child with:

```rust
Screen::Detail => selected.map_or_else(
    || div().into_any_element(),
    |entry| {
        self.entry_view(entry, Layout::Detail, window, cx)
            .track_scroll(self.detail_scroll.clone())
            .into_any_element()
    },
),
```

```rust
.child(div().h_full().flex_1().min_w_0().child(self.entry_view(entry, Layout::Split, window, cx)))
```

    and the split list's width `.w(relative(1. / 3.))` with `.w(relative(tokens.components.split.list))`, its
    `.border_r_1().border_color(cx.theme().border)` with `.border_r(pt(tokens.components.border.hairline)).border_color(tokens.colors.edge)`.

- [ ] **Step 3: No metadata; → and ← do nothing on detail; ⌘n opens links.** In `search_view.rs`:
  - delete the field `show_metadata`, its initializer, `is_showing_metadata()`, and the `self.show_metadata = false;`
    line in `show_detail_screen`;
  - in `show_details` and `hide_details`, change the `Screen::Detail => self.show_metadata = …,` arms to
    `Screen::Detail => {}` (D20: → and ← do nothing there);
  - replace `run_row`'s first `if` with:

```rust
if matches!(self.screen, Screen::Detail) {
    let url = self
        .list
        .read(cx)
        .delegate()
        .selected_entry()
        .and_then(|entry| entry.links.get(n - 1))
        .map(|link| link.url.clone());
    if let Some(url) = url {
        cx.open_url(&url);
    }
    return;
}
if !matches!(self.screen, Screen::Search) {
    return;
}
```

- move `next_offset` and its test from `notes.rs` into `search_view.rs` (a private `fn next_offset` and a
  `#[cfg(test)] mod tests` with `scrolls_within_the_notes_and_stops_at_their_ends`), since `scroll_notes` uses it.

- [ ] **Step 4: Delete the old pane.**

```bash
git rm crates/kb-app/src/notes.rs crates/kb-app/src/metadata.rs
```

In `lib.rs`, delete `mod metadata;` and `mod notes;`. In `presentation.rs`, delete `metadata`, `notes_markdown`,
`split_description`, `NotesParts`, `NO_NOTES` and their tests if nothing else uses them
(`grep -rn "split_description\|notes_markdown(\|fn metadata\|NO_NOTES" crates/kb-app/src` must print nothing
outside `presentation.rs`).

- [ ] **Step 5: Run every kb-app test.**

Run: `cargo nextest run -p kb-app`
Expected: PASS, including the three new UI tests and `scrolls_the_detail_screen_with_the_arrow_keys`.

- [ ] **Step 6: Look at it.** Run the app over the showcase (8a Task 7 Step 6's scratch home), open
      `/k8s-rollout-restart` (→ for the split, ↵ for detail), the Tokio bookmark and the Kubernetes cheat sheet, and
      compare each with `04-entry-template.html`, `04-command-cards.html` and `05-detail-links.html`. Note any
      difference in the commit body; 8h's checklist records them.

- [ ] **Step 7: Gate and commit.**

```bash
mise run gate
git add -A crates/kb-app
git commit -m "feat(ui): Show entries in the new template, split and detail" \
  -m "No metadata column; ⌘1–⌘9 open links on detail; header tags filter (D11, D20)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: The notes body — owner checkpoint

**Files:** none committed unless the owner chooses zorite-markdown.

- [ ] **Step 1: Capture the body as it is.** With the app on the showcase, open the Kubernetes cheat sheet on the
      detail screen (headings, a list, a YAML code block) and capture the window:
      `screencapture -x -o -l $(tmp/bin/probe window-id kb 2>/dev/null || osascript -e 'tell app "System Events" to id of window 1 of process "kb"') tmp/design/body-textview.png`
      (if neither returns an id, capture the screen with `screencapture -x tmp/design/body-textview.png`).

- [ ] **Step 2: Render the same notes with zorite-markdown, in a throwaway.** In the scratchpad (never in the
      repository), create a binary crate with `zorite-markdown = "=0.10.0"` and kb's `gpui-kit` dependency line and
      patches (copy them from the workspace `Cargo.toml`), build it from the kb directory
      (`CARGO_TARGET_DIR=$PWD/target/spike cargo build --manifest-path <scratch>/Cargo.toml`), and draw the same body
      text at the same width with its `MarkdownView`. If it does not build against `gpui-pre 0.3.8`, stop here: that
      is the answer (TextView stays), recorded in the TODO comment below.

- [ ] **Step 3: Stop and ask the owner.** Show both images side by side and ask: "Notes body: keep GPUI Kit's
      TextView, or adopt zorite-markdown?" Record the answer as a comment on TODO task `rPyR`
      (`todo add --comment "2026-10-13: notes body — <answer> (owner)" --task-id rPyR`). If zorite-markdown is
      chosen, its adoption is a new task, not part of this plan.

---

## Done when

- `cargo nextest run -p kb-app` and `mise run gate` pass.
- The split and detail screens match `04-entry-template.html` and `05-detail-links.html` to the eye, with differences
  noted.
- The owner has chosen the notes body (Task 5).
