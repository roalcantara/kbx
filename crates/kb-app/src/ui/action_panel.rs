//! D9's action panel (02-actions, 07-filter): sections of 28 pt rows separated by a hairline, a
//! `‹` header on submenus, the search at the bottom. Pure: the page, what the query matches and
//! the selection in, an element out; `search_view/panel.rs` owns the state.

use std::rc::Rc;

use crate::tag_images::TagImage;
use crate::ui::chip::icon_chip;
use crate::ui::crumb::{CrumbSize, crumb};
use crate::ui::design::{TextRoleExt as _, design, pt};
use crate::ui::keycap::{ChipState, keycap};
use crate::ui::tile::{TileSize, tile};
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, Div, Hsla, InteractiveElement as _, IntoElement as _, ParentElement as _, ScrollHandle,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, svg,
};

/// What leads a row (the filter's rows; the action rows have none).
#[derive(Debug, Clone, PartialEq)]
pub enum RowLead {
    /// A type's mini tile.
    Tile {
        /// The Lucide icon.
        icon: String,
        /// Its accent.
        tint: Hsla,
    },
    /// A tag's image, tinted in its hue when it is a shape.
    Image(TagImage, Hsla),
    /// A tag without an image: `#`.
    Hash,
    /// The Clear All row's ✕.
    Clear,
    /// A theme's accents, as dots.
    Swatch(Vec<Hsla>),
}

/// One row.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PanelRow {
    /// Its label.
    pub label: String,
    /// Its keycap text, if its action has keys.
    pub caps: Option<String>,
    /// Opens a submenu (`›`).
    pub submenu: bool,
    /// The submenu it comes from, when the root's search found it there.
    pub origin: Option<String>,
    /// The filter's lead.
    pub lead: Option<RowLead>,
    /// A chosen filter row (✓).
    pub checked: bool,
    /// Drawn in muted text (Clear All).
    pub muted: bool,
}

/// A section: an optional label (drawn upper-case) and its rows.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PanelSection {
    /// The label.
    pub label: Option<String>,
    /// The rows.
    pub rows: Vec<PanelRow>,
}

/// A page: the root, a submenu (with its `‹` header) or the filter.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PanelPage {
    /// The submenu's header.
    pub crumb: Option<String>,
    /// The sections, separated by a hairline.
    pub sections: Vec<PanelSection>,
    /// Whether it has a search field (submenus have none).
    pub searchable: bool,
}

/// The rows `query` matches, as (section, row), in order: a case-insensitive substring of the
/// label, or of the origin; every row for an empty query.
#[must_use]
pub fn matches(page: &PanelPage, query: &str) -> Vec<(usize, usize)> {
    let query = query.trim().to_lowercase();
    page.sections
        .iter()
        .enumerate()
        .flat_map(|(s, section)| section.rows.iter().enumerate().map(move |(r, row)| (s, r, row)))
        .filter(|(_, _, row)| {
            query.is_empty()
                || row.label.to_lowercase().contains(&query)
                || row.origin.as_deref().is_some_and(|o| o.to_lowercase().contains(&query))
        })
        .map(|(s, r, _)| (s, r))
        .collect()
}

/// What runs when a row is clicked: its index among the visible rows.
pub type OnRow = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// One child of the panel's scroll container, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PanelChild {
    /// The rule between two sections.
    Hairline,
    /// A section's label.
    Label(usize),
    /// Visible row `ix`, of section `section`, row `row`.
    Row { ix: usize, section: usize, row: usize },
}

/// The scroll container's children for `visible`: the one walk `panel()` draws and `child_index` counts, so the two
/// cannot drift (close-out D1).
fn children(page: &PanelPage, visible: &[(usize, usize)]) -> Vec<PanelChild> {
    let mut out = Vec::new();
    let mut shown_section = None;
    for (ix, &(section, row)) in visible.iter().enumerate() {
        if shown_section != Some(section) {
            if shown_section.is_some() {
                out.push(PanelChild::Hairline);
            }
            if page.sections[section].label.is_some() {
                out.push(PanelChild::Label(section));
            }
            shown_section = Some(section);
        }
        out.push(PanelChild::Row { ix, section, row });
    }
    out
}

/// The scroll container's child index of visible row `ix`, for `ScrollHandle::scroll_to_item`.
#[must_use]
pub fn child_index(page: &PanelPage, visible: &[(usize, usize)], ix: usize) -> usize {
    children(page, visible)
        .iter()
        .position(|child| matches!(child, PanelChild::Row { ix: i, .. } if *i == ix))
        .unwrap_or(0)
}

/// The panel: `page`'s `visible` rows (from [`matches`]) with `selected` highlighted, `search` (the
/// search field) at the bottom when the page is searchable, `width` points wide.
#[must_use]
pub fn panel(
    page: &PanelPage,
    visible: &[(usize, usize)],
    selected: usize,
    scroll: &ScrollHandle,
    search: Option<AnyElement>,
    width: f32,
    on_row: &OnRow,
) -> Div {
    let d = design();
    let (c, s, k) = (&d.colors, &d.spacing.panel, &d.components.panel);
    let hairline = || div().h(pt(d.components.border.hairline)).bg(c.edge).flex_none();
    let mut list = v_flex()
        .id("panel-rows")
        .track_scroll(scroll)
        .p(pt(s.pad))
        .max_h(pt(k.max_height))
        .overflow_y_scroll();
    for child in children(page, visible) {
        list = match child {
            PanelChild::Hairline => list.child(hairline().mx(pt(s.rule_margin)).my(pt(s.rule_margin))),
            PanelChild::Label(section) => list.child(
                div()
                    .pt(pt(s.section_pad_top))
                    .px(pt(s.section_pad_x))
                    .pb(pt(s.section_pad_bottom))
                    .text_role(&d.typography.roles.section)
                    .text_color(c.faint)
                    .child(page.sections[section].label.clone().unwrap_or_default().to_uppercase()),
            ),
            PanelChild::Row {
                ix,
                section,
                row: row_ix,
            } => list.child(row(&page.sections[section].rows[row_ix], ix, ix == selected, on_row)),
        };
    }
    v_flex()
        .w(pt(width))
        .overflow_hidden()
        .when_some(page.crumb.clone(), |panel, name| {
            panel.child(
                div()
                    .h(pt(k.crumb))
                    .flex()
                    .items_center()
                    .px(pt(s.crumb_pad_x))
                    .border_b(pt(d.components.border.hairline))
                    .border_color(c.edge)
                    .child(crumb(name, false, CrumbSize::Panel)),
            )
        })
        .child(list)
        .when_some(search.filter(|_| page.searchable), |panel, search| {
            panel.child(
                h_flex()
                    .h(pt(k.search))
                    .flex_none()
                    .items_center()
                    .gap(pt(s.search_gap))
                    .px(pt(s.search_pad_x))
                    .border_t(pt(d.components.border.hairline))
                    .border_color(c.edge)
                    .text_role(&d.typography.roles.description)
                    .text_color(c.text)
                    .child(
                        svg()
                            .path("icons/search.svg")
                            .size(pt(k.search_icon))
                            .text_color(c.faint),
                    )
                    .child(div().flex_1().min_w_0().child(search)),
            )
        })
}

fn row(row: &PanelRow, ix: usize, selected: bool, on_row: &OnRow) -> impl gpui_kit::IntoElement {
    let d = design();
    let (c, s, k) = (&d.colors, &d.spacing.panel, &d.components.panel);
    let on_row = on_row.clone();
    let lead = row.lead.clone().map(|lead| match lead {
        RowLead::Tile { icon, tint } => tile(&icon, tint, TileSize::Mini).into_any_element(),
        RowLead::Image(image, hue) => icon_chip(
            SharedString::from(format!("filter-tag-{ix}")),
            &image,
            hue,
            ChipState::Rest,
            String::new(),
        )
        .size(pt(d.components.tile.mini))
        .into_any_element(),
        RowLead::Hash => div()
            .w(pt(d.components.tile.mini))
            .flex()
            .justify_center()
            .text_color(c.faint)
            .child("#")
            .into_any_element(),
        RowLead::Clear => div()
            .w(pt(d.components.tile.mini))
            .flex()
            .justify_center()
            .child(
                svg()
                    .path("icons/x.svg")
                    .size(pt(d.components.tile.mini_icon))
                    .text_color(c.faint),
            )
            .into_any_element(),
        RowLead::Swatch(colors) => h_flex()
            .flex_none()
            .gap(pt(s.swatch_gap))
            .children(
                colors
                    .into_iter()
                    .map(|color| div().size(pt(k.swatch)).rounded_full().bg(color)),
            )
            .into_any_element(),
    });
    let caps = row
        .caps
        .clone()
        .or_else(|| selected.then(|| crate::ui::keycap::caps("enter")));
    h_flex()
        .id(SharedString::from(format!("panel-row-{ix}")))
        .h(pt(k.row))
        .flex_none()
        .items_center()
        .gap(pt(s.row_gap))
        .px(pt(s.row_pad_x))
        .rounded(pt(d.rounded.row))
        .text_role(&d.typography.roles.label)
        .text_color(if row.muted { c.muted } else { c.text })
        .when(selected, |r| r.bg(c.selected))
        .on_click(move |_, window, cx| on_row(ix, window, cx))
        .children(lead)
        .child(div().flex_1().min_w_0().truncate().child(row.label.clone()))
        .when_some(row.origin.clone(), |r, origin| {
            r.child(div().text_color(c.faint).child(origin))
        })
        .when(row.checked, |r| {
            r.child(svg().path("icons/check.svg").size(pt(k.check)).text_color(c.green))
        })
        .map(|r| {
            if row.submenu {
                r.child(
                    svg()
                        .path("icons/chevron-right.svg")
                        .size(pt(k.chevron))
                        .text_color(c.faint),
                )
            } else if let Some(caps) = caps {
                r.child(keycap(caps, ChipState::of(selected)))
            } else {
                r
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(label: &str) -> PanelRow {
        PanelRow {
            label: label.into(),
            ..PanelRow::default()
        }
    }

    #[test]
    fn matches_labels_and_origins_case_insensitively() {
        let page = PanelPage {
            crumb: None,
            searchable: true,
            sections: vec![
                PanelSection {
                    label: None,
                    rows: vec![row("Details"), row("Copy")],
                },
                PanelSection {
                    label: Some("Copy".into()),
                    rows: vec![PanelRow {
                        origin: Some("More".into()),
                        ..row("Path")
                    }],
                },
            ],
        };
        assert_eq!(matches(&page, ""), [(0, 0), (0, 1), (1, 0)]);
        assert_eq!(matches(&page, "pa"), [(1, 0)]);
        assert_eq!(matches(&page, "MORE"), [(1, 0)]);
        assert_eq!(matches(&page, "zz"), [] as [(usize, usize); 0]);
    }

    #[test]
    fn counts_the_labels_and_hairlines_above_a_row() {
        let page = PanelPage {
            sections: vec![
                PanelSection {
                    label: Some("Types".into()),
                    rows: vec![PanelRow::default(); 2],
                },
                PanelSection {
                    label: Some("Tags".into()),
                    rows: vec![PanelRow::default(); 2],
                },
            ],
            ..Default::default()
        };
        let visible = [(0, 0), (0, 1), (1, 0), (1, 1)];
        // Types label, row 0, row 1, hairline, Tags label, row 2, row 3
        assert_eq!(child_index(&page, &visible, 0), 1);
        assert_eq!(child_index(&page, &visible, 1), 2);
        assert_eq!(child_index(&page, &visible, 2), 5);
        assert_eq!(child_index(&page, &visible, 3), 6);
    }
}
