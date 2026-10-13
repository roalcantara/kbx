# Pre-preflight: four agent tools and their features

- Date: 2026-10-13
- Purpose: read four tools that manage coding agents, list what each provides, and compare them, so the next pass can
  define which agentic-AI features the investigation evaluates. Read-only: README and docs, with spot checks in the
  code (shallow clones; nothing built or run).
- Asked by: the owner, before re-planning the AI launchers investigation (its first plan was not approved).

## The four tools at a glance

| Tool                                               | What it calls itself                                                | Stack                                 | Platforms                                        | Licence                                                     | Activity (2026-10-13)                  |
| -------------------------------------------------- | ------------------------------------------------------------------- | ------------------------------------- | ------------------------------------------------ | ----------------------------------------------------------- | -------------------------------------- |
| [agterm](https://github.com/umputun/agterm)        | "A simply good terminal with a full control API"                    | Swift, AppKit/SwiftUI, libghostty     | macOS 14+, Apple Silicon (Linux fork)            | MIT                                                         | 669★, created 2026-06, pushed daily    |
| [Supacode](https://github.com/supabitapp/supacode) | "Native macOS command center for running coding agents in parallel" | Swift, TCA, libghostty, zmx           | macOS 26+                                        | FSL-1.1-ALv2 (source-available; Apache-2.0 after two years) | 2,393★, created 2026-01, pushed daily  |
| [Zentty](https://github.com/dedene/zentty)         | "A native macOS terminal for agent-driven development"              | Swift, AppKit, libghostty             | macOS 14+                                        | GPL-3.0 (commercial licence available)                      | 313★, created 2026-04, pushed daily    |
| [Orca](https://github.com/stablyai/orca)           | "The ADE for working with a fleet of parallel agents"               | TypeScript, Electron, WebGL terminals | macOS, Windows, Linux; iOS and Android companion | MIT                                                         | 90,950★, created 2026-03, pushed daily |

Commits read: agterm `8c5ac7a07047`, Supacode `af5b4e32eb77`, Zentty `2aac548da434`, Orca `f84451842d0d`.

## Features, tool by tool

### agterm — a terminal you can script, which agents can drive

- **Organisation:** named workspaces holding long-lived named sessions; splits, a scratch terminal, overlays (a program
  run over a session without disturbing it); several windows.
- **Agent status:** an agent reports active, blocked or completed onto its session's sidebar row; hooks for Claude
  Code, Codex, Pi, OpenCode and others install from the Help menu (cookbook recipes add Copilot, Kimi, Kiro, containers).
- **Watching many at once:** a view-only dashboard grid of several sessions' live output; attention glyphs; session
  notifications.
- **Control API:** `agtermctl` over a local socket — create sessions, type into them, read their text back, run an
  overlay and get its exit status, set status, open a picker, ask a yes/no question, dump the model as JSON, events.
- **Agents drive it:** an installable skill (Claude Code and Codex plugin) teaches an agent the control model, so the
  agent builds its own layout and runs overlays.
- **Persistence:** three restore modes, the strongest keeping processes alive through zmx across restarts.
- **Recipes, not features:** a cookbook (session resume for Claude, Codex, OpenCode, Kimi; account swap; conversation
  picker; a flagged-sessions dashboard; remote Claude). Its stated design: no deep agent integration; build the rest
  yourself on the API.
- Not there: worktrees, git or PR awareness, prompting several agents at once, model choice, cost tracking.

### Supacode — one git worktree per task, one terminal per worktree

- **Worktree-first:** each task gets its own git worktree and terminal so parallel agents never collide; created from
  the sidebar, a hotkey, the palette, the CLI or a deeplink; pin, archive, auto-delete after N days, ⌃1–⌃9.
- **Agent presence:** detects the agent in each pane (busy, awaiting input, idle) through hooks it installs; supported
  agents in code: Antigravity, Claude, Codex, Copilot, Grok, Hermes, Kimi, Kiro, OMP, OpenCode, Pi
  (`SupacodeSettingsShared/Models/SkillAgent.swift`); worktrees that need you rise to the top.
- **Git and PRs:** live branch, file and pull request state per worktree; PR checks and merge readiness; the full set of
  PR actions in the palette.
- **Persistence and remote:** sessions in zmx survive quitting the app; SSH repositories (beta) with one multiplexed
  connection and reattach after a drop.
- **Scripting:** the `supacode` CLI and `supacode://` deeplinks for worktrees, tabs, splits, repos; per-repo scripts,
  setup and archive scripts.
- **Notifications:** in-app and system, sounds, per-surface mute.
- Not there: talking to the agents (only watching them), several agents on one prompt, model choice, cost tracking.

### Zentty — a terminal that understands what each agent is doing

- **Layout:** "worklanes", horizontally scrolling columns of panes (niri/Hyprland style); visual pane switcher; command
  palette; restore of worklanes and agent sessions, resuming the agent on relaunch.
- **Agent status, the richest of the four:** working, waiting, asking for approval, compacting; **subagents and teams**
  (a badge counts running subagents and names their models); a menu-bar status; crash detection by PID; 17 agents
  supported out of the box (Amp, Antigravity, Claude Code, Codex, Copilot CLI, Cursor, Devin, Droid, Gemini CLI, Grok
  Build, Hermes, Kimi, Mistral Vibe, Oh My Pi, OpenCode, Pi, Small Harness).
- **An open protocol:** the [Agent Status Protocol](https://github.com/dedene/zentty/blob/main/docs/agent-status-protocol.md)
  (JSON events to the bundled CLI; phases `starting`, `running`, `needs-input`, `idle`; session hierarchy via
  `parentId`; interaction kinds; signal confidence), so any agent joins without changes to Zentty.
- **Around the agent:** 1Password and ssh-agent prompts point to the pane that asked; keeps the Mac awake while agents
  run; running dev servers and PR checks in the sidebar; task runners detected (package.json, Make, just, Task, mise).
- **Terminal comforts:** paste files into ssh sessions, clean copy (no prompts or box drawing; copy as Markdown),
  global search across panes, Ghostty themes, a scriptable `zentty` CLI and an agent skill.
- Not there: worktree management, several agents on one prompt, model choice, cost tracking.

### Orca — an environment for a fleet of agents

- **Parallel worktrees:** fan one prompt across several agents, each in its own worktree; compare the results and
  merge the winner.
- **Orchestration, the only one of the four:** an `orchestration` skill for supervised workers — threaded messages,
  blocking ask/reply, task dispatch, `worker_done` and escalation waits, **task DAGs**, decision gates, coordinator
  loops, decomposing work across agents (`skills/orchestration/SKILL.md`; the full guide is served by the `orca`
  binary).
- **Any agent, your own subscription:** Claude Code, Codex, OpenCode, Pi or any CLI agent; an account switcher and
  usage tracking (Claude and Codex usage and rate-limit resets; hot-swap accounts).
- **Review:** annotate AI diffs line by line and send the comments back to the agent; GitHub and Linear (PRs, issues,
  boards) in the app; an editor (VS Code's) with drag-and-drop of files into a prompt.
- **Beyond the terminal:** a Chromium window with design mode (click an element to send its HTML, CSS and a screenshot
  to the agent); computer use (agents operate desktop apps); Android emulator skills.
- **Remote and mobile:** SSH worktrees with port forwarding; a mobile companion (iOS, Android) to watch and steer agents
  and get notified; a relay service in the repository.
- **Scripting:** the `orca` CLI (`worktree create`, `snapshot`, `click`, `fill`), which agents use too.
- Telemetry is on by default (opt-out documented).

## How they compare

All four exist for the same reason: a developer now runs several coding agents at once, in long-lived terminal
sessions, and an ordinary tabbed terminal loses track of which one needs attention. The three Swift apps answer that
by being a better terminal; Orca answers it by being a workspace in which the terminal is one part.

**The common floor.** Every tool runs real terminals (three of them on Ghostty's engine, libghostty; Orca on WebGL),
keeps sessions across restarts (agterm and Supacode both lean on zmx), shows each agent's state — working, waiting for
you, done — and sends a notification when one needs you. Every tool can be scripted from a CLI, and three ship an
agent skill so the agents can drive the tool themselves. Status is collected the same way everywhere: hooks installed
into each agent CLI, not by reading the screen. These are table stakes; a product without them is not in this
category.

**Where they part.** agterm deliberately stops at a scriptable terminal: workspaces, a dashboard, and a complete
control API, leaving the rest to recipes. Supacode adds the **git worktree** as the unit of work — one task, one
worktree, one terminal — and folds in branches, pull requests and remote hosts, which is what a team doing parallel
feature work needs. Zentty goes deepest on **observing agents**: an open status protocol, subagent and team badges with
their models, approval and compaction states, crash detection, and seventeen agents supported. Orca is the only one
that **coordinates** agents rather than watching them: one prompt fanned to several agents, threaded messages between
them, task DAGs and decision gates, then review of their diffs and a merge; it also adds what the others leave out —
usage and rate-limit tracking across accounts, a browser and computer use, and phone control.

**On the owner's three axes** (many agents × many models × many workflows):

- **Many agents:** all four run many agents side by side; Zentty also sees an agent's own subagents; Orca alone lets
  agents talk to each other.
- **Many models:** none of the four picks or routes models itself. They are model-agnostic by running whichever agent
  CLI you install, each with its own model; Zentty displays the models subagents use; Orca tracks usage per account.
- **Many workflows:** agterm offers recipes; Supacode the worktree lifecycle and scripts; Zentty task runners; Orca
  defined workflows (DAGs, gates, coordinator loops, fan-out and pick-the-winner).

**Openness and reuse.** agterm and Orca are MIT; Zentty is GPL-3.0 (reference only for kb); Supacode is FSL (source
available, not open source until each version turns two). Zentty's Agent Status Protocol is the one artefact written to
be adopted by others; zmx is shared infrastructure for two of them.

**Native or not.** The three Swift apps stay native and macOS-only and are an order of magnitude smaller in scope;
Orca is Electron, cross-platform with mobile, and by far the most adopted (about 91k stars against a few thousand).

## What this suggests for the next pass

Feature families that recur, as candidates for the investigation's taxonomy (to be refined, not frozen):

1. **Session management:** named, long-lived, persistent sessions; workspaces; restore.
2. **Agent status and attention:** state per agent, attention ranking, notifications, menu bar; a status protocol.
3. **Observation at scale:** dashboards of live output, subagent and team visibility, crash detection.
4. **Isolation of work:** a git worktree per task, setup and archive scripts.
5. **Coordination:** fan-out of one prompt, agent-to-agent messages, task DAGs, decision gates, coordinator loops.
6. **Review and merge:** diff annotation sent back to the agent, compare results, PR state.
7. **Accounts, usage and cost:** subscriptions, usage and rate limits, account switching.
8. **Control surface:** a CLI and socket API, deeplinks, agent skills so agents drive the tool.
9. **Reach:** remote hosts over SSH, mobile companion, browser and computer use.

Model routing (choosing models per task) appears in none of the four, which makes it either a gap or out of this
category; the next pass should decide which.
