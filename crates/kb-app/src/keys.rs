//! The commands as typed Actions, and their key bindings: the catalog's keys, plus Esc, ⌘Q and the arrows.
//!
//! One logical command is one Action. The palette, its key binding and any other entry point
//! dispatch the same Action, so their label, shortcut and result cannot drift apart.

use std::ops::Range;

use gpui_kit::base::actions::Confirm;

use gpui_kit::{KeyBinding, actions};

use crate::catalog::Catalog;

/// The key context of the search panel; every binding below applies inside it.
pub const CONTEXT: &str = "Search";

/// Added to [`CONTEXT`] while the detail screen shows.
pub const DETAIL_CONTEXT: &str = "Detail";

/// Added to [`CONTEXT`] while a refresh screen shows.
pub const REFRESH_CONTEXT: &str = "Refresh";

/// Added to [`CONTEXT`] on the report and its detail: the report-only keys (Copy Location, Copy Error, Reveal).
pub const REPORT_CONTEXT: &str = "Report";

/// The action panel's own key context: it holds the keyboard on the submenus (More…, Links…),
/// which have no search field, so ↵, Esc and the arrows still reach the panel there.
pub const PANEL_CONTEXT: &str = "ActionPanel";

actions!(
    kb,
    [
        /// Open the action palette for the selected entry.
        OpenActions,
        /// Open the type and tag filter.
        OpenFilter,
        /// Run the selected entry's primary action.
        RunPrimary,
        /// Run the selected entry's secondary action.
        RunSecondary,
        /// The screen's ↵ (its catalog key `enter`).
        ScreenEnter,
        /// The screen's ⌘↵.
        ScreenCmdEnter,
        /// The screen's ⇧⌘↵.
        ScreenShiftCmdEnter,
        /// Open the selected entry's source file.
        OpenSourceFile,
        /// Copy the selected entry's source file path, absolute.
        CopyPath,
        /// Rebuild the index from the source files.
        Rebuild,
        /// Open the preferences file.
        OpenPreferences,
        /// Leave the current screen: back to the list, or hide the panel.
        Back,
        /// Quit the app: it has no Dock icon or menu bar to quit it from.
        Quit,
        /// Show the split pane (list); nothing on the detail screen (D20).
        ShowDetails,
        /// Hide the split pane (list); nothing on the detail screen (D20).
        HideDetails,
        /// Switch the window between its compact and expanded sizes.
        ToggleWindowSize,
        /// Open the selected entry's detail screen (Return in the list, which the query field
        /// reports; no binding of its own).
        OpenDetail,
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
        /// Copy the selected report row's location.
        CopyLocation,
        /// Copy the selected report row's error.
        CopyError,
        /// Show the selected report row's file in Finder.
        RevealFile,
        /// Page up through a list or a detail pane.
        PageUp,
        /// Page down through a list or a detail pane.
        PageDown,
        /// The next report row.
        NextRow,
        /// The previous report row.
        PreviousRow,
        /// The last report row.
        LastRow,
        /// The first report row.
        FirstRow,
        /// Move the keys between a split's list and its pane.
        TogglePane,
    ]
);

/// The panel's key bindings: every keyed action of the catalog, then the keys kb fixes (Esc, ⌘Q,
/// the arrows, Return on the detail screen).
#[must_use]
pub fn key_bindings(catalog: &Catalog) -> Vec<KeyBinding> {
    let context = Some(CONTEXT);
    let mut bindings: Vec<KeyBinding> = catalog
        .actions
        .iter()
        .filter_map(|(id, action)| Some((id.as_str(), action.keys.as_deref()?)))
        .filter_map(|(id, keys)| {
            Some(match id {
                "actions" => KeyBinding::new(keys, OpenActions, context),
                "filter" => KeyBinding::new(keys, OpenFilter, context),
                "resize" => KeyBinding::new(keys, ToggleWindowSize, context),
                "edit" => KeyBinding::new(keys, OpenSourceFile, context),
                "refresh" => KeyBinding::new(keys, Rebuild, context),
                "configure" => KeyBinding::new(keys, OpenPreferences, context),
                "copy-path" => KeyBinding::new(keys, CopyPath, context),
                "copy-location" => KeyBinding::new(keys, CopyLocation, Some(REPORT_CONTEXT)),
                "copy-error" => KeyBinding::new(keys, CopyError, Some(REPORT_CONTEXT)),
                "reveal" => KeyBinding::new(keys, RevealFile, Some(REPORT_CONTEXT)),
                _ => return None,
            })
        })
        .collect();
    bindings.extend([
        // The list's ↵ and ⌘↵ come from the query field (it reports Enter, then propagates it), so
        // the ladder is bound here only where no text field has the keyboard: the detail screen.
        KeyBinding::new("enter", ScreenEnter, Some(DETAIL_CONTEXT)),
        KeyBinding::new("cmd-enter", ScreenCmdEnter, Some(DETAIL_CONTEXT)),
        KeyBinding::new("enter", ScreenEnter, Some(REFRESH_CONTEXT)),
        KeyBinding::new("cmd-enter", ScreenCmdEnter, Some(REFRESH_CONTEXT)),
        // The query field does not bind ⇧⌘↵, so it reaches the panel everywhere.
        KeyBinding::new("shift-cmd-enter", ScreenShiftCmdEnter, context),
        KeyBinding::new("escape", Back, context),
        KeyBinding::new("cmd-q", Quit, context),
        // → and ← reach these only where no text field takes them first: the detail screen. In
        // the list the query field binds them deeper, so `SearchView` captures them there and
        // acts only with the caret at the query's end (`caret_at_end`).
        KeyBinding::new("right", ShowDetails, context),
        KeyBinding::new("left", HideDetails, context),
        // One navigation-key model (close-out D2): the same six keys on every surface; `SearchView::nav` sends each
        // to its owner. The text fields capture them first (`search_view/header.rs`, `panel.rs`).
        KeyBinding::new("up", PreviousRow, context),
        KeyBinding::new("down", NextRow, context),
        KeyBinding::new("cmd-up", FirstRow, context),
        KeyBinding::new("cmd-down", LastRow, context),
        KeyBinding::new("pageup", PageUp, context),
        KeyBinding::new("pagedown", PageDown, context),
        // ⌘1–⌘9: row n's primary action in the list; link n on the detail screen (R7).
        // ⇥ / ⇧⇥: the keys between a split's list and its pane (close-out D3); the query field captures them first.
        KeyBinding::new("tab", TogglePane, context),
        KeyBinding::new("shift-tab", TogglePane, context),
        KeyBinding::new("cmd-1", RunRow1, context),
        KeyBinding::new("cmd-2", RunRow2, context),
        KeyBinding::new("cmd-3", RunRow3, context),
        KeyBinding::new("cmd-4", RunRow4, context),
        KeyBinding::new("cmd-5", RunRow5, context),
        KeyBinding::new("cmd-6", RunRow6, context),
        KeyBinding::new("cmd-7", RunRow7, context),
        KeyBinding::new("cmd-8", RunRow8, context),
        KeyBinding::new("cmd-9", RunRow9, context),
        // The action panel's own keys (D9), so ↵, ⌘↵, Esc and the arrows reach it on the
        // submenus too, where the search field (and its input context) is gone.
        KeyBinding::new("enter", Confirm { secondary: false }, Some(PANEL_CONTEXT)),
        KeyBinding::new("secondary-enter", Confirm { secondary: true }, Some(PANEL_CONTEXT)),
        KeyBinding::new("shift-cmd-enter", ScreenShiftCmdEnter, Some(PANEL_CONTEXT)),
        KeyBinding::new("escape", Back, Some(PANEL_CONTEXT)),
    ]);
    bindings
}

/// Whether → and ← act on the split pane rather than the caret: nothing selected, and the caret
/// after the query's last character, where typing leaves it.
#[must_use]
pub fn caret_at_end(cursor: usize, selection: Range<usize>, len: usize) -> bool {
    selection.is_empty() && cursor == len
}

#[cfg(test)]
mod tests {
    use gpui_kit::AsKeystroke as _;

    use super::*;

    #[test]
    fn binds_the_catalog_keys_and_the_fixed_ones() {
        let bindings = key_bindings(&Catalog::default());
        assert_eq!(bindings.len(), 40);
    }

    #[test]
    fn follows_an_overridden_key() {
        let (catalog, _) = Catalog::load(&toml::from_str("[actions.resize]\nkeys = \"cmd-shift-m\"\n").unwrap());
        let keys: Vec<String> = key_bindings(&catalog)
            .iter()
            .flat_map(|binding| binding.keystrokes().iter().map(|k| k.as_keystroke().unparse()))
            .collect();
        assert!(keys.iter().any(|k| k == "cmd-shift-m"), "{keys:?}");
        assert!(!keys.iter().any(|k| k == "cmd-shift-f"), "{keys:?}");
    }

    #[test]
    fn acts_on_the_pane_only_with_the_caret_at_the_end_and_nothing_selected() {
        assert!(caret_at_end(7, 7..7, 7));
        assert!(caret_at_end(0, 0..0, 0));
        assert!(!caret_at_end(3, 3..3, 7));
        assert!(!caret_at_end(7, 2..7, 7));
    }
}
