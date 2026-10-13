# The Orca Raycast extension: a launcher in front of an orchestrator

- Date: 2026-10-13
- Read: [`raycast/extensions/extensions/orca`](https://github.com/raycast/extensions/tree/main/extensions/orca)
  (unofficial, by `dmitry_s`, in the Store since 2026-10-02; about 1,200 lines of TypeScript plus tests), read only
- Why: the closest existing case of "the launcher as the control tower, a resident daemon as the engine"
  ([coverage and publishability](2026-10-13-raycast-coverage-publishability.md), next move 3)

## What it does

Its README states the job: "which agent is stuck waiting on me?"

| Command             | Mode               | What it does                                                                                                                                                                                                                                                               |
| ------------------- | ------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| List All Agents     | view               | Every agent Orca runs, grouped by project; the ones waiting for input lead, longest wait first; state, wait time, task, last reply, raw terminal, and what it is blocked on (`AskUserQuestion`, a `Bash` command awaiting permission); ↵ switches to that terminal in Orca |
| Show Waiting Agents | no-view, every 1 m | Writes the waiting agents (`❓ checkout: Fix the VAT…`) or counts (`2 waiting · 1 working`) into its own root-search subtitle, "refreshed every minute even while Raycast is closed"                                                                                       |
| Add Prompt          | view               | A builder (project, agent, new worktree or not, prompt, name) saved as a Raycast Quicklink to Run Prompt                                                                                                                                                                   |
| Run Prompt          | no-view            | Starts the agent in Orca in the background: `worktree create … --agent … --prompt …`, or `terminal create` + `terminal send`, or sends to the project's most recent agent; Orca stays behind, a HUD confirms                                                               |

## How it is built

- **The extension holds no state and runs no agent.** Every read and every action is one call to Orca's CLI with
  `--json` (`terminal list`, `worktree ps`, `worktree create`, `terminal create`, `terminal send`), parsed from
  Orca's common envelope `{ ok, result, error }` (`src/orca.ts`). Orca, the resident app, owns the agents, the
  worktrees and their state.
- **Status is polled.** The open list re-reads Orca every 5 seconds (`setInterval(…, 5000)`, `src/list-agents.tsx`);
  the root-search summary every minute in the background.
- **Two calls make one row:** `terminal list` never reports agent state, so it is joined with `worktree ps` by pane
  key (`mergeAgents`).
- **Focus is handed back to Orca** with `open -b com.stablyai.orca` after selecting the terminal.

## Where Raycast's API shows through

The README documents each limit it worked around:

- **Background work only in `no-view` and `menu-bar` commands**, so the live summary is a separate command; a count
  in the list itself "freezes at whatever it was the last time the list was opened".
- **`updateCommandMetadata` writes only its own command's subtitle**, which is the whole notification channel: the
  root-search row is the "push".
- **No commands added at runtime** ("the manifest is fixed"), so saved prompts are Quicklinks carrying the whole spec
  in a deeplink; the extension keeps nothing.
- **A trimmed `PATH`:** it cannot probe which agents are installed, so it lists every agent Orca supports, and the
  Orca binary path is a preference.
- **Icons:** a Quicklink's icon must be a built-in Raycast icon; vendor logos cannot be used.

## Insights

1. **The control-tower split already works, and it is cheap on the launcher's side.** About 1,200 lines answer "who
   needs me?" and "start this task" for an orchestrator that does everything else. The launcher is a thin client of a
   resident engine's CLI; the engine never needs the launcher. This is the resident-daemon idea, shipped.
2. **The contract between them is a CLI with JSON envelopes.** Orca's `--json` and `{ ok, result, error }` are what
   made the extension possible; agterm (`agtermctl … --json`), Supacode (`supacode` CLI) and Zentty (`zentty` CLI and
   its Agent Status Protocol) expose the same kind of surface. A daemon of one's own would need nothing more to serve
   Raycast and a GPUI launcher alike.
3. **Raycast's ceiling is the transport, not the logic.** What the extension cannot do is: be told — it asks every 5
   s or 60 s; notify beyond its own root-search subtitle; keep a view live in the background; grow its own command
   list. A native launcher would remove exactly those, and nothing else in this extension would change.
4. **It overlaps with Orca's own mobile companion and notifications** — the same "who is waiting" question answered on
   another surface. The launcher is one more surface for a daemon's state, not the home of the orchestration.

## Searched

`gh api repos/raycast/extensions/contents/extensions/orca/...`: `package.json`, `README.md`, `CHANGELOG.md`, all of
`src/` and `tests/`.
