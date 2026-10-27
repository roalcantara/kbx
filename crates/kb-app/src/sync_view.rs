//! The rebuild screen: a sync on the background executor, its progress, and its report.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures::StreamExt as _;
use futures::channel::mpsc;
use gpui_kit::base::Selectable as _;
use gpui_kit::component::v_flex;
use gpui_kit::{
    AppContext as _, Context, EventEmitter, InteractiveElement as _, IntoElement, ParentElement as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Task, Window, div,
};
use kb_domain::selection::Selection;
use kb_domain::sync::{Progress, SyncError, SyncReport, SyncRequest, sync_sources};

use crate::catalog::{Catalog, Messages, fill};
use crate::report_list::{ReportKind, ReportRow, report_sections};
use crate::ui::chip::pill;
use crate::ui::design::{TextRoleExt as _, design, pt, tabular_digits};
use crate::ui::empty_view::{EmptyView, Sentence, progress_bar};
use crate::ui::keycap::ChipState;
use crate::ui::row::Row;
use crate::ui::tile::{TileSize, tile};

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
/// Where a sync stands.
#[derive(Debug)]
pub enum SyncStatus {
    /// Files are being read.
    Running {
        /// Files processed.
        files_scanned: usize,
        /// Files in all, once known.
        files_total: usize,
        /// The file just processed.
        current: String,
    },
    /// The Skills command is running.
    RunningSkills,
    /// The selected sources were refreshed.
    Done(SyncReport),
    /// The previous index is unchanged.
    Cancelled,
    /// The sync stopped; the previous index is unchanged.
    Failed(String),
}

/// Emitted once, when a sync stops for any reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncFinished {
    /// Whether the index may have changed: it finished, or it stopped after committing a source.
    pub changed: bool,
    pub message: String,
}

/// Owns one sync run.
#[derive(Debug)]
pub struct SyncView {
    status: SyncStatus,
    catalog: Rc<Catalog>,
    cancel: Arc<AtomicBool>,
    job: SyncJob,
    _run: Task<()>,
}

impl EventEmitter<SyncFinished> for SyncView {}

/// One progress report, owned, sent from the sync thread.
enum SyncTick {
    File {
        files_scanned: usize,
        files_total: usize,
        current: String,
    },
    Skills,
}

/// Which refresh screen shows (D16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Files are being read.
    Running,
    /// Finished: the report.
    Report,
    /// Stopped by Esc; the previous index is unchanged.
    Stopped,
    /// Failed; the previous index is unchanged.
    Failed,
}

impl SyncView {
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

    /// Whether the sync is still running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self.status, SyncStatus::Running { .. } | SyncStatus::RunningSkills)
    }

    /// The screen to show.
    #[must_use]
    pub fn phase(&self) -> Phase {
        match self.status {
            SyncStatus::Running { .. } | SyncStatus::RunningSkills => Phase::Running,
            SyncStatus::Done(_) => Phase::Report,
            SyncStatus::Cancelled => Phase::Stopped,
            SyncStatus::Failed(_) => Phase::Failed,
        }
    }

    /// Asks a running sync to stop; it rolls back and reports [`SyncStatus::Cancelled`].
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

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

fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (ix, c) in digits.chars().enumerate() {
        if ix > 0 && (digits.len() - ix).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

impl Render for SyncView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let d = design();
        let (c, m) = (&d.colors, &self.catalog.messages);
        match &self.status {
            SyncStatus::Running {
                files_scanned,
                files_total,
                current,
            } => {
                #[expect(clippy::cast_precision_loss, reason = "a fraction of a few thousand files")]
                let fraction = if *files_total == 0 {
                    0.
                } else {
                    *files_scanned as f32 / *files_total as f32
                };
                let progress = fill(
                    &m.refreshing_progress,
                    &[
                        ("done", &files_scanned.to_string()),
                        ("total", &files_total.to_string()),
                        ("file", current),
                    ],
                );
                EmptyView::new(
                    "refresh-cw",
                    c.faint,
                    m.refreshing_title.clone(),
                    Sentence::of(&progress, ""),
                )
                .extra(progress_bar(fraction))
                .into_any_element()
            }
            SyncStatus::RunningSkills => EmptyView::new(
                "refresh-cw",
                c.faint,
                m.refreshing_title.clone(),
                Sentence::of(&m.running_skills, ""),
            )
            .into_any_element(),
            SyncStatus::Cancelled => EmptyView::new(
                "circle-stop",
                c.yellow,
                m.stopped_title.clone(),
                Sentence::of(&m.stopped, ""),
            )
            .into_any_element(),
            SyncStatus::Failed(error) => {
                let text = fill(&m.failed, &[("error", error)]);
                EmptyView::new("circle-alert", c.red, m.failed_title.clone(), Sentence::of(&text, ""))
                    .into_any_element()
            }
            SyncStatus::Done(report) => self.render_report(report, &d, c, m).into_any_element(),
        }
    }
}

impl SyncView {
    fn render_report(
        &self,
        report: &SyncReport,
        d: &crate::ui::design::Design,
        c: &crate::ui::design::Colors,
        m: &Messages,
    ) -> impl IntoElement {
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
                    .font_features(tabular_digits())
                    .text_color(c.muted)
                    .child(line),
            )
            .children(children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_thousands() {
        assert_eq!(
            [thousands(7), thousands(4233), thousands(42_330), thousands(1_000_000)],
            ["7", "4,233", "42,330", "1,000,000"]
        );
    }

    use crate::report_list::tests::{file, yaml_summary};
    use kb_domain::sync::{SkillsOutcome, YamlOutcome};

    #[test]
    fn summarizes_every_source() {
        let report = SyncReport {
            yaml: Some(YamlOutcome::Indexed(yaml_summary(
                vec![file("errors/x.yml", 0, true, false)],
                4233,
            ))),
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
}
