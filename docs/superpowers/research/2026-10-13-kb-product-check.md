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
