# Refresh Report Inspection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The refresh report navigates like the search list (↑↓, → split, ↵ detail, Esc back), shows each row's
pane, and acts on the selected row: Edit (in the configured editor, at the failing line), Copy Location, Copy Error,
Reveal in Finder, and Refresh again.

**Architecture:** Spec: `docs/superpowers/specs/2026-10-27-refresh-report-inspection-design.md` (D1–D8). The engine
gains `FileReport::location()`, a `SkillsWarning` that names its `SKILL.md`, and a failed Skills outcome that keeps its
command and exit code. The app gains `report_list.rs` (a GPUI Kit `ListDelegate` over the report's sections),
`ui/report_pane.rs` (a row's pane, for the split and the detail screen) and `editor.rs` (the `editor` template).
`SyncView` owns the report list and its `pane` / `detail` flags; `SearchView` routes keys and actions to it, as it does
for its own list.

**Tech Stack:** Rust 2024, GPUI Kit 0.7.1 (`gpui-component` list), `shlex` 1 (word splitting, already in
`Cargo.lock`), `cargo nextest`, `mise run gate`.

**Approved pages:** `docs/guides/design/12-report-navigation.html`, `docs/guides/design/12-report-pane.html` (option A).

---

## Ground rules for every task

- Read `AGENTS.md`. Tests first; run them failing, then passing. Commit at the end of each task with
  `git add <paths>` then `git commit` (never `git commit -a`, never `git commit <paths>`). Conventional Commits,
  sentence-case subject ≤ 72 characters, ending with the `Co-Authored-By` line the session gives.
- `kb-domain` never depends on `gpui*` or `kb-app`.
- Run one crate's tests with `cargo nextest run -p <crate> <filter>`; before each commit run
  `cargo clippy --workspace --all-targets -- -D warnings` and `hk fix --all`.
- Words the UI shows live in `crates/kb-app/assets/catalog.toml` `[messages]`, never as literals in Rust (the
  `no-literal-design` ast-grep rule also forbids literal colours and sizes: take them from `design()`).

## File map

| File                                                                                | Change | Responsibility                                                                                             |
| ----------------------------------------------------------------------------------- | ------ | ---------------------------------------------------------------------------------------------------------- |
| `crates/kb-domain/src/report.rs`                                                    | modify | `FileReport::location()`                                                                                   |
| `crates/kb-domain/src/skills.rs`                                                    | modify | `SkillsWarning`; `skill_entry` returns one with its file                                                   |
| `crates/kb-domain/src/sync.rs`                                                      | modify | `SkillsOutcome` carries `SkillsWarning`s; `Failed` keeps `command`, `exit_code`                            |
| `crates/kb-app/src/cli.rs`                                                          | modify | prints `SkillsWarning::message`                                                                            |
| `Cargo.toml`, `crates/kb-app/Cargo.toml`                                            | modify | `shlex = "1"`                                                                                              |
| `crates/kb-app/src/command_path.rs`                                                 | modify | `find_command`: a bare name found on `PATH` and the install folders                                        |
| `crates/kb-app/src/editor.rs`                                                       | create | the `editor` template: check, fill, run, fall back                                                         |
| `crates/kb-app/src/config.rs`                                                       | modify | `editor` setting, checked at load                                                                          |
| `crates/kb-app/src/catalog.rs`, `assets/catalog.toml`                               | modify | actions `copy-error`, `copy-location`, `reveal`; screen `report-detail`; messages; Configure template line |
| `crates/kb-app/src/report_list.rs`                                                  | create | report rows, sections, selection steps, a row's file / location / error; the `ListDelegate`                |
| `crates/kb-app/src/ui/report_pane.rs`                                               | create | a row's pane: content model (tested) and its element                                                       |
| `crates/kb-app/src/sync_view.rs`                                                    | modify | owns the report list, `pane` / `detail`; renders list, split, detail                                       |
| `crates/kb-app/src/keys.rs`                                                         | modify | actions and bindings for the report                                                                        |
| `crates/kb-app/src/actions.rs`                                                      | modify | `report_page`: the ⌘K page for a report row                                                                |
| `crates/kb-app/src/search_view.rs`, `search_view/footer.rs`, `search_view/panel.rs` | modify | routing                                                                                                    |
| `crates/kb-app/tests/ui.rs`                                                         | modify | the report's navigation and actions, end to end                                                            |
| `tools/design/tasks.toml`                                                           | modify | `design:shots` captures the report's split and detail                                                      |
| `docs/guides/parity.md`, `DESIGN.md`, `CONTEXT.md`, `TODO.md`                       | modify | docs                                                                                                       |

---

### Task 1: `FileReport::location()`

**Files:**

- Modify: `crates/kb-domain/src/report.rs` (after `file_report`, and its `tests` module)

- [ ] **Step 1: Write the failing test** — add to `mod tests` in `report.rs`:

```rust
#[test]
fn locates_a_failed_file_at_its_line_and_column() {
    let failed = |line, column| ParsedSourceFile {
        error: Some(SourceError {
            file_path: "/s/x.yml".into(),
            message: "bad".into(),
            line,
            column,
        }),
        ..ParsedSourceFile::default()
    };
    assert_eq!(file_report("/s/x.yml", "x.yml", &failed(Some(4), Some(11))).location(), "/s/x.yml:4:11");
    assert_eq!(file_report("/s/x.yml", "x.yml", &failed(Some(4), None)).location(), "/s/x.yml:4");
    assert_eq!(file_report("/s/x.yml", "x.yml", &failed(None, None)).location(), "/s/x.yml");
    assert_eq!(
        file_report("/s/y.yml", "y.yml", &ParsedSourceFile::default()).location(),
        "/s/y.yml",
        "a file without an error is just its path"
    );
}
```

Add `use crate::source::SourceError;` to the test module's imports if it is not there.

- [ ] **Step 2: Run it and see it fail**

Run: `cargo nextest run -p kb-domain locates_a_failed_file`
Expected: compile error, `no method named location`.

- [ ] **Step 3: Implement** — after `file_report` in `report.rs`:

```rust
impl FileReport {
    /// Where to look: `path:line:column` for an error with a line, else the absolute path.
    #[must_use]
    pub fn location(&self) -> String {
        match self.error.as_ref().and_then(|e| Some((e.line?, e.column))) {
            Some((line, Some(column))) => format!("{}:{line}:{column}", self.file_path),
            Some((line, None)) => format!("{}:{line}", self.file_path),
            None => self.file_path.clone(),
        }
    }
}
```

- [ ] **Step 4: Run it and see it pass**

Run: `cargo nextest run -p kb-domain report`
Expected: PASS.

- [ ] **Step 5: Commit** — `feat(domain): Locate a failed file at its line and column`

---

### Task 2: Skills warnings name their file; a failed run keeps its command

**Files:**

- Modify: `crates/kb-domain/src/skills.rs` (`skill_entry` and its tests)
- Modify: `crates/kb-domain/src/sync.rs:119-136` (`SkillsOutcome`), `:218-269` (`index_skills`), tests at `:650-725`
- Modify: `crates/kb-app/src/cli.rs:99-131`
- Modify: `crates/kb-app/src/sync_view.rs` (`report_sections`, tests)

- [ ] **Step 1: Write the failing tests**

In `skills.rs` tests, change the two warning tests so they check the new shape:

```rust
#[test]
fn indexes_a_record_whose_skill_md_is_missing_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let (entry, warning) = skill_entry(&record("gone", &dir.path().join("gone"), None));
    let warning = warning.unwrap();
    assert!(warning.message.starts_with("/gone: cannot read "), "{warning:?}");
    assert_eq!(
        warning.file.as_deref(),
        Some(dir.path().join("gone/SKILL.md").to_string_lossy().as_ref()),
        "the warning names the SKILL.md it is about"
    );
    // keep the existing assertions on `entry` below this line unchanged
```

and in the front-matter test: `assert!(warning.as_ref().unwrap().message.starts_with("/bad: SKILL.md front matter: "));`
plus `assert!(warning.unwrap().file.unwrap().ends_with("bad/SKILL.md"));`. Keep every other assertion as it is.

In `sync.rs` tests, replace `warnings: vec!["⚠ Skipped x".into()]` with
`warnings: vec![SkillsWarning::stderr("⚠ Skipped x")]` (import `crate::skills::SkillsWarning`), and add:

```rust
    #[test]
    fn keeps_the_command_and_its_exit_code_when_skills_fail() {
        let root = sources(&[]);
        let home = tempfile::tempdir().unwrap();
        let db = home.path().join("kb.db");
        let failing = fake_skills(home.path(), "not json", "careful\n", 0);
        let report = run(Selection::Skills, root.path(), Some(&failing), &db, false);
        let Some(SkillsOutcome::Failed { command, exit_code, warnings, .. }) = report.skills else {
            panic!("expected a failed Skills outcome: {:?}", report.skills);
        };
        assert_eq!(command, format!("{} ls --global --agent universal --json", failing.display()));
        assert_eq!(exit_code, Some(0), "the output was not JSON, the command still exited 0");
        assert_eq!(warnings, [SkillsWarning::stderr("careful")]);

        let missing = home.path().join("no-such-skills");
        let report = run(Selection::Skills, root.path(), Some(&missing), &db, false);
        assert!(
            matches!(report.skills, Some(SkillsOutcome::Failed { exit_code: None, .. })),
            "a command that never ran has no exit code"
        );
    }
```

- [ ] **Step 2: Run them and see them fail**

Run: `cargo nextest run -p kb-domain skills`
Expected: compile errors (`SkillsWarning` not found, no field `message`).

- [ ] **Step 3: Implement**

In `skills.rs`, above `skill_entry`:

```rust
/// Something the Skills source warns about: a line the command wrote to stderr, or a record whose
/// `SKILL.md` could not be read (then `file` names it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillsWarning {
    /// What to tell the user.
    pub message: String,
    /// The `SKILL.md` it is about, absolute; `None` for a stderr line.
    pub file: Option<String>,
}

impl SkillsWarning {
    /// A line of the command's stderr.
    #[must_use]
    pub fn stderr(line: &str) -> Self {
        Self {
            message: line.to_owned(),
            file: None,
        }
    }
}
```

Change `skill_entry` to return `(Entry, Option<SkillsWarning>)`; build each warning as
`SkillsWarning { message: format!(…same text…), file: Some(skill_md.to_string_lossy().into_owned()) }`.

In `sync.rs`, `SkillsOutcome` becomes:

```rust
/// Its entries were replaced.
Indexed {
    /// Entries indexed.
    entries: usize,
    /// The command's stderr lines, then records whose `SKILL.md` could not be read.
    warnings: Vec<SkillsWarning>,
},
/// The command or its output failed; the previous Skills entries are unchanged.
Failed {
    /// Why.
    reason: String,
    /// The command line kb ran: the program, then [`SKILLS_ARGS`].
    command: String,
    /// Its exit code; `None` when it never ran or a signal stopped it.
    exit_code: Option<i32>,
    /// The command's stderr lines.
    warnings: Vec<SkillsWarning>,
},
```

In `index_skills`: compute `let command = format!("{} {}", program.display(), SKILLS_ARGS.join(" "));` first; map
stderr lines with `.map(SkillsWarning::stderr)`; the spawn failure returns `exit_code: None`; the non-zero exit and
the parse failure return `exit_code: output.status.code()`; each `Failed` carries `command: command.clone()`.

In `cli.rs`, `write_warnings(out, warnings: &[SkillsWarning])` prints `warning.message`; the `Failed` arm binds
`Failed { reason, warnings, .. }`.

In `sync_view.rs` `report_sections`: `warnings` becomes `&[SkillsWarning]` and the warning row's detail is
`w.message.clone()`; `SkillsOutcome::Failed { reason, warnings, .. }`. In its tests, `warnings: vec!["⚠ Skipped
recon".into()]` becomes `vec![SkillsWarning::stderr("⚠ Skipped recon")]` and the `Failed` literal gains
`command: "skills ls".into(), exit_code: Some(1),`.

- [ ] **Step 4: Run them and see them pass**

Run: `cargo nextest run --workspace`
Expected: PASS.

- [ ] **Step 5: Commit** — `feat(domain): Name the SKILL.md a warning is about`

---

### Task 3: The editor template

**Files:**

- Modify: `Cargo.toml` (`[workspace.dependencies]`: `shlex = "1"`), `crates/kb-app/Cargo.toml` (`shlex.workspace = true`)
- Modify: `crates/kb-app/src/command_path.rs`
- Create: `crates/kb-app/src/editor.rs`; register `pub mod editor;` in `crates/kb-app/src/lib.rs` (alphabetical)

- [ ] **Step 1: Write the failing tests**

In `command_path.rs` tests:

```rust
#[test]
fn finds_a_command_by_name_or_keeps_what_it_was_given() {
    let home = tempfile::tempdir().unwrap();
    let zed = executable(&home.path().join(".local/bin"), "zed");
    assert_eq!(find_command("zed", None, home.path()), zed);
    assert_eq!(find_command("~/bin/code", None, home.path()), home.path().join("bin/code"));
    assert_eq!(find_command("kb-test-no-such", None, home.path()), PathBuf::from("kb-test-no-such"));
}
```

`crates/kb-app/src/editor.rs`, tests first (the module body comes in Step 3):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_the_file_line_and_column() {
        assert_eq!(
            command("zed {file}:{line}:{column}", "/s/a b.yml", Some(4), Some(11)).unwrap(),
            ["zed", "/s/a b.yml:4:11"],
            "a path with a space stays one word"
        );
        assert_eq!(
            command("code -g {file}:{line}", "/s/x.yml", Some(4), None).unwrap(),
            ["code", "-g", "/s/x.yml:4"]
        );
    }

    #[test]
    fn drops_the_line_and_column_it_does_not_know() {
        assert_eq!(command("zed {file}:{line}:{column}", "/s/x.yml", None, None).unwrap(), ["zed", "/s/x.yml"]);
        assert_eq!(command("zed {file}:{line}:{column}", "/s/x.yml", Some(4), None).unwrap(), ["zed", "/s/x.yml:4"]);
        assert_eq!(
            command("subl --wait {file}", "/s/x.yml", Some(4), Some(1)).unwrap(),
            ["subl", "--wait", "/s/x.yml"],
            "a template without {{line}} opens the file"
        );
    }

    #[test]
    fn groups_quoted_words_and_rejects_a_broken_template() {
        assert_eq!(
            command("'/Applications/My Editor.app/bin/ed' {file}", "/x", None, None).unwrap(),
            ["/Applications/My Editor.app/bin/ed", "/x"]
        );
        assert_eq!(check(""), Err(EditorError::Empty));
        assert_eq!(check("   "), Err(EditorError::Empty));
        assert_eq!(check("zed '{file}"), Err(EditorError::Unbalanced));
        assert_eq!(check("zed {file}"), Ok(()));
    }
}
```

- [ ] **Step 2: Run them and see them fail**

Run: `cargo nextest run -p kb-app editor command_path`
Expected: compile errors (`find_command`, `command`, `check` not found).

- [ ] **Step 3: Implement**

`command_path.rs`, after `skills_command`:

```rust
/// The program `name` names: a path as is (`~/` expanded); a bare name where `PATH` or the install
/// folders have it, else the name itself (spawning it then fails and says why).
#[must_use]
pub fn find_command(name: &str, path_var: Option<&OsStr>, home: &Path) -> PathBuf {
    if name.contains('/') {
        return expand_home(name, home);
    }
    find_executable(name, &search_dirs(path_var, home)).unwrap_or_else(|| PathBuf::from(name))
}
```

`editor.rs` (above the tests):

```rust
//! The `editor` setting (spec 2026-10-27 D6): a command template that opens a file at a line, e.g.
//! `zed {file}:{line}` or `code -g {file}:{line}:{column}`. The template is split into words first
//! (quotes group; no shell runs), then `{file}`, `{line}` and `{column}` are filled in, so a path with
//! spaces stays one argument. Without a known line, `:{line}` and `:{column}` go with their colon.
use std::path::Path;

/// Why a template cannot be used.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EditorError {
    /// It names no program.
    #[error("it names no program")]
    Empty,
    /// A quote is never closed.
    #[error("a quote is never closed")]
    Unbalanced,
}

/// Checks `template` once, when config.toml is read.
///
/// # Errors
///
/// The template is empty or has an unclosed quote.
pub fn check(template: &str) -> Result<(), EditorError> {
    words(template).map(|_| ())
}

fn words(template: &str) -> Result<Vec<String>, EditorError> {
    let words = shlex::split(template).ok_or(EditorError::Unbalanced)?;
    if words.is_empty() {
        return Err(EditorError::Empty);
    }
    Ok(words)
}

/// The program and its arguments for `file` at `line`:`column`.
///
/// # Errors
///
/// As [`check`].
pub fn command(template: &str, file: &str, line: Option<usize>, column: Option<usize>) -> Result<Vec<String>, EditorError> {
    let fill = |word: String| {
        let mut word = word;
        if line.is_none() {
            word = word.replace(":{line}", "").replace("{line}", "");
        }
        if line.is_none() || column.is_none() {
            word = word.replace(":{column}", "").replace("{column}", "");
        }
        word.replace("{file}", file)
            .replace("{line}", &line.map(|n| n.to_string()).unwrap_or_default())
            .replace("{column}", &column.map(|n| n.to_string()).unwrap_or_default())
    };
    Ok(words(template)?.into_iter().map(fill).filter(|w| !w.is_empty()).collect())
}

/// How a file was opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opened {
    /// In the configured editor.
    Editor,
    /// With the system's default app: no editor is set.
    DefaultApp,
    /// With the default app, because the editor could not be run: the program and the OS error.
    Fallback {
        /// The program kb tried.
        program: String,
        /// Why it could not run.
        error: String,
    },
    /// Not at all: the file is gone.
    Missing,
}

/// Opens `file` in `editor` (the template) at `line`:`column`, else with the default app.
pub fn open(
    editor: Option<&str>,
    file: &Path,
    line: Option<usize>,
    column: Option<usize>,
    home: &Path,
    cx: &gpui_kit::App,
) -> Opened {
    if !file.exists() {
        return Opened::Missing;
    }
    let Some(words) = editor.and_then(|t| command(t, &file.to_string_lossy(), line, column).ok()) else {
        cx.open_with_system(file);
        return Opened::DefaultApp;
    };
    let program = crate::command_path::find_command(&words[0], std::env::var_os("PATH").as_deref(), home);
    match std::process::Command::new(&program).args(&words[1..]).spawn() {
        Ok(_) => Opened::Editor,
        Err(error) => {
            cx.open_with_system(file);
            Opened::Fallback {
                program: program.display().to_string(),
                error: error.to_string(),
            }
        }
    }
}
```

If clippy flags `words[0]` (`indexing_slicing` is not in the workspace lints, but check), use
`let (program, args) = words.split_first()` instead.

- [ ] **Step 4: Run them and see them pass**

Run: `cargo nextest run -p kb-app editor command_path` then `cargo deny check`
Expected: PASS; deny ok (shlex is MIT/Apache-2.0).

- [ ] **Step 5: Commit** — `feat(app): Open files with an editor template`

---

### Task 4: The `editor` setting, and the list's Edit uses it

**Files:**

- Modify: `crates/kb-app/src/config.rs` (doc comment, `Preferences`, `Config`, `load`, accessor, tests)
- Modify: `crates/kb-app/src/catalog.rs` (`preferences_template`) and `assets/catalog.toml` `[messages]`
- Modify: `crates/kb-app/src/search_view.rs` (`open_source_file`)

- [ ] **Step 1: Write the failing tests**

In `config.rs` tests (follow the file's existing helper for writing a config.toml under a temp home; if there is
none, write it with `fs::write(home.join(".config/kb/config.toml"), …)` after `create_dir_all`):

```rust
    #[test]
    fn reads_the_editor_and_ignores_a_broken_one() {
        let home = tempfile::tempdir().unwrap();
        let file = home.path().join(".config/kb/config.toml");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "editor = \"zed {file}:{line}\"\n").unwrap();
        let config = Config::load(home.path(), home.path()).unwrap();
        assert_eq!(config.editor(), Some("zed {file}:{line}"));

        fs::write(&file, "editor = \"zed '{file}\"\n").unwrap();
        let config = Config::load(home.path(), home.path()).unwrap();
        assert_eq!(config.editor(), None);
        assert!(
            config.notices().iter().any(|n| n.contains("`editor`") && n.contains("a quote is never closed")),
            "{:?}",
            config.notices()
        );
    }
```

In `catalog.rs` `writes_a_preferences_file_with_every_default_commented_out`, add:

```rust
assert!(text.contains("\n# editor = \"zed {file}:{line}\"\n"), "{text}");
```

- [ ] **Step 2: Run them and see them fail**

Run: `cargo nextest run -p kb-app config catalog`
Expected: FAIL (`editor` is an unknown setting; no template line).

- [ ] **Step 3: Implement**

`config.rs`:

- Doc comment: add `//! editor = "zed {file}:{line}"` to the example and a line: ``//! `editor` opens files at a
  line (`crate::editor`); unset, files open with their default app.``
- `Preferences` gains `editor: Option<String>`; `Config` gains `editor: Option<String>`.
- In `load`, add `"editor"` to the `"sources_path" | … | "theme"` arm.
- In `from_preferences`, before building `Self`, check it (the notice joins the others):

```rust
let mut notices = notices;
let editor = parsed.editor.filter(|template| match crate::editor::check(template) {
    Ok(()) => true,
    Err(error) => {
        notices.push(format!("Ignored `editor` in {}: {error}", home.join(PREFERENCES).display()));
        false
    }
});
```

and set `editor` in the struct.

- Accessor:

```rust
    /// The editor template config.toml names (`crate::editor`), if any and valid.
    #[must_use]
    pub fn editor(&self) -> Option<&str> {
        self.editor.as_deref()
    }

    /// The home folder.
    #[must_use]
    pub fn home(&self) -> &Path {
        &self.home
    }
```

`catalog.rs` `preferences_template`: after the `sources_path` line, push:

```rust
text.push_str("# Opens files at a line; unset, they open with their default app.\n# editor = \"zed {file}:{line}\"\n\n");
```

`catalog.toml` `[messages]`, after `theme_weak`:

```toml
open_missing = "Couldn’t open {file}: it is no longer there."
editor_failed = "Couldn’t run {program}: {error}. Opened it with its default app."
```

and the matching `pub open_missing: String, pub editor_failed: String,` fields at the end of `Messages`.

`search_view.rs` `open_source_file` becomes (entries open at the top of their file):

```rust
    fn open_source_file(&mut self, _: &OpenSourceFile, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = self.selected_source_file(cx) {
            self.open_in_editor(std::path::Path::new(&path), None, None, window, cx);
        }
    }

    /// Opens `file` in the editor at `line`:`column` (D6), saying so when it cannot.
    pub(super) fn open_in_editor(
        &mut self,
        file: &std::path::Path,
        line: Option<usize>,
        column: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let m = &self.catalog.messages;
        let file_text = file.display().to_string();
        let notice = match crate::editor::open(self.config.editor(), file, line, column, self.config.home(), cx) {
            crate::editor::Opened::Editor | crate::editor::Opened::DefaultApp => None,
            crate::editor::Opened::Missing => Some(fill(&m.open_missing, &[("file", &file_text)])),
            crate::editor::Opened::Fallback { program, error } => {
                Some(fill(&m.editor_failed, &[("program", &program), ("error", &error)]))
            }
        };
        if let Some(notice) = notice {
            self.last_notice = Some(notice.clone());
            window.push_notification(notice, cx);
        }
    }
```

(import `crate::catalog::fill` if `search_view.rs` does not already).

- [ ] **Step 4: Run them and see them pass**

Run: `cargo nextest run -p kb-app`
Expected: PASS.

- [ ] **Step 5: Commit** — `feat(app): Read an editor template from config.toml`

---

### Task 5: Catalog: report actions, the report-detail screen, the pane's words

**Files:**

- Modify: `crates/kb-app/assets/catalog.toml`
- Modify: `crates/kb-app/src/catalog.rs` (`ACTION_IDS`, `Screen`, `Screens`, `screen()`, `validate`, `Messages`, tests)

- [ ] **Step 1: Write the failing test** — in `catalog.rs` tests:

```rust
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
    for (id, keys) in [("copy-location", "cmd-shift-l"), ("copy-error", "cmd-shift-e"), ("reveal", "cmd-shift-r")] {
        assert_eq!(catalog.actions[id].keys.as_deref(), Some(keys), "{id}");
    }
    assert_eq!(catalog.label("reveal"), "Reveal in Finder");
}
```

- [ ] **Step 2: Run it and see it fail**

Run: `cargo nextest run -p kb-app ships_the_report`
Expected: compile error (`Screen::ReportDetail`).

- [ ] **Step 3: Implement**

`catalog.toml`, after `[actions.copy-path]`:

```toml
[actions.copy-location]
label = "Copy Location"
keys = "cmd-shift-l"

[actions.copy-error]
label = "Copy Error"
keys = "cmd-shift-e"

[actions.reveal]
label = "Reveal in Finder"
keys = "cmd-shift-r"
```

`[screens.report]` becomes, and `[screens.report-detail]` follows it:

```toml
[screens.report]
enter = "details"
cmd-enter = "edit"
shift-cmd-enter = "copy-location"

[screens.report-detail]
enter = "edit"
cmd-enter = "copy-location"
```

Update the comment above `[screens.list]` to mention the report and its detail. `[messages]`, after
`report_not_configured_chip`:

```toml
report_pane_location = "Location"
report_pane_file = "File"
report_pane_entries = "Entries"
report_pane_index = "Index"
report_pane_kept = "its previous entries are kept"
report_pane_skills_kept = "the previous skills are kept"
report_pane_exit = "Exit"
report_pane_stderr = "Stderr"
report_pane_nothing = "(nothing)"
report_pane_line = "line {line}:{column}"
report_pane_skipped = "{count} skipped: only a map of fields becomes an entry."
report_pane_one_skipped = "1 entry was skipped: only a map of fields becomes an entry."
report_pane_skipped_key = "{type} / {key} — not a map"
report_pane_skipped_chip = "{entries} · {count} skipped"
```

`catalog.rs`:

- `ACTION_IDS` grows to 25: add `"copy-location"`, `"copy-error"`, `"reveal"` after `"copy-path"`.
- `Screen` gains, after `Report`: `/// A report row, full screen. ReportDetail,`.
- `Screens` gains `pub report_detail: ScreenKeys,` (the struct is `rename_all = "kebab-case"`, so it reads
  `report-detail`); `screen()` maps `Screen::ReportDetail => &s.report_detail`; `validate`'s list gains
  `("report-detail", false, &self.screens.report_detail)`.
- `Messages` gains the fourteen `report_pane_*` fields, in the order above.

Fix the other tests that enumerate screens or action counts (`ships_a_valid_catalog` uses `ACTION_IDS.len()`, so it
follows).

- [ ] **Step 4: Run it and see it pass**

Run: `cargo nextest run -p kb-app catalog`
Expected: PASS. `tests/ui.rs::reports_a_refresh_and_goes_back_with_return` now fails (the report's keys changed);
Task 10 replaces it. Mark it `#[ignore = "replaced in Task 10"]` for this commit only.

- [ ] **Step 5: Commit** — `feat(catalog): Add the report's actions, keys and words`

---

### Task 6: `report_list.rs`: rows, sections, selection

**Files:**

- Create: `crates/kb-app/src/report_list.rs`; register `pub mod report_list;` in `lib.rs`
- Modify: `crates/kb-app/src/sync_view.rs` (move `ReportKind`, `ReportRow`, `report_rows`, `ReportSection`,
  `report_sections`, `issues_line` and their tests out; `sync_view.rs` re-imports them from `crate::report_list`)

`ReportRow` gains what the pane and the actions need: the subject it reports on.

- [ ] **Step 1: Move the code, then write the failing tests**

Move the items listed above, unchanged, into `report_list.rs` with their tests; `sync_view.rs` gets
`use crate::report_list::{ReportKind, ReportRow, report_sections};` and `summary_line` stays in `sync_view.rs`. Run
`cargo nextest run -p kb-app` — everything still passes. Then add to `report_list.rs` tests:

```rust
    #[test]
    fn steps_through_rows_across_sections_and_stops_at_the_ends() {
        let counts = [3, 2]; // YAML: 3 files; Skills: outcome + 1 warning
        let at = |section, row| IndexPath { section, row, ..IndexPath::default() };
        assert_eq!(step(at(0, 2), 1, &counts), at(1, 0), "↓ from YAML's last row enters Skills");
        assert_eq!(step(at(1, 0), -1, &counts), at(0, 2), "↑ goes back");
        assert_eq!(step(at(1, 1), 1, &counts), at(1, 1), "stops at the last row");
        assert_eq!(step(at(0, 0), -1, &counts), at(0, 0), "stops at the first row");
        assert_eq!(step(at(0, 0), isize::MAX, &counts), at(1, 1), "⌘↓ jumps to the last row");
        assert_eq!(step(at(1, 1), isize::MIN, &counts), at(0, 0), "⌘↑ jumps to the first row");
        assert_eq!(step(at(0, 0), 1, &[0, 2]), at(1, 0), "an empty section is skipped");
    }

    #[test]
    fn knows_each_row_s_file_location_and_error() {
        let m = Catalog::default().messages;
        let failed = file("errors/x.yml", 0, true, false); // line 4, column 1, see `file`
        let report = SyncReport {
            yaml: Some(YamlOutcome::Indexed(yaml_summary(vec![failed, file("a.yml", 3, false, false)], 3))),
            skills: Some(SkillsOutcome::Indexed {
                entries: 1,
                warnings: vec![
                    SkillsWarning { message: "/x: cannot read".into(), file: Some("/k/x/SKILL.md".into()) },
                    SkillsWarning::stderr("⚠ careful"),
                ],
            }),
            duration_ms: 0,
        };
        let rows: Vec<ReportRow> = report_sections(&report, "/s", &m).into_iter().flat_map(|s| s.rows).collect();
        let facts: Vec<(Option<String>, Option<String>, Option<String>)> = rows
            .iter()
            .map(|r| (r.file().map(str::to_owned), r.location(), r.error()))
            .collect();
        assert_eq!(
            facts,
            [
                (Some("/s/errors/x.yml".into()), Some("/s/errors/x.yml:4:1".into()), Some("Line 4: a flow sequence must be closed".into())),
                (Some("/s/a.yml".into()), Some("/s/a.yml".into()), None),
                (None, None, None),
                (Some("/k/x/SKILL.md".into()), Some("/k/x/SKILL.md".into()), None),
                (None, None, None),
            ]
        );
        assert_eq!(rows[0].line_and_column(), (Some(4), Some(1)));
    }
```

(`yaml_summary` and `file` move here with the tests; import `kb_domain::skills::SkillsWarning`,
`gpui_kit::component::IndexPath`.)

- [ ] **Step 2: Run them and see them fail**

Run: `cargo nextest run -p kb-app report_list`
Expected: compile errors (`step`, `file`, `location`, `error`, `line_and_column`).

- [ ] **Step 3: Implement**

Add the subject to `ReportRow`:

```rust
/// What a report row reports on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subject {
    /// A YAML source file.
    File(FileReport),
    /// The sources directory, which could not be read: its path.
    SourcesDir(String),
    /// The Skills source's outcome.
    Skills(SkillsOutcome),
    /// One of its warnings.
    Warning(SkillsWarning),
}
```

`ReportRow` gains `pub subject: Subject` (doc: `/// What it is about.`). `report_rows` sets
`subject: Subject::File(file.clone())`; the failed-YAML row `Subject::SourcesDir(sources_dir.to_owned())`; the Skills
outcome row `Subject::Skills(skills.clone())`; each warning row `Subject::Warning(w.clone())`. Update the `row`
closure in `report_sections` to take the subject.

```rust
impl ReportRow {
    /// The file this row is about, absolute: Edit, Reveal and Copy Path act on it.
    #[must_use]
    pub fn file(&self) -> Option<&str> {
        match &self.subject {
            Subject::File(file) => Some(&file.file_path),
            Subject::Warning(warning) => warning.file.as_deref(),
            Subject::SourcesDir(_) | Subject::Skills(_) => None,
        }
    }

    /// What Copy Location copies: `path:line:column` for a failed file, else the file.
    #[must_use]
    pub fn location(&self) -> Option<String> {
        match &self.subject {
            Subject::File(file) => Some(file.location()),
            _ => self.file().map(str::to_owned),
        }
    }

    /// Where Edit opens the file.
    #[must_use]
    pub fn line_and_column(&self) -> (Option<usize>, Option<usize>) {
        match &self.subject {
            Subject::File(FileReport { error: Some(e), .. }) => (e.line, e.column),
            _ => (None, None),
        }
    }

    /// What Copy Error copies: the bare message of a failed file, sources directory or Skills run.
    #[must_use]
    pub fn error(&self) -> Option<String> {
        match &self.subject {
            Subject::File(file) => file.error.as_ref().map(|e| e.message.clone()),
            Subject::SourcesDir(_) => Some(self.detail.clone()),
            Subject::Skills(SkillsOutcome::Failed { reason, .. }) => Some(reason.clone()),
            Subject::Skills(_) | Subject::Warning(_) => None,
        }
    }
}

/// The row `delta` rows from `from` (`isize::MAX` / `MIN`: the last / first), across sections of
/// `counts` rows each, clamped to the ends; empty sections are skipped.
#[must_use]
pub fn step(from: IndexPath, delta: isize, counts: &[usize]) -> IndexPath {
    let flat: Vec<IndexPath> = counts
        .iter()
        .enumerate()
        .flat_map(|(section, &n)| (0..n).map(move |row| IndexPath { section, row, ..IndexPath::default() }))
        .collect();
    let Some(last) = flat.len().checked_sub(1) else { return from };
    let at = flat.iter().position(|ix| ix.section == from.section && ix.row == from.row).unwrap_or(0);
    flat[at.saturating_add_signed(delta).min(last)]
}
```

Note: `SourcesDir`'s `detail` is the filled `report_source_failed` message; Copy Error copies that sentence, which is
the reason plus _The previous entries are unchanged._ That is acceptable (spec D4 says "the message"); do not add a
field for it.

- [ ] **Step 4: Run them and see them pass**

Run: `cargo nextest run -p kb-app report_list sync_view`
Expected: PASS.

- [ ] **Step 5: Commit** — `refactor(app): Move the report's rows into report_list` (one commit for the move and the
      additions is fine; say both in the body)

---

### Task 7: `ReportList`, the `ListDelegate`

**Files:**

- Modify: `crates/kb-app/src/report_list.rs`
- Modify: `crates/kb-app/src/sync_view.rs` (the row-drawing code of `render_report` moves into `render_item`)

- [ ] **Step 1: Write the failing test** — in `report_list.rs` tests:

```rust
#[test]
fn selects_the_first_row_and_counts_rows_per_section() {
    let m = Catalog::default().messages;
    let report = SyncReport {
        yaml: Some(YamlOutcome::Indexed(yaml_summary(vec![file("a.yml", 3, false, false), file("e.yml", 0, true, false)], 3))),
        skills: Some(SkillsOutcome::NotConfigured),
        duration_ms: 0,
    };
    let list = ReportList::new(report_sections(&report, "/s", &m), Rc::new(Catalog::default()));
    assert_eq!(list.counts(), [2, 1]);
    assert_eq!(list.selected_row().map(|r| r.path.as_str()), Some("e.yml"), "failed first, selected");
}
```

- [ ] **Step 2: Run it and see it fail**

Run: `cargo nextest run -p kb-app selects_the_first_row`
Expected: compile error (`ReportList`).

- [ ] **Step 3: Implement**

```rust
/// The report as a GPUI Kit list: one section per refreshed source, its label as the header.
#[derive(Debug)]
pub struct ReportList {
    sections: Vec<ReportSection>,
    catalog: Rc<Catalog>,
    selected: Option<IndexPath>,
}

impl ReportList {
    /// The list over `sections`, its first row selected (failed rows sort first).
    #[must_use]
    pub fn new(sections: Vec<ReportSection>, catalog: Rc<Catalog>) -> Self {
        let counts: Vec<usize> = sections.iter().map(|s| s.rows.len()).collect();
        let selected = counts
            .iter()
            .any(|&n| n > 0)
            .then(|| step(IndexPath::default(), 0, &counts));
        Self { sections, catalog, selected }
    }

    /// Rows per section.
    #[must_use]
    pub fn counts(&self) -> Vec<usize> {
        self.sections.iter().map(|s| s.rows.len()).collect()
    }

    /// The selected row.
    #[must_use]
    pub fn selected_row(&self) -> Option<&ReportRow> {
        let ix = self.selected?;
        self.sections.get(ix.section)?.rows.get(ix.row)
    }

    /// The selected index.
    #[must_use]
    pub fn selected(&self) -> Option<IndexPath> {
        self.selected
    }
}

impl ListDelegate for ReportList {
    type Item = Row;

    fn sections_count(&self, _: &App) -> usize {
        self.sections.len().max(1)
    }

    fn items_count(&self, section: usize, _: &App) -> usize {
        self.sections.get(section).map_or(0, |s| s.rows.len())
    }

    fn render_section_header(&mut self, section: usize, _: &mut Window, _: &mut Context<ListState<Self>>) -> Option<impl IntoElement> {
        let d = design();
        let s = &d.spacing.empty;
        Some(
            div()
                .px(pt(s.summary_pad_x))
                .pt(pt(s.summary_pad_top))
                .text_role(&d.typography.roles.section)
                .text_color(d.colors.faint)
                .child(self.sections.get(section)?.label.clone()),
        )
    }

    fn render_item(&mut self, ix: IndexPath, _: &mut Window, _: &mut Context<ListState<Self>>) -> Option<Row> {
        let row = self.sections.get(ix.section)?.rows.get(ix.row)?;
        let selected = self.selected == Some(ix);
        Some(report_row(row, &format!("report-{}-{}", ix.section, ix.row), selected, &self.catalog.messages))
    }

    fn set_selected_index(&mut self, ix: Option<IndexPath>, _: &mut Window, cx: &mut Context<ListState<Self>>) {
        self.selected = ix;
        cx.notify();
    }
}
```

`report_row(row, id, selected, m) -> Row` is the per-row body of today's `render_report` loop in `sync_view.rs`
(the `match row.kind { … }` giving icon, tint, chip, detail, then `Row::new(…).lead(…).mono(true).description(…)
.trailing(vec![pill(chip, ChipState::of(selected))…]).selected(selected)`), moved verbatim with `ix == 0` replaced by
`selected`. For a `Skipped` row with skipped entries the chip becomes
`fill(&m.report_pane_skipped_chip, &[("entries", …), ("count", …)])` only in the pane, not here: keep the row as is.

Imports: `std::rc::Rc`, `gpui_kit::component::list::{ListDelegate, ListState}`,
`gpui_kit::{App, Context, IntoElement, ParentElement as _, Styled as _, Window, div}`, `crate::catalog::{Catalog,
Messages, fill}`, `crate::ui::{chip::pill, design::{TextRoleExt as _, design, pt}, keycap::ChipState, row::Row,
tile::{TileSize, tile}}`.

- [ ] **Step 4: Run it and see it pass**

Run: `cargo nextest run -p kb-app report_list`
Expected: PASS.

- [ ] **Step 5: Commit** — `feat(app): List the report's rows in a GPUI Kit list`

---

### Task 8: `ui/report_pane.rs`: what a row's pane shows

**Files:**

- Create: `crates/kb-app/src/ui/report_pane.rs`; register `pub mod report_pane;` in `crates/kb-app/src/ui.rs`

The content is a plain model (tested without a window); the element draws it with existing tokens.

- [ ] **Step 1: Write the failing tests** — in `report_pane.rs`:

```rust
#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use kb_domain::report::{FileIssue, FileReport};
    use kb_domain::skills::SkillsWarning;
    use kb_domain::source::SourceError;
    use kb_domain::sync::SkillsOutcome;

    use super::*;
    use crate::catalog::Catalog;
    use crate::report_list::{ReportKind, ReportRow, Subject};

    fn row(kind: ReportKind, subject: Subject) -> ReportRow {
        ReportRow { path: "x.yml".into(), kind, entries: 9, detail: String::new(), subject }
    }

    fn file(error: bool, skipped: bool) -> FileReport {
        FileReport {
            rel_path: "x.yml".into(),
            file_path: "/s/x.yml".into(),
            entries: 9,
            types: IndexMap::from([("bookmarks".into(), 9)]),
            error: error.then(|| SourceError { file_path: "/s/x.yml".into(), message: "A flow sequence must be closed".into(), line: Some(4), column: Some(11) }),
            excerpt: error.then(|| "  3 | k8s-pods:\n> 4 |   tags: [k8s, pods\n  5 |   cmd: kubectl".into()),
            issues: if skipped { vec![FileIssue::SkippedEntry { entry_type: "bookmarks".into(), key: "not-a-map".into() }] } else { Vec::new() },
        }
    }

    #[test]
    fn shows_a_failed_file_s_error_excerpt_and_location() {
        let m = Catalog::default().messages;
        let pane = content(&row(ReportKind::Failed, Subject::File(file(true, false))), &m);
        assert_eq!(pane.quote, Some((Tone::Error, "A flow sequence must be closed".into())));
        assert_eq!(
            pane.excerpt,
            [
                ExcerptLine { number: 3, text: "k8s-pods:".into(), marked: false },
                ExcerptLine { number: 4, text: "  tags: [k8s, pods".into(), marked: true },
                ExcerptLine { number: 5, text: "  cmd: kubectl".into(), marked: false },
            ]
        );
        assert_eq!(
            pane.facts,
            [
                ("Location".into(), Fact::Pills(vec!["line 4:11".into()])),
                ("File".into(), Fact::Mono("/s/x.yml".into())),
                ("Index".into(), Fact::Text("its previous entries are kept".into())),
            ]
        );
    }

    #[test]
    fn lists_skipped_entries_then_the_types() {
        let m = Catalog::default().messages;
        let pane = content(&row(ReportKind::Skipped, Subject::File(file(false, true))), &m);
        assert_eq!(pane.quote, Some((Tone::Warning, "1 entry was skipped: only a map of fields becomes an entry.".into())));
        assert_eq!(pane.list, ["bookmarks / not-a-map — not a map"]);
        assert_eq!(pane.facts[0], ("Entries".into(), Fact::Pills(vec!["bookmarks 9".into()])));
        assert_eq!(pane.chip, "9 · 1 skipped");
    }

    #[test]
    fn shows_an_indexed_file_s_facts_only() {
        let m = Catalog::default().messages;
        let pane = content(&row(ReportKind::Indexed, Subject::File(file(false, false))), &m);
        assert_eq!((pane.quote, pane.excerpt.len(), pane.list.len()), (None, 0, 0));
        assert_eq!(pane.facts.len(), 2, "Entries and File");
    }

    #[test]
    fn shows_a_failed_skills_run_s_command_exit_and_stderr() {
        let m = Catalog::default().messages;
        let failed = SkillsOutcome::Failed {
            reason: "the output is not Skills JSON".into(),
            command: "/opt/homebrew/bin/skills ls --global --agent universal --json".into(),
            exit_code: Some(0),
            warnings: Vec::new(),
        };
        let pane = content(&row(ReportKind::Failed, Subject::Skills(failed)), &m);
        assert_eq!(pane.quote, Some((Tone::Error, "the output is not Skills JSON".into())));
        assert_eq!(pane.command.as_deref(), Some("/opt/homebrew/bin/skills ls --global --agent universal --json"));
        assert_eq!(
            pane.facts,
            [
                ("Exit".into(), Fact::Pills(vec!["0".into()])),
                ("Stderr".into(), Fact::Mono("(nothing)".into())),
                ("Index".into(), Fact::Text("the previous skills are kept".into())),
            ]
        );
    }

    #[test]
    fn shows_a_warning_in_full_with_its_file() {
        let m = Catalog::default().messages;
        let warning = SkillsWarning { message: "/x: cannot read".into(), file: Some("/k/x/SKILL.md".into()) };
        let pane = content(&row(ReportKind::Warning, Subject::Warning(warning)), &m);
        assert_eq!(pane.quote, Some((Tone::Warning, "/x: cannot read".into())));
        assert_eq!(pane.facts, [("File".into(), Fact::Mono("/k/x/SKILL.md".into()))]);
    }
}
```

- [ ] **Step 2: Run them and see them fail**

Run: `cargo nextest run -p kb-app report_pane`
Expected: compile errors.

- [ ] **Step 3: Implement the model**

```rust
//! A report row's pane (spec 2026-10-27 D5; `docs/guides/design/12-report-pane.html`): its header,
//! the error or warning as a quote, the lines around an error, the skipped keys, and its facts.
//! The split and the detail screen draw the same pane.

use kb_domain::report::FileIssue;
use kb_domain::sync::SkillsOutcome;

use crate::catalog::{Messages, fill};
use crate::report_list::{ReportKind, ReportRow, Subject};

/// A quote's bar colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Red: something failed.
    Error,
    /// Yellow: skipped entries, a warning.
    Warning,
}

/// One line of an error's excerpt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcerptLine {
    /// Its 1-based number.
    pub number: usize,
    /// Its text.
    pub text: String,
    /// Whether it is the failing line.
    pub marked: bool,
}

/// A fact's value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fact {
    /// Pills.
    Pills(Vec<String>),
    /// Monospace text: a path, stderr.
    Mono(String),
    /// Plain text.
    Text(String),
}

/// What a row's pane shows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PaneContent {
    /// The header's chip.
    pub chip: String,
    /// The quote and its tone.
    pub quote: Option<(Tone, String)>,
    /// The lines around the error.
    pub excerpt: Vec<ExcerptLine>,
    /// The command kb ran (Skills, failed).
    pub command: Option<String>,
    /// Skipped keys.
    pub list: Vec<String>,
    /// Label → value.
    pub facts: Vec<(String, Fact)>,
}

/// Reads `report::excerpt`'s `"> 4 | text"` lines back.
fn excerpt_lines(excerpt: &str) -> Vec<ExcerptLine> {
    excerpt
        .lines()
        .filter_map(|line| {
            let (head, text) = line.split_once(" | ")?;
            Some(ExcerptLine {
                number: head.trim_start_matches('>').trim().parse().ok()?,
                text: text.to_owned(),
                marked: head.starts_with('>'),
            })
        })
        .collect()
}

/// The pane of `row`.
#[must_use]
pub fn content(row: &ReportRow, m: &Messages) -> PaneContent {
    let mut pane = PaneContent { chip: row.entries.to_string(), ..PaneContent::default() };
    let file_fact = |path: &str| (m.report_pane_file.clone(), Fact::Mono(path.to_owned()));
    match &row.subject {
        Subject::File(file) => {
            if let Some(error) = &file.error {
                pane.chip = m.report_failed_chip.clone();
                pane.quote = Some((Tone::Error, error.message.clone()));
                pane.excerpt = file.excerpt.as_deref().map(excerpt_lines).unwrap_or_default();
                if let Some(line) = error.line {
                    let column = error.column.map(|c| c.to_string()).unwrap_or_default();
                    let text = fill(&m.report_pane_line, &[("line", &line.to_string()), ("column", &column)]);
                    pane.facts.push((m.report_pane_location.clone(), Fact::Pills(vec![text.trim_end_matches(':').to_owned()])));
                }
                pane.facts.push(file_fact(&file.file_path));
                pane.facts.push((m.report_pane_index.clone(), Fact::Text(m.report_pane_kept.clone())));
                return pane;
            }
            let skipped: Vec<(&str, &str)> = file
                .issues
                .iter()
                .filter_map(|i| match i {
                    FileIssue::SkippedEntry { entry_type, key } => Some((entry_type.as_str(), key.as_str())),
                    FileIssue::Empty => None,
                })
                .collect();
            if file.issues.contains(&FileIssue::Empty) {
                pane.quote = Some((Tone::Warning, m.report_empty_file.clone()));
            } else if !skipped.is_empty() {
                let count = skipped.len().to_string();
                pane.chip = fill(&m.report_pane_skipped_chip, &[("entries", &row.entries.to_string()), ("count", &count)]);
                let text = if skipped.len() == 1 { m.report_pane_one_skipped.clone() } else { fill(&m.report_pane_skipped, &[("count", &count)]) };
                pane.quote = Some((Tone::Warning, text));
                pane.list = skipped.iter().map(|(t, k)| fill(&m.report_pane_skipped_key, &[("type", t), ("key", k)])).collect();
            }
            if !file.types.is_empty() {
                let pills = file.types.iter().map(|(t, n)| format!("{t} {n}")).collect();
                pane.facts.push((m.report_pane_entries.clone(), Fact::Pills(pills)));
            }
            pane.facts.push(file_fact(&file.file_path));
        }
        Subject::SourcesDir(path) => {
            pane.chip = m.report_failed_chip.clone();
            pane.quote = Some((Tone::Error, row.detail.clone()));
            pane.facts.push(file_fact(path));
        }
        Subject::Skills(SkillsOutcome::Failed { reason, command, exit_code, warnings }) => {
            pane.chip = m.report_failed_chip.clone();
            pane.quote = Some((Tone::Error, reason.clone()));
            pane.command = Some(command.clone());
            if let Some(code) = exit_code {
                pane.facts.push((m.report_pane_exit.clone(), Fact::Pills(vec![code.to_string()])));
            }
            let stderr = warnings.iter().filter(|w| w.file.is_none()).map(|w| w.message.as_str()).collect::<Vec<_>>().join("\n");
            let stderr = if stderr.is_empty() { m.report_pane_nothing.clone() } else { stderr };
            pane.facts.push((m.report_pane_stderr.clone(), Fact::Mono(stderr)));
            pane.facts.push((m.report_pane_index.clone(), Fact::Text(m.report_pane_skills_kept.clone())));
        }
        Subject::Skills(SkillsOutcome::NotConfigured) => {
            pane.chip = m.report_not_configured_chip.clone();
            pane.quote = Some((Tone::Warning, m.report_not_configured.clone()));
        }
        Subject::Skills(SkillsOutcome::Indexed { .. }) => {}
        Subject::Warning(warning) => {
            pane.chip = m.report_warning_chip.clone();
            pane.quote = Some((Tone::Warning, warning.message.clone()));
            if let Some(file) = &warning.file {
                pane.facts.push(file_fact(file));
            }
        }
    }
    pane
}
```

The model does not use `ReportKind`; the element below takes the header's tile and chip colour from `row.kind`, so
import it there. The `report_pane_line` message is `line {line}:{column}`; with no
column the trailing `:` is trimmed above.

- [ ] **Step 4: Run them and see them pass**

Run: `cargo nextest run -p kb-app report_pane`
Expected: PASS.

- [ ] **Step 5: Implement the element** — append to `report_pane.rs`:

```rust
/// The pane of `row`, padded for the split or the detail screen (`detail`), as `entry_view` pads.
#[must_use]
pub fn report_pane(row: &ReportRow, m: &Messages, detail: bool) -> impl gpui_kit::IntoElement {
    use gpui_kit::component::{h_flex, v_flex};
    use gpui_kit::prelude::FluentBuilder as _;
    use gpui_kit::{InteractiveElement as _, ParentElement as _, StatefulInteractiveElement as _, Styled as _, div};

    use crate::ui::chip::pill;
    use crate::ui::design::{TextRoleExt as _, design, pt};
    use crate::ui::keycap::ChipState;
    use crate::ui::tile::{TileSize, tile};

    let d = design();
    let c = &d.colors;
    let p = &d.spacing.pane;
    let pane = content(row, m);
    let (icon, tint) = crate::report_list::kind_tile(row.kind, c);
    let chip_color = match row.kind {
        ReportKind::Failed => c.red,
        ReportKind::Skipped | ReportKind::Warning => c.yellow,
        ReportKind::Indexed | ReportKind::NotConfigured => c.muted,
    };
    let header = h_flex()
        .gap(pt(p.header_gap))
        .child(tile(icon, tint, TileSize::Row))
        .child(div().flex_1().min_w_0().truncate().font_family(d.typography.mono.clone()).text_role(&d.typography.roles.title).child(row.path.clone()))
        .child(pill(pane.chip.clone(), ChipState::Rest).text_color(chip_color));
    let quote = pane.quote.map(|(tone, text)| {
        div()
            .border_l(pt(d.components.quote.bar))
            .border_color(if tone == Tone::Error { c.red } else { c.yellow })
            .pl(pt(d.spacing.quote.pad_left))
            .py(pt(d.spacing.quote.pad_y))
            .text_role(&d.typography.roles.quote)
            .text_color(c.text)
            .child(text)
    });
    let mono_block = |lines: Vec<gpui_kit::AnyElement>| {
        v_flex()
            .bg(c.mantle)
            .rounded(pt(d.rounded.card))
            .py(pt(d.spacing.card.pad_y))
            .font_family(d.typography.mono.clone())
            .text_role(&d.typography.roles.card_split)
            .text_color(c.muted)
            .children(lines)
    };
    let excerpt = (!pane.excerpt.is_empty()).then(|| {
        mono_block(
            pane.excerpt
                .iter()
                .map(|line| {
                    h_flex()
                        .px(pt(d.spacing.card.pad_x))
                        .gap(pt(d.spacing.card.pad_x))
                        .when(line.marked, |l| l.bg(c.red.opacity(0.12)).border_l(pt(d.components.quote.bar)).border_color(c.red).text_color(c.text))
                        .child(div().text_color(c.faint).child(line.number.to_string()))
                        .child(div().whitespace_nowrap().child(line.text.clone()))
                        .into_any_element()
                })
                .collect(),
        )
    });
    let command = pane.command.map(|cmd| mono_block(vec![div().px(pt(d.spacing.card.pad_x)).child(cmd).into_any_element()]));
    let list = (!pane.list.is_empty()).then(|| {
        v_flex().font_family(d.typography.mono.clone()).text_role(&d.typography.roles.description).text_color(c.muted)
            .children(pane.list.iter().map(|item| div().child(format!("• {item}"))))
    });
    let facts = (!pane.facts.is_empty()).then(|| {
        v_flex().gap(pt(p.header_gap)).text_role(&d.typography.roles.description).children(pane.facts.into_iter().map(|(label, value)| {
            h_flex()
                .gap(pt(p.gap))
                .child(div().w(pt(d.components.pane.fact_label)).flex_none().text_color(c.faint).child(label))
                .child(match value {
                    Fact::Pills(pills) => h_flex().gap(pt(d.spacing.row.tags_gap)).children(pills.into_iter().map(|t| pill(t, ChipState::Rest))).into_any_element(),
                    Fact::Mono(text) => div().min_w_0().font_family(d.typography.mono.clone()).text_color(c.muted).child(text).into_any_element(),
                    Fact::Text(text) => div().text_color(c.muted).child(text).into_any_element(),
                })
        }))
    });
    v_flex()
        .id("report-pane")
        .size_full()
        .min_w_0()
        .overflow_y_scroll()
        .px(pt(d.content_inset()))
        .py(pt(if detail { p.detail_pad_y } else { p.split_pad_y }))
        .gap(pt(p.gap))
        .child(header)
        .children(quote)
        .children(excerpt)
        .children(command)
        .children(list)
        .children(facts)
}
```

Notes for the implementer:

- `kind_tile(kind, colors) -> (&'static str, Hsla)`: extract it in `report_list.rs` from `report_row`'s match (icon
  and tint per `ReportKind`), so the row and the pane share it.
- `d.components.pane.fact_label` is a **new design token**: add `fact_label: 72` to DESIGN.md's front matter under
  `components.pane` (create the `pane` group if absent) and the field to `crate::ui::design` (`PaneSize` or the
  existing pane struct), then run `mise run design:validate`. If an existing token already sizes a label column, use
  it instead and skip the new one.
- Every colour and size comes from `design()`; `c.red.opacity(0.12)` mirrors the companion's tint — if the
  `no-literal-design` rule rejects the literal, add `tint_alpha: 0.12` beside `fact_label` and use it.
- If a method name differs in this GPUI Kit version (`truncate`, `whitespace_nowrap`, `font_family`), follow what
  `entry_view.rs` and `command_card.rs` use.

- [ ] **Step 6: Build and lint**

Run: `cargo clippy -p kb-app --all-targets -- -D warnings && hk check --all`
Expected: clean (bench-ts-typecheck may fail for the unrelated `node_modules` reason; say so, do not fix it here).

- [ ] **Step 7: Commit** — `feat(ui): Draw a report row's pane`

---

### Task 9: `SyncView` navigates its report

**Files:**

- Modify: `crates/kb-app/src/sync_view.rs`

- [ ] **Step 1: Write the failing test** — `sync_view.rs` has no GPUI test; navigation is covered end to end in Task
      10. Here, add a unit test for the level logic, kept free of GPUI:

```rust
#[test]
fn steps_back_one_level_at_a_time() {
    let mut level = ReportLevel::List;
    level = level.forward(Move::Pane);
    assert_eq!(level, ReportLevel::Split);
    level = level.forward(Move::Detail);
    assert_eq!(level, ReportLevel::Detail { from_split: true });
    assert_eq!(level.back(), Some(ReportLevel::Split), "Esc on the detail returns to the split");
    assert_eq!(ReportLevel::Split.back(), None, "Esc on the report leaves it");
    assert_eq!(ReportLevel::List.forward(Move::Detail).back(), Some(ReportLevel::List));
    assert_eq!(ReportLevel::Split.forward(Move::NoPane), ReportLevel::List);
    assert_eq!(ReportLevel::Detail { from_split: false }.forward(Move::Pane), ReportLevel::Detail { from_split: false }, "→ does nothing on the detail");
}
```

- [ ] **Step 2: Run it and see it fail**

Run: `cargo nextest run -p kb-app steps_back_one_level`
Expected: compile error.

- [ ] **Step 3: Implement**

In `sync_view.rs`:

```rust
/// Where the report stands (spec 2026-10-27 D1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportLevel {
    /// The rows alone.
    List,
    /// The rows and the selected row's pane.
    Split,
    /// The selected row, full screen; `from_split` restores the pane on Esc.
    Detail {
        /// Whether the pane was open.
        from_split: bool,
    },
}

/// A move on the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    /// →.
    Pane,
    /// ←.
    NoPane,
    /// ↵ (Details).
    Detail,
}

impl ReportLevel {
    /// The level after `m`.
    #[must_use]
    pub fn forward(self, m: Move) -> Self {
        match (self, m) {
            (Self::Detail { .. }, _) => self,
            (_, Move::Pane) => Self::Split,
            (_, Move::NoPane) => Self::List,
            (level, Move::Detail) => Self::Detail { from_split: level == Self::Split },
        }
    }

    /// The level Esc returns to; `None` leaves the report.
    #[must_use]
    pub fn back(self) -> Option<Self> {
        match self {
            Self::Detail { from_split: true } => Some(Self::Split),
            Self::Detail { from_split: false } => Some(Self::List),
            Self::List | Self::Split => None,
        }
    }
}
```

`SyncView` gains `report: Option<Entity<ListState<ReportList>>>` and `level: ReportLevel` (`List` at start). When the
run finishes with `Ok(report)` (in `start`'s final `view.update`), build the list:

```rust
if let SyncStatus::Done(report) = &view.status {
    let sections = report_sections(report, &view.job.sources.display().to_string(), &view.catalog.messages);
    let catalog = view.catalog.clone();
    view.report = Some(cx.new(|cx| ListState::new(ReportList::new(sections, catalog), window, cx).searchable(false)));
}
```

`ListState::new` needs a `Window`: change `start`'s spawn to `cx.spawn_in(window, …)` and `view.update_in(cx, |view,
window, cx| …)` for the final update, and add `window: &mut Window` to `SyncView::start`'s parameters (its one caller,
`SearchView::refresh`, has it). Then:

```rust
    /// The report's level.
    #[must_use]
    pub fn level(&self) -> ReportLevel {
        self.level
    }

    /// The selected report row.
    #[must_use]
    pub fn selected_row<'a>(&self, cx: &'a App) -> Option<&'a ReportRow> {
        self.report.as_ref()?.read(cx).delegate().selected_row()
    }

    /// Moves the selection `delta` rows (`isize::MAX` / `MIN`: last / first) and scrolls to it.
    pub fn move_selection(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(list) = &self.report else { return };
        list.update(cx, |list, cx| {
            let Some(from) = list.delegate().selected() else { return };
            let to = step(from, delta, &list.delegate().counts());
            list.set_selected_index(Some(to), window, cx);
            list.scroll_to_selected_item(window, cx);
        });
        cx.notify();
    }

    /// →, ← or ↵ on the report; nothing without a selected row.
    pub fn go(&mut self, m: Move, cx: &mut Context<Self>) {
        if self.selected_row(cx).is_some() {
            self.level = self.level.forward(m);
            cx.notify();
        }
    }

    /// Esc: one level back; `false` when the report itself should close.
    pub fn back(&mut self, cx: &mut Context<Self>) -> bool {
        match self.level.back() {
            Some(level) => {
                self.level = level;
                cx.notify();
                true
            }
            None => false,
        }
    }
```

`phase()` is unchanged. Replace `render_report` with:

```rust
fn render_report(&self, report: &SyncReport, window: &mut Window, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
    let d = design();
    let (c, m) = (&d.colors, &self.catalog.messages);
    let s = &d.spacing.empty;
    let Some(list) = &self.report else { return div().into_any_element() };
    let summary = div()
        .px(pt(d.spacing.list.pad + s.summary_pad_x))
        .pt(pt(s.summary_pad_top))
        .pb(pt(s.summary_pad_bottom))
        .text_role(&d.typography.roles.description)
        .font_features(tabular_digits())
        .text_color(c.muted)
        .child(summary_line(report, m));
    let rows = || List::new(list).px(pt(d.spacing.list.pad)).py(pt(d.spacing.list.pad - d.spacing.list.gap / 2.)).into_any_element();
    let selected = list.read(cx).delegate().selected_row().cloned();
    match (self.level, selected) {
        (ReportLevel::Detail { .. }, Some(row)) => report_pane(&row, m, true).into_any_element(),
        (ReportLevel::Split, Some(row)) => h_flex()
            .size_full()
            .child(div().h_full().w(relative(d.components.split.list)).flex_none().border_r(pt(d.components.border.hairline)).border_color(c.edge).child(rows()))
            .child(div().h_full().flex_1().min_w_0().child(report_pane(&row, m, false)))
            .into_any_element(),
        _ => v_flex().size_full().child(summary).child(div().flex_1().min_h_0().child(rows())).into_any_element(),
    }
}
```

The split hides the summary line (as the approved split page does); the list shows it. Rows in the split are compact
the way `EntryList::set_compact` makes entry rows; if `Row` truncates on its own at 34 %, leave it.

`Render::render` passes `window` and `cx` to `render_report` (its signature becomes
`fn render(&mut self, window: &mut Window, cx: &mut Context<Self>)`).

- [ ] **Step 4: Run tests and build**

Run: `cargo nextest run -p kb-app && cargo clippy -p kb-app --all-targets -- -D warnings`
Expected: PASS, clean.

- [ ] **Step 5: Commit** — `feat(app): Navigate the refresh report like the list`

---

### Task 10: `SearchView` routes the report's keys, actions, page and footer

**Files:**

- Modify: `crates/kb-app/src/keys.rs`, `crates/kb-app/src/actions.rs`, `crates/kb-app/src/search_view.rs`,
  `crates/kb-app/src/search_view/footer.rs`, `crates/kb-app/src/search_view/panel.rs`
- Modify: `crates/kb-app/tests/ui.rs` (replace `reports_a_refresh_and_goes_back_with_return`)

- [ ] **Step 1: Write the failing tests**

`actions.rs` tests:

```rust
#[test]
fn offers_a_report_row_s_actions() {
    let catalog = Catalog::default();
    let labels = |row: Option<&crate::report_list::ReportRow>| {
        report_page(&catalog, Screen::Report, row).page.sections[0].rows.iter().map(|r| r.label.clone()).collect::<Vec<_>>()
    };
    let failed = failed_row(); // a ReportRow with Subject::File and an error, see below
    assert_eq!(
        labels(Some(&failed)),
        ["Details", "Edit", "Copy Location", "Copy Error", "Reveal in Finder", "Refresh", "Resize", "More…"]
    );
    let mut indexed = failed.clone();
    if let crate::report_list::Subject::File(f) = &mut indexed.subject { f.error = None; }
    assert!(!labels(Some(&indexed)).contains(&"Copy Error".to_owned()));
    assert_eq!(labels(None), ["Refresh", "Resize", "More…"], "no row: only what needs none");
}
```

Write `failed_row()` in the test module, building a `ReportRow { path: "errors/x.yml", kind: ReportKind::Failed,
entries: 0, detail: "…", subject: Subject::File(FileReport { error: Some(SourceError { line: Some(4), column: Some(1),
… }), … }) }`.

`tests/ui.rs` — replace `reports_a_refresh_and_goes_back_with_return` (drop its `#[ignore]`) with:

```rust
#[gpui_kit::test]
fn navigates_the_report_like_the_list(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    press(cx, window, "cmd-r");
    cx.run_until_parked();
    let keys = cx.read(|cx| view.read(cx).footer_keys(cx));
    assert_eq!(keys, ["Details ↵", "Edit ⌘↵", "Actions ⌘K"], "the report's keys");
    let row = || cx.read(|cx| view.read(cx).report_row_path(cx));
    assert_eq!(row().as_deref(), Some("errors/broken.yml"), "a failed file is selected first");
    press(cx, window, "down");
    assert_eq!(row().as_deref(), Some("errors/list-type.yml"));
    press(cx, window, "up");
    press(cx, window, "right");
    assert_eq!(cx.read(|cx| view.read(cx).report_level(cx)), Some(ReportLevel::Split));
    press(cx, window, "enter");
    assert_eq!(cx.read(|cx| view.read(cx).report_level(cx)), Some(ReportLevel::Detail { from_split: true }));
    assert_eq!(cx.read(|cx| view.read(cx).footer_keys(cx)), ["Edit ↵", "Copy Location ⌘↵", "Actions ⌘K"]);
    press(cx, window, "cmd-enter");
    let copied = cx.read_from_clipboard().and_then(|item| item.text()).unwrap();
    assert!(copied.ends_with("/errors/broken.yml:3:11") || copied.contains("/errors/broken.yml:"), "{copied}");
    press(cx, window, "escape");
    assert_eq!(cx.read(|cx| view.read(cx).report_level(cx)), Some(ReportLevel::Split), "Esc: back to the split");
    press(cx, window, "escape");
    assert!(!cx.read(|cx| view.read(cx).is_rebuilding()), "Esc: back to the list");
}

#[gpui_kit::test]
fn copies_a_failed_file_s_error_from_the_report(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    press(cx, window, "cmd-r");
    cx.run_until_parked();
    press(cx, window, "cmd-shift-e");
    let copied = cx.read_from_clipboard().and_then(|item| item.text()).unwrap();
    assert!(!copied.is_empty() && !copied.contains("/errors/"), "the bare message: {copied}");
    assert_eq!(cx.read(|cx| view.read(cx).last_notice().map(str::to_owned)).as_deref(), Some("Copied to clipboard"));
    press(cx, window, "cmd-k");
    let labels = cx.read(|cx| view.read(cx).panel_labels(cx));
    assert!(labels[0].contains(&"Reveal in Finder".to_owned()), "{labels:?}");
}
```

Replace the `copied.ends_with(…) || copied.contains(…)` assertion with the exact `path:line:column` once you see what
the sample's `broken.yml` error reports (run the test once and read the failure). Import
`kb_app::sync_view::ReportLevel`.

- [ ] **Step 2: Run them and see them fail**

Run: `cargo nextest run -p kb-app offers_a_report_row navigates_the_report copies_a_failed_file`
Expected: compile errors (`report_page`, `report_row_path`, `report_level`).

- [ ] **Step 3: Implement**

`keys.rs`:

- New actions in `actions!`: `/// Copy the selected report row's location. CopyLocation,`, `/// Copy the selected
  report row's error. CopyError,`, `/// Show the selected report row's file in Finder. RevealFile,`, `/// The next
  report row. NextRow,`, `/// The previous report row. PreviousRow,`, `/// The last report row. LastRow,`, `/// The
  first report row. FirstRow,`.
- `key_bindings`: `"copy-location" => KeyBinding::new(keys, CopyLocation, context)`, `"copy-error" => …CopyError…`,
  `"reveal" => …RevealFile…`; and, in the fixed bindings:

```rust
// The report has no text field: the arrows move its selection, ⌘ to its ends (spec 2026-10-27 D1).
KeyBinding::new("down", NextRow, Some(REFRESH_CONTEXT)),
KeyBinding::new("up", PreviousRow, Some(REFRESH_CONTEXT)),
KeyBinding::new("cmd-down", LastRow, Some(REFRESH_CONTEXT)),
KeyBinding::new("cmd-up", FirstRow, Some(REFRESH_CONTEXT)),
```

`actions.rs`:

```rust
/// The root on the report (spec 2026-10-27 D3): the screen's keys, then what the row allows of Edit,
/// Copy Location, Copy Error and Reveal in Finder, then Refresh, the catalog's root rows and More….
#[must_use]
pub fn report_page(catalog: &Catalog, screen: Screen, row: Option<&crate::report_list::ReportRow>) -> Page {
    let allowed = |id: &str| match (id, row) {
        ("refresh" | "resize", _) => true,
        (_, None) => false,
        ("details", Some(_)) => true,
        ("edit" | "reveal", Some(r)) => r.file().is_some(),
        ("copy-location", Some(r)) => r.location().is_some(),
        ("copy-error", Some(r)) => r.error().is_some(),
        _ => false,
    };
    let mut ids: Vec<String> = catalog.keys_for(screen, None).into_iter().map(|k| k.action).collect();
    for id in ["edit", "copy-location", "copy-error", "reveal", "refresh"] {
        if !ids.iter().any(|i| i == id) {
            ids.push(id.to_owned());
        }
    }
    let mut rows: Vec<(PanelRow, PanelCommand)> = ids.iter().filter(|id| allowed(id)).map(|id| action_row(catalog, id)).collect();
    rows.extend(catalog.panel.root.iter().map(|id| action_row(catalog, id)));
    rows.push((
        PanelRow { label: catalog.panel.more.clone(), submenu: true, ..PanelRow::default() },
        PanelCommand::Open(Submenu::More),
    ));
    let mut page = Page::default();
    page.page.searchable = true;
    page.section(None, rows);
    page
}
```

The screen keys' rows show their screen caps in `root_page`; here `action_row` shows the action's own keys. If the
footer test or the panel test expects the screen caps (`↵`, `⌘↵`), build those rows as `root_page` does
(`PanelRow { caps: Some(item.caps), … }`) for the ids that came from `keys_for`.

`search_view.rs`:

- `key_screen`: `Phase::Report` returns `KeyScreen::ReportDetail` when `sync.read(cx).level()` is
  `ReportLevel::Detail { .. }`, else `KeyScreen::Report`.
- A helper:

```rust
    /// The refresh screen's view while it shows a report, else `None`.
    fn report(&self, cx: &App) -> Option<Entity<SyncView>> {
        match &self.screen {
            Screen::Rebuild(sync) if sync.read(cx).phase() == Phase::Report => Some(sync.clone()),
            _ => None,
        }
    }

    /// The selected report row's path, for tests.
    #[must_use]
    pub fn report_row_path(&self, cx: &App) -> Option<String> {
        self.report(cx)?.read(cx).selected_row(cx).map(|r| r.path.clone())
    }

    /// The report's level, for tests.
    #[must_use]
    pub fn report_level(&self, cx: &App) -> Option<ReportLevel> {
        Some(self.report(cx)?.read(cx).level())
    }
```

- `run_action`, before the existing match, route the report's actions:

```rust
if let Some(sync) = self.report(cx) {
    let row = sync.read(cx).selected_row(cx).cloned();
    match (action, row) {
        ("details", Some(_)) => return sync.update(cx, |s, cx| s.go(Move::Detail, cx)),
        ("edit", Some(row)) => {
            if let Some(file) = row.file() {
                let (line, column) = row.line_and_column();
                self.open_in_editor(std::path::Path::new(file), line, column, window, cx);
            } else {
                self.open_preferences(&OpenPreferences, window, cx);
            }
            return;
        }
        ("copy-location", Some(row)) => return if let Some(text) = row.location() { self.copy(text, window, cx) },
        ("copy-error", Some(row)) => return if let Some(text) = row.error() { self.copy(text, window, cx) },
        ("copy-path", Some(row)) => return if let Some(file) = row.file() { self.copy(file.to_owned(), window, cx) },
        ("reveal", Some(row)) => {
            if let Some(file) = row.file() {
                cx.reveal_path(std::path::Path::new(file));
            }
            return;
        }
        ("refresh", _) => return self.refresh(sync.read(cx).selection(), window, cx),
        _ => {}
    }
}
```

(Rewrite the `return if let …` lines as blocks if rustfmt or clippy object.)

- New handlers, registered in `render` with the others:

```rust
fn copy_location(&mut self, _: &CopyLocation, window: &mut Window, cx: &mut Context<Self>) {
    self.run_action("copy-location", window, cx);
}
fn copy_error(&mut self, _: &CopyError, window: &mut Window, cx: &mut Context<Self>) {
    self.run_action("copy-error", window, cx);
}
fn reveal_file(&mut self, _: &RevealFile, window: &mut Window, cx: &mut Context<Self>) {
    self.run_action("reveal", window, cx);
}
fn report_step(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
    if self.overlay == Overlay::None && let Some(sync) = self.report(cx) {
        sync.update(cx, |s, cx| s.move_selection(delta, window, cx));
    }
}
```

with `.on_action(cx.listener(|this, _: &NextRow, w, cx| this.report_step(1, w, cx)))` and likewise `PreviousRow`
(−1), `LastRow` (`isize::MAX`), `FirstRow` (`isize::MIN`).

- `open_source_file` (⌘O) on the report: `if self.report(cx).is_some() { return self.run_action("edit", window, cx); }`
  first; `copy_path` (⌥⌘C) likewise with `"copy-path"`.
- `show_details` / `hide_details`: when `self.report(cx)` is `Some`, call `s.go(Move::Pane, cx)` /
  `s.go(Move::NoPane, cx)` and return.
- `back`: in the `match &self.screen`, before `Screen::Detail | Screen::Rebuild(_)`, add
  `Screen::Rebuild(sync) if sync.update(cx, |s, cx| s.back(cx)) => {}`; the guard runs `back` only on the report, so
  first check `sync.read(cx).phase() == Phase::Report`. Mind the borrow: clone the entity before the match.
- `refresh`: pass `window` to `SyncView::start(job, catalog, window, cx)` (Task 9).
- `retry` keeps working for stopped and failed refreshes.

`search_view/panel.rs` `current_page`: when `self.report(cx)` is `Some(sync)` and the overlay is
`Overlay::Actions(None)` with an empty query, return `report_page(&self.catalog, screen, sync.read(cx).selected_row(cx))`;
with a query, `search_page` stays (it searches the root, More…, Refresh More…, Theme…), but build its root from
`report_page` too — change `search_page` to take the root `Page` as a parameter, or add `report_search_page` that does
the same with `report_page` as the root. Pick the smaller diff.

`search_view/footer.rs`: on the report at `Split` or `Detail`, the left side is the row's tile, path and ⌥⌘C:

```rust
let report_row = self
    .report(cx)
    .filter(|s| s.read(cx).level() != ReportLevel::List)
    .and_then(|s| s.read(cx).selected_row(cx).cloned());
```

and when it is `Some(row)`: `entry_status(tile(icon, tint, TileSize::Small), row.path.clone(), copy_caps)` with
`(icon, tint) = crate::report_list::kind_tile(row.kind, colors)`.

- [ ] **Step 4: Run them and see them pass**

Run: `cargo nextest run --workspace`
Expected: PASS. Fix the copied-location assertion to the exact value (Step 1 note).

- [ ] **Step 5: Lint and gate**

Run: `cargo clippy --workspace --all-targets -- -D warnings && hk fix --all && mise run gate`
Expected: clean, except the known `bench-ts-typecheck` failure if `tools/bench/ts/node_modules` is still missing
(report it; do not change it).

- [ ] **Step 6: Commit** — `feat(app): Inspect and act on a refresh report's rows`

---

### Task 11: Shots, docs, task

**Files:**

- Modify: `tools/design/tasks.toml` (`design:shots`)
- Modify: `docs/guides/parity.md` (row Y03), `DESIGN.md` (§4 components; front matter if Task 8 added tokens),
  `CONTEXT.md` (Sync report; new Editor row), `TODO.md` (2YnZ)

- [ ] **Step 1: Shots** — in `design:shots`'s `screens(size)`, replace the report line with:

```js
key(15, 'cmd')
sleep(2500)
shot(`${size}-12-report`)
key(124)
sleep(400)
shot(`${size}-12-report-split`) // →
key(36)
sleep(400)
shot(`${size}-12-report-detail`)
reset() // ↵
```

(`reset()` presses Esc up to six times, which leaves the detail, the report and the list.) Run `mise run design:shots`
on a Mac with the app built (`mise run package` first if the task needs it) and compare the three report shots with
`docs/guides/design/12-report-pane.html` (`mise run design:prototypes` renders the page). Commit the new ONGs under
`tools/design/shots/` with the task.

- [ ] **Step 2: Docs**

- `docs/guides/parity.md` Y03: status **Ported**; port column `report_list`, `ui::report_pane`, `sync_view`; notes:
  "List, split (→) and detail (↵) as on the search list; Edit opens the configured editor at the failing line,
  Copy Location, Copy Error, Reveal in Finder; Refresh reruns the same sources. Copy Report is not ported."
- `DESIGN.md` §4, after **Empty view**: "**Report pane** (`report_pane.rs`; 12-report-pane): the row's tile, path in
  mono and status chip; the error as the red-bar quote, skipped entries and warnings in a yellow one; the lines around
  an error on mantle, the failing line with a red bar and tint; skipped keys as a list; facts as label and value
  (`Location`, `File`, `Entries`, `Exit`, `Stderr`, `Index`). The split and the detail screen draw it, as the entry
  view." Add any token from Task 8 to the front matter and run `mise run design:validate`.
- `CONTEXT.md`: _Sync report_ — append "The report navigates like the list: → its pane, ↵ full screen." Add a row:

  `| Editor | The `editor`command template in`~/.config/kb/config.toml` (`zed {file}:{line}`) that Edit opens a file
  with, at a line when known; unset, the file's default app. | `crates/kb-app/src/editor.rs` |`

- `TODO.md`: mark 2YnZ done with the `todo` CLI (usage in `mise.toml`'s REF comments), and add, as a comment on the
  task, the date and "spec 2026-10-27-refresh-report-inspection; plan of the same name".

- [ ] **Step 3: Check** — `hk fix --all && hk check --all && mise run gate`. Then by hand: `cargo run -p kb-app` with
      your sources holding one broken file: ⌘R, ↓, →, ↵, ⌘↵ (copies the location), ⌘O (opens your `editor` at the
      line), ⇧⌘R (Finder), Esc Esc.

- [ ] **Step 4: Commit** — `docs: Describe the refresh report's inspection`

---

## Spec coverage

| Spec                                                                   | Task                                                             |
| ---------------------------------------------------------------------- | ---------------------------------------------------------------- |
| D1 navigation, levels, Esc                                             | 6 (`step`), 9 (`ReportLevel`, `move_selection`), 10 (keys, back) |
| D2 screen keys                                                         | 5 (catalog), 10 (routing, footer)                                |
| D3 row actions and keys                                                | 5, 10 (`report_page`, handlers)                                  |
| D4 per-row behaviour                                                   | 6 (`file`, `location`, `error`), 10 (`run_action`)               |
| D5 pane                                                                | 8                                                                |
| D6 editor                                                              | 3, 4                                                             |
| D7 engine additions                                                    | 1, 2                                                             |
| D8 structure                                                           | 6–10                                                             |
| Error handling (missing file, bad editor, spawn failure, empty report) | 3 (`Opened`), 4 (notices), 6 (`step` on empty), 10               |
| Testing                                                                | each task; 10 end to end                                         |
| Docs and design kit                                                    | 11 (the showcase already has `errors/broken.yml`)                |
