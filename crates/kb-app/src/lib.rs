//! The kb search panel: a resident GPUI Kit app over `kb-domain`, opened by a global hotkey.
//!
//! `main` only resolves the configuration and calls [`run`]; everything else is a module here,
//! so the UI integration tests build the same views the app shows. Only what `main` and those
//! tests use is public.

#![warn(missing_docs)]

mod actions;
mod assets;
pub mod catalog;
pub mod cli;
pub mod command_path;
pub mod config;
pub mod entry_list;
mod filter;
#[cfg(test)]
mod fixture;
mod http;
pub mod keys;
mod panel;
mod presentation;
pub mod search_view;
mod sync_view;
mod tag_images;
pub mod theme;
mod trace;
mod ui;
pub mod window_size;

use gpui_kit::component::{Theme, ThemeMode, ThemeRegistry};

use crate::config::Config;

/// The bundle identifier, shared by the `.app`'s `Info.plist` and GPUI's app identity.
pub const APP_ID: &str = "dev.roalcantara.kb";

/// kb's GPUI Kit theme, always dark (D1): built from the active theme's palette (`theme::gpui`), Inter
/// and `JetBrains` Mono, and GPUI Kit's base font size kept at the 16 points the design module counts
/// rems in. Called again after a theme change.
pub fn init_theme(cx: &mut gpui_kit::App) {
    let registry = ThemeRegistry::global_mut(cx);
    let json = theme::gpui::theme_json(&theme::active_palette());
    // The JSON is built from a palette and a UI test loads it: should it ever fail, the default dark stays.
    if registry.load_themes_from_str(&json).is_ok()
        && let Some(dark) = registry.themes().get(theme::gpui::THEME_NAME).cloned()
    {
        Theme::global_mut(cx).dark_theme = dark;
    }
    Theme::change(ThemeMode::Dark, None, cx);
    let typography = &ui::design::design().typography;
    let theme = Theme::global_mut(cx);
    theme.font_family = typography.ui.clone().into();
    theme.mono_font_family = typography.mono.clone().into();
    theme.font_size = ui::design::pixels(ui::design::POINTS_PER_REM);
    ui::notice::configure(Theme::global_mut(cx));
}

/// Makes `palette` kb's colours now: the views' tokens, GPUI Kit's theme, and a redraw (themes spec D5).
pub fn apply_palette(palette: theme::scheme::Palette, cx: &mut gpui_kit::App) {
    theme::set_active(palette);
    init_theme(cx);
    cx.refresh_windows();
}

/// Starts the app and blocks until it quits. `notices` are shown in the panel once it opens:
/// problems found before there was a window to report them in, such as unreadable preferences.
/// `trace` logs the show and keystroke latencies to stderr (`kb --trace`).
pub fn run(config: Config, notices: Vec<String>, trace: bool) {
    gpui_kit::application()
        .with_assets(assets::KbAssets)
        // Remote images (favicons, thumbnails) need a client; GPUI gives native apps none.
        .with_http_client(std::sync::Arc::new(http::CurlClient))
        .run(move |cx| {
            gpui_kit::init(cx);
            cx.set_app_identity(APP_ID, "kb");
            if let Err(error) = ui::design::load_fonts(cx) {
                #[expect(clippy::print_stderr, reason = "a font failed to load before any window exists")]
                {
                    eprintln!("kb: {error}");
                }
            }
            let mut notices = notices;
            let dir = config.themes_dir();
            if let Err(error) = theme::folder::seed(&dir) {
                notices.push(format!("Couldn’t write kb’s themes to {}: {error}", dir.display()));
            }
            let (_, palette, theme_notices) =
                theme::resolve(&dir, config.theme(), &ui::design::design().theme, config.colors());
            notices.extend(theme_notices);
            theme::set_active(palette);
            init_theme(cx);
            if trace {
                trace::enable(cx);
            }
            if let Err(error) = panel::open(config, notices, cx) {
                #[expect(
                    clippy::print_stderr,
                    reason = "the window failed to open: stderr is the only channel left"
                )]
                {
                    eprintln!("kb: {error}");
                }
                cx.quit();
            }
        });
}
