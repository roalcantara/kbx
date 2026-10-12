//! The commands as typed Actions, and their key bindings: Raycast's keys, plus ⌘⇧F and ⌘Q.
//!
//! One logical command is one Action. The palette, its key binding and any other entry point
//! dispatch the same Action, so their label, shortcut and result cannot drift apart.

use std::ops::Range;

use gpui_kit::{KeyBinding, actions};

/// The key context of the search panel; every binding below applies inside it.
pub const CONTEXT: &str = "Search";

/// Added to [`CONTEXT`] while the detail screen shows.
pub const DETAIL_CONTEXT: &str = "Detail";

actions!(
    kb,
    [
        /// Open the action palette for the selected entry.
        OpenActions,
        /// Open the type and tag filter.
        OpenFilter,
        /// Run the selected entry's primary action.
        RunPrimary,
        /// Copy the selected bookmark's URL.
        CopyUrl,
        /// Show or hide the details pane (list) or the metadata (detail screen).
        ToggleDetails,
        /// Open the selected entry's source file.
        OpenSourceFile,
        /// Rebuild the index from the source files.
        Rebuild,
        /// Open the preferences file.
        OpenPreferences,
        /// Leave the current screen: back to the list, or hide the panel.
        Back,
        /// Quit the app: it has no Dock icon or menu bar to quit it from.
        Quit,
        /// Show the split pane (list) or the metadata (detail screen).
        ShowDetails,
        /// Hide the split pane (list) or the metadata (detail screen).
        HideDetails,
        /// Switch the window between its compact and expanded sizes.
        ToggleWindowSize,
        /// Scroll the detail screen's notes down a few lines.
        ScrollNotesDown,
        /// Scroll the detail screen's notes up a few lines.
        ScrollNotesUp,
        /// Scroll the detail screen's notes down a page.
        PageNotesDown,
        /// Scroll the detail screen's notes up a page.
        PageNotesUp,
        /// Open the selected entry's detail screen (Return in the list, which the query field
        /// reports; no binding of its own).
        OpenDetail,
    ]
);

/// The panel's key bindings.
#[must_use]
pub fn key_bindings() -> Vec<KeyBinding> {
    let context = Some(CONTEXT);
    vec![
        KeyBinding::new("cmd-k", OpenActions, context),
        // Enter runs the primary action only on the detail screen. In the list, the search field
        // propagates Enter after reporting it, so a panel-wide binding would also fire there.
        KeyBinding::new("enter", RunPrimary, Some(DETAIL_CONTEXT)),
        KeyBinding::new("cmd-p", OpenFilter, context),
        KeyBinding::new("cmd-shift-c", CopyUrl, context),
        KeyBinding::new("cmd-d", ToggleDetails, context),
        KeyBinding::new("cmd-o", OpenSourceFile, context),
        KeyBinding::new("cmd-r", Rebuild, context),
        KeyBinding::new("cmd-shift-,", OpenPreferences, context),
        KeyBinding::new("escape", Back, context),
        KeyBinding::new("cmd-q", Quit, context),
        // → and ← reach these only where no text field takes them first: the detail screen. In
        // the list the query field binds them deeper, so `SearchView` captures them there and
        // acts only with the caret at the query's end (`caret_at_end`).
        KeyBinding::new("right", ShowDetails, context),
        KeyBinding::new("left", HideDetails, context),
        KeyBinding::new("cmd-shift-f", ToggleWindowSize, context),
        // The detail screen has no text field: the arrows scroll its notes, ⌘ a page at a time.
        KeyBinding::new("down", ScrollNotesDown, Some(DETAIL_CONTEXT)),
        KeyBinding::new("up", ScrollNotesUp, Some(DETAIL_CONTEXT)),
        KeyBinding::new("cmd-down", PageNotesDown, Some(DETAIL_CONTEXT)),
        KeyBinding::new("cmd-up", PageNotesUp, Some(DETAIL_CONTEXT)),
    ]
}

/// Whether → and ← act on the split pane rather than the caret: nothing selected, and the caret
/// after the query's last character, where typing leaves it.
#[must_use]
pub fn caret_at_end(cursor: usize, selection: Range<usize>, len: usize) -> bool {
    selection.is_empty() && cursor == len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binds_every_shortcut_in_the_search_context() {
        assert_eq!(key_bindings().len(), 17);
    }

    #[test]
    fn acts_on_the_pane_only_with_the_caret_at_the_end_and_nothing_selected() {
        assert!(caret_at_end(7, 7..7, 7));
        assert!(caret_at_end(0, 0..0, 0));
        assert!(!caret_at_end(3, 3..3, 7));
        assert!(!caret_at_end(7, 2..7, 7));
    }
}
