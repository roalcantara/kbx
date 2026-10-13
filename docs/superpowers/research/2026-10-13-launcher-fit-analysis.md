# Which agent-management features fit a launcher?

- Date: 2026-10-13
- Builds on: [2026-10-13-agent-tools-preflight.md](2026-10-13-agent-tools-preflight.md) (the features of agterm,
  Supacode, Zentty, Orca and the model-routing tools)
- Question (owner): could a Raycast-like app — a launcher — host these features instead of a terminal-like tool? None,
  some or all, which, and why?

## Five questions, kept apart

Answering "can Raycast do it?" mixes separate questions. This analysis answers the first and gathers evidence for the
second; the others are left open for the investigation.

| # | Question                                                                      | Status                                                   |
| - | ----------------------------------------------------------------------------- | -------------------------------------------------------- |
| 1 | **Fit:** is a launcher's form (summoned, transient, keyboard-first) suitable? | Answered below, by reasoning, with one piece of evidence |
| 2 | **Capability:** can Raycast's extension API host it?                          | Partly answered by the owner's private extension (below) |
| 3 | **Coverage:** does Raycast already provide it natively?                       | Open                                                     |
| 4 | **Publishability:** would the Raycast Store accept a third party doing it?    | Open; a private extension is no evidence of it           |
| 5 | **Ports:** what do the Rust/GPUI launchers do?                                | Open                                                     |

Questions 2–5 should only be asked of the features that pass question 1.

## 1. Fit: the clustering

Every feature has an interaction shape:

- **Glance:** see a state in a second.
- **Act:** one command, and the window goes away.
- **Dwell:** stay for minutes, reading or typing.

A launcher is made for glance and act; dwell belongs to a terminal or an IDE.

| Family                  | Glance or act — fits a launcher                                 | Dwell — stays in the terminal-like tool         |
| ----------------------- | --------------------------------------------------------------- | ----------------------------------------------- |
| 1. Sessions             | Create, name, find, jump to, resume a session                   | Typing into it                                  |
| 2. Status and attention | Every agent's state, sorted by "needs you"; notifications       | —                                               |
| 3. Observation          | The subagent tree, crash alerts, a peek at the last lines       | The live output grid                            |
| 4. Worktrees            | Create, archive, run setup scripts                              | —                                               |
| 5. Coordination         | Fan one prompt out to several agents; approve a decision gate   | Authoring and watching a task graph             |
| 6. Review               | PR state, checks, merge                                         | Annotating diffs, editing                       |
| 7. Accounts and cost    | Usage, rate limits, cost per agent; switch account              | —                                               |
| 8. Control surface      | The launcher is a client of the tools' CLIs and sockets         | —                                               |
| 9. Reach                | —                                                               | SSH terminals, mobile, browser and computer use |
| Model routing           | See and switch routes; cost per request; pick a tier or a model | —                                               |
| Policy and governance   | Choose a policy for a run; approve; read the record             | Writing policies                                |

**Verdict: some — about half, and arguably the most valuable half.** Knowing which agent needs you, starting and
resuming work, dispatching, approving, routing models and watching cost fit a launcher; typing into sessions, watching
live output grids, reviewing diffs, editing and remote shells stay in terminal-like tools. A launcher would be the
control tower, the terminal the workshop. The two meet in the glance-and-act gestures the terminal tools already reach
for: Supacode's ⌃1–⌃9 jump, Zentty's menu-bar status, agterm's native picker and yes/no prompt.

## 2. Capability: evidence from the owner's private extension

The owner's private Raycast extension (`~/Work/raycast/kb`, commit `3de16d2`, read only) runs agentic work from a
Raycast view in its `jobx` feature:

- **Dispatch and live output.** It spawns the `omp` agent CLI with the prompt on stdin and streams its JSONL events
  into the screen as they arrive (`apps/kb/src/features/jobx/run/omp.ts`).
- **Model routing**, which none of the four terminal tools has. Each prompt declares a tier (`Model tier: flash` or
  `strong`); each tier is a fallback chain across different providers, so one provider's exhausted quota does not end
  the run; a picker overrides the model for one run (`packages/core/src/jobx/omp/models.ts`).
- **Policy.** A tool allowlist per prompt, a write guard loaded as a hook, skills and extensions passed explicitly;
  the run is refused before anything is spent when the guard files are missing; outputs are recorded
  (`apps/kb/src/features/jobx/run/launch.ts`).

So a Raycast extension **can** host dispatch, a live view of one run, model routing and policy — families 5 (one run
at a time), routing and governance.

It also shows the limit: the run lives as long as its screen ("a screen that closes at any moment — before the spawn
or after — leaves nothing running", `omp.ts`). Raycast keeps nothing alive once the view closes, so anything that must
outlive the window — a long agent run, a coordinator loop, a hook socket receiving status, a model gateway — needs a
resident process Raycast's extensions do not have. That is the line between a Raycast extension and a launcher one
owns.

## Open for the investigation

- **Coverage (3):** which fitting features Raycast ships itself — its AI Chat, AI Commands, AI Extensions, local
  models, MCP — and on which plan. Native coverage is what triggers the Store's "already a feature" refusals.
- **Publishability (4):** would the Store accept an extension that spawns a user-installed agent CLI at a configured
  path (the binary rules), routes models (is that a duplicate of Raycast AI?), or carries guard hooks? Evidence to
  look for: Store extensions that already do each (ClaudeCast, PromptCast, Claude Code Launcher) and closed PRs with
  the reviewers' reasons.
- **Capability, the rest (2):** push versus polling for status (menu-bar commands refresh on an interval), and
  whether any background mechanism can keep an agent alive past its view.
- **Ports (5):** which Rust/GPUI launchers host the fitting features, and whether any has the resident process the
  Raycast line calls for.
