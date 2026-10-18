//! Where the app reads its sources and keeps its index.
//!
//! The preferences are one optional TOML file, `~/.config/kb/config.toml`: the two locations, and
//! any key of the catalog (`assets/catalog.toml`) to override:
//!
//! ```toml
//! sources_path = "~/Work/knowledge"
//! tag_images_path = "~/Pictures/kb-tags"
//! skills_command = "skills"
//! ```
//!
//! `skills_command` names the Skills command (spec D7); unset, kb looks for `skills` (`crate::command_path`).
//!
//! Without it the sources are `~/.config/kodexb/sources`, as in the Raycast extension. The index
//! is the app's own file in its data directory; the Raycast index is never opened.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The sources directory when the preferences name none.
const DEFAULT_SOURCES: &str = ".config/kodexb/sources";
/// The tag-images folder when the preferences name none, relative to the home directory.
const DEFAULT_TAG_IMAGES: &str = ".config/kb/images";
/// The preferences file, relative to the home directory.
const PREFERENCES: &str = ".config/kb/config.toml";

/// Why the preferences could not be read.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The file exists but could not be read.
    #[error("cannot read {}: {source}", path.display())]
    Read {
        /// The preferences file.
        path: PathBuf,
        /// The I/O error.
        source: io::Error,
    },
    /// The file is not the expected TOML.
    #[error("{} is not valid: {source}", path.display())]
    Parse {
        /// The preferences file.
        path: PathBuf,
        /// The parser's error.
        source: Box<toml::de::Error>,
    },
}

/// The location settings; the catalog's tables are kept apart as overrides.
#[derive(Debug, Default, Deserialize)]
struct Preferences {
    sources_path: Option<String>,
    tag_images_path: Option<String>,
    skills_command: Option<String>,
}

/// The top-level tables of config.toml that override the catalog (`crate::catalog`).
const CATALOG_TABLES: [&str; 5] = ["types", "actions", "screens", "panel", "messages"];

/// The resolved locations, the catalog overrides and what was ignored.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    home: PathBuf,
    sources_dir: PathBuf,
    tag_images: PathBuf,
    skills_command: Option<String>,
    database: PathBuf,
    preferences: PathBuf,
    overrides: toml::Table,
    notices: Vec<String>,
}

impl Config {
    /// Resolves the locations under `home` and `data_dir`, reading the preferences if present.
    ///
    /// # Errors
    ///
    /// The preferences file exists but cannot be read or parsed.
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
                "sources_path" | "tag_images_path" | "skills_command" => {
                    locations.insert(key, value);
                }
                k if CATALOG_TABLES.contains(&k) => {
                    overrides.insert(key, value);
                }
                _ => notices.push(format!(
                    "Ignored the unknown setting `{key}` in {}",
                    preferences.display()
                )),
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
            home: home.to_path_buf(),
            sources_dir,
            tag_images,
            skills_command: parsed.skills_command,
            database: data_dir.join("kb-native").join("kb.db"),
            preferences: home.join(PREFERENCES),
            overrides,
            notices,
        }
    }

    /// The directory the source files are found under.
    #[must_use]
    pub fn sources_dir(&self) -> &Path {
        &self.sources_dir
    }

    /// The folder of tag images: `<tag>.svg` draws `#tag` (`mise run images` fills the default).
    #[must_use]
    pub fn tag_images(&self) -> &Path {
        &self.tag_images
    }

    /// The Skills command to run, found as [`crate::command_path::skills_command`] finds it;
    /// `None` when it is not configured.
    #[must_use]
    pub fn skills_command(&self) -> Option<PathBuf> {
        crate::command_path::skills_command(
            self.skills_command.as_deref(),
            std::env::var_os("PATH").as_deref(),
            &self.home,
        )
    }
    /// The index file.
    #[must_use]
    pub fn database(&self) -> &Path {
        &self.database
    }

    /// The preferences file, which may not exist yet.
    #[must_use]
    pub fn preferences(&self) -> &Path {
        &self.preferences
    }

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
}

/// `~` and `~/…` relative to `home`; any other path as written.
pub(crate) fn expand_home(path: &str, home: &Path) -> PathBuf {
    match path.strip_prefix('~') {
        Some("") => home.to_path_buf(),
        Some(rest) if rest.starts_with('/') => home.join(&rest[1..]),
        _ => PathBuf::from(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home_with(preferences: Option<&str>) -> tempfile::TempDir {
        let home = tempfile::tempdir().unwrap();
        if let Some(text) = preferences {
            let file = home.path().join(PREFERENCES);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, text).unwrap();
        }
        home
    }

    #[test]
    fn defaults_to_the_raycast_sources_directory() {
        let home = home_with(None);
        let config = Config::load(home.path(), Path::new("/data")).unwrap();
        assert_eq!(config.sources_dir(), home.path().join(".config/kodexb/sources"));
        assert_eq!(config.database(), Path::new("/data/kb-native/kb.db"));
    }

    #[test]
    fn reads_the_skills_command_and_expands_home() {
        let home = home_with(Some("skills_command = \"~/bin/skills\"\n"));
        let config = Config::load(home.path(), &home.path().join("data")).unwrap();
        assert_eq!(config.skills_command(), Some(home.path().join("bin/skills")));
        assert_eq!(config.notices(), [] as [String; 0]);
    }

    #[test]
    fn reads_the_sources_path_and_expands_home() {
        let home = home_with(Some("sources_path = \"~/Work/knowledge\"\n"));
        let config = Config::load(home.path(), Path::new("/data")).unwrap();
        assert_eq!(config.sources_dir(), home.path().join("Work/knowledge"));
        assert_eq!(expand_home("/abs", home.path()), PathBuf::from("/abs"));
        assert_eq!(expand_home("~", home.path()), home.path());
    }

    #[test]
    fn falls_back_to_the_defaults() {
        let home = home_with(Some("sources_path = \"~/elsewhere\"\n"));
        let config = Config::defaults(home.path(), Path::new("/data"));
        assert_eq!(config.sources_dir(), home.path().join(".config/kodexb/sources"));
        assert_eq!(config.preferences(), home.path().join(".config/kb/config.toml"));
    }

    #[test]
    fn reads_the_tag_images_path_and_defaults_beside_the_preferences() {
        let home = home_with(Some("tag_images_path = \"~/art\"\n"));
        let config = Config::load(home.path(), Path::new("/data")).unwrap();
        assert_eq!(config.tag_images(), home.path().join("art"));
        let config = Config::defaults(home.path(), Path::new("/data"));
        assert_eq!(config.tag_images(), home.path().join(".config/kb/images"));
    }

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
        assert!(config.notices().is_empty(), "{:?}", config.notices());
    }

    #[test]
    fn rejects_a_file_that_is_not_toml() {
        let home = home_with(Some("sources_path = \n"));
        assert!(matches!(
            Config::load(home.path(), Path::new("/data")),
            Err(ConfigError::Parse { .. })
        ));
    }
}
