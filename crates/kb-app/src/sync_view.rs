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
use kb_domain::report::{FileIssue, FileReport};
use kb_domain::sync::{SyncError, SyncSummary, sync_knowledge_base};

use crate::catalog::{Catalog, fill};
use crate::ui::chip::pill;
use crate::ui::design::{TextRoleExt as _, design, pt};
use crate::ui::empty_view::{EmptyView, Sentence, progress_bar};
use crate::ui::keycap::ChipState;
use crate::ui::row::Row;
use crate::ui::tile::{TileSize, tile};

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
    /// The index was rebuilt.
    Done(SyncSummary),
    /// The previous index is unchanged.
    Cancelled,
    /// The sync stopped; the previous index is unchanged.
    Failed(String),
}

/// Emitted once, when a sync stops for any reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncFinished {
    /// Whether the index was rebuilt.
    pub rebuilt: bool,
    /// What to tell the user: the summary line, or why it stopped.
    pub message: String,
}

/// Owns one sync run.
#[derive(Debug)]
pub struct SyncView {
    status: SyncStatus,
    catalog: Rc<Catalog>,
    cancel: Arc<AtomicBool>,
    _run: Task<()>,
}

impl EventEmitter<SyncFinished> for SyncView {}

/// One progress report, owned, sent from the sync thread.
struct SyncTick {
    files_scanned: usize,
    files_total: usize,
    current: String,
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

/// What a report row says about its file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportKind {
    /// The file failed to parse.
    Failed,
    /// Some of its entries were skipped.
    Skipped,
    /// Indexed in full.
    Indexed,
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
            }
        })
        .collect();
    rows.sort_by_key(|row| match row.kind {
        ReportKind::Failed => 0,
        ReportKind::Skipped => 1,
        ReportKind::Indexed => 2,
    });
    rows
}

impl SyncView {
    /// Starts rebuilding `database` from `sources`.
    pub fn start(sources: PathBuf, database: PathBuf, catalog: Rc<Catalog>, cx: &mut Context<Self>) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let run = cx.spawn(async move |view, cx| {
            let (sender, mut progress) = mpsc::unbounded::<SyncTick>();
            let job = cx.background_spawn(async move {
                sync_knowledge_base(&sources, &database, &flag, |p| {
                    let _ = sender.unbounded_send(SyncTick {
                        files_scanned: p.files_scanned,
                        files_total: p.files_total,
                        current: p.file.rel_path.clone(),
                    });
                })
            });
            while let Some(p) = progress.next().await {
                let _ = view.update(cx, |view, cx| {
                    view.status = SyncStatus::Running {
                        files_scanned: p.files_scanned,
                        files_total: p.files_total,
                        current: p.current,
                    };
                    cx.notify();
                });
            }
            let outcome = job.await;
            let _ = view.update(cx, |view, cx| {
                let rebuilt = outcome.is_ok();
                view.status = match outcome {
                    Ok(summary) => SyncStatus::Done(summary),
                    Err(SyncError::Cancelled) => SyncStatus::Cancelled,
                    Err(error) => SyncStatus::Failed(error.to_string()),
                };
                let m = &view.catalog.messages;
                let message = match &view.status {
                    SyncStatus::Done(summary) => summary_line(summary, m),
                    SyncStatus::Cancelled => m.stopped_title.clone(),
                    SyncStatus::Failed(error) => fill(&m.failed, &[("error", error)]),
                    SyncStatus::Running { .. } => String::new(),
                };
                cx.emit(SyncFinished { rebuilt, message });
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
            _run: run,
        }
    }

    /// Whether the sync is still running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self.status, SyncStatus::Running { .. })
    }

    /// The screen to show.
    #[must_use]
    pub fn phase(&self) -> Phase {
        match self.status {
            SyncStatus::Running { .. } => Phase::Running,
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

fn issues_line(file: &FileReport) -> String {
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

/// Summarizes a sync from the catalog's message template, with failed and skipped file counts.
#[must_use]
pub fn summary_line(summary: &SyncSummary, messages: &crate::catalog::Messages) -> String {
    let rows = report_rows(&summary.files);
    let failed = rows.iter().filter(|r| r.kind == ReportKind::Failed).count();
    let skipped = rows.iter().filter(|r| r.kind == ReportKind::Skipped).count();
    let mut line = fill(
        &messages.report_summary,
        &[
            ("entries", &thousands(summary.total_indexed)),
            ("files", &summary.files_scanned.to_string()),
            (
                "seconds",
                &format!(
                    "{:.1}",
                    Duration::from_millis(u64::try_from(summary.duration_ms).unwrap_or(u64::MAX)).as_secs_f64()
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
            SyncStatus::Done(summary) => {
                let rows = report_rows(&summary.files);
                let line = summary_line(summary, m);
                let s = &d.spacing.empty;
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
                    .children(rows.into_iter().enumerate().map(|(ix, row)| {
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
                        };
                        Row::new(SharedString::from(format!("report-{ix}")), row.path.clone())
                            .lead(tile(icon, tint, TileSize::Row))
                            .mono(true)
                            .description(Some(detail))
                            .trailing(vec![pill(chip, ChipState::of(ix == 0)).into_any_element()])
                            .selected(ix == 0)
                    }))
                    .into_any_element()
            }
        }
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

    #[test]
    fn summarizes_a_sync() {
        let summary = SyncSummary {
            total_indexed: 4233,
            files_scanned: 68,
            files_failed: 1,
            duration_ms: 812,
            errors: Vec::new(),
            files: vec![file("errors/x.yml", 0, true, false)],
        };
        let catalog = Catalog::default();
        assert_eq!(
            summary_line(&summary, &catalog.messages),
            "4,233 entries from 68 files in 0.8 s · 1 failed"
        );
    }

    use indexmap::IndexMap;
    use kb_domain::report::{FileIssue, FileReport};
    use kb_domain::source::SourceError;

    fn file(rel: &str, entries: usize, error: bool, skipped: bool) -> FileReport {
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
            file("b.yml", 9, false, true),
            file("a.yml", 12, false, false),
            file("errors/x.yml", 0, true, false),
        ];
        let order: Vec<(String, ReportKind)> = report_rows(&files).into_iter().map(|r| (r.path, r.kind)).collect();
        assert_eq!(
            order,
            [
                ("errors/x.yml".into(), ReportKind::Failed),
                ("b.yml".into(), ReportKind::Skipped),
                ("a.yml".into(), ReportKind::Indexed)
            ]
        );
    }
}
