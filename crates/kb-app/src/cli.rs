//! `kb [--trace | --sync [--source all|yaml|skills]]`: what the command line asks for, and the headless sync.

use std::io::Write;
use std::sync::atomic::AtomicBool;

use kb_domain::selection::Selection;
use kb_domain::sync::{SkillsOutcome, SyncError, SyncReport, SyncRequest, YamlOutcome, sync_sources};

use crate::config::Config;

/// What `kb` was asked to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Run the panel.
    Panel {
        /// Log the show and keystroke latencies to stderr (`--trace`).
        trace: bool,
    },
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
/// An unknown argument, or `--trace` with `--sync`.
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
            let _ = writeln!(
                out,
                "kb: yaml: failed: {reason}; the previous YAML entries are unchanged"
            );
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
            let _ = writeln!(
                out,
                "kb: skills: not configured (install skills, or set skills_command)"
            );
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn runs_the_panel_without_arguments() {
        assert_eq!(parse(args(&[])), Ok(Mode::Panel { trace: false }));
    }

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
    /// A Skills command printing `stdout`, so a test never runs the machine's own `skills`.
    fn fake_skills(dir: &std::path::Path, stdout: &str, code: i32) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt as _;
        let path = dir.join("skills");
        fs::write(
            &path,
            format!("#!/bin/sh\necho '{stdout}'\necho 'careful' >&2\nexit {code}\n"),
        )
        .unwrap();
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
        assert!(
            out.contains("kb: yaml: indexed 1 entries from 2 files (1 failed)"),
            "{out}"
        );
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
}
