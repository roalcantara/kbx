# Spacing ramp Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** every spacing value in DESIGN.md is a step of the GPUI Kit ramp, and the header, rows, footer, panes and
notice share one content edge (20 = `list.pad` + `row.pad_x`).

**Architecture:** DESIGN.md gains `spacing.scale`; `ui/design.rs` gains `Design::content_inset()` and a test that
fails on off-ramp values. Tokens the shared edge replaces are removed from DESIGN.md and their `points!` structs, so
the compiler finds every reader; each reader switches to `content_inset()` or the remaining token.

**Tech Stack:** Rust, GPUI Kit, DESIGN.md front matter (`ui/design.rs`), cargo-nextest, mise.

**Spec:** [2026-10-19-spacing-ramp-design.md](../specs/2026-10-19-spacing-ramp-design.md) (D1–D7, the mapping
table).

**Expects** (verified at `093607a`): the `points!` structs `HeaderSpacing`, `RowSpacing`, `PaneSpacing`,
`FooterSpacing`, `NoticeSpacing`, `FooterSize` in `ui/design.rs`; readers at `ui/header.rs:19`, `ui/row.rs:118-122`,
`ui/footer.rs:24-44,127`, `ui/entry_view.rs:430-433`, `search_view.rs:658-664`, `ui/notice.rs:22-26`,
`search_view/footer.rs:142`; `Row::compact` used only for padding (`entry_list.rs:293`).

**Conventions:** failing test first; `cargo nextest run -p kb-app <filter>`; `git add <paths>` then `git commit`;
Conventional Commits with the harness's `Co-Authored-By` trailer; `hk fix --all` before committing. The crate denies
dead code, so a token leaves DESIGN.md in the same commit as its last reader. No design shots in this pass (owner).

---

### Task 1: The ramp, the shared edge and the values

**Files:** `DESIGN.md`, `crates/kb-app/src/ui/design.rs`, `ui/header.rs`, `ui/row.rs`, `entry_list.rs`,
`ui/footer.rs`, `search_view/footer.rs`, `ui/entry_view.rs`, `search_view.rs`, `ui/notice.rs`.

- [ ] **Step 1: Write the failing tests** (`design.rs` `mod tests`; replace `reads_the_keycap_and_slot_tokens`'s
      `label_slot` assertion line with nothing)

```rust
    #[test]
    fn keeps_every_spacing_on_the_ramp() {
        let d = parse(DESIGN_MD).unwrap();
        let s = &d.spacing.scale;
        let ramp = [0., s.xxs, s.xs, s.sm, s.md, s.lg, s.xl, s.xxl];
        assert_eq!(ramp, [0., 2., 4., 8., 12., 16., 24., 32.]);
        // Composite clearances, not gaps (spec D6).
        let exceptions = ["card.pad_right", "notice.bottom"];
        let json = serde_json::to_value(kb_domain::yaml::to_json(
            &kb_domain::yaml::load(front_matter(DESIGN_MD)).unwrap().unwrap(),
        ))
        .unwrap();
        for (component, fields) in json["spacing"].as_object().unwrap() {
            if component == "scale" {
                continue;
            }
            for (field, value) in fields.as_object().unwrap() {
                let name = format!("{component}.{field}");
                let v = value.as_f64().unwrap() as f32;
                assert!(
                    exceptions.contains(&name.as_str()) || ramp.iter().any(|r| (r - v).abs() < f32::EPSILON),
                    "spacing.{name} = {v} is off the ramp"
                );
            }
        }
    }

    #[test]
    fn shares_one_content_edge() {
        let d = parse(DESIGN_MD).unwrap();
        assert!((d.content_inset() - (d.spacing.list.pad + d.spacing.row.pad_x)).abs() < f32::EPSILON);
        assert!((d.content_inset() - 20.).abs() < f32::EPSILON);
    }
```

Add beside `parse` the helper the test uses (and have `parse` use it):

```rust
/// The YAML between the leading `---` lines.
fn front_matter(markdown: &str) -> &str {
    let rest = markdown.strip_prefix("---\n").unwrap_or(markdown);
    rest.find("\n---\n").map_or(rest, |end| &rest[..end])
}
```

(`parse` keeps returning `DesignError::NoFrontMatter` when the markers are missing; only the slicing is shared.)

Run: `cargo nextest run -p kb-app keeps_every_spacing shares_one_content`
Expected: FAIL to compile (`no field scale`, `no method content_inset`).

- [ ] **Step 2: The ramp and the shared edge in code**

`design.rs`:

```rust
points!(/// The GPUI Kit Design Guides' spacing ramp (spec D1): every spacing value is one of these.
    Scale { xxs, xs, sm, md, lg, xl, xxl });
```

add `pub scale: Scale,` first in `Spacing`, and:

```rust
impl Design {
    /// The window's content edge (spec D2): the list's padding plus a row's, so the header, footer, panes and
    /// notice line up with the rows' content.
    #[must_use]
    pub fn content_inset(&self) -> f32 {
        self.spacing.list.pad + self.spacing.row.pad_x
    }
}
```

- [ ] **Step 3: The values** (DESIGN.md front matter, per the spec's mapping)

```yaml
spacing:
  scale: { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16, xl: 24, xxl: 32 }
  header: { gap: 12 }
  list: { pad: 8, gap: 4 }
  row: { pad_x: 12, gap: 12, text_gap: 2, trailing_gap: 12, tags_gap: 8 }
  pane: { split_pad_y: 16, detail_pad_y: 24, gap: 12, header_gap: 8 }
  footer: { gap: 16, item_gap: 8, left_gap: 8 }
  panel: { pad: 4, row_pad_x: 8, row_gap: 8, section_pad_top: 8, section_pad_x: 8, section_pad_bottom: 4, rule_margin: 4, search_pad_x: 12, search_gap: 8, crumb_pad_x: 12, crumb_gap: 4 }
  pill: { pad_x: 8, pad_y: 4, gap: 8 }
  chip: { pad_x: 8, link_pad_x: 8, link_gap: 4 }
  keycap: { gap: 2 }
  empty: { pad_x: 16, icon_gap: 12, title_gap: 4, bar_gap: 8, summary_pad_x: 12, summary_pad_top: 8, summary_pad_bottom: 4 }
  quote: { pad_left: 12, pad_y: 2 }
  card: { pad_x: 12, pad_y: 12, pad_right: 40, copy_inset: 8 }
  link_hero: { pad_x: 12, pad_y: 8, gap: 8 }
  links: { gap: 4, label_gap: 4, label_pad_top: 2, label_pad_bottom: 4, row_pad_x: 8, row_gap: 8 }
  notice: { gap: 12, hint_gap: 8, bottom: 48 }
```

and `components.footer: { height: 40, dot: 6, dot_glow: 6, dot_glow_alpha: 0.7 }` (no `label_slot`).

Matching structs in `design.rs`: `HeaderSpacing { gap }`, `RowSpacing { pad_x, gap, text_gap, trailing_gap,
tags_gap }`, `PaneSpacing { split_pad_y, detail_pad_y, gap, header_gap }`, `FooterSpacing { gap, item_gap,
left_gap }`, `NoticeSpacing { gap, hint_gap, bottom }`, `FooterSize { height, dot, dot_glow, dot_glow_alpha }`.

- [ ] **Step 4: The readers** (the compiler lists each)

- `ui/header.rs:19`: `.px(pt(d.content_inset()))`.
- `ui/row.rs`: drop the `compact` field, its `Debug` line, its default and the `compact` builder; use
  `spacing.pad_x` and `spacing.gap` directly. `entry_list.rs:293`: remove `.compact(self.compact)` (the list keeps
  its own `compact`, which hides tags in split view).
- `ui/footer.rs`: `footer(left, right)` without `entry`; `.px(pt(d.content_inset()))`; `.gap(pt(s.gap))`; in
  `key_item`, the label back to `.child(label.into())` (no slot). `search_view/footer.rs:142`: `footer(left, right)`;
  `names_entry` still chooses the left side.
- `ui/entry_view.rs:430-433` and `search_view.rs:658-664`: the horizontal pad is `d.content_inset()` in both layouts;
  the vertical pad stays `p.detail_pad_y` / `p.split_pad_y`.
- `ui/notice.rs:22-26`: `top`, `right` and `left` margins are `pixels(d.content_inset())`; `bottom` stays.

- [ ] **Step 5: Run the tests and clippy**

Run: `cargo nextest run -p kb-app && cargo clippy -p kb-app --all-targets -- -D warnings`
Expected: PASS. If a UI test asserted a removed token or a footer width, update it to the new value.

- [ ] **Step 6: Commit**

```bash
git add DESIGN.md crates/kb-app/src
git commit -m "feat(design): Put every spacing on the ramp with one content edge"
```

---

### Task 2: Prose, change map, gate

**Files:** `DESIGN.md` (body), `tools/design/CHANGE-MAP.md`

- [ ] **Step 1:** DESIGN.md's body: a short "Spacing" paragraph under section 2 or 3 stating the ramp, the spatial
      grammar (parts of one control `xs`, related `sm`, group `md`, groups `lg`, sections `xl`, regions `xxl`), the
      shared content edge (`content_inset()`, list 8 + row 12) and the two exceptions; in section 4 remove "each label in a
      fixed slot" from **Footer** and say the items are 16 apart, a label 8 from its keys.
- [ ] **Step 2:** CHANGE-MAP.md, the "A size or spacing" row: add "a spacing value must be a step of `spacing.scale`
      (`ui::design` test `keeps_every_spacing_on_the_ramp`); window insets come from `content_inset()`".
- [ ] **Step 3:** `mise run design:validate && hk fix --all && mise run gate` — all pass.
- [ ] **Step 4: Commit**

```bash
git add DESIGN.md tools/design/CHANGE-MAP.md
git commit -m "docs(design): Describe the spacing ramp and the shared edge"
```

- [ ] **Step 5:** Ask the owner to check the running app (`cargo run --release -p kb-app`): the filter pill, the
      rows' keys and the footer's keys end on one line; the detail screen's text starts under the back chevron.

## Spec coverage

| Spec               | Task                           |
| ------------------ | ------------------------------ |
| D1 ramp            | 1 (scale, test)                |
| D2 content inset   | 1 (`content_inset`, readers)   |
| D3 footer one bar  | 1 (footer, label slot removed) |
| D4 panel edge 12   | 1 (values)                     |
| D5 one row padding | 1 (row)                        |
| D6 exceptions      | 1 (test)                       |
| D7 test            | 1                              |
| Mapping table      | 1 (values)                     |
| Docs               | 2                              |
