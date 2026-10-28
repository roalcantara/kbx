//! D11's entry template (04-entry-template, 05-detail-links): the same component in the split
//! (beside a 34 % list) and on the detail screen. Four slots — Header (the row's icon, the title on
//! one line, the tags on the right), Hero by type (D12), Quote (the description), Body (the notes,
//! then the Links section, D20) — and an empty slot collapses. No type badge, path or metadata.

use std::rc::Rc;

use gpui_kit::component::text::{TextView, TextViewStyle};
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled as _, StyledImage as _, TestSupportExt as _, Window, div, img, relative,
    svg,
};
use kb_domain::entry::Entry;

use crate::catalog::{Catalog, EntryType, Hero};
use crate::presentation::{favicon_url, row_tags, tag_hue, trim_url};
use crate::tag_images::TagImages;
use crate::ui::chip::{icon_chip, pill};
use crate::ui::command_card::{CardStyle, CommandCard, SyntaxColors};
use crate::ui::design::{TextRoleExt as _, design, no_ligatures, pixels, pt};
use crate::ui::keycap::{ChipState, caps, keycap};
use crate::ui::tile::{TileSize, favicon, tile};

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
    /// A `YouTube` thumbnail (`url`, `fallback`).
    Thumbnail(String, String),
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
    match kind.and_then(|k| k.hero) {
        Some(Hero::Command) => HeroKind::Card(entry.primary_content.clone()),
        Some(Hero::Snippet) => snippet(entry).map_or(HeroKind::None, HeroKind::Card),
        Some(Hero::Media) => match kb_domain::markdown::youtube_video_id(&entry.primary_content) {
            Some(id) => HeroKind::Thumbnail(
                format!("https://img.youtube.com/vi/{id}/maxresdefault.jpg"),
                format!("https://img.youtube.com/vi/{id}/hqdefault.jpg"),
            ),
            None => HeroKind::LinkLine(entry.primary_content.clone()),
        },
        Some(Hero::Task) => {
            let chips: Vec<String> = entry.task_status.iter().chain(&entry.task_priority).cloned().collect();
            if chips.is_empty() {
                HeroKind::None
            } else {
                HeroKind::TaskChips(chips)
            }
        }
        None => HeroKind::None,
    }
}

/// A cheat sheet's `cheat:` field, when it has one: its snippet.
fn snippet(entry: &Entry) -> Option<String> {
    let raw = kb_domain::yaml::load(&entry.raw_yaml).ok().flatten()?;
    kb_domain::entry::raw_string(&raw, "cheat")
}

/// The text columns a command card of `width` points holds, for a monospace `advance` (the width of
/// one character, in points) — fmt-rs breaks to this, and so does [`crate::ui::command_card::wrap`].
#[must_use]
pub fn columns_for(width: f32, advance: f32) -> usize {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a positive count of columns"
    )]
    let columns = (width / advance.max(1.)).floor() as usize;
    columns.max(20)
}

/// What the view does when a tag, a link or the card's copy is used.
pub type OnTag = Rc<dyn Fn(&str, &mut Window, &mut App)>;
/// Opens a link.
pub type OnLink = Rc<dyn Fn(&str, &mut Window, &mut App)>;
/// After the card copied its command.
pub use crate::ui::command_card::OnCopied;

/// The entry template.
#[derive(IntoElement)]
pub struct EntryView {
    entry: Entry,
    catalog: Rc<Catalog>,
    images: TagImages,
    layout: Layout,
    columns: usize,
    scroll: Option<ScrollHandle>,
    focused: bool,
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
            focused: false,
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

    /// Whether the pane currently has the keys (close-out D3).
    #[must_use]
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl RenderOnce for EntryView {
    #[expect(clippy::too_many_lines, reason = "the template's slots and Links, in reading order")]
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let d = design();
        let (c, p) = (&d.colors, &d.spacing.pane);
        let detail = self.layout == Layout::Detail;
        let kind = self.catalog.entry_type(&self.entry.entry_type).cloned();
        let tint = kind.as_ref().and_then(|k| c.named(&k.tint)).unwrap_or(c.faint);
        let icon = kind.as_ref().map_or("file-text", |k| k.icon.as_str()).to_owned();
        let id = |part: &str| SharedString::from(format!("entry-{part}-{}", self.entry.id));

        let lead = match kind
            .as_ref()
            .filter(|k| k.hero == Some(Hero::Media))
            .and_then(|_| favicon_url(&self.entry.primary_content))
        {
            Some(url) => favicon(url, icon.clone(), tint, TileSize::Small).into_any_element(),
            None => tile(&icon, tint, TileSize::Small).into_any_element(),
        };
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a token of a few tags"
        )]
        let max = d.components.row.max_tags as usize;
        let (tags, more) = row_tags(&self.entry.tags, max);
        let tags = tags.iter().map(|tag| {
            let chip: AnyElement = match self.images.image(tag) {
                Some(image) => icon_chip(
                    id(&format!("tagimg-{tag}")),
                    image,
                    c.hue(tag_hue(tag)),
                    ChipState::Rest,
                    format!("#{tag}"),
                )
                .into_any_element(),
                None => pill(format!("#{tag}"), ChipState::Rest).into_any_element(),
            };
            let (on_tag, tag) = (self.on_tag.clone(), tag.clone());
            div()
                .id(id(&format!("tag-{tag}")))
                .test_support()
                .cursor_pointer()
                .child(chip)
                .on_click(move |_, window, cx| on_tag(&tag, window, cx))
        });
        let header = h_flex()
            .items_center()
            .gap(pt(p.header_gap))
            .child(lead)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_role(&d.typography.roles.heading)
                    .text_color(c.text)
                    .child(self.entry.title.clone()),
            )
            .child(
                h_flex()
                    .flex_shrink(1.)
                    .min_w_0()
                    .overflow_hidden()
                    .gap(pt(d.spacing.row.tags_gap))
                    .children(tags)
                    .when(more > 0, |group| group.child(pill(format!("+{more}"), ChipState::Rest))),
            );

        let hero: Option<AnyElement> = match hero_of(&self.entry, kind.as_ref()) {
            HeroKind::Card(command) => {
                let role = if detail {
                    &d.typography.roles.card_detail
                } else {
                    &d.typography.roles.card_split
                };
                let style = CardStyle {
                    background: c.mantle,
                    border: c.selected,
                    text: c.text,
                    syntax: SyntaxColors {
                        function: c.blue,
                        constant: c.peach,
                        string: c.yellow,
                        variable: c.pink,
                        keyword: c.mauve,
                        punctuation: c.faint,
                    },
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
                Some(
                    CommandCard::new(id("card"), command, self.columns, style)
                        .on_copied(move |value, window, cx| on_copied(value, window, cx))
                        .into_any_element(),
                )
            }
            HeroKind::Thumbnail(url, fallback) => {
                let height = if detail {
                    d.components.hero.thumb_detail
                } else {
                    d.components.hero.thumb_split
                };
                Some(
                    div()
                        .w_full()
                        .h(pt(height))
                        .flex_none()
                        .rounded(pt(d.rounded.hero))
                        .overflow_hidden()
                        .child(
                            img(url)
                                .size_full()
                                .object_fit(gpui_kit::ObjectFit::Cover)
                                .with_fallback(move || {
                                    img(fallback.clone())
                                        .size_full()
                                        .object_fit(gpui_kit::ObjectFit::Cover)
                                        .into_any_element()
                                }),
                        )
                        .into_any_element(),
                )
            }
            HeroKind::LinkLine(url) => {
                let shown = trim_url(&url, usize::MAX);
                let (host, path) = shown
                    .split_once('/')
                    .map_or((shown.as_str(), String::new()), |(h, rest)| (h, format!("/{rest}")));
                let (host, on_link) = (host.to_owned(), self.on_link.clone());
                Some(
                    h_flex()
                        .id(id("linkline"))
                        .cursor_pointer()
                        .items_center()
                        .gap(pt(d.spacing.link_hero.gap))
                        .px(pt(d.spacing.link_hero.pad_x))
                        .py(pt(d.spacing.link_hero.pad_y))
                        .rounded(pt(d.rounded.card))
                        .bg(c.mantle)
                        .border_1()
                        .border_color(c.selected)
                        .text_role(&d.typography.roles.link_hero)
                        .text_color(c.blue)
                        .child(
                            svg()
                                .path("icons/arrow-up-right.svg")
                                .size(pt(d.components.hero.link_icon))
                                .text_color(c.blue),
                        )
                        .child(div().text_color(c.text).child(host))
                        .child(div().min_w_0().truncate().child(path))
                        .on_click(move |_, window, cx| on_link(&url, window, cx))
                        .into_any_element(),
                )
            }
            HeroKind::TaskChips(chips) => Some(
                h_flex()
                    .gap(pt(d.spacing.row.tags_gap))
                    .children(chips.into_iter().map(|chip| pill(chip, ChipState::Rest)))
                    .into_any_element(),
            ),
            HeroKind::None => None,
        };

        let quote = self.entry.description.clone().map(|text| {
            div()
                .border_l(pt(d.components.quote.bar))
                .border_color(c.red)
                .pl(pt(d.spacing.quote.pad_left))
                .py(pt(d.spacing.quote.pad_y))
                .text_role(&d.typography.roles.quote)
                .text_color(c.text)
                .child(text)
        });

        let body_text = kb_domain::markdown::notes_body(&self.entry).to_owned();
        let roles = &d.typography.roles;
        let (h1, h2, h3) = (
            pixels(roles.notes_h1.size),
            pixels(roles.notes_h2.size),
            pixels(roles.notes_h3.size),
        );
        let body = (!body_text.trim().is_empty()).then(|| {
            div()
                .text_role(&roles.body)
                .text_color(c.text)
                .child(
                    TextView::markdown(id("body"), body_text).style(TextViewStyle::default().heading_font_size(
                        move |level, _| match level {
                            1 => h1,
                            2 => h2,
                            _ => h3,
                        },
                    )),
                )
        });

        let links = (!self.entry.links.is_empty()).then(|| {
            let l = &d.spacing.links;
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a token of a few dozen characters"
            )]
            let chars = d.components.links.url_chars as usize;
            v_flex()
                .gap(pt(l.gap))
                .child(
                    h_flex()
                        .gap(pt(l.label_gap))
                        .pt(pt(l.label_pad_top))
                        .pb(pt(l.label_pad_bottom))
                        .text_role(&roles.section)
                        .text_color(c.faint)
                        .child(self.catalog.messages.links_label.to_uppercase())
                        .child(format!("· {}", self.entry.links.len())),
                )
                .children(self.entry.links.iter().enumerate().map(|(ix, link)| {
                    let url_text = trim_url(&link.url, chars);
                    let title = link
                        .title
                        .clone()
                        .filter(|t| !t.is_empty())
                        .unwrap_or_else(|| url_text.clone());
                    let full = format!("{} — {}", title, link.url);
                    let (on_link, url) = (self.on_link.clone(), link.url.clone());
                    let fav = favicon_url(&link.url).map_or_else(
                        || tile(&icon, tint, TileSize::Mini).into_any_element(),
                        |u| favicon(u, icon.clone(), tint, TileSize::Mini).into_any_element(),
                    );
                    h_flex()
                        .id(id(&format!("link-{ix}")))
                        .test_support()
                        .h(pt(d.components.links.row))
                        .items_center()
                        .gap(pt(l.row_gap))
                        .px(pt(l.row_pad_x))
                        .rounded(pt(d.rounded.row))
                        .cursor_pointer()
                        .hover(|s| s.bg(c.selected))
                        .tooltip(move |window, cx| {
                            gpui_kit::component::tooltip::Tooltip::new(full.clone()).build(window, cx)
                        })
                        .on_click(move |_, window, cx| on_link(&url, window, cx))
                        .child(fav)
                        .child(
                            div()
                                .max_w(relative(d.components.links.title_share))
                                .truncate()
                                .text_role(&roles.link_title)
                                .text_color(c.text)
                                .child(title),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .text_role(&roles.link_url)
                                .text_color(c.faint)
                                .child(url_text),
                        )
                        .when(detail && ix < 9, |row| {
                            row.child(keycap(caps(&format!("cmd-{}", ix + 1)), ChipState::Live))
                        })
                }))
        });

        // The window's content edge in both layouts (spec 2026-10-19 spacing D2).
        let (pad_x, pad_y) = (d.content_inset(), if detail { p.detail_pad_y } else { p.split_pad_y });
        crate::ui::design::focus_bar(
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
                .children(links),
            self.focused,
        )
    }
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
        assert_eq!(
            hero("cheats", "k", "notes: only notes\n"),
            HeroKind::None,
            "no snippet, no card"
        );
        assert_eq!(
            hero("bookmarks", "https://www.youtube.com/watch?v=aqz-KE-bpKQ", "title: t\n"),
            HeroKind::Thumbnail(
                "https://img.youtube.com/vi/aqz-KE-bpKQ/maxresdefault.jpg".into(),
                "https://img.youtube.com/vi/aqz-KE-bpKQ/hqdefault.jpg".into(),
            )
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
        assert_eq!(
            columns_for(10., 7.8),
            20,
            "never below 20: a card that narrow still breaks somewhere"
        );
    }

    #[test]
    fn caps_eight_tags_to_three() {
        let entry = entry("cheats", "k", "tags: [a, b, c, d, e, f, g, h]\n");
        let (tags, more) = row_tags(&entry.tags, 3);
        assert_eq!(tags, &["a", "b", "c"]);
        assert_eq!(more, 5);
    }
}
