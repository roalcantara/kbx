# Does a current-state brief at session start improve the next task?

- Date: 2026-10-13
- Question: does starting a session with a current-state brief (git log and status, pending TODO tasks, active plans,
  recent ADRs and research verdicts, graph overview) improve an agent's outcome on the next task in this repository,
  compared with no brief?
- Purpose: decide whether to automate the brief as a global agent command, at the lowest token cost.
- Audience: the owner, as operator of the agents.
- Bar: at least 20% fewer total tokens with the brief (its own cost included) and no more operator corrections.
- Scope: a code task in kb (e.g. Plan 8g Task 1, the empty view).
- Givens:
  - Session transcripts (`~/.claude/projects/<repo>/*.jsonl`) record each message's `input_tokens`,
    `cache_creation_input_tokens`, `cache_read_input_tokens` and `output_tokens`, so totals are exact.
  - Both arms already load CLAUDE.md, AGENTS.md, CONTEXT.md, auto-memory and the SessionStart hooks (global and
    project); the brief is measured on top of them.
  - No usage tool is installed.
- Method: a web preflight of published controlled studies (asked by the owner), then the ladder of disproof passes,
  each closed by a gate that re-evaluates its own kill signal.

# Preflight (10 minutes): what controlled studies say

Acceptance criteria, written before searching:

- **Proven:** at least two independent controlled studies show a saving of 20% or more with no loss of success, from
  context similar to a state brief. Rung 2 becomes optional.
- **Refuted:** at least two controlled studies show such context raises cost or does not help.
- **Mixed or thin:** fall back to rung 1 and rung 2 (the two-session test).

## Data

| Study                                                                                                         | Setting                                                                                                                        | Success                                            | Cost                                                                                      |
| ------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| Gloaguen et al., ETH Zürich, [arXiv 2602.11988](https://arxiv.org/abs/2602.11988) (v3, 2026-09-29; ICLR 2026) | SWE-bench tasks with LLM-generated context files, plus issues from repos with developer-written files; several LLMs and agents | "does not generally improve task success rates"    | "increasing inference cost by over 20% on average"                                        |
| Lulla et al., [arXiv 2601.20404](https://arxiv.org/abs/2601.20404v2) (v2, 2026-03-30)                         | 10 repositories, 124 pull requests, with and without AGENTS.md                                                                 | "comparable task completion behavior"              | median runtime −28.64%, output tokens −16.58%; total and input tokens not in the abstract |
| Khatri, [arXiv 2607.27250](https://arxiv.org/abs/2607.27250) (2026-07-28)                                     | Claude Code and Codex; 17 tasks in 3 Python repositories; 288 runs; none, full file, on-demand files                           | "does not measurably move correctness" (±10–15 pp) | not in the abstract                                                                       |

All read 2026-10-13. Secondary coverage, not relied on: [heise](https://heise.de/-11245317),
[D. Vaughan on three studies](https://codex.danielvaughan.com/2026/07/27/do-auto-generated-agents-md-files-actually-help-codex-cli/)
(reports Lulla's median total tokens about 1% higher; not verified against the paper).

## What it shows

1. **Success does not move.** Two independent controlled studies (Gloaguen, Khatri) find that repository context
   files do not change task success; Khatri bounds the effect at 10–15 points and attributes failures to
   implementation skill, not missing repository knowledge.
2. **Cost moves in both directions.** Gloaguen finds +20% inference cost; Lulla finds −16.6% output tokens and −28.6%
   runtime. Neither shows a 20% saving in total tokens.
3. **Repository overviews do not help.** Gloaguen's abstract: "repository overviews, although popular and recommended
   by model providers, are not helpful"; context files are useful for "non-standard coding practices". The brief's
   graph overview is such an overview.
4. **The studied context is static; this brief is dynamic.** No study tests what changed, what is pending, or which
   plan is active. That part is untested.

## Gate

| Signal                                               | Fires?                          | Sound?                                                                                 |
| ---------------------------------------------------- | ------------------------------- | -------------------------------------------------------------------------------------- |
| Proven: two studies show a saving of 20% or more     | No (best: −16.6% output only)   | Yes                                                                                    |
| Refuted: two studies show no help or a cost increase | On success, yes; on cost, split | Partly: the studies test static conventions files, which kb already loads in both arms |

Result: **mixed.** The evidence refutes the brief's static parts (overview, conventions) and is silent on its dynamic
parts (state, pending work, active plans).

Refined question for rung 1: does a **state-only** brief (git log and status, pending TODO tasks, active plans,
recent decisions; no repository overview) save 20% of total tokens on the next kb task, with no more corrections?

# Restated card (owner, 2026-10-13)

The preflight's question was too narrow (one artifact, pass or fail) and its method too heavy for the stakes: a global
command is one file, cheap and reversible, so it needs good priors and judgement in use, not a measured bar.

- Question: which start-of-session context do practitioners and vendors recommend, and report as worth its tokens, and
  what is the cheapest way to automate it for any coding agent?
- Method: a 15-minute sweep of vendor docs, known practitioners, skill registries and forums.
- Acceptance: a practice is adopted when at least three independent credible sources agree; contradictions are recorded,
  with the controlled studies above as the tiebreaker.

# Rung 1 (15 minutes): practitioner and vendor sweep

## Data

| Source                                                                                                                                                                                                                                                                           | Kind                 | What it does at session start                                                                                                     |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| Anthropic, [Effective harnesses for long-running agents](https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents) (2025-11-26)                                                                                                                         | vendor               | `pwd`; read `git log` and `claude-progress.txt`; pick the highest-priority unfinished feature; run `init.sh` and a basic test     |
| Anthropic, Effective context engineering for AI agents (2025-09; via [summary](https://agentic-ai.readthedocs.io/en/latest/ContextEngineering/anthropic/))                                                                                                                       | vendor               | keep lightweight pointers (paths, URLs), load just in time; structured notes outside the window                                   |
| Yegge, Beads ([pkg.go.dev](https://pkg.go.dev/github.com/steveyegge/beads@v0.21.1), [summary](https://www.morphllm.com/beads-agent-memory))                                                                                                                                      | practitioner         | query the tracker for the highest-priority **unblocked** work; long Markdown plans waste context on finished items                |
| Huntley, Ralph loop ([summary](https://howaiworks.ai/blog/geoffrey-huntley-ralph-agentic-coding-loop))                                                                                                                                                                           | practitioner         | fresh context each loop; read the prioritised `fix_plan.md`; one task per loop; bloated specs push the agent into the "dumb zone" |
| Cline memory bank ([docs](https://docs.cline.bot/features)); Cursor port ([Lullabot](https://www.lullabot.com/articles/supercharge-your-ai-coding-cursor-rules-and-memory-banks))                                                                                                | vendor, practitioner | read `activeContext.md` and `progress.md`; keep them to current state, not a log                                                  |
| Handoff skills ([skills.sh](https://www.skills.sh/ferueda/agent-skills/handoff-work), [tessl](https://tessl.io/registry/skills/github/lucianghinda/superpowers-ruby/handoff), [GovZero](https://gzkit.readthedocs.io/en/latest/governance/GovZero/session-handoff-obligations/)) | practitioner         | the previous session writes what the next needs (goal, decisions, state, next step, do-not-repeat); stale handoffs are the risk   |
| Aider repo map ([guide](https://fast.io/resources/aider-token-limit/))                                                                                                                                                                                                           | tool                 | a repository map capped at about 1k tokens by default                                                                             |

All read 2026-10-13. None reports a measured token or time effect; Anthropic's claim is qualitative ("saves Claude some
tokens in every session").

## What it shows

Practices with three or more independent sources, adopted:

1. **Recent history plus a short state note.** `git log` and a progress or state file: Anthropic harness, Cline memory
   bank, Ralph, handoff skills.
2. **The next ready task, not the whole plan.** Beads (unblocked work only), Anthropic (highest-priority unfinished
   feature), Ralph (one task per loop).
3. **Lean, current state, never a log.** Cline docs, handoff skills, Yegge, Ralph.
4. **Pointers, not contents; no overviews.** Anthropic context engineering (just-in-time), Gloaguen ("repository
   overviews … are not helpful"), Aider (overview capped small).
5. **Written at the end, read at the start.** Anthropic (progress update with each commit), Cline, Ralph, handoff
   skills: the state the repository cannot derive (why, what is half done, what not to retry) comes from the last
   session.

Single source, not adopted: run the app and a basic test before new work (Anthropic). In kb, `mise run gate` costs
minutes; the agent runs it when its task needs it.

## Gate

| Signal                                          | Fires? | Sound?                                                     |
| ----------------------------------------------- | ------ | ---------------------------------------------------------- |
| Kill: no practice has three independent sources | No     | Yes                                                        |
| Kill: sources contradict the controlled studies | No     | Yes: both say overviews cost and state or conventions help |

## Verdict

Holds, refined: automate a **small, state-only brief, plus a handoff written at the end**. In kb most of the state is
derivable (`git`, the `todo` CLI, plan checkboxes); the handoff carries only what is not.
