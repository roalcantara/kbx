# Handoff: the MVP's final verification

Run this after plan `docs/superpowers/plans/2026-10-27-mvp-close-out.md` lands. It holds what that plan left out on
purpose: the long checks and the ones that take over the screen. Report each result. Fix nothing here: a failure
becomes a TODO with its evidence.

## 1. Gates

- `mise run gate`: it must pass. The audit's reproductions are green now.
- `mise run size`: within `tools/size/budget`.

## 2. Screenshots (takes over the screen and the keyboard; tell the owner before starting)

`mise run design:shots`, then compare each shot in `tools/design/shots/`, compact and expanded, with its approved page
in `docs/guides/design/` (render them with `mise run design:prototypes`). New since the last shots:

- the `‹ <query>` and `‹ Refresh` headers (20-navigation);
- the report's split and detail (12-report-*);
- a split with the keys in the pane (21-pane-focus: the 1 pt bar, the list's faint bar);
- the footer under ⌘K (_Run ↵ · Back esc · Close ⌘K_).

Add any of these that `tools/design/tasks.toml` does not capture yet.

## 3. By hand (`mise run app`, against the sample or showcase corpus, never the private one)

- [ ] Long titles and descriptions are cut off with an ellipsis, in compact and expanded.
- [ ] Tag images draw at their size beside `#tag` pills.
- [ ] Hover: list rows, ⌘K rows and filter rows; the selected row shows no hover.
- [ ] Focus: the pane's bar after ⇥; Esc's levels: pane → list → clear the query → hide.
- [ ] Hide by focus loss (click another app), reopen with ⌥K: the same place, query and selection.
- [ ] Every navigation key on the list, filter, ⌘K and a submenu, Theme…, the report, and both detail screens.
- [ ] Each of the eight themes: More… → Theme…, preview with ↑↓, Esc restores, ↵ keeps; text is legible.
- [ ] VoiceOver (⌘F5): rows read their labels; the back button reads its label.

## 4. Bookkeeping

- [ ] Every finding in `docs/superpowers/audits/2026-10-27-mvp-ui-ux-audit.md` is marked fixed with its commit.
- [ ] TODO.md: 2YnZ and the report-detail scrolling task are closed (`todo status`), each with a comment naming
      its plan.
- [ ] `docs/guides/parity.md` and `docs/guides/compliance.md` reflect the navigation model and the close-out.
- [ ] The verdict: the MVP is closed, or the list of what still blocks it.
