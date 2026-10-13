# Plan 8b: The catalog and its overrides

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** one compiled-in file, `crates/kb-app/assets/catalog.toml`, says what kb offers (the four entry types, every
action with its label and keys, each screen's keys, the action panel's rows, every message); `~/.config/kb/config.toml`
overrides any of it key by key; a bad override shows a notice and falls back for that key only; the app's key
bindings come from the catalog; Configure (⇧⌘,) writes the defaults as comments (spec D18, §2).

**Architecture:** `catalog.rs` deserializes the TOML into typed structs and validates them (icons exist, tints are
tokens, keys parse, references resolve, no key bound twice). `config.rs` splits `config.toml` into the two location
settings and the catalog overrides. `Catalog::load` applies the overrides one leaf at a time, keeping a leaf only if
the whole catalog still validates. `keys::key_bindings(&Catalog)` binds the catalog's keyed actions.

**Tech Stack:** Rust, `toml 0.9`, `serde`, GPUI Kit 0.7.1 (`Keystroke`, `KeyBinding`).

**Expects:** Plan 8a done (`crate::ui::design::design()` exists).

**Index:** [2026-10-12-kb-native-08-design-system.md](2026-10-12-kb-native-08-design-system.md).

---

## File structure

| File                                    | Change  | Responsibility                                                         |
| --------------------------------------- | ------- | ---------------------------------------------------------------------- |
| `crates/kb-app/assets/catalog.toml`     | create  | What kb offers                                                         |
| `crates/kb-app/src/catalog.rs`          | create  | Types, validation, overrides, message templates                        |
| `crates/kb-app/src/config.rs`           | modify  | Splits config.toml into locations, catalog overrides and notices       |
| `crates/kb-app/src/assets.rs`           | modify  | `has_icon`; the catalog's icons bundled                                |
| `crates/kb-app/src/keys.rs`             | modify  | Bindings from the catalog; `CopyPath`                                  |
| `crates/kb-app/src/panel.rs`            | modify  | Builds the catalog, binds its keys, shows its notices                  |
| `crates/kb-app/src/search_view.rs`      | modify  | Holds the catalog; Configure writes the defaults; Copy Path            |
| `crates/kb-app/src/lib.rs`              | modify  | `pub mod catalog`                                                      |
| `crates/kb-app/tests/ui.rs`             | modify  | Builds the catalog in the test window                                  |

---

### Task 1: The catalog file and its types

**Files:** create `crates/kb-app/assets/catalog.toml`, `crates/kb-app/src/catalog.rs`; modify
`crates/kb-app/src/lib.rs`, `crates/kb-app/src/assets.rs`.

- [ ] **Step 1: Write the catalog.** Create `crates/kb-app/assets/catalog.toml` with exactly:

```toml
# What kb offers: its entry types, its actions, each screen's keys, the action panel's rows and the
# words it shows (spec D18). ~/.config/kb/config.toml overrides any key below by the same path, for
# example `[actions.copy] label = "Yank"`. Colours and sizes are not here: they are DESIGN.md's.
#
# Keys use GPUI's spelling: modifiers joined by `-` before the key (`cmd-shift-f`, `shift-cmd-enter`).
# A screen's keys are `enter`, `cmd-enter`, `shift-cmd-enter` and `escape`; their value is an action,
# or `entry.primary` / `entry.secondary` (the selected entry's actions, list and detail only).

# Entry types (D6): any other type in the sources is ignored by the app. The table key is the type as
# the sources write it. `tint` is a colour token of DESIGN.md; `hero` is media, command, snippet or task.
[types.bookmarks]
label = "Bookmarks"
icon = "bookmark"
tint = "blue"
hero = "media"
primary = "open"
secondary = "copy"

[types.cheats]
label = "Cheat Sheets"
icon = "book-open"
tint = "mauve"
hero = "snippet"
primary = "copy"

[types.commands]
label = "Commands"
icon = "square-terminal"
tint = "green"
hero = "command"
mono_title = true
primary = "copy"

[types.tasks]
label = "Tasks"
icon = "circle-check"
tint = "pink"
hero = "task"
primary = "copy"

# Actions (D7, D9). `keys` binds the action everywhere in the panel.
[actions.details]
label = "Details"

[actions.open]
label = "Open"

[actions.copy]
label = "Copy"

[actions.resize]
label = "Resize"
keys = "cmd-shift-f"

[actions.copy-description]
label = "Description"

[actions.copy-notes]
label = "Notes"

[actions.copy-path]
label = "Path"
keys = "cmd-alt-c"

[actions.edit]
label = "Edit"
keys = "cmd-o"

[actions.refresh]
label = "Refresh"
keys = "cmd-r"

[actions.configure]
label = "Configure"
keys = "cmd-shift-,"

[actions.actions]
label = "Actions"
keys = "cmd-k"

[actions.filter]
label = "Filter"
keys = "cmd-p"

[actions.stop]
label = "Stop"

[actions.back]
label = "Back"

[actions.retry]
label = "Retry"

[actions.toggle]
label = "Toggle"

[actions.only]
label = "Only"

[actions.clear-filters]
label = "Clear All"

[actions.open-link]
label = "Open"

[actions.copy-link]
label = "Copy"

# Screen keys (D8, D10, D15, D16): the ladder ↵ / ⌘↵ / ⇧⌘↵, and Esc while refreshing.
[screens.list]
enter = "details"
cmd-enter = "entry.primary"
shift-cmd-enter = "entry.secondary"

[screens.detail]
enter = "entry.primary"
cmd-enter = "entry.secondary"

[screens.empty]
enter = "refresh"

[screens.no-index]
enter = "refresh"

[screens.filter]
enter = "toggle"
cmd-enter = "only"

[screens.links]
enter = "open-link"
cmd-enter = "copy-link"

[screens.refreshing]
escape = "stop"

[screens.report]
enter = "back"

[screens.stopped]
enter = "retry"

[screens.failed]
enter = "retry"
cmd-enter = "configure"

# The action panel (D9): after the screen's keys, the root lists `root`, then Links… (with links) and
# More…; More… lists `more_copy` under `copy_section`, then `more_admin` under `admin_section`.
[panel]
links = "Links…"
more = "More…"
search = "Search"
copy_section = "Copy"
admin_section = "Administration"
root = ["resize"]
more_copy = ["copy-description", "copy-notes", "copy-path"]
more_admin = ["edit", "refresh", "configure"]

# The words kb shows. `{name}` is filled in by the app.
[messages]
search_placeholder = "Search Knowledge Base…"
all_entries = "All Entries"
copied = "Copied to clipboard"
paste_hint = "to paste"
ready = "Ready"
results = "{count} results"
one_result = "1 result"
refreshing_status = "Refreshing…"
no_index_status = "No index"
no_match_title = "No matching entries found"
no_match = "No entries match “{query}”."
no_match_filtered = "No entries match “{query}” with this filter."
no_entries = "Your sources have no bookmarks, cheat sheets, commands or tasks."
no_index_title = "No entries indexed yet"
no_index = "Press Refresh to build the index from your sources."
refreshing_title = "Refreshing the index…"
refreshing_progress = "{done} of {total} files · {file}"
report_summary = "{entries} entries from {files} files in {seconds} s"
report_failed = "{count} failed"
report_skipped = "{count} with skipped entries"
report_indexed = "Indexed"
report_skipped_entries = "Skipped: {entries} (not a map)"
report_empty_file = "No entries"
stopped_title = "Refresh stopped"
stopped = "The previous index is unchanged."
failed_title = "Couldn’t refresh the index"
failed = "{error} Check it in Configure."
filters = "{count} filters"
one_filter = "1 filter"
links_label = "Links"
more_origin = "More"
links_origin = "Links"
```

- [ ] **Step 2: Write the module with its tests and failing bodies.** Create `crates/kb-app/src/catalog.rs`:

```rust
//! The catalog (spec D18): what kb offers — entry types, actions, each screen's keys, the action
//! panel's rows and the words it shows — compiled in from `assets/catalog.toml`, overridden key by
//! key from `~/.config/kb/config.toml`.

use std::collections::BTreeMap;

use gpui_kit::Keystroke;
use serde::Deserialize;

/// The catalog as shipped.
const CATALOG: &str = include_str!("../assets/catalog.toml");

/// The actions kb knows how to run; a catalog names exactly these.
pub const ACTION_IDS: [&str; 20] = [
    "details",
    "open",
    "copy",
    "resize",
    "copy-description",
    "copy-notes",
    "copy-path",
    "edit",
    "refresh",
    "configure",
    "actions",
    "filter",
    "stop",
    "back",
    "retry",
    "toggle",
    "only",
    "clear-filters",
    "open-link",
    "copy-link",
];

/// What an entry type's primary or secondary action may be.
pub const ENTRY_ACTIONS: [&str; 2] = ["open", "copy"];

/// The keys a screen maps, in the order the footer shows them.
pub const SCREEN_KEYS: [&str; 4] = ["enter", "cmd-enter", "shift-cmd-enter", "escape"];

/// A screen key's value naming the selected entry's primary action.
pub const ENTRY_PRIMARY: &str = "entry.primary";
/// A screen key's value naming the selected entry's secondary action.
pub const ENTRY_SECONDARY: &str = "entry.secondary";

/// What a type's hero shows (D12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Hero {
    /// The YouTube thumbnail, else the URL as a link line.
    Media,
    /// The command card of the primary content.
    Command,
    /// The command card of the `cheat` field, when there is one.
    Snippet,
    /// The status and priority chips.
    Task,
}

/// An entry type.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntryType {
    /// How it is named: the filter, tooltips.
    pub label: String,
    /// A Lucide icon name, bundled in `assets.rs`.
    pub icon: String,
    /// A colour token: the tile's accent.
    pub tint: String,
    /// What the entry view's hero shows.
    pub hero: Hero,
    /// Titles in JetBrains Mono.
    #[serde(default)]
    pub mono_title: bool,
    /// The primary action: `open` or `copy`.
    pub primary: String,
    /// The secondary action, if any.
    pub secondary: Option<String>,
}

/// An action.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    /// Its label.
    pub label: String,
    /// The keys that run it anywhere in the panel.
    pub keys: Option<String>,
}

/// A screen's keys: a key of [`SCREEN_KEYS`] → an action id, [`ENTRY_PRIMARY`] or [`ENTRY_SECONDARY`].
pub type ScreenKeys = BTreeMap<String, String>;

/// The screens that map keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// The list.
    List,
    /// The detail screen.
    Detail,
    /// The list with no result.
    Empty,
    /// No index yet.
    NoIndex,
    /// The filter is open.
    Filter,
    /// The Links… submenu is open.
    Links,
    /// A refresh is running.
    Refreshing,
    /// A refresh finished: its report.
    Report,
    /// A refresh was stopped.
    Stopped,
    /// A refresh failed.
    Failed,
}

/// Every screen's keys.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
#[expect(missing_docs, reason = "each field is the screen of the same name")]
pub struct Screens {
    pub list: ScreenKeys,
    pub detail: ScreenKeys,
    pub empty: ScreenKeys,
    pub no_index: ScreenKeys,
    pub filter: ScreenKeys,
    pub links: ScreenKeys,
    pub refreshing: ScreenKeys,
    pub report: ScreenKeys,
    pub stopped: ScreenKeys,
    pub failed: ScreenKeys,
}

/// The action panel's rows and words (D9).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Panel {
    /// The Links… row.
    pub links: String,
    /// The More… row.
    pub more: String,
    /// The search field's placeholder.
    pub search: String,
    /// More…'s first section.
    pub copy_section: String,
    /// More…'s second section.
    pub admin_section: String,
    /// The root's rows after the screen's keys.
    pub root: Vec<String>,
    /// More…'s copy rows.
    pub more_copy: Vec<String>,
    /// More…'s administration rows.
    pub more_admin: Vec<String>,
}

/// The words kb shows; `{name}` is filled in with [`fill`].
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[expect(missing_docs, reason = "catalog.toml describes each message by its text")]
pub struct Messages {
    pub search_placeholder: String,
    pub all_entries: String,
    pub copied: String,
    pub paste_hint: String,
    pub ready: String,
    pub results: String,
    pub one_result: String,
    pub refreshing_status: String,
    pub no_index_status: String,
    pub no_match_title: String,
    pub no_match: String,
    pub no_match_filtered: String,
    pub no_entries: String,
    pub no_index_title: String,
    pub no_index: String,
    pub refreshing_title: String,
    pub refreshing_progress: String,
    pub report_summary: String,
    pub report_failed: String,
    pub report_skipped: String,
    pub report_indexed: String,
    pub report_skipped_entries: String,
    pub report_empty_file: String,
    pub stopped_title: String,
    pub stopped: String,
    pub failed_title: String,
    pub failed: String,
    pub filters: String,
    pub one_filter: String,
    pub links_label: String,
    pub more_origin: String,
    pub links_origin: String,
}

/// The catalog.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    /// Entry types, by the type as the sources write it (lowercase).
    pub types: BTreeMap<String, EntryType>,
    /// Actions, by id.
    pub actions: BTreeMap<String, Action>,
    /// Each screen's keys.
    pub screens: Screens,
    /// The action panel.
    pub panel: Panel,
    /// The words kb shows.
    pub messages: Messages,
}

/// Why a catalog is not valid.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CatalogError {
    /// The TOML does not have the catalog's shape.
    #[error("{0}")]
    Shape(String),
    /// A value is out of bounds.
    #[error("{0}")]
    Invalid(String),
}

impl Default for Catalog {
    /// The catalog as shipped; a unit test proves it valid.
    #[expect(clippy::expect_used, reason = "the compiled-in catalog is checked by a unit test")]
    fn default() -> Self {
        let table: toml::Table = toml::from_str(CATALOG).expect("catalog.toml is TOML");
        Self::from_table(table).expect("catalog.toml is valid")
    }
}

impl Catalog {
    /// A catalog from a TOML table, validated.
    ///
    /// # Errors
    ///
    /// The table is not a catalog, or a value is out of bounds.
    pub fn from_table(table: toml::Table) -> Result<Self, CatalogError> {
        let _ = table;
        Err(CatalogError::Shape("not yet".into()))
    }
}

/// `template` with each `{name}` replaced by its value.
#[must_use]
pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let _ = values;
    template.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ships_a_valid_catalog() {
        let catalog = Catalog::default();
        assert_eq!(
            catalog.types.keys().map(String::as_str).collect::<Vec<_>>(),
            ["bookmarks", "cheats", "commands", "tasks"]
        );
        assert_eq!(catalog.types["bookmarks"].secondary.as_deref(), Some("copy"));
        assert_eq!(catalog.screens.list["shift-cmd-enter"], ENTRY_SECONDARY);
        assert_eq!(catalog.actions["resize"].keys.as_deref(), Some("cmd-shift-f"));
        assert_eq!(catalog.actions.len(), ACTION_IDS.len());
    }

    fn with(edit: impl FnOnce(&mut toml::Table)) -> Result<Catalog, CatalogError> {
        let mut table: toml::Table = toml::from_str(CATALOG).unwrap();
        edit(&mut table);
        Catalog::from_table(table)
    }

    fn set(table: &mut toml::Table, path: &[&str], value: toml::Value) {
        let (last, parents) = path.split_last().unwrap();
        let mut node = table;
        for key in parents {
            node = node.get_mut(*key).unwrap().as_table_mut().unwrap();
        }
        node.insert((*last).to_owned(), value);
    }

    #[test]
    fn rejects_what_kb_cannot_draw_or_run() {
        let cases: [(&[&str], toml::Value); 7] = [
            (&["types", "tasks", "icon"], "no-such-icon".into()),
            (&["types", "tasks", "tint"], "teal".into()),
            (&["types", "tasks", "primary"], "resize".into()),
            (&["actions", "resize", "keys"], "cmd-".into()),
            (&["actions", "resize", "keys"], "cmd-k".into()),
            (&["screens", "empty", "enter"], "entry.primary".into()),
            (&["panel", "root"], toml::Value::Array(vec!["fly".into()])),
        ];
        for (path, value) in cases {
            let result = with(|t| set(t, path, value.clone()));
            assert!(matches!(result, Err(CatalogError::Invalid(_))), "{path:?} = {value}: {result:?}");
        }
        assert!(matches!(
            with(|t| set(t, &["screens", "list", "tab"], "details".into())),
            Err(CatalogError::Invalid(_))
        ));
        assert!(matches!(
            with(|t| set(t, &["actions", "resize", "keys"], "enter".into())),
            Err(CatalogError::Invalid(_))
        ));
    }

    #[test]
    fn fills_message_templates() {
        assert_eq!(fill("{count} results", &[("count", "12")]), "12 results");
        assert_eq!(fill("{a} of {b}", &[("a", "1"), ("b", "2")]), "1 of 2");
        assert_eq!(fill("no names", &[("a", "1")]), "no names");
    }
}
```

In `crates/kb-app/src/lib.rs` add `pub mod catalog;` after `mod assets;` (keep the order: `mod accents;`,
`mod assets;`, `pub mod catalog;`, `pub mod cli;` …).

- [ ] **Step 3: Run the tests and see them fail.**

Run: `cargo nextest run -p kb-app catalog`
Expected: FAIL: `ships_a_valid_catalog` panics (`catalog.toml is valid: Shape("not yet")`), the other two fail.

- [ ] **Step 4: Bundle the catalog's icons.** In `crates/kb-app/src/assets.rs`, change the `icon_assets!` line to:

```rust
icon_assets!(
    ExtraIcons,
    [ArrowUpRight, Bookmark, Circle, CircleStop, Database, Funnel, Hash, Link, NotebookText, Power, SlidersHorizontal, X]
);
```

and add, after `impl AssetSource for KbAssets { … }`:

```rust
/// Whether the Lucide icon `name` (`book-open`) is bundled, by GPUI Kit or by kb.
#[must_use]
pub fn has_icon(name: &str) -> bool {
    matches!(KbAssets.load(&format!("icons/{name}.svg")), Ok(Some(_)))
}
```

(`assets` is a private module; `catalog.rs` reaches it as `crate::assets::has_icon`.)

- [ ] **Step 5: Write `from_table`, the validation and `fill`.** Replace the `impl Catalog { … }` block and the
      `fill` stub in `catalog.rs` with:

```rust
impl Catalog {
    /// A catalog from a TOML table, validated.
    ///
    /// # Errors
    ///
    /// The table is not a catalog, or a value is out of bounds.
    pub fn from_table(table: toml::Table) -> Result<Self, CatalogError> {
        let catalog: Self = toml::Value::Table(table)
            .try_into()
            .map_err(|error: toml::de::Error| CatalogError::Shape(error.message().to_owned()))?;
        catalog.validate()?;
        Ok(catalog)
    }

    /// The type an entry's type key names, matched trimmed and lowercased; `None` for a type kb
    /// ignores (D6).
    #[must_use]
    pub fn entry_type(&self, entry_type: &str) -> Option<&EntryType> {
        self.types.get(&entry_type.trim().to_lowercase())
    }

    /// A screen's keys.
    #[must_use]
    pub fn screen(&self, screen: Screen) -> &ScreenKeys {
        let s = &self.screens;
        match screen {
            Screen::List => &s.list,
            Screen::Detail => &s.detail,
            Screen::Empty => &s.empty,
            Screen::NoIndex => &s.no_index,
            Screen::Filter => &s.filter,
            Screen::Links => &s.links,
            Screen::Refreshing => &s.refreshing,
            Screen::Report => &s.report,
            Screen::Stopped => &s.stopped,
            Screen::Failed => &s.failed,
        }
    }

    /// An action's label; the id itself if the catalog has none (validation prevents it).
    #[must_use]
    pub fn label<'a>(&'a self, id: &'a str) -> &'a str {
        self.actions.get(id).map_or(id, |action| action.label.as_str())
    }

    fn validate(&self) -> Result<(), CatalogError> {
        let invalid = |message: String| Err(CatalogError::Invalid(message));
        let names: Vec<&str> = self.actions.keys().map(String::as_str).collect();
        let mut expected = ACTION_IDS.to_vec();
        expected.sort_unstable();
        if names != expected {
            return invalid(format!("actions must be exactly {}", ACTION_IDS.join(", ")));
        }
        let colors = &crate::ui::design::design().colors;
        for (key, entry_type) in &self.types {
            if !crate::assets::has_icon(&entry_type.icon) {
                return invalid(format!("types.{key}.icon: no icon {}", entry_type.icon));
            }
            if colors.named(&entry_type.tint).is_none() {
                return invalid(format!("types.{key}.tint: {} is not a colour token", entry_type.tint));
            }
            for action in std::iter::once(&entry_type.primary).chain(&entry_type.secondary) {
                if !ENTRY_ACTIONS.contains(&action.as_str()) {
                    return invalid(format!("types.{key}: {action} is not open or copy"));
                }
            }
        }
        let mut bound: Vec<(String, &str)> = Vec::new();
        for (id, action) in &self.actions {
            let Some(keys) = &action.keys else { continue };
            let Some(normal) = normalize(keys) else {
                return invalid(format!("actions.{id}.keys: {keys} is not a key"));
            };
            if SCREEN_KEYS.iter().any(|screen_key| normalize(screen_key).as_deref() == Some(normal.as_str())) {
                return invalid(format!("actions.{id}.keys: {keys} belongs to the screens"));
            }
            if let Some((_, other)) = bound.iter().find(|(k, _)| *k == normal) {
                return invalid(format!("actions.{id}.keys: {keys} is already {other}'s"));
            }
            bound.push((normal, id));
        }
        for (screen, entry_allowed) in [
            ("list", true),
            ("detail", true),
            ("empty", false),
            ("no-index", false),
            ("filter", false),
            ("links", false),
            ("refreshing", false),
            ("report", false),
            ("stopped", false),
            ("failed", false),
        ] {
            let keys = match screen {
                "list" => &self.screens.list,
                "detail" => &self.screens.detail,
                "empty" => &self.screens.empty,
                "no-index" => &self.screens.no_index,
                "filter" => &self.screens.filter,
                "links" => &self.screens.links,
                "refreshing" => &self.screens.refreshing,
                "report" => &self.screens.report,
                "stopped" => &self.screens.stopped,
                _ => &self.screens.failed,
            };
            for (key, value) in keys {
                if !SCREEN_KEYS.contains(&key.as_str()) {
                    return invalid(format!("screens.{screen}: {key} is not one of {}", SCREEN_KEYS.join(", ")));
                }
                let entry = value == ENTRY_PRIMARY || value == ENTRY_SECONDARY;
                if (entry && !entry_allowed) || (!entry && !self.actions.contains_key(value)) {
                    return invalid(format!("screens.{screen}.{key}: {value} is not an action here"));
                }
            }
        }
        for id in self.panel.root.iter().chain(&self.panel.more_copy).chain(&self.panel.more_admin) {
            if !self.actions.contains_key(id) {
                return invalid(format!("panel: {id} is not an action"));
            }
        }
        Ok(())
    }
}

/// `keys` as a canonical keystroke (modifiers in a fixed order), or `None` if it does not parse
/// or is more than one keystroke.
#[must_use]
pub fn normalize(keys: &str) -> Option<String> {
    if keys.contains(char::is_whitespace) {
        return None;
    }
    let keystroke = Keystroke::parse(keys).ok()?;
    if keystroke.key.is_empty() {
        return None;
    }
    let m = keystroke.modifiers;
    Some(format!(
        "{}{}{}{}{}",
        if m.control { "ctrl-" } else { "" },
        if m.alt { "alt-" } else { "" },
        if m.shift { "shift-" } else { "" },
        if m.platform { "cmd-" } else { "" },
        keystroke.key
    ))
}

/// `template` with each `{name}` replaced by its value.
#[must_use]
pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
    values
        .iter()
        .fold(template.to_owned(), |text, (name, value)| text.replace(&format!("{{{name}}}"), value))
}
```

- [ ] **Step 6: Run the tests and see them pass.**

Run: `cargo nextest run -p kb-app catalog`
Expected: PASS, 3 tests. If `Keystroke::parse("cmd-")` succeeds with an empty key, the `keystroke.key.is_empty()`
check rejects it; if it parses into something else, the case fails — then reject any `keys` ending in `-` before
parsing (`if keys.ends_with('-') { return None; }`) and note it under "Deviation:".

- [ ] **Step 7: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/assets/catalog.toml crates/kb-app/src/catalog.rs crates/kb-app/src/lib.rs crates/kb-app/src/assets.rs
git commit -m "feat(app): Add the catalog of entry types, actions, keys and words" \
  -m "Validated: icons exist, tints are tokens, keys parse and bind once (D18)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: config.toml carries the catalog's overrides

**Files:** modify `crates/kb-app/src/config.rs`.

- [ ] **Step 1: Write the failing tests.** In `config.rs`'s `tests` module, replace `rejects_unknown_keys` with:

```rust
    #[test]
    fn ignores_unknown_settings_with_a_notice() {
        let home = home_with(Some("source_path = \"x\"\n"));
        let config = Config::load(home.path(), Path::new("/data")).unwrap();
        assert_eq!(config.notices().len(), 1);
        assert!(config.notices()[0].contains("source_path"), "{:?}", config.notices());
    }

    #[test]
    fn keeps_the_catalog_tables_as_overrides() {
        let home = home_with(Some(
            "sources_path = \"~/k\"\n[actions.copy]\nlabel = \"Yank\"\n[messages]\nready = \"Idle\"\n",
        ));
        let config = Config::load(home.path(), Path::new("/data")).unwrap();
        assert_eq!(config.sources_dir(), home.path().join("k"));
        assert_eq!(config.overrides()["actions"]["copy"]["label"].as_str(), Some("Yank"));
        assert_eq!(config.overrides()["messages"]["ready"].as_str(), Some("Idle"));
        assert!(config.notices().is_empty());
    }

    #[test]
    fn rejects_a_file_that_is_not_toml() {
        let home = home_with(Some("sources_path = \n"));
        assert!(matches!(
            Config::load(home.path(), Path::new("/data")),
            Err(ConfigError::Parse { .. })
        ));
    }
```

Run: `cargo nextest run -p kb-app config`
Expected: FAIL to compile: no method `notices` / `overrides`.

- [ ] **Step 2: Split the file.** In `config.rs`:

Replace the `Preferences` struct with:

```rust
/// The location settings; the catalog's tables are kept apart as overrides.
#[derive(Debug, Default, Deserialize)]
struct Preferences {
    sources_path: Option<String>,
    tag_images_path: Option<String>,
}

/// The top-level tables of config.toml that override the catalog (`crate::catalog`).
const CATALOG_TABLES: [&str; 5] = ["types", "actions", "screens", "panel", "messages"];
```

Replace the `Config` struct's derive and fields with:

```rust
/// The resolved locations, the catalog overrides and what was ignored.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    sources_dir: PathBuf,
    tag_images: PathBuf,
    database: PathBuf,
    preferences: PathBuf,
    overrides: toml::Table,
    notices: Vec<String>,
}
```

Replace `load`'s body, `defaults` and `from_preferences` with:

```rust
    pub fn load(home: &Path, data_dir: &Path) -> Result<Self, ConfigError> {
        let preferences = home.join(PREFERENCES);
        let table = match fs::read_to_string(&preferences) {
            Ok(text) => toml::from_str::<toml::Table>(&text).map_err(|source| ConfigError::Parse {
                path: preferences.clone(),
                source: Box::new(source),
            })?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => toml::Table::new(),
            Err(source) => {
                return Err(ConfigError::Read {
                    path: preferences,
                    source,
                });
            }
        };
        let (mut locations, mut overrides, mut notices) = (toml::Table::new(), toml::Table::new(), Vec::new());
        for (key, value) in table {
            match key.as_str() {
                "sources_path" | "tag_images_path" => {
                    locations.insert(key, value);
                }
                k if CATALOG_TABLES.contains(&k) => {
                    overrides.insert(key, value);
                }
                _ => notices.push(format!("Ignored the unknown setting `{key}` in {}", preferences.display())),
            }
        }
        let parsed: Preferences = toml::Value::Table(locations)
            .try_into()
            .map_err(|source| ConfigError::Parse {
                path: preferences.clone(),
                source: Box::new(source),
            })?;
        Ok(Self::from_preferences(parsed, overrides, notices, home, data_dir))
    }

    /// The locations without preferences: the Raycast sources directory and the app's index.
    #[must_use]
    pub fn defaults(home: &Path, data_dir: &Path) -> Self {
        Self::from_preferences(Preferences::default(), toml::Table::new(), Vec::new(), home, data_dir)
    }

    fn from_preferences(
        parsed: Preferences,
        overrides: toml::Table,
        notices: Vec<String>,
        home: &Path,
        data_dir: &Path,
    ) -> Self {
        let sources_dir = parsed
            .sources_path
            .map_or_else(|| home.join(DEFAULT_SOURCES), |path| expand_home(&path, home));
        let tag_images = parsed
            .tag_images_path
            .map_or_else(|| home.join(DEFAULT_TAG_IMAGES), |path| expand_home(&path, home));
        Self {
            sources_dir,
            tag_images,
            database: data_dir.join("kb-native").join("kb.db"),
            preferences: home.join(PREFERENCES),
            overrides,
            notices,
        }
    }
```

Add these accessors after `preferences()`:

```rust
    /// The catalog tables of config.toml, applied by `Catalog::load`.
    #[must_use]
    pub fn overrides(&self) -> &toml::Table {
        &self.overrides
    }

    /// What config.toml held that kb ignored, to show as notices.
    #[must_use]
    pub fn notices(&self) -> &[String] {
        &self.notices
    }
```

Update the module comment's example (lines 3–8) to:

```rust
//! The preferences are one optional TOML file, `~/.config/kb/config.toml`: the two locations, and
//! any key of the catalog (`assets/catalog.toml`) to override:
//!
//! ```toml
//! sources_path = "~/Work/knowledge"
//! tag_images_path = "~/Pictures/kb-tags"
//!
//! [actions.copy]
//! label = "Yank"
//! ```
```

- [ ] **Step 3: Run the tests and see them pass.**

Run: `cargo nextest run -p kb-app config`
Expected: PASS. (`toml::de::Error` from `try_into` is the same type `ConfigError::Parse` boxes; if the compiler says
otherwise, map it with `toml::de::Error::custom(error.to_string())` and note it under "Deviation:".)

- [ ] **Step 4: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/src/config.rs
git commit -m "feat(app): Read catalog overrides from config.toml" \
  -m "Unknown settings become a notice instead of failing the whole file." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Overrides applied key by key

**Files:** modify `crates/kb-app/src/catalog.rs`.

- [ ] **Step 1: Write the failing tests.** Append to `catalog.rs`'s `tests` module:

```rust
    fn overrides(text: &str) -> toml::Table {
        toml::from_str(text).unwrap()
    }

    #[test]
    fn overrides_a_label_a_key_and_a_primary_action() {
        let (catalog, notices) = Catalog::load(&overrides(
            "[actions.copy]\nlabel = \"Yank\"\n[actions.resize]\nkeys = \"cmd-shift-m\"\n[types.commands]\nprimary = \"open\"\n",
        ));
        assert!(notices.is_empty(), "{notices:?}");
        assert_eq!(catalog.actions["copy"].label, "Yank");
        assert_eq!(catalog.actions["resize"].keys.as_deref(), Some("cmd-shift-m"));
        assert_eq!(catalog.types["commands"].primary, "open");
    }

    #[test]
    fn falls_back_key_by_key_with_a_notice() {
        let (catalog, notices) = Catalog::load(&overrides(
            "[actions.copy]\nlabel = \"Yank\"\n[actions.resize]\nkeys = \"cmd-k\"\n[types.tasks]\ntint = \"red\"\n",
        ));
        assert_eq!(catalog.actions["copy"].label, "Yank", "the good key is kept");
        assert_eq!(catalog.actions["resize"].keys.as_deref(), Some("cmd-shift-f"), "the bad key falls back");
        assert_eq!(catalog.types["tasks"].tint, "pink", "colours are not overridable");
        assert_eq!(notices.len(), 2, "{notices:?}");
        assert!(notices[0].contains("actions.resize.keys"), "{notices:?}");
        assert!(notices[1].contains("types.tasks.tint"), "{notices:?}");
    }
```

Run: `cargo nextest run -p kb-app catalog`
Expected: FAIL to compile: no function `load`.

- [ ] **Step 2: Write `load`.** Add to `impl Catalog`:

```rust
    /// The shipped catalog with `overrides` (config.toml's catalog tables) applied one leaf at a
    /// time: a leaf is kept only if the whole catalog still validates; a rejected leaf becomes a
    /// notice and keeps its default (spec D18). Colours (`types.*.tint`) are not overridable.
    #[must_use]
    pub fn load(overrides: &toml::Table) -> (Self, Vec<String>) {
        let defaults = Self::default();
        #[expect(clippy::expect_used, reason = "the compiled-in catalog is checked by a unit test")]
        let mut merged: toml::Table = toml::from_str(CATALOG).expect("catalog.toml is TOML");
        let mut notices = Vec::new();
        for (path, value) in leaves(overrides) {
            let name = path.join(".");
            if path.first().map(String::as_str) == Some("types") && path.last().map(String::as_str) == Some("tint") {
                notices.push(format!("Ignored {name} in config.toml: colours live in DESIGN.md"));
                continue;
            }
            let mut candidate = merged.clone();
            set_leaf(&mut candidate, &path, value);
            match Self::from_table(candidate.clone()) {
                Ok(_) => merged = candidate,
                Err(error) => notices.push(format!("Ignored {name} in config.toml: {error}")),
            }
        }
        // Every kept leaf left the catalog valid, so this cannot fail; the defaults are the fallback.
        (Self::from_table(merged).unwrap_or(defaults), notices)
    }
```

and these free functions after `fill`:

```rust
/// Every leaf of `table` (a value that is not a table) with its path, in the file's order.
fn leaves(table: &toml::Table) -> Vec<(Vec<String>, toml::Value)> {
    let mut out = Vec::new();
    for (key, value) in table {
        match value {
            toml::Value::Table(inner) => {
                for (mut path, leaf) in leaves(inner) {
                    path.insert(0, key.clone());
                    out.push((path, leaf));
                }
            }
            leaf => out.push((vec![key.clone()], leaf.clone())),
        }
    }
    out
}

/// Sets `path` in `table` to `value`, creating the tables on the way.
fn set_leaf(table: &mut toml::Table, path: &[String], value: toml::Value) {
    let Some((last, parents)) = path.split_last() else { return };
    let mut node = table;
    for key in parents {
        let next = node
            .entry(key.clone())
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        if !next.is_table() {
            *next = toml::Value::Table(toml::Table::new());
        }
        let toml::Value::Table(inner) = next else { return };
        node = inner;
    }
    node.insert(last.clone(), value);
}
```

(`toml::Table` is `toml::map::Map<String, Value>`; with the `preserve_order` feature off, keys iterate sorted: the
test's notice order — `actions.resize.keys` before `types.tasks.tint` — holds either way.)

- [ ] **Step 3: Run the tests and see them pass.**

Run: `cargo nextest run -p kb-app catalog`
Expected: PASS, 5 tests.

- [ ] **Step 4: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/src/catalog.rs
git commit -m "feat(app): Apply config.toml overrides to the catalog key by key" \
  -m "A rejected key keeps its default and becomes a notice; colours are not overridable." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Key bindings from the catalog; the catalog in the panel

**Files:** modify `crates/kb-app/src/keys.rs`, `crates/kb-app/src/panel.rs`, `crates/kb-app/src/search_view.rs`,
`crates/kb-app/tests/ui.rs`.

- [ ] **Step 1: Write the failing tests.** In `keys.rs`'s `tests` module, replace
      `binds_every_shortcut_in_the_search_context` with:

```rust
    use crate::catalog::Catalog;

    #[test]
    fn binds_the_catalog_keys_and_the_fixed_ones() {
        let bindings = key_bindings(&Catalog::default());
        // 7 keyed catalog actions + escape, cmd-q, right, left, enter (detail), down, up, cmd-down, cmd-up.
        assert_eq!(bindings.len(), 16);
    }

    #[test]
    fn follows_an_overridden_key() {
        let (catalog, _) =
            Catalog::load(&toml::from_str("[actions.resize]\nkeys = \"cmd-shift-m\"\n").unwrap());
        let keys: Vec<String> = key_bindings(&catalog)
            .iter()
            .flat_map(|binding| binding.keystrokes().iter().map(|k| k.as_keystroke().unparse()))
            .collect();
        assert!(keys.iter().any(|k| k == "cmd-shift-m"), "{keys:?}");
        assert!(!keys.iter().any(|k| k == "cmd-shift-f"), "{keys:?}");
    }
```

Run: `cargo nextest run -p kb-app keys`
Expected: FAIL to compile: `key_bindings` takes no argument.

- [ ] **Step 2: Bind from the catalog.** In `keys.rs`:

  - In the `actions!` list, remove `CopyUrl`'s neighbour `ToggleDetails` and its doc comment (⌘D is dropped: D9,
    `jm7r`), and add, after `OpenSourceFile`:

```rust
        /// Copy the selected entry's source file path, absolute.
        CopyPath,
```

  - Replace `key_bindings` with:

```rust
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
                _ => return None,
            })
        })
        .collect();
    bindings.extend([
        // Enter runs the primary action only on the detail screen. In the list, the search field
        // propagates Enter after reporting it, so a panel-wide binding would also fire there.
        KeyBinding::new("enter", RunPrimary, Some(DETAIL_CONTEXT)),
        KeyBinding::new("escape", Back, context),
        KeyBinding::new("cmd-q", Quit, context),
        // → and ← reach these only where no text field takes them first: the detail screen. In
        // the list the query field binds them deeper, so `SearchView` captures them there and
        // acts only with the caret at the query's end (`caret_at_end`).
        KeyBinding::new("right", ShowDetails, context),
        KeyBinding::new("left", HideDetails, context),
        // The detail screen has no text field: the arrows scroll its notes, ⌘ a page at a time.
        KeyBinding::new("down", ScrollNotesDown, Some(DETAIL_CONTEXT)),
        KeyBinding::new("up", ScrollNotesUp, Some(DETAIL_CONTEXT)),
        KeyBinding::new("cmd-down", PageNotesDown, Some(DETAIL_CONTEXT)),
        KeyBinding::new("cmd-up", PageNotesUp, Some(DETAIL_CONTEXT)),
    ]);
    bindings
}
```

  - Add `use crate::catalog::Catalog;` to the imports, and update the module comment's first line to
    `//! The commands as typed Actions, and their key bindings: the catalog's keys, plus Esc, ⌘Q and the arrows.`
  - The list `CopyUrl, … cmd-shift-c` binding is gone with the old list; `CopyUrl` stays an Action (the palette's
    "Copy URL" row still dispatches it) until 8e removes it.

- [ ] **Step 3: Remove ⌘D's handler and add Copy Path's.** In `search_view.rs`:
  - delete the method `toggle_details` and its `.on_action(cx.listener(Self::toggle_details))` line, and remove
    `ToggleDetails` from the `use crate::keys::{…}` list; add `CopyPath` to it;
  - add this method after `copy_url`:

```rust
    fn copy_path(&mut self, _: &CopyPath, _: &mut Window, cx: &mut Context<Self>) {
        let path = self
            .list
            .read(cx)
            .delegate()
            .selected_entry()
            .map(|e| e.source_file.clone());
        if let Some(path) = path {
            cx.write_to_clipboard(ClipboardItem::new_string(path));
        }
    }
```

  - add `.on_action(cx.listener(Self::copy_path))` after `.on_action(cx.listener(Self::copy_url))` in `render`.

In `crates/kb-app/tests/ui.rs`, the tests `opens_on_the_list_alone_and_toggles_the_split_pane_with_cmd_d` and
`shows_titles_alone_beside_the_split_pane` press `cmd-d`; change each `press(cx, window, "cmd-d")` to
`press(cx, window, "right")` for showing and `press(cx, window, "left")` for hiding (the second `cmd-d` in
`shows_titles_alone_beside_the_split_pane` becomes `left`), and rename the first test to
`opens_on_the_list_alone_and_shows_the_split_pane_with_right`.

In `palette.rs`, the rows built with `ShowDetails`/`HideDetails` stay; nothing there uses `ToggleDetails`.

- [ ] **Step 4: Hold the catalog in the view.** In `search_view.rs`:
  - add `use std::rc::Rc;` and `use crate::catalog::Catalog;`;
  - add the field `catalog: Rc<Catalog>,` after `config: Config,` in `SearchView`;
  - change `pub fn new(config: Config, window: &mut Window, cx: &mut Context<Self>) -> Self` to
    `pub fn new(config: Config, catalog: Catalog, window: &mut Window, cx: &mut Context<Self>) -> Self`, and add
    `catalog: Rc::new(catalog),` in the returned `Self { … }` after `config,`;
  - change the placeholder `"Search Knowledge Base…"` to `catalog.messages.search_placeholder.clone()` (the line
    builds `query_input`; it must come before `catalog` is moved into `Rc::new`, so build the placeholder first:
    `let placeholder = catalog.messages.search_placeholder.clone();` at the top of `new`, then
    `.placeholder(placeholder)`).

- [ ] **Step 5: Build the catalog in the panel.** In `panel.rs`, replace the first lines of `open`:

```rust
pub fn open(config: Config, mut notices: Vec<String>, cx: &mut App) -> Result<(), PanelError> {
    cx.bind_keys(key_bindings());
```

with:

```rust
pub fn open(config: Config, mut notices: Vec<String>, cx: &mut App) -> Result<(), PanelError> {
    let (catalog, catalog_notices) = Catalog::load(config.overrides());
    notices.extend(config.notices().iter().cloned());
    notices.extend(catalog_notices);
    cx.bind_keys(key_bindings(&catalog));
```

and `cx.new(|cx| SearchView::new(config, window, cx))` with `cx.new(|cx| SearchView::new(config, catalog, window, cx))`;
add `use crate::catalog::Catalog;`.

- [ ] **Step 6: Build the catalog in the UI tests.** In `crates/kb-app/tests/ui.rs`:
  - add `use kb_app::catalog::Catalog;`;
  - in `open`, change `cx.update(|cx| cx.bind_keys(key_bindings()));` to
    `cx.update(|cx| cx.bind_keys(key_bindings(&Catalog::default())));` and
    `cx.new(|cx| SearchView::new(config, window, cx))` to `cx.new(|cx| SearchView::new(config, Catalog::default(), window, cx))`.

- [ ] **Step 7: Run every kb-app test.**

Run: `cargo nextest run -p kb-app`
Expected: PASS. (`KeybindingKeystroke::as_keystroke` is what `footer.rs` already uses; `Keystroke::unparse` gives
`cmd-shift-m`. If `unparse` is missing in GPUI 0.3.8, compare `format!("{}", k.as_keystroke())` and note it under
"Deviation:".)

- [ ] **Step 8: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/src/keys.rs crates/kb-app/src/panel.rs crates/kb-app/src/search_view.rs crates/kb-app/tests/ui.rs
git commit -m "feat(app): Bind keys from the catalog; add Copy Path (⌥⌘C)" \
  -m "⌘D is dropped: → and ← show and hide the split (D9)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Configure writes the defaults as comments

**Files:** modify `crates/kb-app/src/catalog.rs`, `crates/kb-app/src/search_view.rs`.

- [ ] **Step 1: Write the failing test.** Append to `catalog.rs`'s `tests` module:

```rust
    #[test]
    fn writes_a_preferences_file_with_every_default_commented_out() {
        let text = preferences_template(Path::new("/home/me/knowledge"));
        assert!(text.starts_with("# kb preferences"), "{text}");
        assert!(text.contains("\nsources_path = \"/home/me/knowledge\"\n"), "{text}");
        assert!(text.contains("\n# [types.bookmarks]\n"), "{text}");
        assert!(text.contains("\n# label = \"Bookmarks\"\n"), "{text}");
        let active: toml::Table = toml::from_str(&text).unwrap();
        assert_eq!(active.keys().collect::<Vec<_>>(), ["sources_path"], "only the location is live");
    }
```

and add `use std::path::Path;` at the top of the `tests` module.

Run: `cargo nextest run -p kb-app catalog`
Expected: FAIL to compile: no function `preferences_template`.

- [ ] **Step 2: Write it.** Add to `catalog.rs`, after `fill`:

```rust
/// A new config.toml: the sources path live, then every catalog default as a comment to uncomment
/// and change (Configure, ⇧⌘,).
#[must_use]
pub fn preferences_template(sources_dir: &std::path::Path) -> String {
    let mut text = format!(
        "# kb preferences. Uncomment a line to change it; see crates/kb-app/assets/catalog.toml.\n\nsources_path = \"{}\"\n\n",
        sources_dir.display()
    );
    for line in CATALOG.lines() {
        if line.is_empty() {
            text.push('\n');
        } else if line.starts_with('#') {
            text.push_str(line);
            text.push('\n');
        } else {
            text.push_str("# ");
            text.push_str(line);
            text.push('\n');
        }
    }
    text
}
```

- [ ] **Step 3: Use it.** In `search_view.rs`, `open_preferences`, replace the `let template = format!(…);` statement
      with `let template = crate::catalog::preferences_template(self.config.sources_dir());`.

- [ ] **Step 4: Run the tests and see them pass.**

Run: `cargo nextest run -p kb-app`
Expected: PASS.

- [ ] **Step 5: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/src/catalog.rs crates/kb-app/src/search_view.rs
git commit -m "feat(app): Write config.toml with the catalog defaults as comments" \
  -m "Configure (⇧⌘,) creates the file when it does not exist (D18)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Done when

- `cargo nextest run -p kb-app` and `mise run gate` pass.
- A `config.toml` with `[actions.copy] label = "Yank"` is accepted silently; one with `[actions.resize] keys = "cmd-k"`
  starts with a notice and keeps ⇧⌘F.
- `todo`: set `RKHF` (catalog) done with the comment `"2026-10-13: Plan 8b"`.
