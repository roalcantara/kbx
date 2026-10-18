//! Sync: rebuilding the index from the source files.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use rusqlite::{Connection, ErrorCode, Statement, Transaction, params};

use crate::entry::{Entry, normalize_entry};
use crate::markdown::build_entry_markdown;
use crate::report::{FileReport, file_report};
use crate::row::entry_to_row;
use crate::selection::{Selection, SourceKind};
use crate::skills::command::{CommandError, run_command};
use crate::skills::{SKILLS_ARGS, parse_skills, skill_entry};
use crate::source::{SourceError, find_source_files, parse_source_file};

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

/// Why a sync stopped.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    /// The sources directory could not be read.
    #[error("cannot read the sources directory: {0}")]
    Sources(#[from] std::io::Error),
    /// The index's directory could not be created.
    #[error("cannot create the index directory: {0}")]
    IndexDirectory(std::io::Error),
    /// SQLite failed.
    #[error(transparent)]
    Database(#[from] rusqlite::Error),
    /// The caller cancelled; the index is unchanged.
    #[error("sync cancelled")]
    Cancelled,
}

/// What a sync reports after each file.
#[derive(Debug, Clone, Copy)]
pub struct SyncProgress<'a> {
    /// Files processed so far.
    pub files_scanned: usize,
    /// Files in all.
    pub files_total: usize,
    /// Entries indexed so far.
    pub entries_count: usize,
    /// The file just processed.
    pub file: &'a FileReport,
}

/// A finished sync.
#[derive(Debug, Clone)]
pub struct SyncSummary {
    /// Entries indexed.
    pub total_indexed: usize,
    /// Source files seen.
    pub files_scanned: usize,
    /// Source files that failed.
    pub files_failed: usize,
    /// How long it took.
    pub duration_ms: u128,
    /// One error per failed file.
    pub errors: Vec<SourceError>,
    /// Every file, in processing order.
    pub files: Vec<FileReport>,
}

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

/// Opens the index with foreign keys on.
///
/// # Errors
///
/// An SQLite error.
pub fn open_database(path: &Path) -> rusqlite::Result<Connection> {
    let connection = Connection::open(path)?;
    connection.pragma_update(None, "foreign_keys", true)?;
    Ok(connection)
}

/// Opens the index read-only, as the app reads it: a sync writes through its own connection.
///
/// # Errors
///
/// An SQLite error, such as a missing file.
pub fn open_readonly(path: &Path) -> rusqlite::Result<Connection> {
    Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sources(files: &[(&str, &str)]) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        for (path, content) in files {
            fs::write(root.path().join(path), content).unwrap();
        }
        root
    }

    fn count(path: &Path, table: &str) -> i64 {
        open_database(path)
            .unwrap()
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn indexes_entries_tags_and_full_text() {
        let root = sources(&[
            ("a.yml", "commands:\n  docker ps: {tags: [docker, Ops]}\n  ls: {}\n"),
            ("b.yml", "x: [\n"),
        ]);
        let db = tempfile::tempdir().unwrap();
        let path = db.path().join("nested/kb.db");
        let mut seen = Vec::new();
        let summary = sync_knowledge_base(root.path(), &path, &AtomicBool::new(false), |p| {
            seen.push(p.files_scanned);
        })
        .unwrap();
        assert_eq!(
            (summary.total_indexed, summary.files_scanned, summary.files_failed),
            (2, 2, 1)
        );
        assert_eq!(seen, [1, 2]);
        assert_eq!(count(&path, "entries"), 2);
        assert_eq!(count(&path, "entry_tags"), 2);
        assert_eq!(count(&path, "entries_fts('\"dock\"*')"), 1);
    }

    #[test]
    fn a_cancelled_sync_leaves_the_previous_index() {
        let root = sources(&[("a.yml", "notes:\n  one: {}\n")]);
        let db = tempfile::tempdir().unwrap();
        let path = db.path().join("kb.db");
        sync_knowledge_base(root.path(), &path, &AtomicBool::new(false), |_| {}).unwrap();
        fs::write(root.path().join("a.yml"), "notes:\n  one: {}\n  two: {}\n").unwrap();
        let result = sync_knowledge_base(root.path(), &path, &AtomicBool::new(true), |_| {});
        assert!(matches!(result, Err(SyncError::Cancelled)));
        assert_eq!(count(&path, "entries"), 1);
    }

    #[test]
    fn lists_types_and_tags_from_a_read_only_connection() {
        let root = sources(&[("a.yml", "notes:\n  one: {tags: [b, a]}\ncommands:\n  ls: {tags: [a]}\n")]);
        let db = tempfile::tempdir().unwrap();
        let path = db.path().join("kb.db");
        sync_knowledge_base(root.path(), &path, &AtomicBool::new(false), |_| {}).unwrap();
        let connection = open_readonly(&path).unwrap();
        assert_eq!(crate::search::entry_types(&connection).unwrap(), ["commands", "notes"]);
        assert_eq!(crate::search::tags(&connection).unwrap(), ["a", "b"]);
        assert!(connection.execute("DELETE FROM entries", []).is_err());
    }

    #[test]
    fn replaces_a_corrupt_index() {
        let root = sources(&[("a.yml", "notes:\n  one: {}\n")]);
        let db = tempfile::tempdir().unwrap();
        let path = db.path().join("kb.db");
        fs::write(
            &path,
            "not a database, just text that is long enough to be read as a header....",
        )
        .unwrap();
        let summary = sync_knowledge_base(root.path(), &path, &AtomicBool::new(false), |_| {}).unwrap();
        assert_eq!(summary.total_indexed, 1);
    }

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
        assert_eq!(
            count(&db, "entries_fts('\"agents\"*')"),
            2,
            "descriptions are searchable"
        );

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
            fake_skills(
                home.path(),
                r#"[{"name":"a","path":"/a"},{"name":"a","path":"/b"}]"#,
                "",
                0,
            ),
        ];
        for command in &failing {
            let report = run(Selection::Skills, root.path(), Some(command), &db, false);
            assert!(
                matches!(report.skills, Some(SkillsOutcome::Failed { .. })),
                "{command:?}"
            );
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
}
