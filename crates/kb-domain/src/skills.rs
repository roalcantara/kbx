//! The Skills source: the records `skills ls --global --agent universal --json` prints, and the
//! entries they become.
pub mod command;

use std::collections::HashSet;
use std::path::Path;

use saphyr::{ScalarOwned, YamlOwned};
use serde::{Deserialize, Serialize};

use crate::entry::{Entry, EntryLink, derive_entry_id, raw_string};
use crate::markdown::build_entry_markdown;
use crate::yaml;

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

/// The entry `record` becomes (spec, "Entry mapping"), and a warning when its `SKILL.md` cannot
/// be read or its front matter is not YAML. An unreadable `SKILL.md` leaves the notes empty.
#[must_use]
pub fn skill_entry(record: &SkillRecord) -> (Entry, Option<String>) {
    let name = record.name.trim();
    let key = format!("/{name}");
    let skill_md = Path::new(record.path.trim()).join("SKILL.md");
    let rel_path = format!("{name}/SKILL.md");
    let (description, readable, warning) = match std::fs::read_to_string(&skill_md) {
        Ok(content) => match front_matter_description(&content) {
            Ok(description) => (description, true, None),
            Err(error) => (None, true, Some(format!("{key}: SKILL.md front matter: {error}"))),
        },
        Err(error) => (
            None,
            false,
            Some(format!("{key}: cannot read {}: {error}", skill_md.display())),
        ),
    };
    let mut entry = Entry {
        id: derive_entry_id(&rel_path, SKILLS_TYPE, &key),
        title: key.clone(),
        description,
        entry_type: SKILLS_TYPE.to_owned(),
        tags: Vec::new(),
        notes: readable.then(|| YamlOwned::Value(ScalarOwned::String(format!("file://{}", skill_md.display())))),
        notes_markdown: String::new(),
        links: record
            .source_url
            .iter()
            .map(|url| url.trim())
            .filter(|url| !url.is_empty())
            .map(|url| EntryLink {
                title: None,
                url: url.to_owned(),
            })
            .collect(),
        source_file: skill_md.to_string_lossy().into_owned(),
        source_rel_path: rel_path,
        primary_content: key,
        task_status: None,
        task_priority: None,
        // JSON is YAML: `row_to_entry` reads it back as it reads any entry's map.
        raw_yaml: serde_json::to_string_pretty(record).unwrap_or_default(),
    };
    entry.notes_markdown = build_entry_markdown(&entry);
    (entry, warning)
}

/// The `description` of a Markdown file's leading YAML front matter; `Ok(None)` without front
/// matter or without a description.
fn front_matter_description(content: &str) -> Result<Option<String>, yaml::YamlError> {
    let Some(rest) = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
    else {
        return Ok(None);
    };
    let Some(end) = rest.find("\n---") else {
        return Ok(None);
    };
    Ok(yaml::load(&rest[..end])?.and_then(|front| raw_string(&front, "description")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn record(name: &str, path: &std::path::Path, url: Option<&str>) -> SkillRecord {
        SkillRecord {
            name: name.into(),
            path: path.to_string_lossy().into_owned(),
            scope: Some("global".into()),
            agents: None,
            source: None,
            source_url: url.map(str::to_owned),
            source_type: None,
        }
    }

    #[test]
    fn maps_a_record_to_a_skills_entry() {
        let dir = tempfile::tempdir().unwrap();
        let skill = dir.path().join("tdd");
        fs::create_dir(&skill).unwrap();
        fs::write(
            skill.join("SKILL.md"),
            "---\nname: tdd\ndescription: Red, green, refactor.\n---\n\n# Test-Driven Development\n",
        )
        .unwrap();
        let (entry, warning) = skill_entry(&record("tdd", &skill, Some("https://github.com/o/r.git")));
        assert_eq!(warning, None);
        assert_eq!(entry.id, "tdd/SKILL.md#skills#/tdd");
        assert_eq!((entry.title.as_str(), entry.primary_content.as_str()), ("/tdd", "/tdd"));
        assert_eq!(entry.entry_type, "skills");
        assert_eq!(entry.description.as_deref(), Some("Red, green, refactor."));
        assert_eq!(entry.links[0].url, "https://github.com/o/r.git");
        assert_eq!(entry.source_file, skill.join("SKILL.md").to_string_lossy());
        assert_eq!(entry.source_rel_path, "tdd/SKILL.md");
        assert_eq!(entry.tags, [] as [String; 0]);
        assert!(entry.notes_markdown.contains("# Test-Driven Development"));
        assert!(!entry.notes_markdown.contains("name: tdd"), "front matter stripped");
    }

    #[test]
    fn indexes_a_record_whose_skill_md_is_missing_with_a_warning() {
        let dir = tempfile::tempdir().unwrap();
        let (entry, warning) = skill_entry(&record("gone", &dir.path().join("gone"), None));
        assert!(warning.unwrap().starts_with("/gone: cannot read "));
        assert_eq!((entry.description, entry.notes), (None, None));
        assert_eq!(entry.links, [] as [EntryLink; 0]);
        assert!(!entry.notes_markdown.contains("Couldn’t read"));
    }

    #[test]
    fn warns_about_front_matter_that_is_not_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let skill = dir.path().join("bad");
        fs::create_dir(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "---\ndescription: [unclosed\n---\nBody\n").unwrap();
        let (entry, warning) = skill_entry(&record("bad", &skill, None));
        assert!(warning.unwrap().starts_with("/bad: SKILL.md front matter: "));
        assert_eq!(entry.description, None);
        assert!(entry.notes_markdown.contains("Body"));
    }

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
