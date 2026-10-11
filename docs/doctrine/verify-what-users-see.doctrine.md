---
name: verify-what-users-see
applies_to: ['crates/kb-app/**', 'DESIGN.md', 'tools/design/**']
topics: [ui, visual, design, keys, layout]
always: false
---

# Verify what users see

## Principle

A change to what kb shows, or to how it answers a key, is done when the running app obeys the rules in both window
sizes: the Visual Pass ([docs/experiment/visual-pass.md](../experiment/visual-pass.md)) passes, and its ticks are
committed.

## Why

Tests prove what the code can observe. Colour, alignment, truncation, anchoring and focus are seen, not asserted, and
they drift one change at a time. The pass is where they are checked, against the rules rather than against memory.

## Where the rules live

Each kind of rule has one home. Read it, and change it, there.

| Rule                                            | Home                                                                |
| ----------------------------------------------- | ------------------------------------------------------------------- |
| How kb looks: palette, type, components, layout | [DESIGN.md](../../DESIGN.md) (its values in `tools/design/`)        |
| What kb is for: principles, accessibility, tone | [PRODUCT.md](../../PRODUCT.md)                                      |
| The words: entry, filter, panel, palette…       | [CONTEXT.md](../../CONTEXT.md)                                      |
| What each key does                              | [README, Getting Started](../../README.md#getting-started)          |
| Behaviour against Raycast's kb Search           | [parity.md](../experiment/parity.md)                                |
| The GPUI Kit guidelines kb follows              | [compliance.md](../experiment/compliance.md)                        |
| An expectation none of the above holds          | the check itself, in [visual-pass.md](../experiment/visual-pass.md) |

## How to apply

- Before merging a UI change, run the Visual Pass in the small and the expanded window, and commit the ticks.
- The Visual Pass turns these rules into plain observations a tester can make without reading them. When a rule
  changes in its home, change the check that observes it in the same commit.
- What a test can prove belongs in a test, listed in the Visual Pass's appendix, not in its manual checks.
- A failed check stays unticked with a note of what was seen, and becomes a TODO item.
