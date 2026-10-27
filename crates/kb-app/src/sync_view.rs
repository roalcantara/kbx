//! The rebuild screen: a sync on the background executor, its progress, and its report.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures::StreamExt as _;
use futures::channel::mpsc;
use gpui_kit::component::list::{List, ListState};
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::{
    App, AppContext as _, Context, Entity, EventEmitter, IntoElement, ParentElement as _, Render, Styled as _, Task,
    Window, div, relative,
};
use kb_domain::selection::Selection;
use kb_domain::sync::{Progress, SyncError, SyncReport, SyncRequest, sync_sources};

use crate::catalog::{Catalog, Messages, fill};
use crate::report_list::{ReportKind, ReportList, ReportRow, report_sections, step};
use crate::ui::design::{TextRoleExt as _, design, pt, tabular_digits};
use crate::ui::empty_view::{EmptyView, Sentence, progress_bar};
use crate::ui::report_pane::report_pane;

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

/// Where the report stands (spec 2026-10-27 D1).
#[cfg_attr(not(test), expect(dead_code, reason = "wired into SearchView in Task 10"))]
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
#[cfg_attr(not(test), expect(dead_code, reason = "wired into SearchView in Task 10"))]
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
            (level, Move::Detail) => Self::Detail {
                from_split: level == Self::Split,
            },
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

/// Owns one sync run.
#[derive(Debug)]
pub struct SyncView {
    status: SyncStatus,
    catalog: Rc<Catalog>,
    cancel: Arc<AtomicBool>,
    job: SyncJob,
    report: Option<Entity<ListState<ReportList>>>,
    level: ReportLevel,
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
    pub fn start(job: SyncJob, catalog: Rc<Catalog>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let run_job = job.clone();
        let run = cx.spawn_in(window, async move |view, cx| {
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
            let _ = view.update_in(cx, |view, window, cx| {
                let changed = matches!(outcome, Ok(_) | Err(SyncError::Cancelled));
                view.status = match outcome {
                    Ok(report) => SyncStatus::Done(report),
                    Err(SyncError::Cancelled) => SyncStatus::Cancelled,
                    Err(error) => SyncStatus::Failed(error.to_string()),
                };
                if let SyncStatus::Done(report) = &view.status {
                    let sections =
                        report_sections(report, &view.job.sources.display().to_string(), &view.catalog.messages);
                    let catalog = view.catalog.clone();
                    view.report =
                        Some(cx.new(|cx| {
                            ListState::new(ReportList::new(sections, catalog), window, cx).searchable(false)
                        }));
                }
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
            report: None,
            level: ReportLevel::List,
            _run: run,
        }
    }

    /// The report's level.
    #[expect(dead_code, reason = "wired into SearchView in Task 10")]
    #[must_use]
    pub fn level(&self) -> ReportLevel {
        self.level
    }

    /// The selected report row.
    #[expect(dead_code, reason = "wired into SearchView in Task 10")]
    #[must_use]
    pub fn selected_row<'a>(&self, cx: &'a App) -> Option<&'a ReportRow> {
        self.report.as_ref()?.read(cx).delegate().selected_row()
    }

    /// Moves the selection `delta` rows (`isize::MAX` / `MIN`: last / first) and scrolls to it.
    #[expect(dead_code, reason = "wired into SearchView in Task 10")]
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
    #[expect(dead_code, reason = "wired into SearchView in Task 10")]
    pub fn go(&mut self, m: Move, cx: &mut Context<Self>) {
        if self.selected_row(cx).is_some() {
            self.level = self.level.forward(m);
            cx.notify();
        }
    }

    /// Esc: one level back; `false` when the report itself should close.
    #[expect(dead_code, reason = "wired into SearchView in Task 10")]
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
            SyncStatus::Done(report) => self.render_report(report, window, cx),
        }
    }
}

impl SyncView {
    fn render_report(&self, report: &SyncReport, _: &mut Window, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let d = design();
        let (c, m) = (&d.colors, &self.catalog.messages);
        let s = &d.spacing.empty;
        let Some(list) = &self.report else {
            return div().into_any_element();
        };
        let summary = div()
            .px(pt(d.spacing.list.pad + s.summary_pad_x))
            .pt(pt(s.summary_pad_top))
            .pb(pt(s.summary_pad_bottom))
            .text_role(&d.typography.roles.description)
            .font_features(tabular_digits())
            .text_color(c.muted)
            .child(summary_line(report, m));
        let rows = || {
            List::new(list)
                .px(pt(d.spacing.list.pad))
                .py(pt(d.spacing.list.pad - d.spacing.list.gap / 2.))
                .into_any_element()
        };
        let selected = list.read(cx).delegate().selected_row().cloned();
        match (self.level, selected) {
            (ReportLevel::Detail { .. }, Some(row)) => report_pane(&row, m, true).into_any_element(),
            (ReportLevel::Split, Some(row)) => h_flex()
                .size_full()
                .child(
                    div()
                        .h_full()
                        .w(relative(d.components.split.list))
                        .flex_none()
                        .border_r(pt(d.components.border.hairline))
                        .border_color(c.edge)
                        .child(rows()),
                )
                .child(div().h_full().flex_1().min_w_0().child(report_pane(&row, m, false)))
                .into_any_element(),
            _ => v_flex()
                .size_full()
                .child(summary)
                .child(div().flex_1().min_h_0().child(rows()))
                .into_any_element(),
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
    fn steps_back_one_level_at_a_time() {
        let mut level = ReportLevel::List;
        level = level.forward(Move::Pane);
        assert_eq!(level, ReportLevel::Split);
        level = level.forward(Move::Detail);
        assert_eq!(level, ReportLevel::Detail { from_split: true });
        assert_eq!(
            level.back(),
            Some(ReportLevel::Split),
            "Esc on the detail returns to the split"
        );
        assert_eq!(ReportLevel::Split.back(), None, "Esc on the report leaves it");
        assert_eq!(ReportLevel::List.forward(Move::Detail).back(), Some(ReportLevel::List));
        assert_eq!(ReportLevel::Split.forward(Move::NoPane), ReportLevel::List);
        assert_eq!(
            ReportLevel::Detail { from_split: false }.forward(Move::Pane),
            ReportLevel::Detail { from_split: false },
            "→ does nothing on the detail"
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
