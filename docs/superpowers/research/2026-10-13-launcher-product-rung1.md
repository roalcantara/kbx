# Is a Rust/GPUI launcher worth building as a product? Rung 1 (5 minutes)

- Date: 2026-10-13
- Question (owner): is a Rust/GPUI launcher in Raycast's vein worth building as a product — by adoption and revenue,
  on macOS, Linux and Windows?
- Method: a ladder of cheap disproof passes (5 minutes, 20 minutes, 1 day), each run only if the previous one does not
  kill the idea, each closed by a gate that also re-evaluates its own kill signal (owner, 2026-10-13).
- This rung: the ceiling and saturation, from API data only — Homebrew install analytics and GitHub.

## Data

**Homebrew installs** (casks, users with analytics on; macOS only; read 2026-10-13):

| Cask        | 30 days | 90 days | 365 days | What it is                                      |
| ----------- | ------: | ------: | -------: | ----------------------------------------------- |
| raycast     |   6,265 |  23,951 |  116,159 | The incumbent                                   |
| cmux        |   3,906 |  13,844 |   57,763 | Ghostty-based terminal for coding agents (2026) |
| hammerspoon |   4,898 |  14,982 |   52,801 | Scriptable automation                           |
| alfred      |   1,789 |   6,278 |   24,065 | The previous incumbent                          |
| supacode    |     271 |   1,606 |    8,287 | Agent worktree manager (2026)                   |
| sol         |     220 |     774 |    2,874 | Open-source launcher                            |
| superset    |     206 |     622 |    2,703 | Agent IDE                                       |
| orca        |       — |     739 |    1,127 | Agent orchestrator                              |
| vicinae     |     164 |     452 |      508 | Raycast-compatible launcher (macOS build new)   |
| script-kit  |       7 |      33 |      158 | Script Kit v1                                   |
| loungy      |       — |       4 |       38 | GPUI launcher, archived                         |

**GitHub** (stars; created; last push):

| Launcher                                |   Stars | Created | Notes                                |
| --------------------------------------- | ------: | ------- | ------------------------------------ |
| PowerToys (Run, Command Palette)        | 139,261 | 2019-05 | Microsoft; Windows                   |
| Flow Launcher                           |  15,765 | 2020-04 | Windows                              |
| Vicinae                                 |  10,239 | 2025-07 | Linux first; runs Raycast extensions |
| Albert                                  |   8,006 | 2014-07 | Linux                                |
| Ulauncher                               |   4,523 | 2015-05 | Linux                                |
| Script Kit                              |   4,191 | 2020-12 | macOS; last push 2025-12             |
| Sol                                     |   3,168 | 2022-01 | macOS                                |
| Loungy (Rust/GPUI)                      |   1,730 | 2024-02 | archived                             |
| Kunkun                                  |   1,305 | 2024-11 | Tauri                                |
| RustCast (Rust)                         |   1,107 | 2025-12 | "retired"                            |
| Asyar (Rust/Tauri)                      |     904 | 2025-02 |                                      |
| Gauntlet (Rust)                         |     819 | 2024-01 | last push 2025-10                    |
| _cmux (agent terminal, for comparison)_ |  28,146 | 2026-01 |                                      |

**Saturation** (GitHub search, repositories created since 2025-10-13): `topic:launcher` 2,116, of which 35 above 100
stars (1.7 %) and 5 above 1,000 (0.2 %); "raycast alternative" 33, of which 3 above 100.

## What it shows

1. **The ceiling for an open-source launcher is about 10–16k stars**, and it takes a wedge: Flow Launcher (Windows,
   five years), Vicinae (Linux, 15 months, and it runs Raycast's extensions). Every Rust launcher sits at 0.8–1.7k
   stars, and two of the four are archived or retired.
2. **The space is crowded with stalled entries:** 2,116 new launcher repositories in a year, 98 % under 100 stars.
3. **A breakout is still possible:** Vicinae went from nothing to 10k stars in 15 months. It had a market without the
   incumbent (Linux) and a way to inherit the incumbent's ecosystem (Raycast extensions).
4. **Attention in 2026 went to agent tools, not launchers:** cmux, created in January, has more stars than any
   launcher but PowerToys and more Homebrew installs than Alfred; Supacode and Superset outsell every open-source
   launcher on Homebrew.

## Gate: is the kill signal sound?

The signal was: "the best open-source alternatives plateau far below the incumbents while new launchers stall —
a ceiling too low for adoption or revenue". Re-evaluated against the data:

- **It does not kill.** Most entries stall, but Vicinae shows a new entrant can reach five figures quickly with a
  wedge. The signal conflated "most fail" (true of any GitHub category) with "none can succeed" (false).
- **It cannot kill as formulated.** "Too low" was never quantified: without the owner's target (how many users, how
  much revenue makes it worth it), no ceiling is too low.
- **Its measures are proxies.** Stars mix old and new projects (growth rate matters more than totals); Homebrew counts
  only macOS users of Homebrew with analytics on, and most Raycast installs come from its own download, so absolute
  numbers are low and only comparisons hold; neither measures revenue.

**Refined signal for the next rungs:** "No wedge exists that a Rust/GPUI launcher could own" — where a wedge is a
segment without a strong incumbent, or a way to inherit an incumbent's ecosystem, as Vicinae did. And a number from the
owner: what adoption and revenue, by when, would make it worth building.
