# Visual Pass — design

- Status: approved
- Date: 2026-10-11

## Goal

A structured way to check what tests cannot see: one checklist that walks through kb as a user does and asks, for
every rule the project already states, whether the running app obeys it. It runs by hand now, ticked in place, and
names for each check how it could be automated later.

## Decisions

**V1. One checklist, one router, no copied rules.** The rules already have homes: DESIGN.md (how kb looks),
PRODUCT.md (what it is for, its principles and accessibility), CONTEXT.md (its words), README "Keys" (what each key
does), `docs/experiment/parity.md` (behaviour against Raycast) and `compliance.md` (the GPUI Kit guidelines). The
checklist, `docs/experiment/visual-pass.md`, never restates a value or a rule: each check names the observable and
cites the rule's home (`DESIGN.md §4 Footer`, `README Keys`, `parity S09`). A check states its expectation in its own
words only when no doc holds it, and then that check is the rule's home. The doctrine,
`docs/doctrine/verify-what-users-see.doctrine.md`, is a router: when a change touches the UI, which doc owns which
kind of rule, and that the checklist must pass before merge. It holds no checks and no rules of its own.

**V2. Organised by the user's journey.** Phases follow one sweep through the app, so a manual run goes top to bottom
without jumping between screens; phase 0 holds what applies everywhere. The layout mirrors the earl-speckit tasks
template: numbered phases, then items.

**V3. EARS items.** Each check is one sentence in EARS (kiro SDD): `WHEN <trigger> THE panel SHALL …` for actions,
`WHILE <state> …` for states, `IF <failure> …` for failure paths, `THE panel SHALL …` for invariants. WHERE stays
unused until kb has a mode to name.

**V4. A method tag per check.** `[test]` (a UI or unit test covers it; the test is named), `[probe]` (scriptable from
outside: the CGEvent `probe` and a screenshot), `[pixel]` (needs an image compared with a reference), `[eye]` (human
judgement only). A summary counts the tags per phase, so the road to automation shows.

**V5. Ticked in place.** A run ticks the boxes and commits them; the checklist's "Last run" line names the date, kb
version and window sizes. A failure stays unticked with a one-line note and becomes a TODO item.

**V6. Coverage.** The dark appearance, both window sizes (compact and expanded; phase 8 re-checks the layout in
expanded), the owner's corpus. Out: the light appearance (TODO QTjG), edge-case data, a side-by-side with the Raycast
screenshots.

## Scope

In: the checklist, the doctrine, pointers from AGENTS.md and DESIGN.md, a TODO item for the first run. Out: running
the pass (the owner's next step), any automation (a later plan), changes to the rules themselves.

## Checklist shape

```markdown
# Visual Pass

How to run · Tag legend · Last run: 2026-10-11, v0.4.0, compact + expanded

## 0. Global

- [ ] **VP-0.01** [eye] THE panel SHALL draw every surface in its DESIGN.md §2 role (window, header, selection, raised).

## 3. Split pane

- [ ] **VP-3.02** [test] WHEN the caret is at the query's end and → is pressed THE panel SHALL show the split pane
      (README Keys; parity S09; `shows_details_with_right_at_the_end_of_the_query_and_hides_them_with_left`).

## Summary

| Phase | test | probe | pixel | eye |
```

Phases: 0 Global (surfaces, palette roles, typography, spacing, icons, copy) · 1 Open and hide · 2 Search and list
· 3 Split pane · 4 Detail screen · 5 Footer · 6 Actions popover · 7 Filter dropdown · 8 Window size (⌘⇧F, then
phases 2–7's layout in expanded) · 9 Rebuild · 10 Keyboard map (every README key). IDs are `VP-<phase>.<nn>`, stable
and never reused.

## Doctrine shape

Front-matter `name`, `applies_to: ['crates/kb-app/**', 'DESIGN.md', 'tools/design/**']`, `topics: [ui, visual,
design, keys]`, `always: false`; sections Principle, Why, Where the rules live (one table: kind of rule → owning doc),
How to apply (run the Visual Pass in both sizes; a new rule goes to its owning doc and gets a check; a check never
copies a rule).
