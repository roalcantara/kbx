# Is kb a Rust/GPUI tool worth building? A ladder of cheap disproof passes

- Date: 2026-10-13
- Question (owner): is kb a Rust/GPUI tool worth building, measured by opportunity, features and adoption (not
  revenue), on macOS, Linux and Windows?
- Bar (owner): about 2–3k engaged users within 12 months.
- Scope note (owner): kb's YAML format is not a defining trait — storage can change — so it is neither a
  differentiator nor a blocker. kb's category here: **a hotkey panel over one's own commands, cheat sheets, bookmarks
  and tasks, with a view per type.**
- Method: as [the launcher check](2026-10-13-launcher-product-check.md) — rungs of 5 minutes, 20 minutes and a day,
  each closed by a gate that re-evaluates its own kill signal.

# Rung 1 (5 minutes): the ceiling and the crowd

Read 2026-10-13 from Homebrew's analytics API and GitHub.

## Data

| Tool                        | Kind                               |  Stars | Created |                             Homebrew installs, 365 days |
| --------------------------- | ---------------------------------- | -----: | ------- | ------------------------------------------------------: |
| tldr (pages)                | Community command examples         | 63,934 | 2013-12 | 22,522 (`tldr`) + 30,293 (`tealdeer`) + 17,761 (`tlrc`) |
| cheat.sh                    | Online cheat sheets                | 41,803 | 2017-05 |                                                       — |
| Atuin                       | Shell history, synced, searchable  | 31,986 | 2020-10 |                                                 156,066 |
| navi                        | Interactive cheat sheets, terminal | 17,762 | 2019-09 |                                                   3,920 |
| espanso                     | Text expander                      | 14,623 | 2019-08 |                                           10,911 (cask) |
| cheat                       | Personal cheat sheets, terminal    | 13,478 | 2013-07 |                                                   3,191 |
| massCode                    | Snippet manager, Electron GUI      |  7,068 | 2022-03 |                                              512 (cask) |
| pet                         | Command snippet manager, terminal  |  5,360 | 2017-03 |                                                     328 |
| _Raycast (snippets inside)_ | _Launcher_                         |      — | —       |                                        _116,159 (cask)_ |
| _Obsidian (notes)_          | _Knowledge base_                   |      — | —       |                                        _196,077 (cask)_ |

New entrants since 2025-10-13: `topic:snippet-manager` 81 repositories, 2 above 100 stars; `topic:cheatsheet` 8
above 100; "command snippets manager" 0 above 100.

## What it shows

1. **The job is large:** keeping and finding one's commands is a five-figure-star category (tldr, cheat.sh, Atuin,
   navi, cheat), and Atuin alone draws 156k Homebrew installs a year.
2. **The bar sits where established personal tools land:** navi and cheat each draw 3–4k Homebrew installs a year on
   macOS alone — the order of the owner's 2–3k engaged users, after years in the market.
3. **The crowd is thin:** unlike launchers (2,116 new repositories a year), only a handful of new tools enter this
   category, and almost none break out.
4. **The incumbents live in the terminal.** navi, cheat, pet, tldr and Atuin are command-line tools; the GUI options
   are massCode (Electron; 512 installs) and Raycast's snippets (inside a launcher). None is a native panel from
   anywhere with commands, cheat sheets, bookmarks and tasks together, on all three platforms.

## Gate: are the kill signals sound?

| Signal                                                        | Fires?                                                                                                                                       | Sound?                                                                                                                                                                                                                                             |
| ------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| "No comparable tool reaches the bar"                          | **No.** Several exceed it; navi and cheat sit at its order on Homebrew alone.                                                                | Fair. Stars accumulate over years and installs are not engaged users; Homebrew is macOS only. Good enough to say the bar is not out of reach for the category.                                                                                     |
| "Free incumbents already do kb's core on all three platforms" | **Partly.** "Search and copy my commands" is done, freely, everywhere — in the terminal. A native GUI panel over several entry types is not. | **Under-defined.** It depends on what "kb's core" is. If the core is "find a command", it fires; if it is "one panel from anywhere for one's commands, links, cheat sheets and tasks", it does not. Rung 2 must test which of the two people want. |

**Refined signal for rung 2:** do users of the terminal tools (navi, cheat, pet, Atuin) and of Raycast's snippets ask
for what kb adds — a panel outside the terminal, several entry types in one place, a view per type — or are they
served? And by what channel would they hear of kb?

## Outcome of rungs 1 (owner)

Enough for the owner's bar — and the owner's further purpose, kb as a portfolio piece. Rung 2 not run.

# Next question: build from scratch, fork, port, or another angle?

- Question (owner): is kb worth building from scratch, feature by feature, or is there an alternative — a tool in this
  stack to fork and build on, a tool in another stack to port, or another angle?
- Purpose: adoption (2–3k engaged users in 12 months) and a portfolio piece. Authorship and craft count, not only speed.
- Starting point: kb is not at scratch. The engine (`kb-domain`, TypeScript-golden parity), the app (Plans 3–7), the
  benchmark kit and most of a design system (Plan 8) exist: about 10k lines of Rust plus tooling.

## Rung 1 (5 minutes): the candidates

Read 2026-10-13 from GitHub.

| Option            | Candidate       | Licence           |         Stars | Stack and GPUI line                                      | Job                                                |
| ----------------- | --------------- | ----------------- | ------------: | -------------------------------------------------------- | -------------------------------------------------- |
| Fork, same stack  | Steward         | MIT               |            13 | Rust, `gpui-pre =0.3.7` (kb's line, one minor behind)    | App launcher and plugin platform, Windows          |
|                   | Corvo           | MIT               |             0 | Rust, GPUI                                               | App launcher, many built-in tools; three weeks old |
|                   | zlaunch         | MIT               |           227 | Rust, Zed's git GPUI + gpui-component at a pinned commit | App launcher, Wayland                              |
|                   | Loungy          | MIT               |         1,730 | Rust, Zed's git GPUI (2024)                              | App launcher; archived                             |
|                   | Script Kit GPUI | Apache-2.0        |            29 | Rust, vendored gpui-component 0.5; 706 MB repository     | Script runner and prompts                          |
| Port, other stack | massCode        | AGPL-3.0          |         7,068 | Vue, Electron                                            | Snippet manager — the nearest job to kb's          |
|                   | pet, cheat      | MIT               | 5,360; 13,478 | Go, terminal                                             | Command snippets, cheat sheets                     |
|                   | Vicinae         | GPL-3.0           |        10,239 | C++, Qt                                                  | Launcher running Raycast's extensions              |
| Reuse, same stack | navi            | Apache-2.0        |        17,762 | Rust, **has a library crate** (`[lib] name = "navi"`)    | Cheat sheets with variables, terminal              |
|                   | Atuin           | MIT               |        31,986 | Rust                                                     | Shell history in SQLite                            |
| Reuse, content    | tldr pages      | CC BY 4.0 (pages) |        63,934 | Markdown                                                 | Community command examples                         |

## Gate

| Option    | Kill signal                                                                                                        | Fires?                                                                                                                                                                                                                                                                                             | Sound?                                                                                                                                    |
| --------- | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| **Fork**  | A permissive candidate on a compatible GPUI line covers most of kb's job                                           | **No.** Every candidate is an _app_ launcher; none has kb's job (a panel over one's own knowledge with a view per type). Forking means deleting most of their code and moving to their GPUI line: Steward's is one minor behind kb's; the others are on Zed's git GPUI or an older gpui-component. | Fair: judged from READMEs and manifests, and "job fit" is reasoning. It would change only if a candidate had kb's job, which none claims. |
| **Port**  | An app in another stack does kb's job better, under a licence kb (MIT) can take, for less than kb's remaining work | **No.** massCode is nearest in job, but AGPL (not adoptable into MIT kb) and an Electron app far larger than kb's remaining work; pet and cheat are terminal tools whose value is their format, not their code; Vicinae is GPL and a launcher.                                                     | Sound on licence; fair on size.                                                                                                           |
| **Reuse** | Nothing reusable that serves adoption                                                                              | **No — the opposite.** navi ships a library crate (Apache-2.0); Atuin's history is SQLite (MIT); tldr's pages are open content; pet, cheat and navi use simple formats. These are content and data, not UI: kb can read them.                                                                      | Sound: licences and the library are verified; value for adoption is reasoning (users bring the data they already have).                   |

## What it shows

- **Keep building kb.** No fork or port does kb's job, and kb's GPUI line is the newest of the candidates. For a
  portfolio, kb's own code is also the stronger piece: authorship is clear.
- **Buy the content and the data instead.** The adoption lever the fork question was really after lies here:
  importers for navi, cheat and pet files and Atuin's history (users arrive with their commands already in kb), and
  tldr's pages as bundled content (a useful kb on first launch, before the user writes anything). The storage format,
  which the owner called replaceable, is what makes this cheap.
- **Angles left aside, with reasons:** kb as an extension of another launcher (Raycast, Vicinae, Corvo) gives up the
  native standalone product and the portfolio piece; contributing a GUI to navi upstream gains navi's audience but
  makes kb a feature of someone else's project.
