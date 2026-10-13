# Plan 8: The new design system — index

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** kb takes the design the owner approved screen by screen (spec:
[2026-10-12-design-system-design.md](../specs/2026-10-12-design-system-design.md), D1–D20); every design value lives in
DESIGN.md's front matter, every action, key and label in the catalog, and committed screenshots of the real app prove
each screen against its approved page.

**Architecture:** the plan is eight sub-plans, run in order; each leaves the app building, tested and usable. A design
module (`crates/kb-app/src/ui/design.rs`) parses DESIGN.md's front matter at start and is the only place a size or
colour is turned into GPUI units. A catalog (`crates/kb-app/assets/catalog.toml`, overridable from
`~/.config/kb/config.toml`) says what kb offers. One component per concept in `crates/kb-app/src/ui/` draws what the
approved pages show; `search_view` keeps the state and the handlers and composes the components.

**Tech Stack:** Rust 2024 (toolchain 1.99), GPUI Kit `=0.7.1` on `gpui-pre 0.3.8`, `fmt-rs =0.1.0`, SQLite FTS5
(`kb-domain`), Bun 1.4.2 for `tools/design`, mise, hk, nextest, ast-grep, dprint.

---

## The sub-plans, in order

| #  | File                                                                         | Produces                                                                                                                                          |
| -- | ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| 8a | [08a-foundation](2026-10-12-kb-native-08a-foundation.md)                     | Showcase corpus; DESIGN.md front matter; the design module; bundled fonts; always dark; the rounded transparent window; `design:theme`/`validate` |
| 8b | [08b-catalog](2026-10-12-kb-native-08b-catalog.md)                           | `catalog.toml`, its validation, `config.toml` overrides with per-key fallback, key bindings from the catalog, Configure                           |
| 8c | [08c-engine](2026-10-12-kb-native-08c-engine.md)                             | `search::count`, `search::tags_for_types`, `markdown::notes_body`; the four entry types, app-side                                                 |
| 8d | [08d-list](2026-10-12-kb-native-08d-list.md)                                 | keycap, chip, tile, row, header, footer and notice components; the list screen, the status line, ⌘1–⌘9, copy feedback                            |
| 8e | [08e-action-panel](2026-10-12-kb-native-08e-action-panel.md)                 | The action panel (root, Links…, More…), the screen ladder ↵ / ⌘↵ / ⇧⌘↵, the filter on the same component                                       |
| 8f | [08f-entry-view](2026-10-12-kb-native-08f-entry-view.md)                     | The entry template (split and detail), the command card, heroes, the Links section, clickable tags, the Markdown body                             |
| 8g | [08g-empty-and-refresh](2026-10-12-kb-native-08g-empty-and-refresh.md)       | The empty view, no index, the four refresh screens; old modules removed; the `no-px-outside-design` ast-grep rule                                 |
| 8h | [08h-shots-and-docs](2026-10-12-kb-native-08h-shots-and-docs.md)             | `design:shots`, the checklist review, every document of spec §10, `CHANGE-MAP.md`, TODO.md                                                      |

Run them strictly in this order: each sub-plan's first step lists what it expects from the previous one.

## Rules for every task

- Read the task's whole text before the first step. Steps are in order; never skip the "run it and see it fail" step.
- Write the failing test first; tests sit beside the code in `#[cfg(test)]` modules, UI tests in
  `crates/kb-app/tests/ui.rs`.
- After every step that changes Rust: `cargo nextest run -p <crate>` (the crate the step names).
- Before every commit: `mise run gate`. It must pass. If it fails, fix what it reports before committing; never skip a
  hook (`--no-verify` is forbidden).
- Commits: Conventional Commits, sentence-case subject ≤ 72 characters, body lines ≤ 72, ending with the line
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Each task gives the exact message.
- Never write to `$KB_TS_REPO`; never commit anything under `tools/bench/fixtures/private*`.
- `kb-domain` never depends on `gpui*` or `kb-app`.
- Use the words of [CONTEXT.md](../../../CONTEXT.md). New words this plan adds are listed below; use them exactly.
- **When the code in a step does not compile against GPUI Kit 0.7.1** (a renamed method, a different argument type),
  read the item's source under `~/.local/share/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/<crate>-0.7.1/` (or
  `gpui-pre-0.3.8/` for GPUI itself), make the smallest change that keeps the step's intent, and record the change in
  the commit body under "Deviation:". Do not change behaviour, names or tests to make it compile.

## Words this plan adds (CONTEXT.md takes them in 8h)

| Word         | Meaning                                                                                                          |
| ------------ | ---------------------------------------------------------------------------------------------------------------- |
| Design token | A value in DESIGN.md's front matter: a colour, a type role, a radius, a spacing or a component size.             |
| Catalog      | `crates/kb-app/assets/catalog.toml`: the entry types, actions, each screen's keys and the messages kb shows.     |
| Action panel | The ⌘K list of actions (root, Links…, More…), search at the bottom; replaces "Palette".                         |
| Screen keys  | What ↵, ⌘↵ and ⇧⌘↵ (and Esc on the refresh screen) do on a screen; the catalog's `[screens.*]`.                 |
| Status line  | The footer's left side on the list: `● Ready • N results`, `● Refreshing…`, `● No index`.                        |
| Command card | A command in JetBrains Mono, broken at operators by fmt-rs, wrapped with a hanging indent, with a copy button.   |
| Notice       | A message bottom right above the footer, through GPUI Kit's Notification; _Copied to clipboard_ is one.          |
| Refresh      | The action that runs a sync (⌘R); "Sync" stays the name of the process in the engine and CONTEXT.md.             |

## Refinements to the approved spec

Each refinement keeps the approved look and behaviour; it settles how, or follows the approved page where the spec's
text and the page differ (the pages are the contract).

| #  | Spec says                                                     | Plan does                                                                                                                                                                                                                       | Why                                                                                                                                       |
| -- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| R1 | `design:theme` writes a `design.json` the app loads           | The app parses DESIGN.md's front matter itself at start (`include_str!` + `kb_domain::yaml`); `design:theme` writes only GPUI Kit's `kb.json`                                                                                    | One source, no generated copy to drift; no new dependency                                                                                 |
| R2 | D13: "an extended highlight query (argument values coloured)" | GPUI Kit's stock bash highlighter, mapped to the tokens as in the spike                                                                                                                                                          | The approved page `04-command-cards.html` says "plain values in text colour"                                                              |
| R3 | D9 action panel; the filter reuses it                         | kb draws the action panel itself (`ui/action_panel.rs`) inside GPUI Kit's `Popover`, instead of GPUI Kit's `Command`                                                                                                             | `Command` puts its search at the top and fixes its row padding; the approved pages need the search at the bottom and 28 pt rows         |
| R4 | §1: `design:prototypes` and `design:shots`                    | `design:render`, `design:palettes` and `layout.json` go; `design:prototypes` renders the approved pages to PNG for the side-by-side review                                                                                       | `design:render` drew the old design from `layout.json`, which the spec retires                                                            |
| R5 | D16: the report lists "one row per file worth a look"         | One row per file: failed first, then files with skipped entries, then indexed files                                                                                                                                             | The approved page `11-14-refresh.html` lists indexed files too                                                                            |
| R6 | D15: footer _Toggle ↵ · Only ⌘↵_                              | _Only_ sets the filter to that one choice and closes the filter; _Toggle_ and _Clear All_ keep it open                                                                                                                          | The pages and TODO say only "Only this"; closing after a final choice is Raycast's behaviour                                              |
| R7 | D5: ⌘1…⌘9 on rows                                             | ⌘n is the n-th result of the list (index n − 1), whatever is scrolled; on the detail screen ⌘n opens link n; in the split the links show no ⌘n                                                                                 | The pages number rows from the top; one meaning per key per screen                                                                         |
| R8 | D20: the link title "trimmed with an ellipsis"                | The title takes at most half the row; the URL takes the rest; a URL longer than 32 characters keeps its host and last path segment with `/…/` between                                                                       | The owner asked for a defined limit                                                                                                        |
| R9 | D1: sizes in points                                           | Tokens are points; the design module turns them into rems at 16 pt per rem (`ui::design::pt`)                                                                                                                                    | Keeps the compliance rule "text sizes are in the rem scale" true at no visual cost                                                        |

## Constraints reviewed against the design

The owner asked (2026-10-13) that a kb rule which would degrade the approved design be questioned before the design is.
Each rule the design touches:

| Rule (where)                                                                                     | Clash                                                                                                                         | Resolution                                                                                                                                                                                                                                                                                                        |
| ------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| "Text meets WCAG 2.2 AA contrast on every surface it sits on" (PRODUCT.md; `design:validate`)   | `faint` `#6E738D` is 3.15:1 on the window, 2.70:1 on a rest chip; the pages use it for counts, hints, section labels and URLs | **Owner decision O1 (asked in 8a, Task 4).** Recommended: the rule is the problem — it treats tertiary text like body text. Amend it to "body and label text AA (4.5:1); tertiary text (`faint`) at least 3:1 on the window and popovers; colour never the only cue", and report faint-on-chip as a warning (as D19 did for accents). Declined: `faint` becomes `#8087A2` (Catppuccin overlay1, 4.0:1) everywhere. |
| "The app follows the system appearance" (docs/guides/compliance.md)                              | D1: always dark                                                                                                               | Already decided (D1). 8h rewrites the item: always dark is deliberate; kb has one approved palette.                                                                                                                                                                                                              |
| "Text sizes are in the rem scale" (compliance.md)                                                | The tokens are points                                                                                                          | R9: points become rems in one function; no clash left.                                                                                                                                                                                                                                                            |
| "'Quit kb' in the action palette" (compliance.md)                                                | D9 drops Quit from the panel                                                                                                   | Already decided (D9, `jm7r`); ⌘Q still quits. 8h rewrites the item.                                                                                                                                                                                                                                              |
| hk `large-files`: no committed file over 500 KB (hk.pkl)                                         | `design:shots` commits 2× screenshots; one 1720 × 1040 PNG of a dense screen can pass 500 KB                                  | **Owner decision O2 (asked in 8h, Task 1).** Recommended: the rule guards against accidental binaries, not deliberate evidence; exclude `tools/design/shots/**` from that one step. Declined: shots are captured at 1×.                                                                                            |
| Fonts must fit the same 500 KB limit                                                             | Inter's full static TTFs are ~400 KB each                                                                                      | No clash: Inter in fontsource's Latin subsets (68 KB each; macOS falls back for other scripts) and JetBrains Mono whole (274 KB, it carries ⌘ ⌥ ⇧ for keycaps).                                                                                                                                                   |
| "Unknown keys are kept, not rejected" (CONTEXT.md, Entry type)                                   | D6 shows four types only                                                                                                       | No clash: the app narrows; the engine and the index keep every type (D6).                                                                                                                                                                                                                                         |
| GPUI Kit's `Command` (buy over build)                                                            | Search at the top, fixed row padding                                                                                           | R3: the component cannot be configured to the approved pages; kb's panel is ~300 lines on GPUI primitives, inside GPUI Kit's `Popover`.                                                                                                                                                                         |

Until the owner answers O1, sub-plan 8a stops at its Task 4 checkpoint; nothing after it depends on the answer except
the value of `faint` and the validate thresholds, both written in that task for each answer.

## File map

Created (`crates/kb-app/src/` unless a path is given):

| File                                         | Responsibility                                                                                     | Sub-plan |
| -------------------------------------------- | -------------------------------------------------------------------------------------------------- | -------- |
| `tools/design/showcase/**`                   | The showcase corpus: the entries the approved pages show                                           | 8a       |
| `tools/design/showcase-images/*.svg`         | Stand-in tag images for the showcase (Lucide shapes)                                               | 8a       |
| `crates/kb-app/assets/fonts/*`               | Inter (400, 500, 600, 400 italic), JetBrains Mono (400, 500), their OFL licences                   | 8a       |
| `ui.rs`, `ui/design.rs`                      | The components' module; DESIGN.md's tokens as typed values, `pt()`, fonts                          | 8a       |
| `crates/kb-app/assets/catalog.toml`          | What kb offers                                                                                     | 8b       |
| `catalog.rs`                                 | Loads, merges and validates the catalog                                                            | 8b       |
| `ui/keycap.rs`, `ui/chip.rs`, `ui/tile.rs`   | D2's keycap; D3's chips; the type tile and the favicon                                             | 8d       |
| `ui/row.rs`, `ui/header.rs`, `ui/footer.rs`  | D5's row; D4's header; D14's footer                                                                | 8d       |
| `ui/notice.rs`                               | D17's copy notice                                                                                  | 8d       |
| `ui/action_panel.rs`                         | D9's panel: pages, sections, rows, search at the bottom                                            | 8e       |
| `actions.rs`                                 | The panel's pages for a screen and an entry, as data (replaces `palette.rs`)                       | 8e       |
| `ui/command_card.rs`                         | D13's card; no kb types                                                                            | 8f       |
| `ui/entry_view.rs`                           | D11's template: header, hero, quote, body, links                                                   | 8f       |
| `ui/empty_view.rs`                           | D10's empty view, also the refresh states                                                          | 8g       |
| `tools/ast-grep/rules/no-px-outside-design.yml` and its test | Rejects `px(` outside the design module                                            | 8g       |
| `tools/design/shots/*.png`, `tools/design/CHANGE-MAP.md`, `tools/design/CHECKLISTS.md` | Evidence and the change map                                   | 8h       |

Removed: `accents.rs`, `metadata.rs`, `notes.rs`, `palette.rs`, `search_view/overlays.rs`, `assets/themes/accents.json`,
`tools/design/layout.json`, the `design:render` and `design:palettes` tasks, `docs/guides/design/*` (replaced by the
approved pages in 8h).

## Owner checkpoints

| When                     | What the owner decides                                                    |
| ------------------------ | ------------------------------------------------------------------------- |
| 8a Task 4                | O1: the contrast rule for `faint`                                         |
| 8f Task 7                | The notes body: GPUI Kit's `TextView` or `zorite-markdown`, from two shots |
| 8h Task 1                | O2: shots and the large-files step                                        |
| 8h Task 3                | Each screen's shot against its approved page (the checklists)             |

At each checkpoint the executing agent stops, shows what the task says to show, and waits.

## Acceptance (spec) → where it is proved

| Spec criterion                                                       | Proved by                                                                                   |
| -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| 1. Every checklist passes on the shots, compact and expanded         | 8h Task 3 (`tools/design/CHECKLISTS.md` filled in, owner signs off)                         |
| 2. One place for every value; no literal size in views               | 8a `design` tests; 8b catalog tests; 8g ast-grep rule `no-px-outside-design`                |
| 3. `design:validate` passes, accents as warnings                     | 8a Task 5                                                                                   |
| 4. `config.toml` overrides a label, a key and a primary; bad → notice | 8b Tasks 3–4 tests                                                                          |
| 5. Only four types; goldens still pass                               | 8c Task 4 UI test; `goldens.rs` unchanged and passing                                       |
| 6. True count; ⌘1–⌘9; copy feedback on every copy                    | 8c Task 1, 8d Tasks 7–9 UI tests                                                            |
| 7. Documents updated; CHANGE-MAP.md exists                           | 8h Tasks 4–6                                                                                |
| 8. `mise run gate` passes; only threshold changes D19 (and O1, O2)   | Every commit                                                                                |

## Self-review (done when writing the plan)

- Spec coverage: D1 (8a), D2–D5 (8d), D6 (8c), D7–D9 (8b, 8e), D10 (8g), D11–D13 (8f), D14 (8d), D15 (8e), D16 (8g),
  D17 (8d), D18 (8a, 8b), D19 (8a), D20 (8f); §5 engine (8c); §6 spike (8f Task 7); §7 showcase (8a); §8 shots (8h);
  §9 checklists (8h); §10 documents (8h); §11 change map (8h).
- Names used across sub-plans are defined once: `ui::design::{design, pt, Design}`, `catalog::{Catalog, Ladder,
  ActionId}`, `actions::{PanelCommand, root_page, more_page, links_page, filter_page}`, `ui::action_panel::{ActionPanel,
  PanelPage, PanelSection, PanelRow}`. Each sub-plan's "Expects" line repeats the ones it uses.
