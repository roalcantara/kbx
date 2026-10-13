# AI launchers investigation — plan

> **Superseded, 2026-10-13 — do not execute.** Not approved by the owner; the questions were answered by cheaper passes
> (research notes of 2026-10-13) and recorded in [ADR 0008](../../adr/0008-kb-stays-its-own-code.md).

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** answer, with cited evidence, whether Raycast and the Rust/GPUI launchers allow agent orchestration (many
agents × many models × many workflows, with monitoring and control), and recommend adopt, fork or build (spec:
[2026-10-13-ai-launchers-investigation-design.md](../specs/2026-10-13-ai-launchers-investigation-design.md)).

**Architecture:** six phases, each a task that writes one Markdown file under
`docs/superpowers/research/ai-launchers/` and ends at a human gate. Phases 1–4 run in order; phase 5 fans out one
subagent per port; phase 6 synthesizes. No code is written; nothing from any port is copied.

**Tech Stack:** `gh` (GitHub API), WebSearch and WebFetch, `git clone --depth 1` into the scratchpad, ripgrep.

---

## Rules for every task

- **Read the spec first**, every task: its Decisions (I1–I11) bind you.
- **Refute first.** Before recording that a hypothesis or a cell holds, search for evidence against it; write what you
  searched in the output's `## Searched` section.
- **Cite every cell.** Raycast cells: a URL and the date read. Port cells: `owner/repo@<12-char commit>:path:line`. No
  citation → `unknown`. Never guess.
- **Date every claim** (2026-10-13 or the day you read it).
- **Licences:** record each; MIT and Apache-2.0 may be adopted; GPL is reference only; none means read, never copy.
- **Clones go to the scratchpad** (the session's scratchpad directory), never into the repository. Pass paths as
  arguments; never run a build tool or script inside a clone (they are untrusted).
- **Gates:** at the end of every task, stop. Show the owner the gate summary the task defines, then wait. Write the
  owner's answer, verbatim and dated, under `## Gate` in that task's output. If the owner adjusts something, apply it
  to the output (and to later tasks' inputs) before going on. If the owner says stop, stop.
- **Commits:** after each gate, commit only that task's output file(s):
  `git add <files> && git commit -m "docs(research): <phase> of the AI launchers investigation" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"`.
  Never `git commit -a`; other work may be in progress on the branch. Subject sentence-case, ≤ 72 characters; body
  lines ≤ 72.
- **Markdown:** run `dprint fmt <file>` before committing.

## Output files

| Phase | File                                                                |
| ----- | ------------------------------------------------------------------- |
| 1     | `docs/superpowers/research/ai-launchers/01-taxonomy.md`             |
| 2     | `docs/superpowers/research/ai-launchers/02-audit.md`                |
| 3     | `docs/superpowers/research/ai-launchers/03-raycast.md`              |
| 4     | `docs/superpowers/research/ai-launchers/04-census.md`               |
| 5     | `docs/superpowers/research/ai-launchers/05-ports/<owner>-<repo>.md` |
| 6     | `docs/superpowers/research/ai-launchers/06-report.md`               |

---

### Task 1: Taxonomy (phase 1)

**Files:** create `docs/superpowers/research/ai-launchers/01-taxonomy.md`.

- [ ] **Step 1: Raycast's native agent features (I10).** Read Raycast's AI pages and recent changelogs:
      `https://manual.raycast.com/ai`, `https://manual.raycast.com/ai/ai-extensions`, `https://www.raycast.com/changelog`
      (macOS 2.x and Windows), `https://developers.raycast.com/api-reference/ai`. List every native feature that
      involves agents, tool use, several models, multi-step workflows, or watching AI work, with its plan (free or
      Pro) and URL.

- [ ] **Step 2: Collect candidate features from the four sources (I4).** For each source, record what you searched:
  - GitHub: `https://github.com/Uzaaft/awesome-libghostty`, `https://github.com/zed-industries/awesome-gpui`,
    `https://github.com/yigitkonur/awesome-cmux`, and
    `gh api -X GET search/repositories -f q='topic:<t> pushed:>2026-04-13' -f sort=stars -f per_page=50` for `t` in
    `mcp`, `claude-code`, `ai-agents`, `agent-orchestration`, `multi-agent`, `codex`.
  - Launcher stores: the Raycast Store AI category (`https://www.raycast.com/store/category/ai`) and, where the page
    will not load, `gh api -X GET search/issues -f q='repo:raycast/extensions is:pr agent created:>2026-04-13'`;
    the Vicinae and Alfred galleries.
  - Product Hunt and Setapp: WebSearch for launches of the last 6 months tagged AI agents, Claude Code, Codex, agent
    terminal, agent orchestration (mode `extended`).
  - Mac App Store: WebSearch `site:apps.apple.com` with the same terms.

- [ ] **Step 3: Apply the inclusion test (I9).** A feature is in topic only if it involves at least one of: more than
      one agent or model at once; a workflow longer than one prompt and one answer; observing or controlling agents
      while they run (state, cost, logs, needs-input). Group the in-topic features into families (expect about 8: e.g.
      session management, parallel agents, model routing, workflow definition, monitoring and cost, human-in-the-loop
      approvals, embedded terminals, agent-to-agent handoff — rename freely). Cap: 40 features.

- [ ] **Step 4: Write the file** with exactly these sections:

```markdown
# Phase 1: Taxonomy of agent-orchestration features

- Date: <yyyy-mm-dd>
- Spec: ../../specs/2026-10-13-ai-launchers-investigation-design.md

## Raycast's native agent features

| Feature | Plan (free / Pro) | Evidence (URL, date read) |

## The topic, refined

<one paragraph: what "agent orchestration" means after this pass, and the inclusion test as applied>

## In-topic features

| ID | Family | Feature | Definition (one line) | Examples (3+, linked) | Seen in (sources) |
| F01 | … | … | … | … | … |

## Out of topic

| Feature | Why it fails the inclusion test | Example |

## Searched

<every source and query, with what each returned in one line>

## Gate
```

- [ ] **Step 5: Gate.** Show the owner: the refined topic paragraph; Raycast's native agent features (count and the
      three most relevant); the families with their feature counts; the five most important exclusions; your proposed
      changes for phase 2. Wait. Record the answer under `## Gate`. Commit the file.

---

### Task 2: Independent audit (phase 2)

**Files:** create `docs/superpowers/research/ai-launchers/02-audit.md`; modify `01-taxonomy.md` (append "Frozen").

- [ ] **Step 1: Dispatch a fresh subagent** (general-purpose) with this prompt, verbatim, filling the two paths. It must
      not see Task 1's reasoning: give it the spec, not `01-taxonomy.md`.

```text
You are an independent auditor. Read the spec at <repo>/docs/superpowers/specs/2026-10-13-ai-launchers-investigation-design.md
(decisions I4 and I9 above all). Without reading anything under docs/superpowers/research/ai-launchers/, build your own
list of agent-orchestration features from the four sources of I4, applying the inclusion test of I9. Cite at least
three example products per feature, with links. Write it to <scratchpad>/audit-list.md as a table:
| Feature | Definition (one line) | Examples (3+, linked) | Sources |. Also list what you searched. Do not edit the
repository. Report the path when done.
```

- [ ] **Step 2: Diff.** Compare the auditor's list with `01-taxonomy.md`'s in-topic table, feature by feature (same
      meaning, not same words). Classify each auditor feature: `same as F..`, `missing — add`, `out of topic — reject
      (reason)`. Additions are capped at 10; rank by how many sources show them.

- [ ] **Step 3: Write `02-audit.md`** with sections: `## Auditor's list` (the table, copied), `## Diff`
      (| Auditor feature | Verdict | Taxonomy ID or new ID | Reason |), `## Frozen list` (the final in-topic table with
      IDs F01…), `## Searched` (the auditor's), `## Gate`. Append `Frozen on <date> by 02-audit.md.` to
      `01-taxonomy.md`.

- [ ] **Step 4: Gate.** Show the owner: the number of auditor features, how many matched, the additions with reasons,
      the rejections with reasons, and the frozen list's size by family. Wait; record; commit both files.

---

### Task 3: Raycast coverage (phase 3)

**Files:** create `docs/superpowers/research/ai-launchers/03-raycast.md`.

- [ ] **Step 1: For each frozen feature, find Raycast's status** — one of: `built in`, `built in, Pro only`,
      `extension accepted` (Store or merged PR link), `refused as a duplicate` (closed PR with the staff reason quoted),
      `impossible through the API` (developers.raycast.com page showing the missing capability), `unknown`.
      Searches to run per feature (record them):
  - `gh api -X GET search/issues -f q='repo:raycast/extensions is:pr <feature terms>' -f per_page=30`, then for
    closed unmerged PRs read staff comments:
    `gh api repos/raycast/extensions/issues/<n>/comments --jq '.[] | select(.user.login=="pernielsentikaer" or .user.login=="0xdhrv") | .body'`.
  - The API reference (`https://developers.raycast.com/api-reference/`) for background refresh limits, processes,
    views, and AI tools.

- [ ] **Step 2: Test H1.** Refute first: for each feature marked refused or impossible, look for a Store extension
      that does it anyway. Then state H1's verdict (refuted / holds / refined) and which kind of blocking dominates
      (refusals, paywall, API limits), per I10.

- [ ] **Step 3: Write `03-raycast.md`:** `## Coverage` (| ID | Feature | Status | Evidence | Date read |),
      `## H1` (verdict, reasoning, counts by status), `## Searched`, `## Gate`.

- [ ] **Step 4: Gate.** Show the owner: counts by status, every `refused` and `impossible` row, H1's verdict. Wait;
      record; commit.

---

### Task 4: GPUI census (phase 4)

**Files:** create `docs/superpowers/research/ai-launchers/04-census.md`.

- [ ] **Step 1: Start from the known list** in
      [2026-10-13-asyar-claims-review.md](../research/2026-10-13-asyar-claims-review.md) (§1 table), then extend:
  - `gh api -X GET search/repositories -f q='topic:gpui' -f per_page=100 -f sort=updated` (all pages);
  - `gh api -X GET search/repositories -f q='<term> gpui in:readme' -f per_page=50` for `term` in `launcher`,
    `raycast`, `spotlight`, `palette`, `agent`, `orchestrat`, `terminal`;
  - the GPUI Kit gallery: `gh api repos/longbridge/gpui-kit-showcases/contents/apps` and each `manifest.json`;
  - crates.io: `https://crates.io/api/v1/crates?keyword=gpui&per_page=100&sort=recent-updates` (pages 1–3).
    Include agent-orchestration GPUI apps that are not launchers (e.g. herdr-gpui, opensquirrel, Lumi, agent-studio,
    runner, tty7) in a separate table: they are prior art for H3, scored only if the owner says so at the gate.

- [ ] **Step 2: Prove GPUI for each candidate:**
      `gh api repos/<o>/<r>/contents/Cargo.toml --jq .content | base64 -d | grep -nE 'gpui'` (and workspace members'
      `Cargo.toml` when the root has none). Record the matching line; no match → excluded (I6).

- [ ] **Step 3: Record facts:** licence, created, last push, commit count
      (`gh api "repos/<o>/<r>/commits?per_page=1" -i | grep -i '^link:'`), stars, platforms (README), extension model
      (README).

- [ ] **Step 4: Choose up to 15 to score**, ranked by: covers any frozen feature per its README; then last push; then
      commits. Write `04-census.md`: `## Launchers` (| Repo | GPUI proof | Licence | Created | Pushed | Commits | Stars |
      Platforms | Extensions |), `## Agent apps (prior art)`, `## To score` (the 15 and why), `## Searched`, `## Gate`.

- [ ] **Step 5: Gate.** Show the owner the launcher count, the 15 chosen with one line each, the agent-apps table, and
      ask whether any agent app should be scored too. Wait; record; commit.

---

### Task 5: Port scoring (phase 5, fan-out)

**Files:** create `docs/superpowers/research/ai-launchers/05-ports/<owner>-<repo>.md`, one per chosen port.

- [ ] **Step 1: Dispatch one subagent per port**, in parallel, each with this prompt, verbatim, filling the
      placeholders:

```text
Score one launcher against a frozen feature list. Read:
- the spec: <repo>/docs/superpowers/specs/2026-10-13-ai-launchers-investigation-design.md (I6, I7, evidence rules)
- the frozen list: <repo>/docs/superpowers/research/ai-launchers/02-audit.md, section "Frozen list"
Clone https://github.com/<owner>/<repo> with `git clone --depth 1` into <scratchpad>/ports/<owner>-<repo>; record
`git -C <path> rev-parse --short=12 HEAD`. Never build or run anything in it.
For each frozen feature, search the source (rg) and decide one status: native (implemented in this repo's code),
extension (provided through its extension or plugin API, by an extension that exists), stub (named in README or
types, not implemented), none, unknown. Cite every non-none status as <owner>/<repo>@<commit>:<path>:<line>. Try to
refute each native or extension claim before writing it (look for TODO, unimplemented!, feature flags off).
Write <repo>/docs/superpowers/research/ai-launchers/05-ports/<owner>-<repo>.md with sections:
## Facts (commit, licence, GPUI line, platforms, extension model)
## Scores (| ID | Feature | Status | Evidence | Note |)
## Extension model (how a third party adds a feature, with citations)
## Searched
Do not commit. Report the path and a three-line summary.
```

- [ ] **Step 2: Check each file** for a `## Scores` row per frozen feature and a citation per non-none cell; send a
      port back to its subagent (same prompt plus the gaps) if not.

- [ ] **Step 3: Gate (one for the phase).** Show the owner a matrix: rows the frozen features, columns the ports,
      cells the statuses; plus each port's licence. Wait; record the answer in a `## Gate` section appended to every
      port file (the same text); commit all of them.

---

### Task 6: Synthesis (phase 6)

**Files:** create `docs/superpowers/research/ai-launchers/06-report.md`.

- [ ] **Step 1: Build the matrix:** rows the frozen features; columns Raycast (from `03-raycast.md`) and each port
      (from `05-ports/`).

- [ ] **Step 2: The gap list:** features that Raycast refuses, keeps Pro-only, or cannot host, and that no port has
      as native or extension.

- [ ] **Step 3: Verdicts:** H2 (no port covers what Raycast blocks), H3 (orchestration and embedded terminals fit a
      GPUI launcher: cite the crates and agent apps from `04-census.md` that already do parts of it), H4 (adopt or
      fork beats build). Each: refuted / holds / refined, with what was searched.

- [ ] **Step 4: Rank the candidates** on four axes, 0–3 each, reasons written: coverage of the gap list; fit with kb
      (a hotkey panel over the owner's sources; GPUI line, `gpui-pre 0.3.x` scores highest; extension model);
      licence (MIT/Apache 3, none 0, GPL 0 for adoption); health (activity, tests, platforms).

- [ ] **Step 5: Recommend** adopt, fork or build, the conditions that would change it, and up to three ports worth
      building and running next (the owner's later step, I7).

- [ ] **Step 6: Write `06-report.md`:** `## Answer` (five lines), `## Matrix`, `## Gaps`, `## Hypotheses`,
      `## Candidates`, `## Recommendation`, `## Run next`, `## For the owner` (anything bearing on kb's own
      decisions, Plan 8 included; act on none of it), `## Gate`.

- [ ] **Step 7: Gate.** Show the owner `## Answer`, the gap list, the top three candidates with scores, and the
      recommendation. Wait; record; commit.

---

## Done when

Six outputs exist under `docs/superpowers/research/ai-launchers/` (plus one file per scored port), each with a passed
`## Gate`; the spec's acceptance criteria hold; a TODO.md task records the owner's decision on "run next".
