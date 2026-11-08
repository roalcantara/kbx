//! Theme…: the themes listed from the folder, previewed as the selection moves, kept with ↵ and
//! saved by name in config.toml, restored when the submenu or the panel closes without one.

use gpui_kit::component::WindowExt as _;
use gpui_kit::{App, Context, Window};

use super::{Overlay, SearchView};
use crate::actions::{Submenu, ThemeRow};
use crate::catalog::fill;
use crate::theme::folder;
use crate::theme::scheme::Palette;
use crate::ui::action_panel::matches;

impl SearchView {
    /// Re-reads the themes folder into the rows Theme… and the root's search list.
    pub(super) fn reload_themes(&mut self) {
        self.themes = folder::list(&self.config.themes_dir())
            .into_iter()
            .map(|entry| {
                let label = entry.label().to_owned();
                let (swatch, broken) = match &entry.scheme {
                    Ok(scheme) => (
                        ["mauve", "blue", "green", "peach"]
                            .iter()
                            .map(|role| scheme.palette.role(role).hsla())
                            .collect(),
                        None,
                    ),
                    Err(why) => (Vec::new(), Some(why.clone())),
                };
                ThemeRow {
                    current: entry.id == self.theme_id,
                    id: entry.id,
                    label,
                    swatch,
                    broken,
                }
            })
            .collect();
    }

    /// Opening Theme…: remembers the theme in use, to restore it; the row of that theme, to select.
    pub(super) fn begin_theme_preview(&mut self) -> usize {
        self.theme_preview = Some(crate::theme::active_palette());
        self.themes.iter().position(|t| t.current).unwrap_or(0)
    }

    /// `id`'s palette with config.toml's `[colors]` on top, or why it cannot be used.
    fn theme_palette(&self, id: &str) -> Result<(String, Palette), String> {
        let scheme = folder::load(&self.config.themes_dir(), id)?;
        let mut palette = scheme.palette;
        let _ = palette.apply(self.config.colors());
        Ok((scheme.label, palette))
    }

    /// Previews the theme under the selection, while Theme… is open; a broken theme is skipped.
    pub(super) fn preview_selected_theme(&mut self, cx: &mut Context<Self>) {
        if self.navigator.overlay() != Overlay::Actions(Some(Submenu::Theme)) {
            return;
        }
        let page = self.current_page(cx);
        let visible = matches(&page.page, "");
        let id = visible
            .get(self.panel_selected)
            .and_then(|&(s, r)| page.command(s, r))
            .and_then(|command| match command {
                crate::actions::PanelCommand::Theme(id) => Some(id.clone()),
                _ => None,
            });
        if let Some((_, palette)) = id.and_then(|id| self.theme_palette(&id).ok()) {
            crate::apply_palette(palette, cx);
        }
    }

    /// ↵ on a theme: draws with it, saves it in config.toml and says so; a weak theme also says why.
    pub(super) fn keep_theme(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let (label, palette) = match self.theme_palette(id) {
            Ok(found) => found,
            Err(why) => return window.push_notification(why, cx),
        };
        let weak = crate::theme::legibility::first_failure(&palette);
        crate::apply_palette(palette, cx);
        id.clone_into(&mut self.theme_id);
        self.theme_preview = None;
        self.show_overlay(Overlay::None, window, cx);
        let m = &self.catalog.messages;
        window.push_notification(fill(&m.theme_chosen, &[("name", &label)]), cx);
        if let Some(failure) = weak {
            window.push_notification(fill(&m.theme_weak, &[("name", &label), ("failure", &failure)]), cx);
        }
        if let Err(error) = self.config.save_theme(id) {
            let notice = fill(
                &m.theme_save_failed,
                &[
                    ("file", &self.config.preferences().display().to_string()),
                    ("error", &error.to_string()),
                ],
            );
            window.push_notification(notice, cx);
        }
    }

    /// Leaving Theme… without ↵ (Esc, or the panel hiding): restores the theme kbx had when it opened.
    pub fn cancel_theme_preview(&mut self, cx: &mut App) {
        if let Some(palette) = self.theme_preview.take() {
            crate::apply_palette(palette, cx);
        }
    }
}
