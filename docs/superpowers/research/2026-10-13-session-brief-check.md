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
