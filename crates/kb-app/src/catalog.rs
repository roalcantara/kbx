//! The catalog (spec D18): what kb offers — entry types, actions, each screen's keys, the action
//! panel's rows and the words it shows — compiled in from `assets/catalog.toml`, overridden key by
//! key from `~/.config/kb/config.toml`.

use std::collections::BTreeMap;

use gpui_kit::Keystroke;
use serde::Deserialize;

/// The catalog as shipped.
const CATALOG: &str = include_str!("../assets/catalog.toml");

/// The actions kb knows how to run; a catalog names exactly these.
pub const ACTION_IDS: [&str; 27] = [
    "details",
    "open",
    "copy",
    "resize",
    "copy-description",
    "copy-notes",
    "copy-path",
    "copy-location",
    "copy-error",
    "reveal",
    "edit",
    "refresh",
    "refresh-yaml",
    "refresh-skills",
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
    "run",
    "close",
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
    /// The `YouTube` thumbnail, else the URL as a link line.
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
    /// What the entry view's hero shows; none when absent.
    #[serde(default)]
    pub hero: Option<Hero>,
    /// Titles in `JetBrains` Mono.
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
    /// ⌘K is open (the root or a submenu other than Links…).
    Actions,
    /// The Links… submenu is open.
    Links,
    /// A refresh is running.
    Refreshing,
    /// A refresh finished: its report.
    Report,
    /// A report row, full screen.
    ReportDetail,
    /// A refresh was stopped.
    Stopped,
    /// A refresh failed.
    Failed,
}

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
    pub actions: ScreenKeys,
    pub links: ScreenKeys,
    pub refreshing: ScreenKeys,
    pub report: ScreenKeys,
    pub report_detail: ScreenKeys,
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
    /// The Refresh More… row, after Refresh in More….
    pub refresh_more: String,
    /// The Theme… row, after Refresh More… in More….
    pub theme: String,
    /// The search field's placeholder.
    pub search: String,
    /// More…'s first section.
    pub copy_section: String,
    /// More…'s second section.
    pub admin_section: String,
    /// The filter's types section.
    pub types_section: String,
    /// The filter's tags section.
    pub tags_section: String,
    /// The root's rows after the screen's keys.
    pub root: Vec<String>,
    /// More…'s copy rows.
    pub more_copy: Vec<String>,
    /// More…'s administration rows.
    pub more_admin: Vec<String>,
    /// Refresh More…'s rows.
    pub more_refresh: Vec<String>,
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
    pub no_entries_filtered: String,
    pub no_entries_title: String,
    pub no_entries: String,
    pub list_or: String,
    pub no_index_title: String,
    pub no_index: String,
    pub refreshing_title: String,
    pub refreshing_progress: String,
    pub running_skills: String,
    pub report_summary: String,
    pub report_failed: String,
    pub report_failed_chip: String,
    pub report_skipped: String,
    pub report_indexed: String,
    pub report_skipped_entries: String,
    pub report_empty_file: String,
    pub report_yaml_section: String,
    pub report_skills_section: String,
    pub report_source_failed: String,
    pub report_not_configured: String,
    pub report_warning_chip: String,
    pub report_not_configured_chip: String,
    pub report_pane_location: String,
    pub report_pane_file: String,
    pub report_pane_entries: String,
    pub report_pane_index: String,
    pub report_pane_kept: String,
    pub report_pane_skills_kept: String,
    pub report_pane_exit: String,
    pub report_pane_stderr: String,
    pub report_pane_nothing: String,
    pub report_pane_line: String,
    pub report_pane_skipped: String,
    pub report_pane_one_skipped: String,
    pub report_pane_skipped_key: String,
    pub report_pane_skipped_chip: String,
    pub stopped_title: String,
    pub stopped: String,
    pub failed_title: String,
    pub failed: String,
    pub filters: String,
    pub one_filter: String,
    pub links_label: String,
    pub more_origin: String,
    pub links_origin: String,
    pub refresh_origin: String,
    pub theme_origin: String,
    pub theme_chosen: String,
    pub theme_weak: String,
    pub open_missing: String,
    pub editor_failed: String,
    /// The list's name after ‹ when it has no query.
    pub back_search: String,
    /// A refresh's name after ‹.
    pub back_refresh: String,
    pub config_create_failed: String,
    pub theme_save_failed: String,
    pub hotkey_failed: String,
    pub themes_seed_failed: String,
    pub config_colors_ignored: String,
    pub config_unknown_ignored: String,
    pub config_editor_ignored: String,
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

/// The shipped catalog's words, parsed once: for the notices raised before config.toml's overrides are read
/// (close-out D8). config.toml cannot be trusted to word its own errors.
#[must_use]
pub fn shipped_messages() -> &'static Messages {
    static SHIPPED: std::sync::OnceLock<Messages> = std::sync::OnceLock::new();
    SHIPPED.get_or_init(|| Catalog::default().messages)
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
            Screen::Actions => &s.actions,
            Screen::Links => &s.links,
            Screen::Refreshing => &s.refreshing,
            Screen::Report => &s.report,
            Screen::ReportDetail => &s.report_detail,
            Screen::Stopped => &s.stopped,
            Screen::Failed => &s.failed,
        }
    }

    /// An action's label; the id itself if the catalog has none (validation prevents it).
    #[must_use]
    pub fn label<'a>(&'a self, id: &'a str) -> &'a str {
        self.actions.get(id).map_or(id, |action| action.label.as_str())
    }

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

    /// The shipped catalog with `overrides` (config.toml's catalog tables) applied one leaf at a
    /// time: a leaf is kept only if the whole catalog still validates; a rejected leaf becomes a
    /// notice and keeps its default (spec D18). Colours (`types.*.tint`) are not overridable.
    ///
    /// # Panics
    ///
    /// The compiled-in catalog is not TOML; a unit test proves it is.
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
            if SCREEN_KEYS
                .iter()
                .any(|screen_key| normalize(screen_key).as_deref() == Some(normal.as_str()))
            {
                return invalid(format!("actions.{id}.keys: {keys} belongs to the screens"));
            }
            if let Some((_, other)) = bound.iter().find(|(k, _)| *k == normal) {
                return invalid(format!("actions.{id}.keys: {keys} is already {other}'s"));
            }
            bound.push((normal, id));
        }
        for (screen, entry_allowed, keys) in [
            ("list", true, &self.screens.list),
            ("detail", true, &self.screens.detail),
            ("empty", false, &self.screens.empty),
            ("no-index", false, &self.screens.no_index),
            ("filter", false, &self.screens.filter),
            ("links", false, &self.screens.links),
            ("refreshing", false, &self.screens.refreshing),
            ("report", false, &self.screens.report),
            ("report-detail", false, &self.screens.report_detail),
            ("stopped", false, &self.screens.stopped),
            ("failed", false, &self.screens.failed),
        ] {
            for (key, value) in keys {
                if !SCREEN_KEYS.contains(&key.as_str()) {
                    return invalid(format!(
                        "screens.{screen}: {key} is not one of {}",
                        SCREEN_KEYS.join(", ")
                    ));
                }
                let entry = value == ENTRY_PRIMARY || value == ENTRY_SECONDARY;
                if (entry && !entry_allowed) || (!entry && !self.actions.contains_key(value)) {
                    return invalid(format!("screens.{screen}.{key}: {value} is not an action here"));
                }
            }
        }
        for id in self
            .panel
            .root
            .iter()
            .chain(&self.panel.more_copy)
            .chain(&self.panel.more_admin)
            .chain(&self.panel.more_refresh)
        {
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
    values.iter().fold(template.to_owned(), |text, (name, value)| {
        text.replace(&format!("{{{name}}}"), value)
    })
}

/// A new config.toml: the sources path live, then every catalog default as a comment to uncomment
/// and change (Configure, ⇧⌘,).
#[must_use]
pub fn preferences_template(sources_dir: &std::path::Path) -> String {
    let mut text = format!(
        "# kb preferences. Uncomment a line to change it; see crates/kb-app/assets/catalog.toml.\n\nsources_path = \"{}\"\n\n",
        sources_dir.display()
    );
    text.push_str(
        "# Opens files at a line; unset, they open with their default app.\n# editor = \"zed {file}:{line}\"\n\n",
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
    let Some((last, parents)) = path.split_last() else {
        return;
    };
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn ships_a_valid_catalog() {
        let catalog = Catalog::default();
        assert_eq!(
            catalog.types.keys().map(String::as_str).collect::<Vec<_>>(),
            ["bookmarks", "cheats", "commands", "skills", "tasks"]
        );
        assert_eq!(catalog.types["bookmarks"].secondary.as_deref(), Some("copy"));
        assert_eq!(catalog.screens.list["shift-cmd-enter"], ENTRY_SECONDARY);
        assert_eq!(catalog.actions["resize"].keys.as_deref(), Some("cmd-shift-f"));
        assert_eq!(catalog.actions.len(), ACTION_IDS.len());
    }

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
            assert!(
                matches!(result, Err(CatalogError::Invalid(_))),
                "{path:?} = {value}: {result:?}"
            );
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
        assert_eq!(
            catalog.actions["resize"].keys.as_deref(),
            Some("cmd-shift-f"),
            "the bad key falls back"
        );
        assert_eq!(catalog.types["tasks"].tint, "pink", "colours are not overridable");
        assert_eq!(notices.len(), 2, "{notices:?}");
        assert!(notices[0].contains("actions.resize.keys"), "{notices:?}");
        assert!(notices[1].contains("types.tasks.tint"), "{notices:?}");
    }

    #[test]
    fn writes_a_preferences_file_with_every_default_commented_out() {
        let text = preferences_template(Path::new("/home/me/knowledge"));
        assert!(text.starts_with("# kb preferences"), "{text}");
        assert!(text.contains("\nsources_path = \"/home/me/knowledge\"\n"), "{text}");
        assert!(text.contains("\n# [types.bookmarks]\n"), "{text}");
        assert!(text.contains("\n# label = \"Bookmarks\"\n"), "{text}");
        let active: toml::Table = toml::from_str(&text).unwrap();
        assert_eq!(
            active.keys().collect::<Vec<_>>(),
            ["sources_path"],
            "only the location is live"
        );
        assert!(text.contains("\n# editor = \"zed {file}:{line}\"\n"), "{text}");
    }

    #[test]
    fn ships_skills_without_a_hero_and_the_scoped_refreshes() {
        let catalog = Catalog::default();
        let skills = catalog.entry_type("skills").unwrap();
        assert_eq!(
            (skills.label.as_str(), skills.icon.as_str(), skills.tint.as_str()),
            ("Skills", "notebook-text", "yellow")
        );
        assert_eq!(
            (skills.hero, skills.primary.as_str(), skills.secondary.as_deref()),
            (None, "copy", None)
        );
        assert_eq!(catalog.label("refresh-yaml"), "Refresh YAML");
        assert_eq!(catalog.label("refresh-skills"), "Refresh Skills");
        assert_eq!(catalog.panel.refresh_more, "Refresh More…");
        assert_eq!(catalog.panel.more_refresh, ["refresh-yaml", "refresh-skills"]);
    }

    #[test]
    fn ships_the_report_s_keys_and_actions() {
        let catalog = Catalog::default();
        let keys = |screen| {
            catalog
                .keys_for(screen, None)
                .into_iter()
                .map(|k| format!("{} {}", k.label, k.caps))
                .collect::<Vec<_>>()
        };
        assert_eq!(keys(Screen::Report), ["Details ↵", "Edit ⌘↵", "Copy Location ⇧⌘↵"]);
        assert_eq!(keys(Screen::ReportDetail), ["Edit ↵", "Copy Location ⌘↵"]);
        for (id, keys) in [
            ("copy-location", "cmd-shift-l"),
            ("copy-error", "cmd-shift-e"),
            ("reveal", "cmd-shift-r"),
        ] {
            assert_eq!(catalog.actions[id].keys.as_deref(), Some(keys), "{id}");
        }
        assert_eq!(catalog.label("reveal"), "Reveal in Finder");
    }
}
