# Skills entry and scoped refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** every record of `skills ls --global --agent universal --json` becomes a searchable `skills` entry, and
Refresh YAML / Refresh Skills refresh one source without touching the other.

**Architecture:** the index gains `entries.source` (`yaml` | `skills`), migrated once by `PRAGMA user_version`.
`kb-domain` gets a Skills source adapter (`skills.rs`: JSON records → entries; `skills/command.rs`: runs the command,
cancellable) beside the YAML one, and `sync_sources` runs the selected sources, each in its own transaction, returning
a per-source `SyncReport`. `kb-app` finds the command (`command_path.rs`), adds `--source` to `kb --sync`, the
`skills` catalog type, the Refresh More… submenu, and per-source report sections.

**Tech Stack:** Rust 2024, rusqlite (FTS5), serde_json, GPUI Kit, cargo-nextest, mise.

**Spec:** [2026-10-18-skills-entry-and-scoped-refresh-design.md](../specs/2026-10-18-skills-entry-and-scoped-refresh-design.md)
(D1–D8). TODO task `Rkpg`.

**Expects** (verified at `2dc6d64`): `kb_domain::sync::{sync_knowledge_base, SyncProgress, SyncSummary, SyncError,
open_database, open_readonly, SCHEMA_DDL}`; `kb_domain::entry::{Entry, EntryLink, derive_entry_id, raw_string}`;
`kb_domain::markdown::build_entry_markdown` (strips a `file://` Markdown note's front matter);
`kb_app::catalog::{Catalog, Hero, Panel, Messages, ACTION_IDS}`; `kb_app::actions::{Submenu, more_page, search_page}`;
`kb_app::sync_view::{SyncView, SyncFinished, report_rows, summary_line}`; `NotebookText` is already in
`assets.rs`'s `ExtraIcons`; the `yellow` colour token exists in DESIGN.md.

**Conventions:** write the failing test first; run focused tests with `cargo nextest run -p <crate> <filter>`;
stage with `git add <paths>` then `git commit` (never `git commit -a` or `git commit <paths>`); Conventional Commits,
sentence-case subject ≤ 72 characters, ending with the `Co-Authored-By` trailer your harness gives. Format with
`hk fix --all` before committing if a hook complains.

---

## File map

| File                                                       | Change                                                 |
| ---------------------------------------------------------- | ------------------------------------------------------ |
| `crates/kb-domain/src/schema.sql`                          | `entries.source`, its index                            |
| `crates/kb-domain/src/selection.rs`                        | **new**: `SourceKind`, `Selection`                     |
| `crates/kb-domain/src/skills.rs`                           | **new**: `SkillRecord`, `parse_skills`, `skill_entry`  |
| `crates/kb-domain/src/skills/command.rs`                   | **new**: `run_command` (cancellable child process)     |
| `crates/kb-domain/src/sync.rs`                             | migration, scoped writes, `sync_sources`, `SyncReport` |
| `crates/kb-domain/src/lib.rs`                              | `pub mod selection; pub mod skills;`                   |
| `crates/kb-app/src/command_path.rs`                        | **new**: finds the Skills command without a shell      |
| `crates/kb-app/src/config.rs`                              | `skills_command` setting                               |
| `crates/kb-app/src/cli.rs`, `src/main.rs`                  | `--source`, per-source output, failure exit            |
| `crates/kb-app/assets/catalog.toml`                        | `skills` type, two actions, Refresh More…, messages    |
| `crates/kb-app/src/catalog.rs`                             | optional hero, new ids, panel fields, messages         |
| `crates/kb-app/src/ui/entry_view.rs`, `src/entry_list.rs`  | `Option<Hero>`                                         |
| `crates/kb-app/src/actions.rs`                             | `Submenu::Refresh`, `refresh_page`                     |
| `crates/kb-app/src/search_view.rs`, `search_view/panel.rs` | scoped refresh, retry, back from Refresh More…         |
| `crates/kb-app/src/sync_view.rs`                           | `SyncJob`, Skills progress, report sections            |
| `crates/kb-app/tests/{sync_cli,ui}.rs`                     | fake Skills command; new behaviour                     |
| `CONTEXT.md`, `DESIGN.md`                                  | vocabulary, yellow accent                              |

---

### Task 1: The `source` column and its migration

**Files:**

- Modify: `crates/kb-domain/src/schema.sql`
- Modify: `crates/kb-domain/src/sync.rs` (`SCHEMA_DDL` doc, `open_initialized`, tests)

- [ ] **Step 1: Write the failing tests** (append to `mod tests` in `sync.rs`)

```rust
    #[test]
    fn creates_an_index_at_the_current_schema_version() {
        let db = tempfile::tempdir().unwrap();
        let connection = open_initialized(&db.path().join("kb.db")).unwrap();
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 1);
        connection.execute("SELECT source FROM entries", []).unwrap();
    }

    #[test]
    fn migrates_an_index_without_sources_keeping_its_rows_as_yaml() {
        let db = tempfile::tempdir().unwrap();
        let path = db.path().join("kb.db");
        open_database(&path)
            .unwrap()
            .execute_batch(
                "CREATE TABLE entries (id TEXT PRIMARY KEY, title TEXT NOT NULL, description TEXT NOT NULL, \
                 type TEXT NOT NULL, notes TEXT NOT NULL, notes_markdown TEXT NOT NULL, primary_content TEXT NOT NULL, \
                 links_json TEXT NOT NULL, source_file TEXT NOT NULL, source_rel_path TEXT NOT NULL, \
                 raw_yaml TEXT NOT NULL); \
                 INSERT INTO entries VALUES ('f.yml#notes#one', 'one', '', 'notes', '', '', 'one', '[]', '/s/f.yml', \
                 'f.yml', '{}');",
            )
            .unwrap();
        let connection = open_initialized(&path).unwrap();
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        let source: String = connection
            .query_row("SELECT source FROM entries", [], |r| r.get(0))
            .unwrap();
        assert_eq!((version, source.as_str()), (1, "yaml"));
    }
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo nextest run -p kb-domain schema_version migrates_an_index`
Expected: FAIL (`no such column: source`, version 0).

- [ ] **Step 3: Add the column to the schema**

In `schema.sql`, make the end of `CREATE TABLE IF NOT EXISTS entries` read:

```sql
  source_rel_path TEXT NOT NULL,
  raw_yaml TEXT NOT NULL,
  source TEXT NOT NULL DEFAULT 'yaml'
);
```

and after `CREATE INDEX IF NOT EXISTS idx_entry_tags_tag ON entry_tags(tag);` add:

```sql
CREATE INDEX IF NOT EXISTS idx_entries_source ON entries(source);
```

- [ ] **Step 4: Migrate before applying the schema**

In `sync.rs`, replace the `SCHEMA_DDL` doc comment and `open_initialized` with:

```rust
/// The index schema.
pub const SCHEMA_DDL: &str = include_str!("schema.sql");

/// The schema version this engine writes (`PRAGMA user_version`): 1 added `entries.source`.
const SCHEMA_VERSION: i64 = 1;

/// Brings an index to [`SCHEMA_VERSION`], then applies the schema. An index at 0 that has `entries`
/// gains `source`, every row `yaml`: no other source existed before it.
fn initialize(connection: &Connection) -> rusqlite::Result<()> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let has_entries: bool = connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'entries')",
        [],
        |row| row.get(0),
    )?;
    if version < 1 && has_entries {
        connection.execute_batch("ALTER TABLE entries ADD COLUMN source TEXT NOT NULL DEFAULT 'yaml';")?;
    }
    connection.execute_batch(SCHEMA_DDL)?;
    connection.pragma_update(None, "user_version", SCHEMA_VERSION)
}

/// Opens the index with its schema, replacing the file only when SQLite says it is corrupt: the
/// index is derived, so a corrupt one costs nothing to discard; any other failure is returned.
fn open_initialized(path: &Path) -> rusqlite::Result<Connection> {
    let attempt = open_database(path).and_then(|c| initialize(&c).map(|()| c));
    match attempt {
        Err(rusqlite::Error::SqliteFailure(e, _))
            if matches!(e.code, ErrorCode::DatabaseCorrupt | ErrorCode::NotADatabase) =>
        {
            for suffix in ["", "-journal", "-wal", "-shm"] {
                let _ = fs::remove_file(format!("{}{suffix}", path.display()));
            }
            let connection = open_database(path)?;
            initialize(&connection)?;
            Ok(connection)
        }
        other => other,
    }
}
```

- [ ] **Step 5: Run the crate's tests**

Run: `cargo nextest run -p kb-domain`
Expected: PASS, the goldens included (they select named columns).

- [ ] **Step 6: Commit**

```bash
git add crates/kb-domain/src/schema.sql crates/kb-domain/src/sync.rs
git commit -m "feat(domain): Record each index row's source and migrate old indexes"
```

---

### Task 2: Source kinds and the selection

**Files:**

- Create: `crates/kb-domain/src/selection.rs`
- Modify: `crates/kb-domain/src/lib.rs` (add `pub mod selection;` after `pub mod search;`)

- [ ] **Step 1: Write the module with its failing tests**

```rust
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
```

- [ ] **Step 2: Run the tests**

Run: `cargo nextest run -p kb-domain selection`
Expected: PASS (2 tests). The module is data-only, so test and code land together.

- [ ] **Step 3: Commit**

```bash
git add crates/kb-domain/src/selection.rs crates/kb-domain/src/lib.rs
git commit -m "feat(domain): Name the sources a sync can refresh"
```

---

### Task 3: YAML refresh replaces only YAML rows

**Files:**

- Modify: `crates/kb-domain/src/sync.rs`

- [ ] **Step 1: Write the failing test** (in `mod tests`)

```rust
    fn count_source(path: &Path, source: &str) -> i64 {
        open_database(path)
            .unwrap()
            .query_row("SELECT count(*) FROM entries WHERE source = ?", [source], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn a_yaml_sync_keeps_the_rows_of_other_sources() {
        let root = sources(&[("a.yml", "notes:\n  one: {}\n")]);
        let db = tempfile::tempdir().unwrap();
        let path = db.path().join("kb.db");
        sync_knowledge_base(root.path(), &path, &AtomicBool::new(false), |_| {}).unwrap();
        open_database(&path)
            .unwrap()
            .execute(
                "INSERT INTO entries (id, title, description, type, notes, notes_markdown, primary_content, \
                 links_json, source_file, source_rel_path, raw_yaml, source) \
                 VALUES ('t/SKILL.md#skills#/t', '/t', '', 'skills', '', '', '/t', '[]', '/x/t/SKILL.md', \
                 't/SKILL.md', '{}', 'skills')",
                [],
            )
            .unwrap();
        fs::write(root.path().join("a.yml"), "notes:\n  one: {}\n  two: {}\n").unwrap();
        sync_knowledge_base(root.path(), &path, &AtomicBool::new(false), |_| {}).unwrap();
        assert_eq!((count_source(&path, "yaml"), count_source(&path, "skills")), (2, 1));
        assert_eq!(count(&path, "entries_fts('\"two\"*')"), 1);
    }
```

- [ ] **Step 2: Run it to see it fail**

Run: `cargo nextest run -p kb-domain a_yaml_sync_keeps`
Expected: FAIL: skills count 0 (the YAML sync deletes every row).

- [ ] **Step 3: Split the YAML sync into scoped steps**

In `sync.rs`, add `use crate::selection::SourceKind;` and `use rusqlite::{Statement, Transaction};` (merge into the
existing `rusqlite` import), then replace `sync_knowledge_base` with:

```rust
const INSERT_ENTRY: &str = "INSERT INTO entries (id, title, description, type, notes, notes_markdown, \
    primary_content, links_json, source_file, source_rel_path, raw_yaml, source) \
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
const INSERT_TAG: &str = "INSERT OR IGNORE INTO entry_tags (entry_id, tag) VALUES (?, ?)";

/// Opens the index at `database_path`, creating its directory and schema.
fn open_index(database_path: &Path) -> Result<Connection, SyncError> {
    if let Some(parent) = database_path.parent() {
        fs::create_dir_all(parent).map_err(SyncError::IndexDirectory)?;
    }
    Ok(open_initialized(database_path)?)
}

/// Deletes `kind`'s rows (tags through `ON DELETE CASCADE`, full text through the delete trigger).
/// When no other source has rows, the full-text table is emptied at once instead, as fast as before
/// sources existed.
fn clear_source(transaction: &Transaction<'_>, kind: SourceKind) -> rusqlite::Result<()> {
    let others: bool = transaction.query_row(
        "SELECT EXISTS (SELECT 1 FROM entries WHERE source != ?)",
        [kind.column()],
        |row| row.get(0),
    )?;
    if others {
        transaction.execute("DELETE FROM entries WHERE source = ?", [kind.column()])?;
    } else {
        transaction.execute_batch("DELETE FROM entries; INSERT INTO entries_fts(entries_fts) VALUES('delete-all');")?;
    }
    Ok(())
}

/// Writes `entry` and its tags as a row of `kind`.
fn insert_entry(
    entries: &mut Statement<'_>,
    tags: &mut Statement<'_>,
    entry: &Entry,
    kind: SourceKind,
) -> rusqlite::Result<()> {
    let row = entry_to_row(entry);
    entries.execute(params![
        row.id,
        row.title,
        row.description,
        row.entry_type,
        row.notes,
        row.notes_markdown,
        row.primary_content,
        row.links_json,
        row.source_file,
        row.source_rel_path,
        row.raw_yaml,
        kind.column()
    ])?;
    for tag in &entry.tags {
        tags.execute(params![row.id, tag])?;
    }
    Ok(())
}

/// Rebuilds the YAML entries of the index at `database_path` from the source files under
/// `sources_dir`, in one transaction: a failure or a cancellation leaves the previous index as it
/// was. Other sources' entries are kept.
///
/// # Errors
///
/// The sources directory is unreadable, SQLite fails, or `cancel` was set.
pub fn sync_knowledge_base(
    sources_dir: &Path,
    database_path: &Path,
    cancel: &AtomicBool,
    on_progress: impl FnMut(SyncProgress<'_>),
) -> Result<SyncSummary, SyncError> {
    let mut connection = open_index(database_path)?;
    index_yaml(&mut connection, sources_dir, cancel, on_progress)
}

/// The YAML source: replaces its rows with the source files' entries, in one transaction.
fn index_yaml(
    connection: &mut Connection,
    sources_dir: &Path,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(SyncProgress<'_>),
) -> Result<SyncSummary, SyncError> {
    let started = Instant::now();
    let files = find_source_files(sources_dir)?;
    let transaction = connection.transaction()?;
    clear_source(&transaction, SourceKind::Yaml)?;

    let mut summary = SyncSummary {
        total_indexed: 0,
        files_scanned: files.len(),
        files_failed: 0,
        duration_ms: 0,
        errors: Vec::new(),
        files: Vec::new(),
    };
    {
        let mut entries = transaction.prepare(INSERT_ENTRY)?;
        let mut tags = transaction.prepare(INSERT_TAG)?;
        for (index, file) in files.iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                return Err(SyncError::Cancelled);
            }
            let file_path = file.to_string_lossy().into_owned();
            let rel_path = file
                .strip_prefix(sources_dir)
                .unwrap_or(file)
                .to_string_lossy()
                .into_owned();
            let parsed = parse_source_file(file);
            let report = file_report(&file_path, &rel_path, &parsed);
            if let Some(error) = &parsed.error {
                summary.files_failed += 1;
                summary.errors.push(error.clone());
            }
            for record in &parsed.records {
                let mut entry: Entry =
                    normalize_entry(&file_path, &rel_path, &record.entry_type, &record.key, &record.raw);
                entry.notes_markdown = build_entry_markdown(&entry);
                insert_entry(&mut entries, &mut tags, &entry, SourceKind::Yaml)?;
                summary.total_indexed += 1;
            }
            on_progress(SyncProgress {
                files_scanned: index + 1,
                files_total: files.len(),
                entries_count: summary.total_indexed,
                file: &report,
            });
            summary.files.push(report);
        }
    }
    if cancel.load(Ordering::Relaxed) {
        return Err(SyncError::Cancelled);
    }
    transaction.commit()?;
    summary.duration_ms = started.elapsed().as_millis();
    Ok(summary)
}
```

- [ ] **Step 4: Run the crate's tests**

Run: `cargo nextest run -p kb-domain`
Expected: PASS, the goldens and `a_cancelled_sync_leaves_the_previous_index` included.

- [ ] **Step 5: Commit**

```bash
git add crates/kb-domain/src/sync.rs
git commit -m "refactor(domain): Replace only YAML rows on a YAML sync"
```

---

### Task 4: Parse the Skills JSON

**Files:**

- Create: `crates/kb-domain/src/skills.rs`
- Modify: `crates/kb-domain/src/lib.rs` (add `pub mod skills;` after `pub mod search;`/`pub mod selection;`)

- [ ] **Step 1: Write the failing tests** (create `skills.rs` with only this, so it fails to compile)

```rust
//! The Skills source: the records `skills ls --global --agent universal --json` prints, and the
//! entries they become.

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
        assert_eq!(parse_skills(r#"[{"name":"a","path":""}]"#).unwrap_err(), "a has no path");
        assert_eq!(
            parse_skills(r#"[{"name":"a","path":"/s/a"},{"name":"a","path":"/t/a"}]"#).unwrap_err(),
            "a appears twice"
        );
    }
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo nextest run -p kb-domain skills`
Expected: FAIL to compile: `parse_skills` not found.

- [ ] **Step 3: Implement the record and the parser** (above `mod tests`)

```rust
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
```

- [ ] **Step 4: Run the tests**

Run: `cargo nextest run -p kb-domain skills`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/kb-domain/src/skills.rs crates/kb-domain/src/lib.rs
git commit -m "feat(domain): Parse the Skills command's JSON records"
```

---

### Task 5: A Skills record becomes an entry

**Files:**

- Modify: `crates/kb-domain/src/skills.rs`

- [ ] **Step 1: Write the failing tests** (in `mod tests`)

```rust
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
        assert!(entry.tags.is_empty());
        assert!(entry.notes_markdown.contains("# Test-Driven Development"));
        assert!(!entry.notes_markdown.contains("name: tdd"), "front matter stripped");
    }

    #[test]
    fn indexes_a_record_whose_skill_md_is_missing_with_a_warning() {
        let dir = tempfile::tempdir().unwrap();
        let (entry, warning) = skill_entry(&record("gone", &dir.path().join("gone"), None));
        assert!(warning.unwrap().starts_with("/gone: cannot read "));
        assert_eq!((entry.description, entry.notes), (None, None));
        assert!(entry.links.is_empty());
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
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo nextest run -p kb-domain skills`
Expected: FAIL to compile: `skill_entry` not found.

- [ ] **Step 3: Implement the mapping** (extend the imports, add below `parse_skills`)

```rust
use std::path::Path;

use saphyr::{ScalarOwned, YamlOwned};

use crate::entry::{Entry, EntryLink, derive_entry_id, raw_string};
use crate::markdown::build_entry_markdown;
use crate::yaml;

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
        notes: readable
            .then(|| YamlOwned::Value(ScalarOwned::String(format!("file://{}", skill_md.display())))),
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
```

- [ ] **Step 4: Run the tests**

Run: `cargo nextest run -p kb-domain skills`
Expected: PASS (6 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/kb-domain/src/skills.rs
git commit -m "feat(domain): Map a Skills record to a skills entry"
```

---

### Task 6: Run the command, cancellably

**Files:**

- Create: `crates/kb-domain/src/skills/command.rs`
- Modify: `crates/kb-domain/src/skills.rs` (add `pub mod command;` below the module doc)

- [ ] **Step 1: Write the failing tests** (create `command.rs` with only this)

```rust
//! Runs the Skills command: its status, stdout and stderr in full, or nothing when cancelled.

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn returns_the_status_and_both_streams() {
        let output = run_command(
            Path::new("/bin/sh"),
            &["-c", "echo '[]'; echo warn >&2; exit 3"],
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(
            (output.stdout.as_str(), output.stderr.as_str(), output.status.code()),
            ("[]\n", "warn\n", Some(3))
        );
    }

    #[test]
    fn kills_a_cancelled_command() {
        let started = Instant::now();
        let result = run_command(Path::new("/bin/sleep"), &["5"], &AtomicBool::new(true));
        assert!(matches!(result, Err(CommandError::Cancelled)));
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn says_why_a_command_cannot_run() {
        let error = run_command(Path::new("/nonexistent/skills"), &[], &AtomicBool::new(false)).unwrap_err();
        assert!(error.to_string().starts_with("cannot run /nonexistent/skills: "), "{error}");
    }
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo nextest run -p kb-domain command`
Expected: FAIL to compile: `run_command` not found.

- [ ] **Step 3: Implement it** (above `mod tests`)

```rust
use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// How often a running command checks for a cancellation.
const POLL: Duration = Duration::from_millis(20);

/// What a finished command printed.
#[derive(Debug)]
pub struct CommandOutput {
    /// How it exited.
    pub status: ExitStatus,
    /// Everything it wrote to stdout.
    pub stdout: String,
    /// Everything it wrote to stderr.
    pub stderr: String,
}

/// Why a command gave no output.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    /// It could not be started.
    #[error("cannot run {program}: {source}")]
    Spawn {
        /// The command.
        program: String,
        /// The OS error.
        source: io::Error,
    },
    /// Its output could not be read, or was not UTF-8.
    #[error("cannot read the output of {program}: {source}")]
    Output {
        /// The command.
        program: String,
        /// The I/O error.
        source: io::Error,
    },
    /// `cancel` was set; the command was killed.
    #[error("cancelled")]
    Cancelled,
}

/// Runs `program` with `args`, no stdin, until it exits or `cancel` is set (then it is killed).
///
/// # Errors
///
/// It cannot start, its output cannot be read, or it was cancelled.
pub fn run_command(program: &Path, args: &[&str], cancel: &AtomicBool) -> Result<CommandOutput, CommandError> {
    let name = program.display().to_string();
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| CommandError::Spawn {
            program: name.clone(),
            source,
        })?;
    // Each pipe drains on its own thread, so a command that fills one never blocks on it.
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let status = loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(CommandError::Cancelled);
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(POLL),
            Err(source) => return Err(CommandError::Output { program: name, source }),
        }
    };
    let collect = |handle: JoinHandle<io::Result<String>>| {
        handle
            .join()
            .unwrap_or_else(|_| Err(io::Error::other("the output reader stopped")))
            .map_err(|source| CommandError::Output {
                program: name.clone(),
                source,
            })
    };
    Ok(CommandOutput {
        status,
        stdout: collect(stdout)?,
        stderr: collect(stderr)?,
    })
}

/// Reads `pipe` to the end on a thread of its own.
fn drain(pipe: Option<impl Read + Send + 'static>) -> JoinHandle<io::Result<String>> {
    thread::spawn(move || {
        let mut text = String::new();
        if let Some(mut pipe) = pipe {
            pipe.read_to_string(&mut text)?;
        }
        Ok(text)
    })
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo nextest run -p kb-domain command`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/kb-domain/src/skills.rs crates/kb-domain/src/skills/command.rs
git commit -m "feat(domain): Run the Skills command and kill it on cancel"
```

---

### Task 7: `sync_sources`: each selected source in its own transaction

**Files:**

- Modify: `crates/kb-domain/src/sync.rs`

- [ ] **Step 1: Write the failing tests** (in `mod tests`)

```rust
    use std::os::unix::fs::PermissionsExt as _;
    use std::path::PathBuf;

    use crate::selection::Selection;

    /// An executable that prints `stdout`, then `stderr` to stderr, and exits with `code`.
    fn fake_skills(dir: &Path, stdout: &str, stderr: &str, code: i32) -> PathBuf {
        let path = dir.join(format!("skills-{code}-{}", stdout.len()));
        fs::write(
            &path,
            format!("#!/bin/sh\ncat <<'JSON'\n{stdout}\nJSON\nprintf '%s' '{stderr}' >&2\nexit {code}\n"),
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    /// The JSON of Skills named `names`, each with a `SKILL.md` under `dir`.
    fn skills_json(dir: &Path, names: &[&str]) -> String {
        let records: Vec<String> = names
            .iter()
            .map(|name| {
                let skill = dir.join(name);
                fs::create_dir_all(&skill).unwrap();
                fs::write(
                    skill.join("SKILL.md"),
                    format!("---\ndescription: Drives {name} agents\n---\nBody\n"),
                )
                .unwrap();
                format!(r#"{{"name":"{name}","path":"{}"}}"#, skill.display())
            })
            .collect();
        format!("[{}]", records.join(","))
    }

    fn run(selection: Selection, sources: &Path, skills: Option<&Path>, db: &Path, cancel: bool) -> SyncReport {
        try_run(selection, sources, skills, db, cancel).unwrap()
    }

    fn try_run(
        selection: Selection,
        sources: &Path,
        skills: Option<&Path>,
        db: &Path,
        cancel: bool,
    ) -> Result<SyncReport, SyncError> {
        let request = SyncRequest {
            selection,
            sources_dir: sources,
            skills_command: skills,
        };
        sync_sources(&request, db, &AtomicBool::new(cancel), |_| {})
    }

    #[test]
    fn refreshes_each_source_without_touching_the_other() {
        let root = sources(&[("a.yml", "notes:\n  one: {}\n")]);
        let home = tempfile::tempdir().unwrap();
        let db = home.path().join("kb.db");
        let skills = fake_skills(home.path(), &skills_json(home.path(), &["tdd", "act"]), "", 0);

        run(Selection::Yaml, root.path(), Some(&skills), &db, false);
        let report = run(Selection::Skills, root.path(), Some(&skills), &db, false);
        assert!(report.yaml.is_none());
        assert_eq!(
            report.skills,
            Some(SkillsOutcome::Indexed {
                entries: 2,
                warnings: Vec::new()
            })
        );
        assert_eq!((count_source(&db, "yaml"), count_source(&db, "skills")), (1, 2));
        assert_eq!(count(&db, "entries_fts('\"agents\"*')"), 2, "descriptions are searchable");

        fs::write(root.path().join("a.yml"), "notes:\n  one: {}\n  two: {}\n").unwrap();
        run(Selection::Yaml, root.path(), Some(&skills), &db, false);
        assert_eq!((count_source(&db, "yaml"), count_source(&db, "skills")), (2, 2));
    }

    #[test]
    fn keeps_the_previous_skills_when_the_command_fails() {
        let root = sources(&[]);
        let home = tempfile::tempdir().unwrap();
        let db = home.path().join("kb.db");
        let good = fake_skills(home.path(), &skills_json(home.path(), &["tdd"]), "", 0);
        run(Selection::Skills, root.path(), Some(&good), &db, false);

        let failing = [
            fake_skills(home.path(), "[]", "", 1),
            fake_skills(home.path(), "not json", "", 0),
            fake_skills(home.path(), r#"[{"name":"a","path":"/a"},{"name":"a","path":"/b"}]"#, "", 0),
        ];
        for command in &failing {
            let report = run(Selection::Skills, root.path(), Some(command), &db, false);
            assert!(matches!(report.skills, Some(SkillsOutcome::Failed { .. })), "{command:?}");
            assert!(report.has_failure());
            assert_eq!(count_source(&db, "skills"), 1, "{command:?}");
        }
        let missing = home.path().join("no-such-skills");
        let report = run(Selection::Skills, root.path(), Some(&missing), &db, false);
        assert!(matches!(report.skills, Some(SkillsOutcome::Failed { .. })));
        assert_eq!(count_source(&db, "skills"), 1);
    }

    #[test]
    fn reports_stderr_lines_as_warnings() {
        let root = sources(&[]);
        let home = tempfile::tempdir().unwrap();
        let db = home.path().join("kb.db");
        let skills = fake_skills(home.path(), "[]", "⚠ Skipped x\n\n", 0);
        let report = run(Selection::Skills, root.path(), Some(&skills), &db, false);
        assert_eq!(
            report.skills,
            Some(SkillsOutcome::Indexed {
                entries: 0,
                warnings: vec!["⚠ Skipped x".into()]
            })
        );
        assert!(!report.has_failure());
    }

    #[test]
    fn reports_skills_not_configured_without_failing() {
        let root = sources(&[("a.yml", "notes:\n  one: {}\n")]);
        let db = tempfile::tempdir().unwrap();
        let report = run(Selection::All, root.path(), None, &db.path().join("kb.db"), false);
        assert_eq!(report.skills, Some(SkillsOutcome::NotConfigured));
        assert!(!report.has_failure());
        assert_eq!(report.total_indexed(), 1);
    }

    #[test]
    fn keeps_fresh_yaml_when_skills_fail_or_stop() {
        let root = sources(&[("a.yml", "notes:\n  one: {}\n")]);
        let home = tempfile::tempdir().unwrap();
        let db = home.path().join("kb.db");
        let failing = fake_skills(home.path(), "[]", "", 1);
        run(Selection::All, root.path(), Some(&failing), &db, false);
        assert_eq!(count_source(&db, "yaml"), 1);

        let good = fake_skills(home.path(), &skills_json(home.path(), &["tdd"]), "", 0);
        run(Selection::Skills, root.path(), Some(&good), &db, false);
        let result = try_run(Selection::Skills, root.path(), Some(&good), &db, true);
        assert!(matches!(result, Err(SyncError::Cancelled)));
        assert_eq!(count_source(&db, "skills"), 1, "a cancelled source commits nothing");
    }

    #[test]
    fn runs_skills_when_the_sources_directory_is_missing() {
        let home = tempfile::tempdir().unwrap();
        let db = home.path().join("kb.db");
        let skills = fake_skills(home.path(), &skills_json(home.path(), &["tdd"]), "", 0);
        let report = run(Selection::All, &home.path().join("nowhere"), Some(&skills), &db, false);
        assert!(
            matches!(&report.yaml, Some(YamlOutcome::Failed(reason)) if reason.contains("cannot read the sources directory"))
        );
        assert_eq!(count_source(&db, "skills"), 1);
        assert!(report.has_failure());
    }
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo nextest run -p kb-domain sync::tests`
Expected: FAIL to compile: `SyncRequest`, `sync_sources`, `SyncReport` not found.

- [ ] **Step 3: Implement the request, the report and `sync_sources`**

Add to the imports of `sync.rs`:

```rust
use crate::selection::Selection;
use crate::skills::command::{CommandError, run_command};
use crate::skills::{SKILLS_ARGS, parse_skills, skill_entry};
```

and add after `SyncSummary`:

```rust
/// What a sync refreshes.
#[derive(Debug, Clone, Copy)]
pub struct SyncRequest<'a> {
    /// The sources to refresh.
    pub selection: Selection,
    /// The YAML sources directory.
    pub sources_dir: &'a Path,
    /// The Skills command; `None` when it is not configured.
    pub skills_command: Option<&'a Path>,
}

/// What a sync reports while it runs.
#[derive(Debug, Clone, Copy)]
pub enum Progress<'a> {
    /// A YAML file was indexed.
    Yaml(SyncProgress<'a>),
    /// The Skills command started.
    Skills,
}

/// The YAML source's outcome.
#[derive(Debug, Clone)]
pub enum YamlOutcome {
    /// Its entries were replaced.
    Indexed(SyncSummary),
    /// The sources directory could not be read; the previous YAML entries are unchanged.
    Failed(String),
}

/// The Skills source's outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillsOutcome {
    /// No command is set and none was found: nothing ran, nothing changed.
    NotConfigured,
    /// Its entries were replaced.
    Indexed {
        /// Entries indexed.
        entries: usize,
        /// The command's stderr lines, then records whose `SKILL.md` could not be read.
        warnings: Vec<String>,
    },
    /// The command or its output failed; the previous Skills entries are unchanged.
    Failed {
        /// Why.
        reason: String,
        /// The command's stderr lines.
        warnings: Vec<String>,
    },
}

/// A finished sync: each selected source's outcome (`None` for a source it did not select).
#[derive(Debug, Clone)]
pub struct SyncReport {
    /// The YAML source.
    pub yaml: Option<YamlOutcome>,
    /// The Skills source.
    pub skills: Option<SkillsOutcome>,
    /// How long it took.
    pub duration_ms: u128,
}

impl SyncReport {
    /// Entries indexed, by every source that indexed.
    #[must_use]
    pub fn total_indexed(&self) -> usize {
        let yaml = match &self.yaml {
            Some(YamlOutcome::Indexed(summary)) => summary.total_indexed,
            _ => 0,
        };
        let skills = match &self.skills {
            Some(SkillsOutcome::Indexed { entries, .. }) => *entries,
            _ => 0,
        };
        yaml + skills
    }

    /// Whether a selected source failed; a source not configured has not.
    #[must_use]
    pub fn has_failure(&self) -> bool {
        matches!(self.yaml, Some(YamlOutcome::Failed(_))) || matches!(self.skills, Some(SkillsOutcome::Failed { .. }))
    }
}

/// Refreshes the selected sources of the index at `database_path`: YAML, then Skills, each
/// committed in its own transaction, so a source's failure or cancellation never touches the
/// other's entries, nor its own previous ones.
///
/// # Errors
///
/// SQLite fails, the index directory cannot be created, or `cancel` was set; sources committed
/// before then stay.
pub fn sync_sources(
    request: &SyncRequest<'_>,
    database_path: &Path,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(Progress<'_>),
) -> Result<SyncReport, SyncError> {
    let started = Instant::now();
    let mut connection = open_index(database_path)?;
    let yaml = if request.selection.includes(SourceKind::Yaml) {
        let outcome = index_yaml(&mut connection, request.sources_dir, cancel, |p| {
            on_progress(Progress::Yaml(p));
        });
        Some(match outcome {
            Ok(summary) => YamlOutcome::Indexed(summary),
            Err(error @ SyncError::Sources(_)) => YamlOutcome::Failed(error.to_string()),
            Err(error) => return Err(error),
        })
    } else {
        None
    };
    let skills = if request.selection.includes(SourceKind::Skills) {
        Some(match request.skills_command {
            None => SkillsOutcome::NotConfigured,
            Some(program) => {
                on_progress(Progress::Skills);
                index_skills(&mut connection, program, cancel)?
            }
        })
    } else {
        None
    };
    Ok(SyncReport {
        yaml,
        skills,
        duration_ms: started.elapsed().as_millis(),
    })
}

/// The Skills source: runs the command and, when its output is a valid Skills set, replaces the
/// Skills rows with its records' entries, in one transaction.
fn index_skills(connection: &mut Connection, program: &Path, cancel: &AtomicBool) -> Result<SkillsOutcome, SyncError> {
    let output = match run_command(program, &SKILLS_ARGS, cancel) {
        Ok(output) => output,
        Err(CommandError::Cancelled) => return Err(SyncError::Cancelled),
        Err(error) => {
            return Ok(SkillsOutcome::Failed {
                reason: error.to_string(),
                warnings: Vec::new(),
            });
        }
    };
    let mut warnings: Vec<String> = output
        .stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();
    if !output.status.success() {
        return Ok(SkillsOutcome::Failed {
            reason: format!("{} exited with {}", program.display(), output.status),
            warnings,
        });
    }
    let records = match parse_skills(&output.stdout) {
        Ok(records) => records,
        Err(reason) => return Ok(SkillsOutcome::Failed { reason, warnings }),
    };
    let transaction = connection.transaction()?;
    clear_source(&transaction, SourceKind::Skills)?;
    {
        let mut entries = transaction.prepare(INSERT_ENTRY)?;
        let mut tags = transaction.prepare(INSERT_TAG)?;
        for record in &records {
            if cancel.load(Ordering::Relaxed) {
                return Err(SyncError::Cancelled);
            }
            let (entry, warning) = skill_entry(record);
            warnings.extend(warning);
            insert_entry(&mut entries, &mut tags, &entry, SourceKind::Skills)?;
        }
    }
    if cancel.load(Ordering::Relaxed) {
        return Err(SyncError::Cancelled);
    }
    transaction.commit()?;
    Ok(SkillsOutcome::Indexed {
        entries: records.len(),
        warnings,
    })
}
```

- [ ] **Step 4: Run the crate's tests and clippy**

Run: `cargo nextest run -p kb-domain && cargo clippy -p kb-domain --all-targets -- -D warnings`
Expected: PASS, no warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/kb-domain/src/sync.rs
git commit -m "feat(domain): Sync the selected sources, each in its own transaction"
```

---

### Task 8: Find the Skills command

**Files:**

- Create: `crates/kb-app/src/command_path.rs`
- Modify: `crates/kb-app/src/lib.rs` (add `pub mod command_path;` after `pub mod cli;`)
- Modify: `crates/kb-app/src/config.rs`

- [ ] **Step 1: Write the failing tests** (create `command_path.rs` with only this)

```rust
//! Finds the Skills command without a shell: an app opened from Finder gets
//! `PATH=/usr/bin:/bin:/usr/sbin:/sbin`, so standard install folders are searched after `PATH`.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt as _;

    use super::*;

    fn executable(dir: &Path, name: &str) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[test]
    fn searches_path_then_the_install_folders() {
        let home = Path::new("/h");
        let dirs = search_dirs(Some(std::ffi::OsStr::new("/a:/b")), home);
        assert_eq!(
            dirs,
            ["/a", "/b", "/opt/homebrew/bin", "/usr/local/bin", "/h/.local/bin"].map(PathBuf::from)
        );
    }

    #[test]
    fn finds_the_first_executable_and_skips_plain_files() {
        let root = tempfile::tempdir().unwrap();
        let (first, second) = (root.path().join("first"), root.path().join("second"));
        fs::create_dir_all(&first).unwrap();
        fs::write(first.join("skills"), "not executable").unwrap();
        let found = executable(&second, "skills");
        assert_eq!(find_executable("skills", &[first, second]), Some(found));
        assert_eq!(find_executable("skills", &[root.path().join("none")]), None);
    }

    #[test]
    fn runs_a_set_path_and_keeps_a_set_name_it_cannot_find() {
        let home = Path::new("/h");
        assert_eq!(
            skills_command(Some("~/bin/skills"), None, home),
            Some(PathBuf::from("/h/bin/skills"))
        );
        assert_eq!(
            skills_command(Some("kb-test-no-such-command"), None, home),
            Some(PathBuf::from("kb-test-no-such-command"))
        );
    }
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo nextest run -p kb-app command_path`
Expected: FAIL to compile.

- [ ] **Step 3: Implement it** (above `mod tests`)

```rust
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::config::expand_home;

/// The command's name when `skills_command` is not set.
pub const DEFAULT_SKILLS_COMMAND: &str = "skills";

/// Searched after `PATH`, in order; `~/` is the home folder.
const INSTALL_DIRS: [&str; 3] = ["/opt/homebrew/bin", "/usr/local/bin", "~/.local/bin"];

/// The Skills command to run (spec, "Finding the command"): a set path as is, `~/` expanded; a
/// set name where it is found, else the name itself (running it then fails and says why); with no
/// setting, `skills` where it is found, else `None` (not configured).
#[must_use]
pub fn skills_command(setting: Option<&str>, path_var: Option<&OsStr>, home: &Path) -> Option<PathBuf> {
    let dirs = search_dirs(path_var, home);
    match setting.map(str::trim).filter(|s| !s.is_empty()) {
        Some(path) if path.contains('/') => Some(expand_home(path, home)),
        Some(name) => Some(find_executable(name, &dirs).unwrap_or_else(|| PathBuf::from(name))),
        None => find_executable(DEFAULT_SKILLS_COMMAND, &dirs),
    }
}

/// `PATH`'s folders, then [`INSTALL_DIRS`].
fn search_dirs(path_var: Option<&OsStr>, home: &Path) -> Vec<PathBuf> {
    path_var
        .map(|path| std::env::split_paths(path).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .chain(INSTALL_DIRS.iter().map(|dir| expand_home(dir, home)))
        .collect()
}

/// The first `dir/name` that is an executable file.
fn find_executable(name: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt as _;
    dirs.iter()
        .map(|dir| dir.join(name))
        .find(|path| {
            path.metadata()
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
}
```

In `config.rs`, make `expand_home` crate-visible: `pub(crate) fn expand_home(path: &str, home: &Path) -> PathBuf`.

- [ ] **Step 4: Run the tests**

Run: `cargo nextest run -p kb-app command_path`
Expected: PASS (3 tests).

- [ ] **Step 5: Write the failing config test** (in `config.rs`'s `mod tests`)

```rust
#[test]
fn reads_the_skills_command_and_expands_home() {
    let home = home_with(Some("skills_command = \"~/bin/skills\"\n"));
    let config = Config::load(home.path(), &home.path().join("data")).unwrap();
    assert_eq!(config.skills_command(), Some(home.path().join("bin/skills")));
    assert!(config.notices().is_empty());
}
```

Run: `cargo nextest run -p kb-app reads_the_skills_command`
Expected: FAIL to compile: no method `skills_command`.

- [ ] **Step 6: Read the setting**

In `config.rs`:

1. Module doc example: add `skills_command = "skills"` after `tag_images_path = …`, and the sentence "`skills_command`
   names the Skills command (spec D7); unset, kb looks for `skills` (`crate::command_path`)."
2. `Preferences`: add `skills_command: Option<String>,`.
3. In `load`, the location keys: `"sources_path" | "tag_images_path" | "skills_command" => {`.
4. `Config`: add fields `home: PathBuf,` and `skills_command: Option<String>,`; in `from_preferences` set
   `home: home.to_path_buf(),` and `skills_command: parsed.skills_command,`.
5. Add the accessor after `tag_images`:

```rust
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
```

- [ ] **Step 7: Run the app crate's unit tests**

Run: `cargo nextest run -p kb-app --lib`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add crates/kb-app/src/command_path.rs crates/kb-app/src/lib.rs crates/kb-app/src/config.rs
git commit -m "feat(app): Find the Skills command without a shell"
```

---

### Task 9: `kb --sync [--source all|yaml|skills]`

**Files:**

- Modify: `crates/kb-app/src/cli.rs`
- Modify: `crates/kb-app/src/main.rs`
- Modify: `crates/kb-app/tests/sync_cli.rs`

- [ ] **Step 1: Write the failing parse tests** (replace `reads_trace_and_sync` and
      `rejects_unknown_arguments_and_trace_with_sync` in `cli.rs`'s tests)

```rust
    #[test]
    fn reads_trace_sync_and_a_source() {
        assert_eq!(parse(args(&["--trace"])), Ok(Mode::Panel { trace: true }));
        assert_eq!(
            parse(args(&["--sync"])),
            Ok(Mode::Sync {
                selection: Selection::All
            })
        );
        assert_eq!(
            parse(args(&["--sync", "--source", "skills"])),
            Ok(Mode::Sync {
                selection: Selection::Skills
            })
        );
        assert_eq!(
            parse(args(&["--source", "yaml", "--sync"])),
            Ok(Mode::Sync {
                selection: Selection::Yaml
            })
        );
    }

    #[test]
    fn rejects_unknown_arguments_sources_and_combinations() {
        assert!(parse(args(&["--sink"])).is_err());
        assert!(parse(args(&["--sync", "--trace"])).is_err());
        assert!(parse(args(&["--sync", "--source", "both"])).is_err());
        assert!(parse(args(&["--sync", "--source"])).is_err());
        assert!(parse(args(&["--source", "yaml"])).is_err());
    }
```

Run: `cargo nextest run -p kb-app cli::tests`
Expected: FAIL to compile (`Mode::Sync` has no fields).

- [ ] **Step 2: Parse `--source`**

In `cli.rs`, update the module doc to ``//! `kb [--trace | --sync [--source all|yaml|skills]]`: …``, import
`kb_domain::selection::Selection`, and replace `Mode::Sync`, `UsageError` and `parse`:

```rust
    /// Refresh the index from the sources and exit (`--sync`, `--source`).
    Sync {
        /// The sources to refresh: `all` unless `--source` says otherwise.
        selection: Selection,
    },
}

/// An argument `kb` does not take.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("usage: kb [--trace | --sync [--source all|yaml|skills]]; unexpected {0}")]
pub struct UsageError(String);

/// Reads the arguments after the program name. `-psn_…`, which macOS once passed to apps opened
/// from Finder, is ignored.
///
/// # Errors
///
/// An unknown argument or source, `--source` without a value or without `--sync`, or `--trace`
/// with `--sync`.
pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Mode, UsageError> {
    let (mut trace, mut sync, mut selection) = (false, false, None);
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--trace" => trace = true,
            "--sync" => sync = true,
            "--source" => {
                let value = args
                    .next()
                    .ok_or_else(|| UsageError("`--source` without a value".into()))?;
                selection = Some(value.parse::<Selection>().map_err(|e| UsageError(e.to_string()))?);
            }
            psn if psn.starts_with("-psn_") => {}
            _ => return Err(UsageError(format!("`{arg}`"))),
        }
    }
    match (sync, trace, selection) {
        (true, true, _) => Err(UsageError("`--trace` with `--sync`".into())),
        (true, false, selection) => Ok(Mode::Sync {
            selection: selection.unwrap_or_default(),
        }),
        (false, _, Some(_)) => Err(UsageError("`--source` without `--sync`".into())),
        (false, trace, None) => Ok(Mode::Panel { trace }),
    }
}
```

Run: `cargo nextest run -p kb-app cli::tests` — the parse tests pass; `sync_reports_failed_files_and_a_summary` does not
compile yet (next step).

- [ ] **Step 3: Write the failing output test** (replace `sync_reports_failed_files_and_a_summary`)

```rust
    /// A Skills command printing `stdout`, so a test never runs the machine's own `skills`.
    fn fake_skills(dir: &std::path::Path, stdout: &str, code: i32) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt as _;
        let path = dir.join("skills");
        fs::write(&path, format!("#!/bin/sh\necho '{stdout}'\necho 'careful' >&2\nexit {code}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    fn config_with(home: &std::path::Path, skills: &std::path::Path) -> Config {
        let sources = home.join("sources");
        fs::create_dir_all(&sources).unwrap();
        fs::write(sources.join("good.yml"), "commands:\n  ls:\n    cmd: ls -la\n").unwrap();
        fs::write(sources.join("bad.yml"), "commands: [\n").unwrap();
        fs::create_dir_all(home.join(".config/kb")).unwrap();
        fs::write(
            home.join(".config/kb/config.toml"),
            format!(
                "sources_path = \"{}\"\nskills_command = \"{}\"\n",
                sources.display(),
                skills.display()
            ),
        )
        .unwrap();
        Config::load(home, &home.join("data")).unwrap()
    }

    #[test]
    fn sync_reports_each_source_then_a_summary() {
        let home = tempfile::tempdir().unwrap();
        let config = config_with(home.path(), &fake_skills(home.path(), "[]", 0));
        let mut out = Vec::new();
        let report = sync(&config, Selection::All, &mut out).unwrap();

        let out = String::from_utf8(out).unwrap();
        assert!(out.contains("bad.yml"), "{out}");
        assert!(out.contains("kb: yaml: indexed 1 entries from 2 files (1 failed)"), "{out}");
        assert!(out.contains("kb: skills: warning: careful"), "{out}");
        assert!(out.contains("kb: skills: indexed 0 entries"), "{out}");
        assert!(out.contains("kb: 1 entries in "), "{out}");
        assert!(!report.has_failure());
        assert!(config.database().is_file());
    }

    #[test]
    fn sync_says_a_failed_source_kept_its_entries() {
        let home = tempfile::tempdir().unwrap();
        let config = config_with(home.path(), &fake_skills(home.path(), "[]", 2));
        let mut out = Vec::new();
        let report = sync(&config, Selection::Skills, &mut out).unwrap();

        let out = String::from_utf8(out).unwrap();
        assert!(!out.contains("kb: yaml:"), "{out}");
        assert!(out.contains("kb: skills: failed: "), "{out}");
        assert!(out.contains("the previous Skills entries are unchanged"), "{out}");
        assert!(report.has_failure());
    }
```

Run: `cargo nextest run -p kb-app cli::tests`
Expected: FAIL to compile (`sync` takes two arguments).

- [ ] **Step 4: Print per source**

In `cli.rs`, replace the `sync_knowledge_base` import with
`use kb_domain::sync::{SkillsOutcome, SyncError, SyncReport, SyncRequest, YamlOutcome, sync_sources};` and replace
`sync`:

```rust
/// Refreshes `selection` of `config`'s sources and writes to `out`, per source, its failed files,
/// its warnings and its outcome, then a summary. A failed file or source is reported, not fatal, as
/// in the panel's refresh; the caller exits with a failure when [`SyncReport::has_failure`].
///
/// # Errors
///
/// SQLite failed; sources not yet committed are unchanged.
pub fn sync(config: &Config, selection: Selection, out: &mut impl Write) -> Result<SyncReport, SyncError> {
    let skills = config.skills_command();
    let request = SyncRequest {
        selection,
        sources_dir: config.sources_dir(),
        skills_command: skills.as_deref(),
    };
    let report = sync_sources(&request, config.database(), &AtomicBool::new(false), |_| {})?;
    match &report.yaml {
        Some(YamlOutcome::Indexed(summary)) => {
            for error in &summary.errors {
                let _ = writeln!(out, "kb: {}: {}", error.file_path, error.message);
            }
            let _ = writeln!(
                out,
                "kb: yaml: indexed {} entries from {} files ({} failed)",
                summary.total_indexed, summary.files_scanned, summary.files_failed
            );
        }
        Some(YamlOutcome::Failed(reason)) => {
            let _ = writeln!(out, "kb: yaml: failed: {reason}; the previous YAML entries are unchanged");
        }
        None => {}
    }
    match &report.skills {
        Some(SkillsOutcome::Indexed { entries, warnings }) => {
            write_warnings(out, warnings);
            let _ = writeln!(out, "kb: skills: indexed {entries} entries");
        }
        Some(SkillsOutcome::Failed { reason, warnings }) => {
            write_warnings(out, warnings);
            let _ = writeln!(
                out,
                "kb: skills: failed: {reason}; the previous Skills entries are unchanged"
            );
        }
        Some(SkillsOutcome::NotConfigured) => {
            let _ = writeln!(out, "kb: skills: not configured (install skills, or set skills_command)");
        }
        None => {}
    }
    let _ = writeln!(
        out,
        "kb: {} entries in {} ms into {}",
        report.total_indexed(),
        report.duration_ms,
        config.database().display()
    );
    Ok(report)
}

fn write_warnings(out: &mut impl Write, warnings: &[String]) {
    for warning in warnings {
        let _ = writeln!(out, "kb: skills: warning: {warning}");
    }
}
```

- [ ] **Step 5: Exit with a failure when a source failed**

In `main.rs`, update the module doc to ``//! `kb [--trace | --sync [--source …]]`: …`` and replace the
`Mode::Sync` arm:

```rust
// No panel shows a notice in a headless sync, so unreadable preferences stop it.
Mode::Sync { selection } => match Config::load(&home, &data) {
    Ok(config) => match cli::sync(&config, selection, &mut io::stderr().lock()) {
        Ok(report) if report.has_failure() => ExitCode::FAILURE,
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => fail(error),
    },
    Err(error) => fail(error),
},
```

- [ ] **Step 6: Update the binary's integration tests**

In `tests/sync_cli.rs`, make `sync_under` take extra arguments and configure a fake Skills command:

```rust
fn sync_under(home: &Path, sources: &Path, extra: &[&str]) -> Output {
    use std::os::unix::fs::PermissionsExt as _;
    fs::create_dir_all(home.join(".config/kb")).unwrap();
    let skills = home.join("skills");
    fs::write(&skills, "#!/bin/sh\necho '[]'\n").unwrap();
    fs::set_permissions(&skills, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(
        home.join(".config/kb/config.toml"),
        format!(
            "sources_path = \"{}\"\nskills_command = \"{}\"\n",
            sources.display(),
            skills.display()
        ),
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_kb"))
        .arg("--sync")
        .args(extra)
        .env("HOME", home)
        .output()
        .unwrap()
}
```

Pass `&[]` in the two existing calls, change the first test's assertion to
`assert!(stderr.contains("kb: yaml: indexed "), "{stderr}");`, and add:

```rust
#[test]
fn refreshes_one_source_and_rejects_an_unknown_one() {
    let home = tempfile::tempdir().unwrap();

    let output = sync_under(home.path(), &home.path().join("nowhere"), &["--source", "skills"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(stderr.contains("kb: skills: indexed 0 entries"), "{stderr}");
    assert!(!stderr.contains("kb: yaml:"), "{stderr}");

    let output = sync_under(home.path(), &home.path().join("nowhere"), &["--source", "both"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage: kb"));
}
```

- [ ] **Step 7: Run the CLI tests**

Run: `cargo nextest run -p kb-app cli sync_cli`
Expected: PASS; `fails_when_the_sources_directory_is_missing` still fails the process and names "cannot read the
sources directory".

- [ ] **Step 8: Commit**

```bash
git add crates/kb-app/src/cli.rs crates/kb-app/src/main.rs crates/kb-app/tests/sync_cli.rs
git commit -m "feat(cli): Refresh one source with kb --sync --source"
```

---

### Task 10: The catalog: Skills, an optional hero, the scoped refresh actions

**Files:**

- Modify: `crates/kb-app/assets/catalog.toml`
- Modify: `crates/kb-app/src/catalog.rs`
- Modify: `crates/kb-app/src/ui/entry_view.rs:54,164`
- Modify: `crates/kb-app/src/entry_list.rs:245`

- [ ] **Step 1: Write the failing test** (in `catalog.rs`'s tests)

```rust
#[test]
fn ships_skills_without_a_hero_and_the_scoped_refreshes() {
    let catalog = Catalog::default();
    let skills = catalog.entry_type("skills").unwrap();
    assert_eq!(
        (skills.label.as_str(), skills.icon.as_str(), skills.tint.as_str()),
        ("Skills", "notebook-text", "yellow")
    );
    assert_eq!((skills.hero, skills.primary.as_str(), skills.secondary.as_deref()), (None, "copy", None));
    assert_eq!(catalog.label("refresh-yaml"), "Refresh YAML");
    assert_eq!(catalog.label("refresh-skills"), "Refresh Skills");
    assert_eq!(catalog.panel.refresh_more, "Refresh More…");
    assert_eq!(catalog.panel.more_refresh, ["refresh-yaml", "refresh-skills"]);
}
```

Run: `cargo nextest run -p kb-app ships_skills`
Expected: FAIL to compile (`refresh_more` unknown).

- [ ] **Step 2: Extend the catalog file**

In `catalog.toml`:

1. Types comment: "`hero` is media, command, snippet or task; a type without one shows none."
2. After `[types.tasks]`:

```toml
[types.skills]
label = "Skills"
icon = "notebook-text"
tint = "yellow"
primary = "copy"
```

3. After `[actions.refresh]`:

```toml
[actions.refresh-yaml]
label = "Refresh YAML"

[actions.refresh-skills]
label = "Refresh Skills"
```

4. Panel comment: "…More… lists `more_copy` under `copy_section`, then `more_admin` under `admin_section`, with
   `refresh_more` after Refresh opening `more_refresh`." and in `[panel]` after `more = "More…"` add
   `refresh_more = "Refresh More…"`, after `more_admin` add `more_refresh = ["refresh-yaml", "refresh-skills"]`.
5. `[messages]`:
   - `no_entries = "Your sources have no bookmarks, cheat sheets, commands, tasks or skills."`
   - after `refreshing_progress`: `running_skills = "Running skills…"`
   - after `report_empty_file` (`report_summary` changes in Task 12, with the code that fills it):

```toml
report_yaml_section = "YAML"
report_skills_section = "Skills"
report_source_failed = "{error} The previous entries are unchanged."
report_not_configured = "Not configured: install skills, or set skills_command in Configure."
report_warning_chip = "warning"
report_not_configured_chip = "off"
```

- `stopped = "Sources not refreshed yet are unchanged."` (an `all` refresh stopped during Skills keeps the YAML it
  committed)
- after `links_origin`: `refresh_origin = "Refresh More"`

- [ ] **Step 3: Extend the catalog types**

In `catalog.rs`:

1. `ACTION_IDS: [&str; 22]`, with `"refresh-yaml", "refresh-skills",` after `"refresh",`.
2. `EntryType.hero`:

```rust
/// What the entry view's hero shows; none when absent.
#[serde(default)]
pub hero: Option<Hero>,
```

3. `Panel`: after `more` add

```rust
/// The Refresh More… row, after Refresh in More….
pub refresh_more: String,
```

and after `more_admin`

```rust
/// Refresh More…'s rows.
pub more_refresh: Vec<String>,
```

4. `Messages`: add `pub running_skills: String,` after `refreshing_progress`; `report_yaml_section`,
   `report_skills_section`, `report_source_failed`, `report_not_configured`, `report_warning_chip`,
   `report_not_configured_chip` after `report_empty_file`; `pub refresh_origin: String,` after `links_origin`.
5. `validate`: chain `.chain(&self.panel.more_refresh)` into the panel ids loop.

- [ ] **Step 4: Follow the optional hero**

- `ui/entry_view.rs:54`: `match kind.and_then(|k| k.hero) {`
- `ui/entry_view.rs:164`: `.filter(|k| k.hero == Some(Hero::Media))`
- `entry_list.rs:245`: `kind.hero == Some(Hero::Media),`

- [ ] **Step 5: Run the app's unit tests**

Run: `cargo nextest run -p kb-app`
Expected: PASS. The icon check passes: `NotebookText` is already bundled.

- [ ] **Step 6: Commit**

```bash
git add crates/kb-app/assets/catalog.toml crates/kb-app/src/catalog.rs crates/kb-app/src/ui/entry_view.rs crates/kb-app/src/entry_list.rs
git commit -m "feat(catalog): Add the Skills type and the scoped refresh actions"
```

---

### Task 11: Refresh More… in the action panel

**Files:**

- Modify: `crates/kb-app/src/actions.rs`
- Modify: `crates/kb-app/src/search_view/panel.rs:18,87`
- Modify: `crates/kb-app/src/search_view.rs` (`back`)

- [ ] **Step 1: Write the failing tests** (in `actions.rs`'s tests; replace `copies_and_administers_from_more`)

```rust
    #[test]
    fn copies_and_administers_from_more() {
        let catalog = Catalog::default();
        let page = more_page(&catalog, Some(&bookmark()));
        assert_eq!(page.page.crumb.as_deref(), Some("More"));
        assert_eq!(
            labels(&page),
            [
                vec!["Description", "Notes", "Path"],
                vec!["Edit", "Refresh", "Refresh More…", "Configure"]
            ]
        );
        assert_eq!(page.command(0, 1), Some(&PanelCommand::Copy("N".into())));
        assert_eq!(page.command(1, 2), Some(&PanelCommand::Open(Submenu::Refresh)));
        assert_eq!(
            labels(&more_page(&catalog, None)),
            [["Edit", "Refresh", "Refresh More…", "Configure"]]
        );
    }

    #[test]
    fn lists_the_scoped_refreshes() {
        let catalog = Catalog::default();
        let page = refresh_page(&catalog);
        assert_eq!(page.page.crumb.as_deref(), Some("Refresh More"));
        assert_eq!(labels(&page), [["Refresh YAML", "Refresh Skills"]]);
        assert_eq!(page.command(0, 1), Some(&PanelCommand::Action("refresh-skills".into())));
    }

    #[test]
    fn finds_the_scoped_refreshes_from_the_root() {
        let catalog = Catalog::default();
        let page = search_page(&catalog, Screen::List, None);
        let row = page
            .page
            .sections
            .iter()
            .flat_map(|s| &s.rows)
            .find(|r| r.label == "Refresh Skills")
            .unwrap();
        assert_eq!(row.origin.as_deref(), Some("Refresh More"));
    }
```

Run: `cargo nextest run -p kb-app actions::tests`
Expected: FAIL to compile (`Submenu::Refresh`, `refresh_page`).

- [ ] **Step 2: Add the submenu**

In `actions.rs`:

```rust
/// A submenu of the root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Submenu {
    /// One row per link.
    Links,
    /// COPY and ADMINISTRATION.
    More,
    /// The scoped refreshes, from More….
    Refresh,
}
```

In `more_page`, replace the `admin` binding with:

```rust
let mut admin = Vec::new();
for id in &catalog.panel.more_admin {
    admin.push(action_row(catalog, id));
    if id == "refresh" && !catalog.panel.more_refresh.is_empty() {
        admin.push((
            PanelRow {
                label: catalog.panel.refresh_more.clone(),
                submenu: true,
                ..PanelRow::default()
            },
            PanelCommand::Open(Submenu::Refresh),
        ));
    }
}
```

Add after `more_page`:

```rust
/// Refresh More…: one row per scoped refresh (spec D1).
#[must_use]
pub fn refresh_page(catalog: &Catalog) -> Page {
    let mut page = Page::default();
    page.page.crumb = Some(catalog.panel.refresh_more.trim_end_matches('…').to_owned());
    let rows = catalog
        .panel
        .more_refresh
        .iter()
        .map(|id| action_row(catalog, id))
        .collect();
    page.section(None, rows);
    page
}
```

In `search_page`, after `marked(more_page(…), …);` add
`marked(refresh_page(catalog), &catalog.messages.refresh_origin);` and update its doc: "then More…'s, Refresh
More…'s and Links…'s rows, marked with their submenu (`more_origin`, `refresh_origin`, `links_origin`)."

- [ ] **Step 3: Open it and come back from it**

- `search_view/panel.rs:18`: add `refresh_page` to the `crate::actions` import; after the `Submenu::More` arm of
  `current_page` add `Overlay::Actions(Some(Submenu::Refresh)) => refresh_page(&self.catalog),`.
- `search_view.rs`, `back`: Esc from Refresh More… returns to More…:

```rust
let next = match self.overlay {
    Overlay::Actions(Some(crate::actions::Submenu::Refresh)) => {
        Overlay::Actions(Some(crate::actions::Submenu::More))
    }
    Overlay::Actions(Some(_)) => Overlay::Actions(None),
    _ => Overlay::None,
};
```

- [ ] **Step 4: Update and extend the UI test**

In `tests/ui.rs`, `clears_the_search_then_leaves_more_then_closes_with_escape`: the More… labels become
`[vec!["Notes", "Path"], vec!["Edit", "Refresh", "Refresh More…", "Configure"]]`. Add:

```rust
#[gpui_kit::test]
fn opens_refresh_more_and_returns_to_more_with_escape(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "rebase");
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    type_text(cx, window, "Refresh More");
    cx.run_until_parked();
    press(cx, window, "enter");
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| view.read(cx).panel_labels(cx)),
        [["Refresh YAML", "Refresh Skills"]]
    );
    press(cx, window, "escape");
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| view.read(cx).panel_labels(cx))[1],
        ["Edit", "Refresh", "Refresh More…", "Configure"]
    );
}
```

- [ ] **Step 5: Run the tests**

Run: `cargo nextest run -p kb-app actions refresh_more leaves_more`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/kb-app/src/actions.rs crates/kb-app/src/search_view/panel.rs crates/kb-app/src/search_view.rs crates/kb-app/tests/ui.rs
git commit -m "feat(ui): Open Refresh More… from More… in the action panel"
```

---

### Task 12: The refresh screen runs a selection and reports per source

**Files:**

- Modify: `crates/kb-app/src/sync_view.rs`
- Modify: `crates/kb-app/src/search_view.rs` (`rebuild`, `run_action`, a test accessor)
- Modify: `crates/kb-app/tests/ui.rs` (`sample_config`, a new test)

- [ ] **Step 1: Write the failing report tests** (in `sync_view.rs`'s tests; replace `summarizes_a_sync`)

```rust
    use kb_domain::sync::SyncSummary;

    fn yaml_summary(files: Vec<FileReport>, total: usize) -> SyncSummary {
        SyncSummary {
            total_indexed: total,
            files_scanned: files.len(),
            files_failed: files.iter().filter(|f| f.error.is_some()).count(),
            duration_ms: 0,
            errors: Vec::new(),
            files,
        }
    }

    #[test]
    fn summarizes_every_source() {
        let report = SyncReport {
            yaml: Some(YamlOutcome::Indexed(yaml_summary(vec![file("errors/x.yml", 0, true, false)], 4233))),
            skills: Some(SkillsOutcome::Indexed {
                entries: 438,
                warnings: Vec::new(),
            }),
            duration_ms: 812,
        };
        let catalog = Catalog::default();
        assert_eq!(
            summary_line(&report, &catalog.messages),
            "4,671 entries in 0.8 s · 1 failed"
        );
    }

    #[test]
    fn reports_skills_in_a_section_of_their_own() {
        let m = Catalog::default().messages;
        let report = SyncReport {
            yaml: None,
            skills: Some(SkillsOutcome::Failed {
                reason: "skills exited with exit status: 1".into(),
                warnings: vec!["⚠ Skipped recon".into()],
            }),
            duration_ms: 0,
        };
        let sections = report_sections(&report, "/s", &m);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].label, "Skills");
        let kinds: Vec<ReportKind> = sections[0].rows.iter().map(|r| r.kind).collect();
        assert_eq!(kinds, [ReportKind::Failed, ReportKind::Warning]);
        assert_eq!(
            sections[0].rows[0].detail,
            "skills exited with exit status: 1 The previous entries are unchanged."
        );
        assert_eq!(sections[0].rows[1].detail, "⚠ Skipped recon");
        assert_eq!(summary_line(&report, &m), "0 entries in 0.0 s · 1 failed");
    }

    #[test]
    fn reports_yaml_files_then_skills_not_configured() {
        let m = Catalog::default().messages;
        let report = SyncReport {
            yaml: Some(YamlOutcome::Indexed(yaml_summary(vec![file("a.yml", 3, false, false)], 3))),
            skills: Some(SkillsOutcome::NotConfigured),
            duration_ms: 0,
        };
        let sections = report_sections(&report, "/s", &m);
        let labels: Vec<&str> = sections.iter().map(|s| s.label.as_str()).collect();
        assert_eq!(labels, ["YAML", "Skills"]);
        assert_eq!(sections[0].rows[0].path, "a.yml");
        assert_eq!(sections[1].rows[0].kind, ReportKind::NotConfigured);
    }

    #[test]
    fn reports_an_unreadable_sources_directory_as_a_failed_yaml_row() {
        let m = Catalog::default().messages;
        let report = SyncReport {
            yaml: Some(YamlOutcome::Failed("cannot read the sources directory: gone".into())),
            skills: None,
            duration_ms: 0,
        };
        let row = &report_sections(&report, "/s", &m)[0].rows[0];
        assert_eq!((row.path.as_str(), row.kind), ("/s", ReportKind::Failed));
    }
```

Run: `cargo nextest run -p kb-app sync_view`
Expected: FAIL to compile (`report_sections`, `ReportKind::Warning`).

- [ ] **Step 2: Report sections**

In `catalog.toml`'s `[messages]`, set `report_summary = "{entries} entries in {seconds} s"` (the files count moves to
the YAML section's rows).

In `sync_view.rs`:

1. Imports: replace `use kb_domain::sync::{SyncError, SyncSummary, sync_knowledge_base};` with

```rust
use kb_domain::selection::Selection;
use kb_domain::sync::{Progress, SkillsOutcome, SyncError, SyncReport, SyncRequest, YamlOutcome, sync_sources};
```

and add `use crate::catalog::Messages;`. The tests import `SyncSummary` themselves (Step 1).

2. `ReportKind`: add two variants after `Indexed`:

```rust
/// Something the Skills command or a Skill's `SKILL.md` warned about.
Warning,
/// The Skills command is not configured.
NotConfigured,
```

3. Add after `report_rows`:

```rust
/// One source's rows in the report, under its label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportSection {
    /// `YAML` or `Skills`.
    pub label: String,
    /// Its rows.
    pub rows: Vec<ReportRow>,
}

/// The report's sections, one per refreshed source (spec, "Refresh report and progress"):
/// YAML's files, or one failed row naming `sources_dir`; Skills' outcome row, then its warnings.
#[must_use]
pub fn report_sections(report: &SyncReport, sources_dir: &str, m: &Messages) -> Vec<ReportSection> {
    let mut sections = Vec::new();
    if let Some(yaml) = &report.yaml {
        let rows = match yaml {
            YamlOutcome::Indexed(summary) => report_rows(&summary.files),
            YamlOutcome::Failed(reason) => vec![ReportRow {
                path: sources_dir.to_owned(),
                kind: ReportKind::Failed,
                entries: 0,
                detail: fill(&m.report_source_failed, &[("error", reason)]),
            }],
        };
        sections.push(ReportSection {
            label: m.report_yaml_section.clone(),
            rows,
        });
    }
    if let Some(skills) = &report.skills {
        let row = |kind, entries, detail: String| ReportRow {
            path: kb_domain::skills::SKILLS_TYPE.to_owned(),
            kind,
            entries,
            detail,
        };
        let (outcome, warnings): (ReportRow, &[String]) = match skills {
            SkillsOutcome::NotConfigured => (
                row(ReportKind::NotConfigured, 0, m.report_not_configured.clone()),
                &[],
            ),
            SkillsOutcome::Indexed { entries, warnings } => (
                row(ReportKind::Indexed, *entries, String::new()),
                warnings.as_slice(),
            ),
            SkillsOutcome::Failed { reason, warnings } => (
                row(
                    ReportKind::Failed,
                    0,
                    fill(&m.report_source_failed, &[("error", reason)]),
                ),
                warnings.as_slice(),
            ),
        };
        let mut rows = vec![outcome];
        rows.extend(warnings.iter().map(|w| row(ReportKind::Warning, 0, w.clone())));
        sections.push(ReportSection {
            label: m.report_skills_section.clone(),
            rows,
        });
    }
    sections
}
```

4. Replace `summary_line`:

```rust
/// Summarizes a sync from the catalog's message template: entries from every source, then the
/// failed files and sources, then the files with skipped entries.
#[must_use]
pub fn summary_line(report: &SyncReport, messages: &Messages) -> String {
    let rows: Vec<ReportRow> = report_sections(report, "", messages)
        .into_iter()
        .flat_map(|s| s.rows)
        .collect();
    let failed = rows.iter().filter(|r| r.kind == ReportKind::Failed).count();
    let skipped = rows.iter().filter(|r| r.kind == ReportKind::Skipped).count();
    let mut line = fill(
        &messages.report_summary,
        &[
            ("entries", &thousands(report.total_indexed())),
            (
                "seconds",
                &format!(
                    "{:.1}",
                    Duration::from_millis(u64::try_from(report.duration_ms).unwrap_or(u64::MAX)).as_secs_f64()
                ),
            ),
        ],
    );
    if failed > 0 {
        let _ = write!(
            line,
            " · {}",
            fill(&messages.report_failed, &[("count", &failed.to_string())])
        );
    }
    if skipped > 0 {
        let _ = write!(
            line,
            " · {}",
            fill(&messages.report_skipped, &[("count", &skipped.to_string())])
        );
    }
    line
}
```

The crate still does not compile: `SyncStatus::Done` carries a `SyncSummary` until Step 3. Go on to Step 3, then run
the tests in Step 6.

- [ ] **Step 3: Run a selection; show Skills progress**

In `sync_view.rs`:

1. Add before `SyncStatus`:

```rust
/// What a refresh runs.
#[derive(Debug, Clone)]
pub struct SyncJob {
    /// The sources to refresh.
    pub selection: Selection,
    /// The YAML sources directory.
    pub sources: PathBuf,
    /// The Skills command; `None` when it is not configured.
    pub skills_command: Option<PathBuf>,
    /// The index.
    pub database: PathBuf,
}
```

2. `SyncStatus`: add after `Running { … }`

```rust
/// The Skills command is running.
RunningSkills,
```

and make `Done` carry `SyncReport`: `Done(SyncReport),` ("The selected sources were refreshed.").
3. `SyncFinished`: rename `rebuilt` to `changed`, documented "Whether the index may have changed: it finished, or
it stopped after committing a source."
4. `SyncView`: add a `job: SyncJob` field.
5. `SyncTick` becomes an enum:

```rust
/// One progress report, owned, sent from the sync thread.
enum SyncTick {
    File {
        files_scanned: usize,
        files_total: usize,
        current: String,
    },
    Skills,
}
```

6. Replace `start` with:

```rust
    /// Starts refreshing `job`'s sources.
    pub fn start(job: SyncJob, catalog: Rc<Catalog>, cx: &mut Context<Self>) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let run_job = job.clone();
        let run = cx.spawn(async move |view, cx| {
            let (sender, mut progress) = mpsc::unbounded::<SyncTick>();
            let task = cx.background_spawn(async move {
                let request = SyncRequest {
                    selection: run_job.selection,
                    sources_dir: &run_job.sources,
                    skills_command: run_job.skills_command.as_deref(),
                };
                sync_sources(&request, &run_job.database, &flag, |p| {
                    let _ = sender.unbounded_send(match p {
                        Progress::Yaml(p) => SyncTick::File {
                            files_scanned: p.files_scanned,
                            files_total: p.files_total,
                            current: p.file.rel_path.clone(),
                        },
                        Progress::Skills => SyncTick::Skills,
                    });
                })
            });
            while let Some(tick) = progress.next().await {
                let _ = view.update(cx, |view, cx| {
                    view.status = match tick {
                        SyncTick::File {
                            files_scanned,
                            files_total,
                            current,
                        } => SyncStatus::Running {
                            files_scanned,
                            files_total,
                            current,
                        },
                        SyncTick::Skills => SyncStatus::RunningSkills,
                    };
                    cx.notify();
                });
            }
            let outcome = task.await;
            let _ = view.update(cx, |view, cx| {
                let changed = matches!(outcome, Ok(_) | Err(SyncError::Cancelled));
                view.status = match outcome {
                    Ok(report) => SyncStatus::Done(report),
                    Err(SyncError::Cancelled) => SyncStatus::Cancelled,
                    Err(error) => SyncStatus::Failed(error.to_string()),
                };
                let m = &view.catalog.messages;
                let message = match &view.status {
                    SyncStatus::Done(report) => summary_line(report, m),
                    SyncStatus::Cancelled => m.stopped_title.clone(),
                    SyncStatus::Failed(error) => fill(&m.failed, &[("error", error)]),
                    SyncStatus::Running { .. } | SyncStatus::RunningSkills => String::new(),
                };
                cx.emit(SyncFinished { changed, message });
                cx.notify();
            });
        });
        let status = SyncStatus::Running {
            files_scanned: 0,
            files_total: 0,
            current: String::new(),
        };
        Self {
            status,
            catalog,
            cancel,
            job,
            _run: run,
        }
    }

    /// The sources this refresh runs: Retry runs them again.
    #[must_use]
    pub fn selection(&self) -> Selection {
        self.job.selection
    }
```

7. `is_running`: `matches!(self.status, SyncStatus::Running { .. } | SyncStatus::RunningSkills)`; `phase`: add
   `SyncStatus::RunningSkills => Phase::Running,`.
8. `render`: add an arm after `Running { … }`:

```rust
SyncStatus::RunningSkills => EmptyView::new(
    "refresh-cw",
    c.faint,
    m.refreshing_title.clone(),
    Sentence::of(&m.running_skills, ""),
)
.into_any_element(),
```

and replace the `Done(summary)` arm's body so it draws a section label before each section's rows, numbering rows
across sections:

```rust
SyncStatus::Done(report) => {
    let sources = self.job.sources.display().to_string();
    let sections = report_sections(report, &sources, m);
    let line = summary_line(report, m);
    let s = &d.spacing.empty;
    let mut ix = 0;
    let mut children: Vec<gpui_kit::AnyElement> = Vec::new();
    for section in sections {
        children.push(
            div()
                .px(pt(s.summary_pad_x))
                .pt(pt(s.summary_pad_top))
                .text_role(&d.typography.roles.section)
                .text_color(c.faint)
                .child(section.label)
                .into_any_element(),
        );
        for row in section.rows {
            let (icon, tint, chip, detail) = match row.kind {
                ReportKind::Failed => ("circle-x", c.red, m.report_failed_chip.clone(), row.detail.clone()),
                ReportKind::Skipped => (
                    "triangle-alert",
                    c.yellow,
                    row.entries.to_string(),
                    // A file with no entries at all has no skipped entries to name.
                    if row.detail.is_empty() {
                        m.report_empty_file.clone()
                    } else {
                        fill(&m.report_skipped_entries, &[("entries", &row.detail)])
                    },
                ),
                ReportKind::Indexed => (
                    "circle-check",
                    c.green,
                    row.entries.to_string(),
                    m.report_indexed.clone(),
                ),
                ReportKind::Warning => (
                    "triangle-alert",
                    c.yellow,
                    m.report_warning_chip.clone(),
                    row.detail.clone(),
                ),
                ReportKind::NotConfigured => (
                    "circle",
                    c.faint,
                    m.report_not_configured_chip.clone(),
                    row.detail.clone(),
                ),
            };
            children.push(
                Row::new(SharedString::from(format!("report-{ix}")), row.path.clone())
                    .lead(tile(icon, tint, TileSize::Row))
                    .mono(true)
                    .description(Some(detail))
                    .trailing(vec![pill(chip, ChipState::of(ix == 0)).into_any_element()])
                    .selected(ix == 0)
                    .into_any_element(),
            );
            ix += 1;
        }
    }
    v_flex()
        .id("report")
        .size_full()
        .overflow_y_scroll()
        .px(pt(d.spacing.list.pad))
        .child(
            div()
                .px(pt(s.summary_pad_x))
                .pt(pt(s.summary_pad_top))
                .pb(pt(s.summary_pad_bottom))
                .text_role(&d.typography.roles.description)
                .text_color(c.muted)
                .child(line),
        )
        .children(children)
        .into_any_element()
}
```

`IntoElement` is already imported, so `.into_any_element()` works on `div()` and `Row`; add `AnyElement` to the
`gpui_kit` import instead of the `gpui_kit::AnyElement` path if you prefer. The `circle` icon is bundled
(`ExtraIcons`).

- [ ] **Step 4: Start, retry and scope refreshes from the search view**

In `search_view.rs`:

1. Import `crate::sync_view::SyncJob` beside `SyncView`, and `kb_domain::selection::Selection`.
2. Replace `rebuild` with:

```rust
    fn rebuild(&mut self, _: &Rebuild, window: &mut Window, cx: &mut Context<Self>) {
        self.refresh(Selection::All, window, cx);
    }

    /// Refreshes `selection`'s sources on the refresh screen, unless a refresh is running.
    fn refresh(&mut self, selection: Selection, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(&self.screen, Screen::Rebuild(sync) if sync.read(cx).is_running()) {
            return;
        }
        let job = SyncJob {
            selection,
            sources: self.config.sources_dir().to_path_buf(),
            skills_command: self.config.skills_command(),
            database: self.config.database().to_path_buf(),
        };
        let catalog = self.catalog.clone();
        let sync = cx.new(|cx| SyncView::start(job, catalog, cx));
        let subscription = cx.subscribe_in(&sync, window, |this, _, finished: &SyncFinished, window, cx| {
            window.push_notification(finished.message.clone(), cx);
            if finished.changed {
                let connection = open_readonly(this.config.database()).ok();
                this.list.update(cx, |list, cx| {
                    list.delegate_mut().set_connection(connection);
                    cx.notify();
                });
            }
        });
        self.subscriptions.push(subscription);
        self.screen = Screen::Rebuild(sync);
        self.focus.focus(window, cx);
        cx.notify();
    }

    /// The sources of the refresh on screen, for tests.
    #[must_use]
    pub fn refresh_selection(&self, cx: &App) -> Option<Selection> {
        match &self.screen {
            Screen::Rebuild(sync) => Some(sync.read(cx).selection()),
            _ => None,
        }
    }
```

3. `run_action`: after `("refresh", _) => …` add

```rust
("refresh-yaml", _) => self.refresh(Selection::Yaml, window, cx),
("refresh-skills", _) => self.refresh(Selection::Skills, window, cx),
```

and make `retry` refresh the same sources:

```rust
("retry", _) => {
    // A finished or failed refresh lets `refresh` start a new one, of the same sources.
    let selection = match &self.screen {
        Screen::Rebuild(sync) => sync.read(cx).selection(),
        _ => Selection::All,
    };
    self.refresh(selection, window, cx);
}
```

- [ ] **Step 5: Keep UI tests off the machine's `skills`, and test the scoped refresh**

In `tests/ui.rs`, `sample_config` writes a fake Skills command beside the preferences:

```rust
let preferences = home.join(".config/kb/config.toml");
fs::create_dir_all(preferences.parent().unwrap()).unwrap();
// A Skills command that lists no skills, so a refresh never runs the machine's own `skills`.
let skills = home.join("skills");
fs::write(&skills, "#!/bin/sh\necho '[]'\n").unwrap();
{
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(&skills, fs::Permissions::from_mode(0o755)).unwrap();
}
fs::write(
    &preferences,
    format!(
        "sources_path = \"{}\"\nskills_command = \"{}\"\n",
        sample.display(),
        skills.display()
    ),
)
.unwrap();
```

Add:

```rust
#[gpui_kit::test]
fn refreshes_skills_alone_from_the_action_panel(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    type_text(cx, window, "Refresh Skills");
    cx.run_until_parked();
    press(cx, window, "enter");
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| view.read(cx).refresh_selection(cx)),
        Some(kb_domain::selection::Selection::Skills)
    );
}
```

- [ ] **Step 6: Run the app's tests and clippy**

Run: `cargo nextest run -p kb-app && cargo clippy -p kb-app --all-targets -- -D warnings`
Expected: PASS, no warnings (including `summarizes_every_source`, which Task 10 left failing).

- [ ] **Step 7: Commit**

```bash
git add crates/kb-app/src/sync_view.rs crates/kb-app/src/search_view.rs crates/kb-app/tests/ui.rs
git commit -m "feat(ui): Refresh a source alone and report each source apart"
```

---

### Task 13: Vocabulary, design notes and the gate

**Files:**

- Modify: `CONTEXT.md`
- Modify: `DESIGN.md` (the accents row)
- Modify: `TODO.md` (through the `todo` CLI)

- [ ] **Step 1: Update CONTEXT.md**

Edit these rows of the first table (keep the table aligned; `hk fix --all` reformats it):

- **Entry type**: "The top-level YAML key an entry sits under, or `skills` for a Skills record. The app shows five —
  Bookmarks, Cheat Sheets, Commands, Tasks, Skills (the catalog's `types`); the engine keeps every key." Lives in:
  add `crates/kb-domain/src/skills.rs`.
- **Source file**: append "A Skills entry's source file is its `SKILL.md`."
- **Index**: "`kb.db`: `entries` (each row's `source`: `yaml` or `skills`), the `entries_fts` FTS5 table,
  `entry_tags`, in the app's own support directory; `PRAGMA user_version` migrates it."
- **Sync**: "The manual refresh of the index from its sources: all of them (Refresh, ⌘R), or one (Refresh YAML,
  Refresh Skills, under More… → Refresh More…). Each source is its own transaction; cancellable." Lives in: add
  `crates/kb-domain/src/selection.rs`.
- **Sync report**: "What a sync says per source: for YAML, each source file's entries indexed, skipped, or why it
  failed; for Skills, entries indexed, why the command failed, or not configured, and its warnings."

Add a row after **Source file**:

```markdown
| Skills source | The records of `skills ls --global --agent universal --json` (`skills_command` in `~/.config/kb/config.toml`, else `skills` found on `PATH` or in `/opt/homebrew/bin`, `/usr/local/bin`, `~/.local/bin`); each becomes a `skills` entry keyed `/<name>`. | `crates/kb-domain/src/skills.rs`, `crates/kb-app/src/command_path.rs` |
```

- [ ] **Step 2: Update DESIGN.md's accents row**

In the accents row (around line 142), change `` `yellow` `` to `` `yellow` (Skills) ``.

- [ ] **Step 3: Run the gate**

Run: `hk fix --all && mise run gate`
Expected: every step passes (nextest, clippy `-D warnings`, release build, `hk check --all`, `cargo deny`, docs).

- [ ] **Step 4: Check it by hand** (the real `skills` takes about 8 s)

Run: `cargo run --release -p kb-app -- --sync --source skills`
Expected: `kb: skills: warning: ⚠ Skipped …recon/SKILL.md …` (if that Skill is still broken),
`kb: skills: indexed 438 entries` (or the current count), exit 0. Then `cargo run --release -p kb-app -- --sync
--source yaml` and confirm the Skills count in the index is unchanged:
`sqlite3 ~/Library/Application\ Support/kb-native/kb.db "SELECT source, count(*) FROM entries GROUP BY source"`.

- [ ] **Step 5: Tick the task**

Find the `todo` CLI's usage in `TODO.md`'s REF comments (`grep -n REF TODO.md`), mark `Rkpg` done with it, and add
a dated comment linking this plan.

- [ ] **Step 6: Commit**

```bash
git add CONTEXT.md DESIGN.md TODO.md
git commit -m "docs: Name the Skills source and the scoped refresh"
```

---

## Spec coverage

| Spec                                        | Task                     |
| ------------------------------------------- | ------------------------ |
| D1 Refresh More…, Refresh YAML / Skills     | 10, 11, 12               |
| D2, D3 `skills` entries, key `/<name>`      | 4, 5                     |
| D4 separate adapters behind sync            | 4–7                      |
| D5 failures keep prior entries              | 7                        |
| D6 `sourceUrl` as link, Edit opens SKILL.md | 5 (links, `source_file`) |
| D7 `skills_command`, fixed args             | 4 (`SKILLS_ARGS`), 8     |
| D8 / Schema and migration                   | 1, 3                     |
| Finding the command                         | 8                        |
| Records, entry mapping, warnings            | 4, 5                     |
| Sync behaviour, order, cancellation         | 3, 6, 7                  |
| Refresh report and progress, CLI output     | 9, 12                    |
| Catalog (type, optional hero, icon)         | 10                       |
| CLI and configuration                       | 8, 9                     |
| Documentation                               | 13                       |
| Tests and validation                        | each task; gate in 13    |
