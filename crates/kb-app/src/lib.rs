//! The kb search panel: a resident GPUI Kit app over `kb-domain`, opened by a global hotkey.
//!
//! `main` only resolves the configuration and calls [`run`]; everything else is a module here,
//! so the UI integration tests build the same views the app shows. Only what `main` and those
//! tests use is public.

#![warn(missing_docs)]

mod accents;
pub mod cli;
pub mod config;
pub mod entry_list;
mod filter;
#[cfg(test)]
mod fixture;
mod http;
pub mod keys;
mod notes;
pub mod palette;
mod panel;
mod presentation;
pub mod search_view;
mod sync_view;
mod tag_images;
mod trace;
pub mod window_size;

use gpui_kit::component::{Theme, ThemeRegistry};

use crate::config::Config;

/// The bundle identifier, shared by the `.app`'s `Info.plist` and GPUI's app identity.
pub const APP_ID: &str = "dev.roalcantara.kb";

/// kb's GPUI Kit theme: DESIGN.md's palette, written by `tools/design/theme.mjs`.
const THEME: &str = include_str!("../assets/themes/kb.json");
/// The dark theme's name in [`THEME`].
const DARK_THEME: &str = "kb Dark";

/// Follows the system's light or dark appearance: kb's palette when dark, GPUI Kit's default
/// light theme when light. Every colour the app draws comes from the theme, so both work;
/// `SearchView` re-syncs whenever the appearance changes.
pub fn init_theme(cx: &mut gpui_kit::App) {
    let registry = ThemeRegistry::global_mut(cx);
    // The theme is compiled in and a UI test loads it: should it ever fail, the default dark stays.
    if registry.load_themes_from_str(THEME).is_ok()
        && let Some(dark) = registry.themes().get(DARK_THEME).cloned()
    {
        Theme::global_mut(cx).dark_theme = dark;
    }
    Theme::sync_system_appearance(None, cx);
}

/// Starts the app and blocks until it quits. `notices` are shown in the panel once it opens:
/// problems found before there was a window to report them in, such as unreadable preferences.
/// `trace` logs the show and keystroke latencies to stderr (`kb --trace`).
pub fn run(config: Config, notices: Vec<String>, trace: bool) {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        // Remote images (favicons, thumbnails) need a client; GPUI gives native apps none.
        .with_http_client(std::sync::Arc::new(http::CurlClient))
        .run(move |cx| {
            gpui_kit::init(cx);
            cx.set_app_identity(APP_ID, "kb");
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
