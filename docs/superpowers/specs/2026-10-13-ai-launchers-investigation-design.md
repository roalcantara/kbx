# AI-agent features: Raycast versus the Rust/GPUI launchers — design

- Status: approved by the owner, 2026-10-13
- Date: 2026-10-13
- Kind: an investigation, executed by other agents; it writes reports, never code
- Inputs: [2026-10-13-asyar-claims-review.md](../research/2026-10-13-asyar-claims-review.md) (the GPUI launchers
  found), [2026-10-13-raycast-ai-restrictions-check.md](../research/2026-10-13-raycast-ai-restrictions-check.md)
  (phase 0, done)
- Plan: [2026-10-13-ai-launchers-investigation.md](../plans/2026-10-13-ai-launchers-investigation.md)

## Goal

One report that lets the owner decide whether to **adopt, fork or build** a Rust/GPUI launcher in Raycast's vein,
judged on one topic: **agent orchestration — many agents × many models × many workflows, with their monitoring and
control**. Phase 1 refines that topic into a feature list; only those features are scored. Simple AI integrations
(one model, one prompt, one answer) are out: Raycast already ships them and refuses third-party duplicates (phase 0),
so a port gains nothing there. The report maps each feature to what Raycast does with it (offers, sells, accepts from
extensions, refuses, cannot host) and to what each Rust/GPUI launcher does, so the gaps Raycast leaves and no port
fills are visible, and the candidates are ranked.

## Decisions

| #   | Decision                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| --- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| I1  | **The question is a taxonomy, not a yes/no.** First list the AI-agent features flooding app stores; then score Raycast and each port against that list (owner, 2026-10-13).                                                                                                                                                                                                                                                                                                |
| I2  | **Phase 0 first**: a short attempt to refute the load-bearing assumption before any heavy work. Done: it holds, refined — Raycast refuses, case by case, AI extensions that duplicate its native AI (#25592 local models, #24097 rewrite anywhere), some of which is paid.                                                                                                                                                                                                 |
| I3  | **Phased with gates** (approach A): taxonomy → independent audit → frozen list → Raycast coverage → GPUI census → port scoring (fanned out, one agent per port) → synthesis. Nothing is scored before the list is frozen.                                                                                                                                                                                                                                                  |
| I4  | **Sources for the taxonomy: all four** — GitHub lists and topics (awesome-libghostty, awesome-gpui, awesome-cmux, `gpui`, `mcp`, `claude-code`, `agent`), launcher stores (Raycast Store AI category, Vicinae, Alfred galleries), Product Hunt and Setapp (last 6 months), the Mac App Store (Developer Tools, Productivity).                                                                                                                                              |
| I5  | **An independent auditor** rebuilds the list from the same sources without reading phase 1's reasoning, then diffs; whatever relevant feature it adds goes in before the freeze.                                                                                                                                                                                                                                                                                           |
| I6  | **Columns: Raycast and the Rust/GPUI launchers only.** A launcher counts only if its `Cargo.toml` depends on a GPUI (`gpui`, `gpui-pre`, `gpui-ce`, a Zed git `gpui`, or `gpui-component`/`gpui-kit`).                                                                                                                                                                                                                                                                     |
| I7  | **Ports are scored from their source, not run.** Each cell cites `repo@commit:path:line`. Whether to build and run the top candidates is decided after the report (owner).                                                                                                                                                                                                                                                                                                 |
| I8  | **Maturity does not exclude.** A young or small project is scored like any other; maturity is a column, not a filter (owner: reading an immature port beats building a copy).                                                                                                                                                                                                                                                                                              |
| I9  | **The topic is agent orchestration, refined in phase 1** (owner, 2026-10-13). A feature enters the taxonomy only if it involves at least one of: more than one agent or model at once; a workflow longer than one prompt and one answer; observing or controlling agents while they run (state, cost, logs, needs-input). Everything else is listed once as "simple integration, out of topic", with its reason, and never scored; the auditor may challenge the boundary. |
| I10 | **Phase 1 first asks whether Raycast ships native agents or orchestration.** If it does, its duplicate rule covers them and H1 is about refusals; if not, the topic is open ground and H1 is about what its extension API cannot host (no persistent background process, no embedded terminal).                                                                                                                                                                            |
| I11 | **A human gate closes every phase** (owner, 2026-10-13). The agent stops, shows the owner a one-screen summary (what it found, what it decided, open questions, changes it proposes for the next phase) and waits. The owner approves, adjusts or stops; the answer and its date are recorded in the phase output under `## Gate`. No phase starts before the previous gate is passed.                                                                                     |

## Scope

In: the taxonomy and its audit; Raycast's status per feature; every Rust/GPUI launcher found (up to 15 scored in
full); licences; the gap list; a ranked candidate list; an adopt / fork / build recommendation; a short list of ports
worth building and running next.

Out: building or running any port (I7); non-GPUI launchers as columns (Vicinae, Asyar, Gauntlet, Script Kit v1,
Alfred appear only as taxonomy sources); writing any code; changing kb's Plan 8. A finding that bears on a kb decision
goes to the report's "For the owner" section, nowhere else.

## Design

### Hypotheses (each phase tries to refute first)

- **H1** — Raycast blocks some agent-orchestration features: refused as duplicates of its native AI, paid-only, or
  impossible through its extension API. Phase 0 holds it for simple AI features; phase 3 tests it on the orchestration
  features (I10 decides which kind of blocking to look for).
- **H2** — No Rust/GPUI launcher covers the features Raycast blocks. Refuted by one port covering one blocked feature
  in code; refined by how many and how deeply.
- **H3** — Agent-session management (terminals embedded with libghostty or alacritty_terminal, session lists,
  monitoring, as agterm does) fits a GPUI launcher. Refuted if no GPUI launcher or crate can host it; refined by which
  crates and apps already do parts of it.
- **H4** — Adopting or forking beats building. Refuted if no candidate clears licence (MIT or Apache-2.0), fit and
  coverage.

### Phases and outputs

All outputs go to `docs/superpowers/research/ai-launchers/`.

| Phase | Output                     | Who                 | Content                                                                                                                                 |
| ----- | -------------------------- | ------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| 0     | (done, input)              | —                   | `2026-10-13-raycast-ai-restrictions-check.md`                                                                                           |
| 1     | `01-taxonomy.md`           | researcher          | ≤ 40 features in about 8 families; each with a one-line definition, 3+ example products with links, and the sources it was seen in      |
| 2     | `02-audit.md`              | independent auditor | Its own list from the same sources, the diff, up to 10 additions with evidence; the frozen list                                         |
| 3     | `03-raycast.md`            | researcher          | Per feature: one status (below), the evidence URL and the date read                                                                     |
| 4     | `04-census.md`             | researcher          | Every Rust/GPUI launcher: repo, GPUI proof (`Cargo.toml` line), licence, created, last push, commits, stars, platforms; the 15 to score |
| 5     | `05-ports/<owner-repo>.md` | one agent per port  | Per feature: one status (below) with `repo@commit:path:line`; README claims the code lacks are "stub"                                   |
| 6     | `06-report.md`             | synthesizer         | The matrix; H1–H4 verdicts; the gap list; candidates ranked; adopt / fork / build; ports to run next; "For the owner"                   |

**Raycast statuses:** built in · built in, Pro only · extension accepted (Store link) · refused as a duplicate (PR
link) · impossible through the API (docs link) · unknown.

**Port statuses:** native · extension · stub · none · unknown.

### Evidence rules

- Every cell cites its proof; without one it reads "unknown", never a guess.
- Every hypothesis is attacked first; the report records what was searched to refute it.
- README claims are checked in the code.
- Licence decides use: MIT and Apache-2.0 may be adopted; GPL is reference only; no licence means read, never copy.
- Every claim carries the date it was checked: this field changes weekly.

### Ranking (phase 6)

Each candidate is scored on four axes, 0–3 each, with the reasons written out: coverage of the gap list, fit with kb
(a hotkey panel over the owner's sources; GPUI line; extension model), licence, health (activity, tests, platform).
The recommendation names adopt, fork or build, and the conditions under which it would change.

### Effort limits

Phases 1–4 run in order, each closed by its gate (I11); phase 5 fans out after the census gate and is closed by one gate after the last port; at most 15 ports, chosen by activity. The taxonomy is capped at 40
features plus 10 from the audit.

## Acceptance criteria

1. `02-audit.md` shows the auditor's independent list, the diff and the frozen list.
1. Every phase output ends with a `## Gate` section holding the owner's decision and its date.
1. Every cell of the matrix has a status and a citation, or "unknown".
1. H1–H4 each end with refuted, holds, or refined, and what was searched.
1. `06-report.md` ranks the candidates, recommends adopt, fork or build, and lists the ports worth running next.
1. No code was written; no port's code was copied.

## Risks

| Risk                                                    | Mitigation                                                                  |
| ------------------------------------------------------- | --------------------------------------------------------------------------- |
| The taxonomy reflects one agent's view                  | I5's independent audit before the freeze                                    |
| Raycast's refusals are case by case, hard to generalise | Each "refused" cell cites a PR; a count is reported, not a rule inferred    |
| Ports change during the run                             | Every port cell pins a commit                                               |
| Store pages need a browser or block fetching            | Fall back to the extension repository (`raycast/extensions`) and changelogs |

## Doctrine check

`docs/doctrine/` holds one doctrine, `tag-conventions` (`applies_to: tools/tags/**`); this investigation does not touch
the tag tools.
