//! Which sources a sync refreshes, and the source each index row came from (`entries.source`).

use std::str::FromStr;

/// Where an index row came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// A YAML source file under the sources directory.
    Yaml,
    /// A record of the Skills command.
    Skills,
}

impl SourceKind {
    /// Its `entries.source` value.
    #[must_use]
    pub const fn column(self) -> &'static str {
        match self {
            Self::Yaml => "yaml",
            Self::Skills => "skills",
        }
    }
}

/// The sources a sync refreshes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Selection {
    /// YAML, then Skills: Refresh, and `kb --sync` without `--source`.
    #[default]
    All,
    /// YAML only: Refresh YAML.
    Yaml,
    /// Skills only: Refresh Skills.
    Skills,
}

impl Selection {
    /// Whether it refreshes `kind`.
    #[must_use]
    pub const fn includes(self, kind: SourceKind) -> bool {
        matches!(
            (self, kind),
            (Self::All, _) | (Self::Yaml, SourceKind::Yaml) | (Self::Skills, SourceKind::Skills)
        )
    }
}

/// A `--source` value other than `all`, `yaml` or `skills`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("`{0}` is not all, yaml or skills")]
pub struct UnknownSelection(String);

impl FromStr for Selection {
    type Err = UnknownSelection;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "all" => Ok(Self::All),
            "yaml" => Ok(Self::Yaml),
            "skills" => Ok(Self::Skills),
            other => Err(UnknownSelection(other.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_three_selections_and_rejects_others() {
        assert_eq!("all".parse(), Ok(Selection::All));
        assert_eq!("yaml".parse(), Ok(Selection::Yaml));
        assert_eq!("skills".parse(), Ok(Selection::Skills));
        assert_eq!(
            "both".parse::<Selection>().unwrap_err().to_string(),
            "`both` is not all, yaml or skills"
        );
    }

    #[test]
    fn includes_its_sources() {
        use SourceKind::{Skills, Yaml};
        assert!(Selection::All.includes(Yaml) && Selection::All.includes(Skills));
        assert!(Selection::Yaml.includes(Yaml) && !Selection::Yaml.includes(Skills));
        assert!(Selection::Skills.includes(Skills) && !Selection::Skills.includes(Yaml));
    }
}
