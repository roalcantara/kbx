# Raycast: coverage and publishability of the launcher-fitting agent features

- Date: 2026-10-13 (read that day; time-boxed check, about 25 minutes)
- Builds on: [2026-10-13-launcher-fit-analysis.md](2026-10-13-launcher-fit-analysis.md) — questions 3 (coverage) and 4
  (publishability) for the families that fit a launcher
- Sources: Raycast's manual and pricing page; the 3,419 public extensions in
  [`raycast/extensions/extensions`](https://github.com/raycast/extensions/tree/main/extensions) (their `package.json`);
  closed unmerged pull requests in that repository with staff comments
- Owner's evidence: the Store extension **Agent Usage** (`agent-usage`), in daily use

## Summary

- **Coverage is low.** Raycast's own AI (all of it Pro or above; "AI isn't available on the Free plan") manages
  Raycast's AI — saved chat setups ("Agents"), scheduled prompts ("Automations"), tool approvals, MCP, local models,
  bring-your-own key or subscription — not external coding agents. No agent-management family is native for Claude
  Code, Codex and the like.
- **Publishability is high, and the Store is crowded.** For almost every family that fits a launcher, at least one
  public extension already does it — including extensions for Orca and cmux themselves. The refusals found are of
  new extensions that duplicate **other extensions** ("Similar extensions already in the Store"), not Raycast's own
  features; the reviewers ask authors to contribute to the existing one.
- **What no extension does** is what needs a process alive beyond a view: fanning a task out to several agents and
  coordinating them, policies over agent runs, live output, and hosting a model gateway. Status reaches extensions by
  polling, at best every minute (Agent Usage refreshes in the background with `interval: 1m`).

## By family

| Family                  | Native in Raycast (coverage)                                                                                                          | Store (publishability)                                                                                                                                                                                                                                                                                                                         | Verdict                                     |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| 1. Sessions             | No: [Agents](https://manual.raycast.com/ai/agents) are saved chat setups, one chat each                                               | Open, crowded: `claudecast`, `claude-sessions`, `codex-sessions`, `opencode-sessions`, `devin`, `jules-agents`, `codex-claude-cli`; a newcomer closed as overlap ([#31399](https://github.com/raycast/extensions/pull/31399))                                                                                                                  | **open, saturated**                         |
| 2. Status and attention | No for external agents; for its own [Automations](https://manual.raycast.com/ai/automations), notifications when a run needs approval | Open: `orca` ("see which agents are waiting for input"), `codex-sessions` (live Working/Done), `claudecast` (menu-bar monitor); polling only, background `interval` ≥ 1 minute                                                                                                                                                                 | **open (polling)**                          |
| 3. Observation          | No                                                                                                                                    | Status yes (above); live output of a running agent: none found — the API draws lists, details, forms and grids                                                                                                                                                                                                                                 | **gap** (live output)                       |
| 4. Worktrees            | No                                                                                                                                    | Open: `git-worktrees`, `worktrees`                                                                                                                                                                                                                                                                                                             | **open**                                    |
| 5. Coordination         | Partial, own AI only: one prompt per Automation, "can't create or change other automations"; no parallel agents                       | Dispatch open: `cursor-agents` (background agents), `jules-agents`, `devin` (follow-ups), `orca` (start agents), `codex-claude-cli` (send prompts). Fan-out to several agents, task graphs, coordinator loops: none found                                                                                                                      | **open for dispatch; gap for coordination** |
| 6. Review               | No                                                                                                                                    | Open: `github`, `github-review-requests` (actionable PRs in the menu bar)                                                                                                                                                                                                                                                                      | **open**                                    |
| 7. Accounts and cost    | Partial, own AI: credits, BYOK, Claude or ChatGPT subscription ([AI](https://manual.raycast.com/ai), pricing)                         | Open, crowded: `agent-usage` (20 providers, limits, resets, spend), `cursor-costs`, `codexrunway-reset-tracker`, `codex-reset-forecast`, `litellm` (usage), `openrouter-manager`; profile switchers `claude-code-config-switcher`, `codex-profiles`; a newcomer closed as overlap ([#31698](https://github.com/raycast/extensions/pull/31698)) | **open, saturated**                         |
| Model routing           | Partial: a model per Agent or Automation; local models; custom providers; no automatic routing                                        | As a client: `litellm`, `openrouter-manager`, `claude-code-config-switcher`. Local inference contested: `raycast-ollama` and `lm-studio` accepted, `local-ai` refused as native ([#25592](https://github.com/raycast/extensions/pull/25592)). Hosting a gateway: impossible (no resident process for extensions)                               | **contested; gap for a hosted gateway**     |
| Policy and governance   | Partial: tool approvals; Enterprise AI control centre (provider allow-list, toggles)                                                  | None found. A Hermes update with "runs with approvals" was closed without a staff reason ([#29604](https://github.com/raycast/extensions/pull/29604))                                                                                                                                                                                          | **gap (unknown publishability)**            |
| 8. Control surface      | —                                                                                                                                     | Terminal control is open: `cmux`, `ghostty`, `iterm`, `warp`, `kitty`; MCP management: `mcp`, `kill-mcp`, `codex-manager`; ACP client: `agent-client-protocol`                                                                                                                                                                                 | **open**                                    |

## What it means for a launcher of one's own

- **The glance-and-act half is already served inside Raycast, by third parties.** A new launcher gains nothing there
  on capability; Raycast's Store has a session browser, a status view and a usage dashboard for every major agent.
  Only speed, push instead of polling, and one place for everything remain as arguments.
- **The differential is the coordination layer:** fan-out, multi-agent coordination, policies over runs, live peeks,
  a hosted model gateway — everything that needs a process alive after the window closes. Neither Raycast nor its
  Store has it; Orca has it (Electron), Bernstein has policy (Python), claude-code-router has the gateway (Node).
- **Publishability is not the obstacle.** The Store accepts agent clients readily; its friction is overlap with
  existing extensions and, for local models, with Raycast's own AI.

## Searched

- Raycast manual: `/ai`, `/ai/agents`, `/ai/automations`; `raycast.com/pricing`.
- `gh api repos/raycast/extensions/git/trees/<extensions sha>`: 3,419 names, filtered by agent, session, worktree,
  terminal, model, router, MCP, cost and agent-tool names; `package.json` of 29 matches (description, command modes,
  background intervals, AI tools).
- `search/issues` on `repo:raycast/extensions is:pr is:closed is:unmerged` for router, gateway, orchestrat, parallel
  agents, multi-agent, subagent, guardrail or policy, claude code, codex; staff comments of eight of them.
- Not done in the time box: a full count of refusals; reading extension sources to confirm their descriptions.
