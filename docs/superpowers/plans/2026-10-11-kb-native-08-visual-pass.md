# Plan 8: Visual Pass Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A Visual Pass checklist and the doctrine that routes to it, as the spec
[2026-10-11-visual-pass-design.md](../specs/2026-10-11-visual-pass-design.md) describes, ready for the owner's first
manual run.

**Architecture:** Docs only. `docs/experiment/visual-pass.md` holds every check, in EARS, each citing the doc that
owns the rule it verifies; `docs/doctrine/verify-what-users-see.doctrine.md` routes a UI change to those docs and to
the checklist. No rule or value is copied: the checklist asks, the owning docs state.

**Tech Stack:** Markdown, dprint, hk (typos, ls-lint), the todo CLI.

**Rules for every task:** `dprint fmt` the files; `hk check --all` before each commit; Conventional Commits, subject
≤ 72, body lines ≤ 72, ending `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Never copy a value (hex,
size, key) or a rule into the checklist: cite where it lives.

## File map

| File                                              | Change | Responsibility                                               |
| ------------------------------------------------- | ------ | ------------------------------------------------------------ |
| `docs/doctrine/verify-what-users-see.doctrine.md` | create | the rule to run the pass, and where each kind of rule lives  |
| `docs/experiment/visual-pass.md`                  | create | every check, EARS, tagged, citing its source; ticked per run |
| `AGENTS.md`                                       | modify | repository map + core rule: pointer to the doctrine          |
| `DESIGN.md`                                       | modify | one line: the Visual Pass verifies this file                 |
| `TODO.md`                                         | modify | bQVp points to the checklist                                 |

---

### Task 1: The doctrine

- [ ] **Step 1:** Create `docs/doctrine/verify-what-users-see.doctrine.md`:

```markdown
---
name: verify-what-users-see
applies_to: ['crates/kb-app/**', 'DESIGN.md', 'tools/design/**']
topics: [ui, visual, design, keys, layout]
always: false
---

# Verify what users see

## Principle

A change to what kb shows or how it answers a key is done when the running app obeys the rules, in both window
sizes: the Visual Pass ([docs/experiment/visual-pass.md](../experiment/visual-pass.md)) passes, and its ticks are
committed.

## Why

Tests prove behaviour the code can observe. Colour, alignment, truncation, anchoring and focus are seen, not
asserted, and they drift one change at a time. The pass is where they are checked, against the rules, not against
memory.

## Where the rules live

Each kind of rule has one home. Read it there; never copy it elsewhere, the checklist included.

| Rule                                                   | Home                                                                |
| ------------------------------------------------------ | ------------------------------------------------------------------- |
| How kb looks: palette, type, components, layout        | [DESIGN.md](../../DESIGN.md) (values in `tools/design/`)            |
| What kb is for: principles, accessibility, tone        | [PRODUCT.md](../../PRODUCT.md)                                      |
| The words: entries, filters, the panel, the palette    | [CONTEXT.md](../../CONTEXT.md)                                      |
| What each key does                                     | [README, Keys](../../README.md#getting-started)                     |
| Behaviour against Raycast's kb Search                  | [parity.md](../experiment/parity.md)                                |
| The GPUI Kit guidelines kb follows                     | [compliance.md](../experiment/compliance.md)                        |
| Anything the checklist expects that no doc above holds | the check itself, in [visual-pass.md](../experiment/visual-pass.md) |

## How to apply

- Before merging a UI change, run the Visual Pass in compact and expanded (⌘⇧F) and commit the ticks.
- A new rule goes to its home first; then a check cites it. A check states an expectation in its own words only when
  no home holds it.
- A failed check stays unticked with a one-line note and becomes a TODO item.
```

- [ ] **Step 2:** `dprint fmt docs/doctrine/*.md && hk check --all`. If ls-lint refuses the `.doctrine.md` name or the
      `docs/doctrine/` folder, add them to `.ls-lint.yml` in the same commit, as the repo's naming for doctrines.
- [ ] **Step 3:** Commit `docs: Add the verify-what-users-see doctrine`.

### Task 2: The checklist

- [ ] **Step 1:** Create `docs/experiment/visual-pass.md` with this frame:

```markdown
# Visual Pass

What the tests cannot see, checked by hand in the running app until each check is automated. Every check verifies a
rule that lives elsewhere and cites it; the doctrine
[verify-what-users-see](../doctrine/verify-what-users-see.doctrine.md) says where each kind of rule lives.

**Last run:** none yet.

## How to run

1. `mise run show`, with the system in dark appearance and your own sources indexed (⌘R once).
2. Go through the phases in order, compact first; phase 8 repeats the layout checks in expanded (⌘⇧F).
3. Tick what holds. Leave a failure unticked with ` — <what was seen>` after it, and file it in TODO.md.
4. Update **Last run** (date, kb version, sizes) and commit the ticks: `docs: Run the Visual Pass on <version>`.
5. Before the next run, untick everything: `sed -i '' 's/^- \[x\] \*\*VP-/- [ ] **VP-/' docs/experiment/visual-pass.md`.

## Tags

How each check is verified, and the route to automating it:

- **[test]** a UI or unit test already proves it; the test is named. A manual run only confirms the app as built.
- **[probe]** scriptable from outside: the CGEvent `probe` (tools/bench/probes) drives keys, a screenshot or the
  window list shows the result.
- **[pixel]** needs an image compared with a reference capture.
- **[eye]** human judgement: balance, legibility, calm.

## Sources

`DESIGN §n` = DESIGN.md section; `PRODUCT` = PRODUCT.md; `Keys` = README, Keys; `Sn`, `An`, `Fn`, `Ln` = parity.md
rows; `UI-nn` = the UI comparison findings mapped in DESIGN.md's appendix; `compliance` = compliance.md.
```

- [ ] **Step 2:** Write phases 0–10, items `- [ ] **VP-<phase>.<nn>** [tag] <EARS sentence> (<sources>)`, deriving
      each item from its sources and adding none that repeats another:

| Phase              | Items come from                                                                                                                   |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------- |
| 0. Global          | DESIGN §1–§3, §6 (surfaces in their roles, accents by meaning, type per element, Title Case copy, no blur); PRODUCT accessibility |
| 1. Open and hide   | Keys (⌥K, Esc), parity L01/L02, PRODUCT principle 4 (never block typing), CONTEXT Panel/Hotkey                                    |
| 2. Search and list | DESIGN §4 List row and Tag pill, §5 alignment; parity S01–S08 rows the screen shows; UI tests for typing and ↑↓                   |
| 3. Split pane      | Keys (→ ←), DESIGN §4 Split pane, Metadata card, Notes; parity S09, S12; UI-08…10, UI-20, UI-22                                   |
| 4. Detail screen   | Keys (↩, → ←, Esc), DESIGN §4 Detail screen, Header; parity S11; UI-11, UI-12, UI-23                                              |
| 5. Footer          | DESIGN §4 Footer; UI-07                                                                                                           |
| 6. Actions popover | Keys (⌘K, Esc), DESIGN §4 Actions popover, §6; parity A01–A06; UI-13…16                                                           |
| 7. Filter dropdown | Keys (⌘P, Esc), DESIGN §4 Filter dropdown; parity F01–F03, T3; UI-17…19                                                           |
| 8. Window size     | Keys (⌘⇧F), DESIGN §5 Window; CONTEXT Size toggle; then phases 2–7's layout items, re-checked expanded                            |
| 9. Rebuild         | Keys (⌘R), parity rows on sync and the sync report, CONTEXT Sync/Sync report                                                      |
| 10. Keyboard map   | every row of Keys not checked by an earlier phase                                                                                 |

    Phase 8 re-checks by citing the earlier IDs (`WHILE the window is expanded THE panel SHALL satisfy VP-2.03,
    VP-3.04, …`), not by copying them. A `[test]` item names its test from `crates/kb-app/tests/ui.rs` or the unit
    tests; check each name exists with `grep -n "fn <name>" crates/kb-app/tests/ui.rs crates/kb-app/src -r`.

- [ ] **Step 3:** Append the summary:

```markdown
## Summary

| Phase | Checks | test | probe | pixel | eye |
| ----- | ------ | ---- | ----- | ----- | --- |
```

    one row per phase, counted from the items (Step 4's command prints them).

- [ ] **Step 4: Verify.**
  - IDs unique: `grep -oE 'VP-[0-9]+\.[0-9]+' docs/experiment/visual-pass.md | sort | uniq -d` prints nothing.
  - Every item has one tag and a source:
    `grep -E '^- \[ \] \*\*VP-' docs/experiment/visual-pass.md | grep -vE '\] \*\*VP-[0-9.]+\*\* \[(test|probe|pixel|eye)\] .*\(.+\)$'`
    prints nothing.
  - EARS: `grep -E '^- \[ \] \*\*VP-' docs/experiment/visual-pass.md | grep -vE 'SHALL'` prints nothing.
  - No copied values: `grep -nE '#[0-9A-Fa-f]{6}|[0-9]+ ?(pt|px)\b|[0-9]+ ?×' docs/experiment/visual-pass.md` prints
    nothing.
  - Tag counts for the summary: `for t in test probe pixel eye; do grep -c "\[$t\]" docs/experiment/visual-pass.md; done`.
- [ ] **Step 5:** `dprint fmt docs/experiment/visual-pass.md && hk check --all`; commit
      `docs: Add the Visual Pass checklist`.

### Task 3: Pointers and TODO

- [ ] **Step 1:** ``AGENTS.md``, repository map: ``- \``docs/doctrine/\`: principles agents load when a change matches
      their \`applies_to\`; start with verify-what-users-see for any UI change``. Core engineering rules: ``- A UI change
      passes the Visual Pass before merge (docs/doctrine/verify-what-users-see.doctrine.md).`
- [ ] **Step 2:** `DESIGN.md`, after the source-of-truth table's paragraph: `The Visual Pass
      ([docs/experiment/visual-pass.md](docs/experiment/visual-pass.md)) checks the running app against this file.`
- [ ] **Step 3:** TODO bQVp: `todo update bQVp --description "Run the Visual Pass (docs/experiment/visual-pass.md) on
      v0.4.0, compact and expanded; file failures"`.
- [ ] **Step 4:** `hk check --all`; commit `docs: Point agents and DESIGN.md at the Visual Pass`.
- [ ] **Step 5:** Push `chore/visual-pass`; `gh pr create -R roalcantara/kbx`; merge on green (docs only: no release).

## Self-review

- Spec V1 → Tasks 1 (router, no rules), 2 (citations, Step 4 "no copied values"); V2 → Task 2 Step 2 phases; V3 →
  Step 4 EARS check; V4 → frame Tags, summary; V5 → How to run 3–5; V6 → How to run 1–2, phase 8.
- No duplication: the doctrine's table points, the checklist cites, DESIGN.md and AGENTS.md get one line each.
