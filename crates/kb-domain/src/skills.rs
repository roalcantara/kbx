//! The Skills source: the records `skills ls --global --agent universal --json` prints, and the
//! entries they become.
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// The arguments kb always gives the Skills command (spec D7).
pub const SKILLS_ARGS: [&str; 5] = ["ls", "--global", "--agent", "universal", "--json"];

/// The entry type of every Skills entry.
pub const SKILLS_TYPE: &str = "skills";

/// One record of the command's JSON; unknown fields are ignored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillRecord {
    /// The Skill's name: the entry's key, after a `/`.
    pub name: String,
    /// The Skill's directory, holding its `SKILL.md`.
    pub path: String,
    /// `global`.
    #[serde(default)]
    pub scope: Option<String>,
    /// The agents that load it.
    #[serde(default)]
    pub agents: Option<Vec<String>>,
    /// Where it was installed from (`owner/repo`).
    #[serde(default)]
    pub source: Option<String>,
    /// The repository it was installed from: the entry's link.
    #[serde(default)]
    pub source_url: Option<String>,
    /// `github`, …
    #[serde(default)]
    pub source_type: Option<String>,
}

/// The records of the command's stdout: one JSON array of records with unique, non-empty names
/// and non-empty paths.
///
/// # Errors
///
/// Why the output is not a valid Skills set; the whole source then fails (spec D5).
pub fn parse_skills(stdout: &str) -> Result<Vec<SkillRecord>, String> {
    let records: Vec<SkillRecord> =
        serde_json::from_str(stdout).map_err(|error| format!("the output is not Skills JSON: {error}"))?;
    let mut names = HashSet::new();
    for (index, record) in records.iter().enumerate() {
        let name = record.name.trim();
        if name.is_empty() {
            return Err(format!("record {} has no name", index + 1));
        }
        if record.path.trim().is_empty() {
            return Err(format!("{name} has no path"));
        }
        if !names.insert(name) {
            return Err(format!("{name} appears twice"));
        }
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_records_ignoring_unknown_fields_and_nulls() {
        let records = parse_skills(
            r#"[{"name":"tdd","path":"/s/tdd","scope":"global","agents":["Claude Code"],"source":null,
                "sourceUrl":"https://github.com/o/r.git","sourceType":"github","extra":1},
               {"name":"act","path":"/s/act","agents":null}]"#,
        )
        .unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].source_url.as_deref(), Some("https://github.com/o/r.git"));
        assert_eq!(records[1].agents, None);
    }

    #[test]
    fn rejects_output_that_is_not_one_json_array() {
        for stdout in ["", "{", "⚠ Skipped x\n[]", "[]\nwarning", r#"[{"path":"/s/a"}]"#] {
            assert!(parse_skills(stdout).is_err(), "{stdout:?}");
        }
    }

    #[test]
    fn rejects_empty_names_and_paths_and_duplicate_names() {
        assert_eq!(
            parse_skills(r#"[{"name":" ","path":"/s/a"}]"#).unwrap_err(),
            "record 1 has no name"
        );
        assert_eq!(
            parse_skills(r#"[{"name":"a","path":""}]"#).unwrap_err(),
            "a has no path"
        );
        assert_eq!(
            parse_skills(r#"[{"name":"a","path":"/s/a"},{"name":"a","path":"/t/a"}]"#).unwrap_err(),
            "a appears twice"
        );
    }
}
