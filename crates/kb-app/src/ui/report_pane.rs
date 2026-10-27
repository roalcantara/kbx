//! A report row's pane (spec 2026-10-27 D5; `docs/guides/design/12-report-pane.html`): its header,

//! the error or warning as a quote, the lines around an error, the skipped keys, and its facts.
//! The split and the detail screen draw the same pane.

use kb_domain::report::{FileIssue, FileReport};
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
    let mut pane = PaneContent {
        chip: row.entries.to_string(),
        ..PaneContent::default()
    };
    match &row.subject {
        Subject::File(file) => file_content(&mut pane, row, file, m),
        Subject::SourcesDir(path) => {
            pane.chip.clone_from(&m.report_failed_chip);
            pane.quote = Some((Tone::Error, row.detail.clone()));
            pane.facts.push((m.report_pane_file.clone(), Fact::Mono(path.clone())));
        }
        Subject::Skills(skills) => skills_content(&mut pane, skills, m),
        Subject::Warning(warning) => {
            pane.chip.clone_from(&m.report_warning_chip);
            pane.quote = Some((Tone::Warning, warning.message.clone()));
            if let Some(file) = &warning.file {
                pane.facts.push((m.report_pane_file.clone(), Fact::Mono(file.clone())));
            }
        }
    }
    pane
}

fn file_content(pane: &mut PaneContent, row: &ReportRow, file: &FileReport, m: &Messages) {
    let file_fact = (m.report_pane_file.clone(), Fact::Mono(file.file_path.clone()));
    if let Some(error) = &file.error {
        pane.chip.clone_from(&m.report_failed_chip);
        pane.quote = Some((Tone::Error, error.message.clone()));
        pane.excerpt = file.excerpt.as_deref().map(excerpt_lines).unwrap_or_default();
        if let Some(line) = error.line {
            let column = error.column.map(|c| c.to_string()).unwrap_or_default();
            let text = fill(&m.report_pane_line, &[("line", &line.to_string()), ("column", &column)]);
            pane.facts.push((
                m.report_pane_location.clone(),
                Fact::Pills(vec![text.trim_end_matches(':').to_owned()]),
            ));
        }
        pane.facts.push(file_fact);
        pane.facts
            .push((m.report_pane_index.clone(), Fact::Text(m.report_pane_kept.clone())));
        return;
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
        pane.chip = fill(
            &m.report_pane_skipped_chip,
            &[("entries", &row.entries.to_string()), ("count", &count)],
        );
        let text = if skipped.len() == 1 {
            m.report_pane_one_skipped.clone()
        } else {
            fill(&m.report_pane_skipped, &[("count", &count)])
        };
        pane.quote = Some((Tone::Warning, text));
        pane.list = skipped
            .iter()
            .map(|(t, k)| fill(&m.report_pane_skipped_key, &[("type", t), ("key", k)]))
            .collect();
    }
    if !file.types.is_empty() {
        let pills = file.types.iter().map(|(t, n)| format!("{t} {n}")).collect();
        pane.facts.push((m.report_pane_entries.clone(), Fact::Pills(pills)));
    }
    pane.facts.push(file_fact);
}

fn skills_content(pane: &mut PaneContent, skills: &SkillsOutcome, m: &Messages) {
    match skills {
        SkillsOutcome::Failed {
            reason,
            command,
            exit_code,
            warnings,
        } => {
            pane.chip.clone_from(&m.report_failed_chip);
            pane.quote = Some((Tone::Error, reason.clone()));
            pane.command = Some(command.clone());
            if let Some(code) = exit_code {
                pane.facts
                    .push((m.report_pane_exit.clone(), Fact::Pills(vec![code.to_string()])));
            }
            let stderr = warnings
                .iter()
                .filter(|w| w.file.is_none())
                .map(|w| w.message.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let stderr = if stderr.is_empty() {
                m.report_pane_nothing.clone()
            } else {
                stderr
            };
            pane.facts.push((m.report_pane_stderr.clone(), Fact::Mono(stderr)));
            pane.facts.push((
                m.report_pane_index.clone(),
                Fact::Text(m.report_pane_skills_kept.clone()),
            ));
        }
        SkillsOutcome::NotConfigured => {
            pane.chip.clone_from(&m.report_not_configured_chip);
            pane.quote = Some((Tone::Warning, m.report_not_configured.clone()));
        }
        SkillsOutcome::Indexed { .. } => {}
    }
}

fn facts_element(
    facts: Vec<(String, Fact)>,
    d: &crate::ui::design::Design,
    c: &crate::ui::design::Colors,
) -> Option<impl gpui_kit::IntoElement> {
    use crate::ui::chip::pill;
    use crate::ui::design::{TextRoleExt as _, pt};
    use crate::ui::keycap::ChipState;
    use gpui_kit::component::h_flex;
    use gpui_kit::component::v_flex;
    use gpui_kit::{IntoElement as _, ParentElement as _, Styled as _, div};

    let p = &d.spacing.pane;
    (!facts.is_empty()).then(|| {
        v_flex()
            .gap(pt(p.header_gap))
            .text_role(&d.typography.roles.description)
            .children(facts.into_iter().map(|(label, value)| {
                h_flex()
                    .gap(pt(p.gap))
                    .child(
                        div()
                            .w(pt(d.components.pane.fact_label))
                            .flex_none()
                            .text_color(c.faint)
                            .child(label),
                    )
                    .child(match value {
                        Fact::Pills(pills) => h_flex()
                            .gap(pt(d.spacing.row.tags_gap))
                            .children(pills.into_iter().map(|t| pill(t, ChipState::Rest)))
                            .into_any_element(),
                        Fact::Mono(text) => div()
                            .min_w_0()
                            .font_family(d.typography.mono.clone())
                            .text_color(c.muted)
                            .child(text)
                            .into_any_element(),
                        Fact::Text(text) => div().text_color(c.muted).child(text).into_any_element(),
                    })
            }))
    })
}

/// The pane of `row`, padded for the split or the detail screen (`detail`), as `entry_view` pads.
#[must_use]
#[expect(clippy::too_many_lines, reason = "composes the six sections of a report row's pane")]
pub fn report_pane(
    row: &ReportRow,
    m: &Messages,
    detail: bool,
    scroll: Option<&gpui_kit::ScrollHandle>,
) -> impl gpui_kit::IntoElement {
    use gpui_kit::component::{h_flex, v_flex};
    use gpui_kit::prelude::FluentBuilder as _;
    use gpui_kit::{
        InteractiveElement as _, IntoElement as _, ParentElement as _, StatefulInteractiveElement as _, Styled as _,
        div,
    };

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
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .font_family(d.typography.mono.clone())
                .text_role(&d.typography.roles.title)
                .child(row.path.clone()),
        )
        .child(pill(pane.chip.clone(), ChipState::Rest).text_color(chip_color));
    let quote = pane.quote.map(|(tone, text)| {
        let bar = if tone == Tone::Error { c.red } else { c.yellow };
        div()
            .border_l(pt(d.components.quote.bar))
            .border_color(bar)
            .pl(pt(d.spacing.quote.pad_left))
            .py(pt(d.spacing.quote.pad_y))
            .text_color(c.text)
            .text_role(&d.typography.roles.quote)
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
                        .when(line.marked, |l| {
                            l.bg(c.red.opacity(d.components.pane.tint_alpha))
                                .border_l(pt(d.components.quote.bar))
                                .border_color(c.red)
                                .text_color(c.text)
                        })
                        .child(div().text_color(c.faint).child(line.number.to_string()))
                        .child(div().whitespace_nowrap().child(line.text.clone()))
                        .into_any_element()
                })
                .collect(),
        )
    });
    let command = pane
        .command
        .map(|cmd| mono_block(vec![div().px(pt(d.spacing.card.pad_x)).child(cmd).into_any_element()]));
    let list = (!pane.list.is_empty()).then(|| {
        v_flex()
            .font_family(d.typography.mono.clone())
            .text_role(&d.typography.roles.description)
            .text_color(c.muted)
            .children(pane.list.iter().map(|item| div().child(format!("• {item}"))))
    });
    let facts = facts_element(pane.facts, &d, c);
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
        .when_some(scroll, gpui_kit::StatefulInteractiveElement::track_scroll)
}

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
        ReportRow {
            path: "x.yml".into(),
            kind,
            entries: 9,
            detail: String::new(),
            subject,
        }
    }

    fn sample_file(error: bool, skipped: bool) -> FileReport {
        let mut rep = FileReport {
            rel_path: "x.yml".into(),
            file_path: "/s/x.yml".into(),
            entries: 9,
            types: IndexMap::from([("bookmarks".into(), 9)]),
            error: None,
            excerpt: None,
            issues: Vec::new(),
        };
        if error {
            rep.error = Some(SourceError {
                file_path: "/s/x.yml".into(),
                message: "A flow sequence must be closed".into(),
                line: Some(4),
                column: Some(11),
            });
            rep.excerpt = Some("  3 | k8s-pods:\n> 4 |   tags: [k8s, pods\n  5 |   cmd: kubectl".into());
        }
        if skipped {
            rep.issues.push(FileIssue::SkippedEntry {
                entry_type: "bookmarks".into(),
                key: "not-a-map".into(),
            });
        }
        rep
    }

    #[test]
    fn shows_a_failed_file_s_error_excerpt_and_location() {
        let m = Catalog::default().messages;
        let pane = content(&row(ReportKind::Failed, Subject::File(sample_file(true, false))), &m);
        assert_eq!(pane.quote, Some((Tone::Error, "A flow sequence must be closed".into())));
        assert_eq!(
            pane.excerpt,
            [
                ExcerptLine {
                    number: 3,
                    text: "k8s-pods:".into(),
                    marked: false
                },
                ExcerptLine {
                    number: 4,
                    text: "  tags: [k8s, pods".into(),
                    marked: true
                },
                ExcerptLine {
                    number: 5,
                    text: "  cmd: kubectl".into(),
                    marked: false
                },
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
        let pane = content(&row(ReportKind::Skipped, Subject::File(sample_file(false, true))), &m);
        assert_eq!(
            pane.quote,
            Some((
                Tone::Warning,
                "1 entry was skipped: only a map of fields becomes an entry.".into()
            ))
        );
        assert_eq!(pane.list, ["bookmarks / not-a-map — not a map"]);
        assert_eq!(
            pane.facts[0],
            ("Entries".into(), Fact::Pills(vec!["bookmarks 9".into()]))
        );
        assert_eq!(pane.chip, "9 · 1 skipped");
    }

    #[test]
    fn shows_an_indexed_file_s_facts_only() {
        let m = Catalog::default().messages;
        let pane = content(&row(ReportKind::Indexed, Subject::File(sample_file(false, false))), &m);
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
        assert_eq!(
            pane.command.as_deref(),
            Some("/opt/homebrew/bin/skills ls --global --agent universal --json")
        );
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
        let warning = SkillsWarning {
            message: "/x: cannot read".into(),
            file: Some("/k/x/SKILL.md".into()),
        };
        let pane = content(&row(ReportKind::Warning, Subject::Warning(warning)), &m);
        assert_eq!(pane.quote, Some((Tone::Warning, "/x: cannot read".into())));
        assert_eq!(pane.facts, [("File".into(), Fact::Mono("/k/x/SKILL.md".into()))]);
    }
}
