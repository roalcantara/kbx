//! The report's rows, sections, and selection.

use std::rc::Rc;

use gpui_kit::base::Selectable as _;
use gpui_kit::component::IndexPath;
use gpui_kit::component::list::{ListDelegate, ListState};
use gpui_kit::{App, Context, IntoElement, ParentElement as _, SharedString, Styled as _, Window, div};
use kb_domain::report::{FileIssue, FileReport};
use kb_domain::skills::SkillsWarning;
use kb_domain::sync::{SkillsOutcome, SyncReport, YamlOutcome};

use crate::catalog::{Catalog, Messages, fill};
use crate::ui::chip::pill;
use crate::ui::design::{TextRoleExt as _, design, pt};
use crate::ui::keycap::ChipState;
use crate::ui::row::Row;
use crate::ui::tile::{TileSize, tile};
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

/// What a report row says about its file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportKind {
    /// The file failed to parse.
    Failed,
    /// Some of its entries were skipped.
    Skipped,
    /// Indexed in full.
    Indexed,
    /// Something the Skills command or a Skill's `SKILL.md` warned about.
    Warning,
    /// The Skills command is not configured.
    NotConfigured,
}

/// One report row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportRow {
    /// The file, relative to the sources directory.
    pub path: String,
    /// What happened.
    pub kind: ReportKind,
    /// How many entries it indexed.
    pub entries: usize,
    /// For a failed file, its error; for a skipped one, the skipped entries.
    pub detail: String,
    /// What it is about.
    pub subject: Subject,
}

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

/// The icon and tint for `kind`.
#[must_use]
pub fn kind_tile(kind: ReportKind, c: &crate::ui::design::Colors) -> (&'static str, gpui_kit::Hsla) {
    match kind {
        ReportKind::Failed => ("circle-x", c.red),
        ReportKind::Skipped | ReportKind::Warning => ("triangle-alert", c.yellow),
        ReportKind::Indexed => ("circle-check", c.green),
        ReportKind::NotConfigured => ("circle", c.faint),
    }
}

/// One row in the report list.
#[must_use]
pub fn report_row(row: &ReportRow, id: &str, selected: bool, m: &Messages) -> Row {
    let d = design();
    let c = &d.colors;
    let (icon, tint) = kind_tile(row.kind, c);
    let (chip, detail) = match row.kind {
        ReportKind::Failed => (m.report_failed_chip.clone(), row.detail.clone()),
        ReportKind::Skipped => (
            row.entries.to_string(),
            // A file with no entries at all has no skipped entries to name.
            if row.detail.is_empty() {
                m.report_empty_file.clone()
            } else {
                fill(&m.report_skipped_entries, &[("entries", &row.detail)])
            },
        ),
        ReportKind::Indexed => (row.entries.to_string(), m.report_indexed.clone()),
        ReportKind::Warning => (m.report_warning_chip.clone(), row.detail.clone()),
        ReportKind::NotConfigured => (m.report_not_configured_chip.clone(), row.detail.clone()),
    };
    Row::new(SharedString::from(id.to_owned()), row.path.clone())
        .lead(tile(icon, tint, TileSize::Row))
        .mono(true)
        .description(Some(detail))
        .trailing(vec![pill(chip, ChipState::of(selected)).into_any_element()])
        .selected(selected)
}

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
        Self {
            sections,
            catalog,
            selected,
        }
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

    fn render_section_header(
        &mut self,
        section: usize,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Option<impl IntoElement> {
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
        Some(report_row(
            row,
            &format!("report-{}-{}", ix.section, ix.row),
            selected,
            &self.catalog.messages,
        ))
    }

    fn set_selected_index(&mut self, ix: Option<IndexPath>, _: &mut Window, cx: &mut Context<ListState<Self>>) {
        self.selected = ix;
        cx.notify();
    }
}

impl ReportList {
    /// Sets the selection to `ix`.
    pub fn set_selected(&mut self, ix: Option<IndexPath>) {
        self.selected = ix;
    }
}

/// The row `delta` rows from `from` (`isize::MAX` / `MIN`: the last / first), across sections of
/// `counts` rows each, clamped to the ends; empty sections are skipped.
#[must_use]
pub fn step(from: IndexPath, delta: isize, counts: &[usize]) -> IndexPath {
    let flat: Vec<IndexPath> = counts
        .iter()
        .enumerate()
        .flat_map(|(section, &n)| {
            (0..n).map(move |row| IndexPath {
                section,
                row,
                ..IndexPath::default()
            })
        })
        .collect();
    let Some(last) = flat.len().checked_sub(1) else {
        return from;
    };
    let at = if let Some(pos) = flat
        .iter()
        .position(|ix| ix.section == from.section && ix.row == from.row)
    {
        pos.saturating_add_signed(delta)
    } else {
        // `from` is not in flat (e.g. at(0, 0) with counts [0, 2]).
        // Find the first index whose section >= from.section
        let next = flat.iter().position(|ix| ix.section >= from.section).unwrap_or(0);
        if delta > 0 {
            next.saturating_add_signed(delta - 1)
        } else {
            next.saturating_add_signed(delta)
        }
    };
    flat[at.min(last)]
}

/// Helper formatting skipped entries.
#[must_use]
pub fn issues_line(file: &FileReport) -> String {
    let skipped: Vec<String> = file
        .issues
        .iter()
        .filter_map(|issue| match issue {
            FileIssue::SkippedEntry { entry_type, key } => Some(format!("{entry_type}/{key}")),
            FileIssue::Empty => None,
        })
        .collect();
    skipped.join(", ")
}

/// Every file, failed first, then those with skipped entries, then the indexed ones (R5).
#[must_use]
pub fn report_rows(files: &[FileReport]) -> Vec<ReportRow> {
    let mut rows: Vec<ReportRow> = files
        .iter()
        .map(|file| {
            let (kind, detail) = if let Some(error) = &file.error {
                (
                    ReportKind::Failed,
                    error.message.lines().next().unwrap_or_default().to_owned(),
                )
            } else if !file.issues.is_empty() {
                (ReportKind::Skipped, issues_line(file))
            } else {
                (ReportKind::Indexed, String::new())
            };
            ReportRow {
                path: file.rel_path.clone(),
                kind,
                entries: file.entries,
                detail,
                subject: Subject::File(file.clone()),
            }
        })
        .collect();
    rows.sort_by_key(|row| match row.kind {
        ReportKind::Failed => 0,
        ReportKind::Skipped => 1,
        ReportKind::Indexed => 2,
        ReportKind::Warning => 3,
        ReportKind::NotConfigured => 4,
    });
    rows
}

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
                subject: Subject::SourcesDir(sources_dir.to_owned()),
            }],
        };
        sections.push(ReportSection {
            label: m.report_yaml_section.clone(),
            rows,
        });
    }
    if let Some(skills) = &report.skills {
        let row = |kind, entries, detail: String, subject: Subject| ReportRow {
            path: kb_domain::skills::SKILLS_TYPE.to_owned(),
            kind,
            entries,
            detail,
            subject,
        };
        let (outcome, warnings): (ReportRow, &[SkillsWarning]) = match skills {
            SkillsOutcome::NotConfigured => (
                row(
                    ReportKind::NotConfigured,
                    0,
                    m.report_not_configured.clone(),
                    Subject::Skills(skills.clone()),
                ),
                &[],
            ),
            SkillsOutcome::Indexed { entries, warnings } => (
                row(
                    ReportKind::Indexed,
                    *entries,
                    String::new(),
                    Subject::Skills(skills.clone()),
                ),
                warnings.as_slice(),
            ),
            SkillsOutcome::Failed { reason, warnings, .. } => (
                row(
                    ReportKind::Failed,
                    0,
                    fill(&m.report_source_failed, &[("error", reason)]),
                    Subject::Skills(skills.clone()),
                ),
                warnings.as_slice(),
            ),
        };
        let mut rows = vec![outcome];
        rows.extend(
            warnings
                .iter()
                .map(|w| row(ReportKind::Warning, 0, w.message.clone(), Subject::Warning(w.clone()))),
        );
        sections.push(ReportSection {
            label: m.report_skills_section.clone(),
            rows,
        });
    }
    sections
}

#[cfg(test)]
pub(crate) mod tests {
    use indexmap::IndexMap;
    use kb_domain::report::{FileIssue, FileReport};
    use kb_domain::source::SourceError;
    use kb_domain::sync::SyncSummary;

    use super::*;
    use crate::catalog::Catalog;

    pub(crate) fn yaml_summary(files: Vec<FileReport>, total: usize) -> SyncSummary {
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
    fn selects_the_first_row_and_counts_rows_per_section() {
        let m = Catalog::default().messages;
        let report = SyncReport {
            yaml: Some(YamlOutcome::Indexed(yaml_summary(
                vec![file("a.yml", 3, false, false), file("e.yml", 0, true, false)],
                3,
            ))),
            skills: Some(SkillsOutcome::NotConfigured),
            duration_ms: 0,
        };
        let list = ReportList::new(report_sections(&report, "/s", &m), Rc::new(Catalog::default()));
        assert_eq!(list.counts(), [2, 1]);
        assert_eq!(
            list.selected_row().map(|r| r.path.as_str()),
            Some("e.yml"),
            "failed first, selected"
        );
    }

    pub(crate) fn file(rel: &str, entries: usize, error: bool, skipped: bool) -> FileReport {
        FileReport {
            rel_path: rel.into(),
            file_path: format!("/s/{rel}"),
            entries,
            types: IndexMap::new(),
            error: error.then(|| SourceError {
                file_path: format!("/s/{rel}"),
                message: "Line 4: a flow sequence must be closed".into(),
                line: Some(4),
                column: Some(1),
            }),
            excerpt: None,
            issues: if skipped {
                vec![FileIssue::SkippedEntry {
                    entry_type: "bookmarks".into(),
                    key: "not-a-map".into(),
                }]
            } else {
                Vec::new()
            },
        }
    }

    #[test]
    fn reports_skills_in_a_section_of_their_own() {
        let m = Catalog::default().messages;
        let report = SyncReport {
            yaml: None,
            skills: Some(SkillsOutcome::Failed {
                reason: "skills exited with exit status: 1".into(),
                command: "skills ls".into(),
                exit_code: Some(1),
                warnings: vec![SkillsWarning::stderr("⚠ Skipped recon")],
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
    }

    #[test]
    fn reports_yaml_files_then_skills_not_configured() {
        let m = Catalog::default().messages;
        let report = SyncReport {
            yaml: Some(YamlOutcome::Indexed(yaml_summary(
                vec![file("a.yml", 3, false, false)],
                3,
            ))),
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

    #[test]
    fn leaves_the_detail_empty_for_a_file_with_no_entries() {
        let empty = FileReport {
            issues: vec![FileIssue::Empty],
            ..file("empty.yml", 0, false, false)
        };
        let row = &report_rows(&[empty])[0];
        assert_eq!((row.kind, row.detail.as_str()), (ReportKind::Skipped, ""));
    }

    #[test]
    fn lists_every_file_failed_first_then_skipped_then_indexed() {
        let files = vec![
            file("indexed.yml", 5, false, false),
            file("skipped.yml", 2, false, true),
            file("failed.yml", 0, true, false),
        ];
        let rows = report_rows(&files);
        let paths: Vec<&str> = rows.iter().map(|r| r.path.as_str()).collect();
        assert_eq!(paths, ["failed.yml", "skipped.yml", "indexed.yml"]);
    }

    #[test]
    fn steps_through_rows_across_sections_and_stops_at_the_ends() {
        let counts = [3, 2]; // YAML: 3 files; Skills: outcome + 1 warning
        let at = |section, row| IndexPath {
            section,
            row,
            ..IndexPath::default()
        };
        assert_eq!(
            step(at(0, 2), 1, &counts),
            at(1, 0),
            "↓ from YAML's last row enters Skills"
        );
        assert_eq!(step(at(1, 0), -1, &counts), at(0, 2), "↑ goes back");
        assert_eq!(step(at(1, 1), 1, &counts), at(1, 1), "stops at the last row");
        assert_eq!(step(at(0, 0), -1, &counts), at(0, 0), "stops at the first row");
        assert_eq!(
            step(at(0, 0), isize::MAX, &counts),
            at(1, 1),
            "⌘↓ jumps to the last row"
        );
        assert_eq!(
            step(at(1, 1), isize::MIN, &counts),
            at(0, 0),
            "⌘↑ jumps to the first row"
        );
        assert_eq!(step(at(0, 0), 1, &[0, 2]), at(1, 0), "an empty section is skipped");
    }

    #[test]
    fn knows_each_row_s_file_location_and_error() {
        let m = Catalog::default().messages;
        let failed = file("errors/x.yml", 0, true, false); // line 4, column 1, see `file`
        let report = SyncReport {
            yaml: Some(YamlOutcome::Indexed(yaml_summary(
                vec![failed, file("a.yml", 3, false, false)],
                3,
            ))),
            skills: Some(SkillsOutcome::Indexed {
                entries: 1,
                warnings: vec![
                    SkillsWarning {
                        message: "/x: cannot read".into(),
                        file: Some("/k/x/SKILL.md".into()),
                    },
                    SkillsWarning::stderr("⚠ careful"),
                ],
            }),
            duration_ms: 0,
        };
        let rows: Vec<ReportRow> = report_sections(&report, "/s", &m)
            .into_iter()
            .flat_map(|s| s.rows)
            .collect();
        let facts: Vec<(Option<String>, Option<String>, Option<String>)> = rows
            .iter()
            .map(|r| (r.file().map(str::to_owned), r.location(), r.error()))
            .collect();
        assert_eq!(
            facts,
            [
                (
                    Some("/s/errors/x.yml".into()),
                    Some("/s/errors/x.yml:4:1".into()),
                    Some("Line 4: a flow sequence must be closed".into())
                ),
                (Some("/s/a.yml".into()), Some("/s/a.yml".into()), None),
                (None, None, None),
                (Some("/k/x/SKILL.md".into()), Some("/k/x/SKILL.md".into()), None),
                (None, None, None),
            ]
        );
        assert_eq!(rows[0].line_and_column(), (Some(4), Some(1)));
    }
}
