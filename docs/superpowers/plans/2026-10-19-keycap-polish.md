# Keycap polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** every shortcut is drawn as one 20 × 20 cap per key, and nothing on screen moves when the selection, the
entry type or a count changes.

**Architecture:** `ui/keycap.rs` keeps its API (`caps` spells a shortcut as one string, `keycap(text, state)` draws
it), so no caller changes; inside, `keys` splits the string and `keycap` draws one cap per key. Rows, the footer and
the filter pill get fixed slots from new DESIGN.md tokens; the link count becomes an outlined pill; counts use
tabular digits; the list footer drops its ⇧⌘↵ item.

**Tech Stack:** Rust, GPUI Kit, DESIGN.md front matter tokens (`ui/design.rs`), mise tasks (`design:*`), cargo-nextest.

**Spec:** [2026-10-19-keycap-polish-design.md](../specs/2026-10-19-keycap-polish-design.md) (D1–D9).

**Expects** (verified at `74b860a`): `ui::keycap::{caps, keycap, keycap_width, ChipState}`; `ui::design::{design, pt,
no_ligatures, TextRoleExt}` and its `points!` structs; `ui::chip::{pill, link_chip, link_text}`;
`ui::footer::{footer, status, key_item, entry_status}`; `ui::header` (the filter pill, `pill_max_width`);
`EntryList::row_keycap` (`entry_list.rs:184`); `SearchView::screen_keys` (`search_view.rs:304`), used by both the
footer and `ladder`; the footer test accessor `footer_keys` (`search_view/footer.rs:51`); the UI test
`shows_the_status_line_and_the_screen_keys_in_the_footer` (`tests/ui.rs:705`).

**Conventions:** failing test first; focused runs with `cargo nextest run -p kb-app <filter>`; stage with
`git add <paths>`, then `git commit` (never `-a`, never paths on `git commit`); Conventional Commits, sentence-case
subject ≤ 72 characters, with the `Co-Authored-By` trailer your harness gives. Run `hk fix --all` before committing if
a hook complains about formatting. No literal sizes in views: every size comes from `design()`.

---

## File map

| File                                      | Change                                           |
| ----------------------------------------- | ------------------------------------------------ |
| `DESIGN.md`                               | new tokens (Task 1, 2); component notes (Task 8) |
| `crates/kb-app/src/ui/design.rs`          | token fields; `tabular_digits()`                 |
| `crates/kb-app/src/ui/keycap.rs`          | `keys`, `keycap_width`, one cap per key          |
| `crates/kb-app/src/entry_list.rs`         | the row's key slot; `link_chip` without a state  |
| `crates/kb-app/src/ui/chip.rs`            | the outlined link pill; tabular digits in pills  |
| `crates/kb-app/src/ui/footer.rs`          | label slots; tabular count                       |
| `crates/kb-app/src/search_view/footer.rs` | no ⇧⌘↵ item (D9)                                 |
| `crates/kb-app/src/ui/header.rs`          | the pill's minimum width                         |
| `crates/kb-app/src/sync_view.rs`          | tabular summary line                             |
| `crates/kb-app/tests/ui.rs`               | the footer's three keys                          |
| `tools/design/CHANGE-MAP.md`              | the keycap row                                   |

---

### Task 1: The new tokens

**Files:**

- Modify: `DESIGN.md` (front matter)
- Modify: `crates/kb-app/src/ui/design.rs`

- [ ] **Step 1: Write the failing test** (append to `design.rs`'s `mod tests`)

```rust
#[test]
fn reads_the_keycap_and_slot_tokens() {
    let d = parse(DESIGN_MD).unwrap();
    let near = |a: f32, b: f32| (a - b).abs() < f32::EPSILON;
    let k = &d.components.keycap;
    assert!(near(k.height, 20.) && near(k.width, 20.) && near(k.word_width, 28.));
    assert!(near(d.spacing.keycap.gap, 2.));
    let role = &d.typography.roles.keycap;
    assert_eq!(role.family, Family::Ui);
    assert!(near(role.size, 9.) && near(role.weight, 400.));
    assert!(near(d.components.row.key_slot, 42.));
    assert!(near(d.components.footer.label_slot, 56.));
    assert!(near(d.components.header.pill_min_width, 160.));
    assert!(near(d.components.chip.link_outline_alpha, 0.45));
}
```

- [ ] **Step 2: Run it to see it fail**

Run: `cargo nextest run -p kb-app reads_the_keycap_and_slot_tokens`
Expected: FAIL to compile (`no field width on KeycapSize`).

- [ ] **Step 3: Add the tokens to DESIGN.md**

In the front matter:

- `typography.roles.keycap: { family: ui, size: 9, weight: 400, line_height: 16 }`
- `spacing.keycap: { pad_x: 6, gap: 2 }` (`pad_x` goes in Task 2, with the code that reads it)
- `components.header: { height: 56, icon: 20, pill_icon: 15, pill_max_width: 220, pill_min_width: 160 }`
- `components.row: { height: 56, bar_width: 2.5, bar_inset: 6, glow: 8, glow_alpha: 0.6, max_tags: 3, key_slot: 42 }`
- `components.chip: { height: 20, icon: 14, link_icon: 12, link_outline_alpha: 0.45 }`
- `components.keycap: { height: 20, width: 20, word_width: 28 }`
- `components.footer: { height: 40, dot: 6, dot_glow: 6, dot_glow_alpha: 0.7, label_slot: 56 }`

- [ ] **Step 4: Add the fields and the tabular-digits helper**

In `design.rs`:

```rust
points!(/// A keycap's spacing: `gap` between the caps of one shortcut.
    KeycapSpacing { pad_x, gap });
```

```rust
points!(/// The header's sizes; `pill_min_width` keeps the filter pill's width when its label changes.
    HeaderSize { height, icon, pill_icon, pill_max_width, pill_min_width });
points!(/// A row's sizes; `key_slot` holds `↵` or `⌘ n` at one width.
    RowSize { height, bar_width, bar_inset, glow, glow_alpha, max_tags, key_slot });
points!(/// A chip's sizes; `link_outline_alpha` is the link pill's outline.
    ChipSize { height, icon, link_icon, link_outline_alpha });
points!(/// A keycap's sizes: one key's cap, and a word key's (`esc`).
    KeycapSize { height, width, word_width });
```

```rust
points!(/// The footer's sizes; `label_slot` is a key label's least width.
    FooterSize { height, dot, dot_glow, dot_glow_alpha, label_slot });
```

and after `no_ligatures`:

```rust
/// Tabular digits: a count keeps its width as its digits change (Inter's `tnum`).
#[must_use]
pub fn tabular_digits() -> FontFeatures {
    FontFeatures(Arc::new(vec![("tnum".into(), 1)]))
}
```

- [ ] **Step 5: Run the design tests**

Run: `cargo nextest run -p kb-app design`
Expected: PASS. (The keycap glyphs are already Inter 9 in the running app at this point; Task 2 makes the caps.)

- [ ] **Step 6: Commit**

```bash
git add DESIGN.md crates/kb-app/src/ui/design.rs
git commit -m "feat(design): Add the keycap, slot and link pill tokens"
```

---

### Task 2: One cap per key

**Files:**

- Modify: `crates/kb-app/src/ui/keycap.rs`
- Modify: `DESIGN.md` (`spacing.keycap`), `crates/kb-app/src/ui/design.rs` (`KeycapSpacing`)

- [ ] **Step 1: Write the failing tests** (replace `normalizes_keycap_width_by_visual_bucket` in `keycap.rs`)

```rust
    #[test]
    fn splits_a_shortcut_into_its_keys() {
        assert_eq!(keys("⌘↵"), ["⌘", "↵"]);
        assert_eq!(keys("⇧⌘,"), ["⇧", "⌘", ","]);
        assert_eq!(keys("⌥⌘C"), ["⌥", "⌘", "C"]);
        assert_eq!(keys("⌘1"), ["⌘", "1"]);
        assert_eq!(keys("↵"), ["↵"]);
        assert_eq!(keys("esc"), ["esc"]);
    }

    #[test]
    fn gives_every_key_the_same_cap() {
        let k = &design().components.keycap;
        let gap = design().spacing.keycap.gap;
        for one in ["↵", "⌘", "K", "3", ","] {
            assert!((keycap_width(one) - k.width).abs() < f32::EPSILON, "{one}");
        }
        for two in ["⌘↵", "⌘1", "⌘K", "⌘P"] {
            assert!((keycap_width(two) - (2. * k.width + gap)).abs() < f32::EPSILON, "{two}");
        }
        assert!((keycap_width("esc") - k.word_width).abs() < f32::EPSILON);
        assert!((keycap_width("⇧⌘↵") - (3. * k.width + 2. * gap)).abs() < f32::EPSILON);
    }
```

Run: `cargo nextest run -p kb-app keycap`
Expected: FAIL to compile (`keys` not found).

- [ ] **Step 2: Draw one cap per key**

In `keycap.rs`, update the module doc to
`//! D1's keycaps: one 20 × 20 cap per key (`⌘` `K`), 2 apart, Inter 9; a word key (`esc`) 28 wide.`, add
`use gpui_kit::component::h_flex;`, and replace `keycap_width` and `keycap` with:

```rust
/// The modifier glyphs `caps` puts before the key.
const MODIFIERS: [char; 4] = ['⌃', '⌥', '⇧', '⌘'];

/// A shortcut as `caps` spells it, split into its keys: each leading modifier is a key, the rest
/// is one key (`⇧⌘,` → `⇧` `⌘` `,`; `esc` → `esc`).
#[must_use]
pub fn keys(text: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut rest = String::new();
    for ch in text.chars() {
        if rest.is_empty() && MODIFIERS.contains(&ch) {
            keys.push(ch.to_string());
        } else {
            rest.push(ch);
        }
    }
    if !rest.is_empty() {
        keys.push(rest);
    }
    keys
}

/// One key's cap width: a glyph's, or a word's (`esc`).
fn cap_width(key: &str) -> f32 {
    let k = &design().components.keycap;
    if key.chars().count() > 1 { k.word_width } else { k.width }
}

/// The width `keycap(text)` draws: its caps and the gaps between them.
#[must_use]
pub fn keycap_width(text: &str) -> f32 {
    let keys = keys(text);
    #[expect(clippy::cast_precision_loss, reason = "a shortcut has at most five keys")]
    let gaps = keys.len().saturating_sub(1) as f32 * design().spacing.keycap.gap;
    keys.iter().map(|key| cap_width(key)).sum::<f32>() + gaps
}

/// A shortcut's keycaps: one cap per key, 2 apart. Live (mantle, muted text) or, on the popover
/// and notice surfaces, rest (badge, faint text).
#[must_use]
pub fn keycap(text: impl Into<SharedString>, state: ChipState) -> Div {
    let d = design();
    let text_color = match state {
        ChipState::Live => d.colors.muted,
        ChipState::Rest => d.colors.faint,
    };
    let text: SharedString = text.into();
    h_flex()
        .flex_none()
        .items_center()
        .gap(pt(d.spacing.keycap.gap))
        .children(keys(&text).into_iter().map(|key| {
            div()
                .flex_none()
                .h(pt(d.components.keycap.height))
                .w(pt(cap_width(&key)))
                .flex()
                .items_center()
                .justify_center()
                .rounded(pt(d.rounded.chip))
                .bg(state.fill())
                .text_color(text_color)
                .text_role(&d.typography.roles.keycap)
                .child(key)
        }))
}
```

If clippy reports the `expect` as unfulfilled (the lint is not enabled), drop the attribute.

- [ ] **Step 3: Drop the unused padding token**

`keycap` no longer reads `spacing.keycap.pad_x`: set `spacing.keycap: { gap: 2 }` in DESIGN.md and
`KeycapSpacing { gap }` in `design.rs`.

- [ ] **Step 4: Run the app's tests**

Run: `cargo nextest run -p kb-app`
Expected: PASS. The UI tests still read keys as strings (`"Copy ⌘↵"`): `caps` is unchanged.

- [ ] **Step 5: Commit**

```bash
git add crates/kb-app/src/ui/keycap.rs crates/kb-app/src/ui/design.rs DESIGN.md
git commit -m "feat(ui): Draw one 20 pt cap per key in every shortcut"
```

---

### Task 3: The row's key slot

**Files:**

- Modify: `crates/kb-app/src/entry_list.rs:276-278` and its `mod tests`

- [ ] **Step 1: Write the failing test** (in `entry_list.rs`'s `mod tests`)

```rust
#[test]
fn every_row_key_fits_the_row_slot() {
    let slot = crate::ui::design::design().components.row.key_slot;
    for ix in 0..12 {
        for selected in [true, false] {
            if let Some(key) = EntryList::row_keycap(ix, selected) {
                assert!(crate::ui::keycap::keycap_width(&key) <= slot, "{key} in row {ix}");
            }
        }
    }
}
```

Run: `cargo nextest run -p kb-app every_row_key_fits`
Expected: PASS already (42 fits `⌘ 9`); it guards the token. Go on.

- [ ] **Step 2: Put the row's key in a fixed slot**

Replace

```rust
if let Some(key) = Self::row_keycap(ix, selected) {
    trailing.push(keycap(key, ChipState::Live).into_any_element());
}
```

with

```rust
// A fixed, right-aligned slot: `↵` and `⌘ n` take the same room, so the chips beside them
// never move with the selection (spec D3). Rows past the ninth keep the empty slot.
trailing.push(
    div()
        .flex_none()
        .w(pt(design().components.row.key_slot))
        .flex()
        .justify_end()
        .children(Self::row_keycap(ix, selected).map(|key| keycap(key, ChipState::Live)))
        .into_any_element(),
);
```

Add `div` and `pt` to the imports if the file does not have them (`use gpui_kit::div;`,
`use crate::ui::design::pt;`).

- [ ] **Step 3: Run the tests**

Run: `cargo nextest run -p kb-app`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/kb-app/src/entry_list.rs
git commit -m "fix(ui): Hold each row's key in a fixed slot"
```

---

### Task 4: The outlined link pill

**Files:**

- Modify: `crates/kb-app/src/ui/chip.rs`
- Modify: `crates/kb-app/src/entry_list.rs:273`

- [ ] **Step 1: Replace the link chip**

In `chip.rs`, replace `link_chip` and delete `link_text` (and its two assertions in the tests, lines ~101-102):

```rust
/// The `↗ n` link count (spec D4): a round pill with a faint outline and no fill, the same on
/// every row, so it never reads as a key.
#[must_use]
pub fn link_chip(count: usize) -> Div {
    let d = design();
    let c = &d.components.chip;
    h_flex()
        .flex_none()
        .h(pt(c.height))
        .px(pt(d.spacing.chip.pad_x))
        .gap(pt(d.spacing.chip.link_gap))
        .items_center()
        .rounded_full()
        .border(pt(d.components.border.hairline))
        .border_color(d.colors.faint.opacity(c.link_outline_alpha))
        .text_color(d.colors.faint)
        .text_role(&d.typography.roles.count)
        .child(
            svg()
                .path("icons/arrow-up-right.svg")
                .size(pt(c.link_icon))
                .text_color(d.colors.faint),
        )
        .child(count.to_string())
}
```

`border(…)` is the same helper family as the footer's `border_t(pt(…))`; if it does not exist, use `.border_1()` (the
hairline token is 1). The count is JetBrains Mono (`count` role), whose digits are already tabular. Update the module
doc's list: "the `↗ n` link pill".

In `entry_list.rs:273`: `trailing.push(link_chip(entry.links.len()).into_any_element());`

- [ ] **Step 2: Build, test, lint**

Run: `cargo nextest run -p kb-app && cargo clippy -p kb-app --all-targets -- -D warnings`
Expected: PASS; no unused `Hsla` import left in `chip.rs` (remove it if clippy says so).

- [ ] **Step 3: Commit**

```bash
git add crates/kb-app/src/ui/chip.rs crates/kb-app/src/entry_list.rs
git commit -m "fix(ui): Outline the link count so it never reads as a key"
```

---

### Task 5: A still footer

**Files:**

- Modify: `crates/kb-app/src/search_view/footer.rs`
- Modify: `crates/kb-app/src/ui/footer.rs`
- Modify: `crates/kb-app/tests/ui.rs:705-711`

- [ ] **Step 1: Write the failing UI test** (replace the assertion at `tests/ui.rs:710` and add a command case)

```rust
    assert_eq!(keys, ["Details ↵", "Open ⌘↵", "Actions ⌘K"], "no ⇧⌘↵ in the footer (D9)");
}

#[gpui_kit::test]
fn shows_the_same_three_footer_keys_for_a_command(cx: &mut TestAppContext) {
    let (_home, _window, view) = open_with_query(cx, "rebase");
    let keys = cx.read(|cx| view.read(cx).footer_keys(cx));
    assert_eq!(keys.len(), 3, "{keys:?}");
    assert!(keys[0].starts_with("Details") && keys[2].starts_with("Actions"), "{keys:?}");
}
```

(The first block ends the existing `shows_the_status_line_and_the_screen_keys_in_the_footer`.)

Run: `cargo nextest run -p kb-app footer`
Expected: `shows_the_status_line…` FAILS (four keys); the command test passes.

- [ ] **Step 2: Leave ⇧⌘↵ out of the footer**

In `search_view/footer.rs`, add to the `impl SearchView` block:

```rust
/// The screen's keys the footer shows: all but ⇧⌘↵ (spec D9), which would come and go with
/// the entry type and move the others; it still runs, and the action panel lists it.
fn footer_items(&self, cx: &App) -> Vec<KeyItem> {
    self.screen_keys(cx)
        .into_iter()
        .filter(|item| item.keys != "shift-cmd-enter")
        .collect()
}
```

and use `self.footer_items(cx)` instead of `self.screen_keys(cx)` in `footer_keys` (line ~52) and in the render (line
~105). Import `crate::catalog::KeyItem` if the file does not. `ladder` keeps calling `screen_keys`, so ⇧⌘↵ still
runs.

- [ ] **Step 3: Fixed label slots and a tabular count**

In `ui/footer.rs`, import `tabular_digits` from `crate::ui::design`, and in `key_item` replace `.child(label.into())`
with:

```rust
// A least width, right-aligned: the caps stay put when the label changes (Open ↔ Copy, D5).
.child(
    div()
        .min_w(pt(d.components.footer.label_slot))
        .flex()
        .justify_end()
        .child(label.into()),
)
```

In `status`, add `.font_features(tabular_digits())` to the returned `h_flex()` (before `.when_some_count(count)`), so
the count's digits keep their width (D6).

- [ ] **Step 4: Run the tests**

Run: `cargo nextest run -p kb-app`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kb-app/src/search_view/footer.rs crates/kb-app/src/ui/footer.rs crates/kb-app/tests/ui.rs
git commit -m "fix(ui): Keep the footer's keys still as the selection moves"
```

---

### Task 6: The filter pill keeps its width

**Files:**

- Modify: `crates/kb-app/src/ui/header.rs:50-75`

- [ ] **Step 1: Give the pill a least width and its keys the right edge**

In the filter pill's `h_flex()` chain, add `.min_w(pt(d.components.header.pill_min_width))` after the padding calls,
and make the label take the free room so the caps stay at the pill's right edge:

```rust
.child(
    div()
        .flex_1()
        .max_w(pt(d.components.header.pill_max_width))
        .truncate()
        .child(label.into()),
)
```

- [ ] **Step 2: Run the tests**

Run: `cargo nextest run -p kb-app`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/kb-app/src/ui/header.rs
git commit -m "fix(ui): Keep the filter pill's width when its label changes"
```

---

### Task 7: Tabular counts in pills and the report

**Files:**

- Modify: `crates/kb-app/src/ui/chip.rs` (`pill`)
- Modify: `crates/kb-app/src/sync_view.rs` (the summary line)

- [ ] **Step 1: Add tabular digits**

- `chip.rs`, `pill`: add `.font_features(tabular_digits())` after `.text_role(&d.typography.roles.chip)` (pills carry
  the report's entry counts and `+n`; letters are unaffected).
- `sync_view.rs`, the summary `div()` in the `Done` arm: add `.font_features(tabular_digits())` after its
  `.text_role(…)`. Import `tabular_digits` from `crate::ui::design` in both files.

- [ ] **Step 2: Run the tests and clippy**

Run: `cargo nextest run -p kb-app && cargo clippy -p kb-app --all-targets -- -D warnings`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/kb-app/src/ui/chip.rs crates/kb-app/src/sync_view.rs
git commit -m "fix(ui): Use tabular digits for counts"
```

---

### Task 8: Design notes, shots and the gate

**Files:**

- Modify: `DESIGN.md` (sections 3 and 4)
- Modify: `tools/design/CHANGE-MAP.md`

- [ ] **Step 1: Update DESIGN.md's prose**

- Typography paragraph: move "keycaps" from the JetBrains Mono list to Inter ("Inter for the interface and keycaps").
- Section 4, **Keycap**: "(`keycap.rs`; spec 2026-10-19 D1): one 20 × 20 cap per key, 2 apart (`⌘` `K`, `⇧` `⌘`
  `,`); a word key (`esc`) 28 wide; Inter 9 / 400; live or rest."
- **Chip**: "… tag pills, tag icon chips, task chips; live or rest. The `↗ n` link count is a round pill with a faint
  outline and no fill, never a key's shape."
- **Row**: "… link pill, then a fixed key slot (`↵` selected, `⌘ 1`…`⌘ 9` the first nine rows, empty after)…"
- **Header**: "… the filter pill (sliders, the filter's name, `⌘ P`), never narrower than `pill_min_width`."
- **Footer**: "… right the screen's ↵ and ⌘↵ keys and _Actions ⌘K_ (never ⇧⌘↵, which the action panel lists), each
  label in a fixed slot; counts in tabular digits."

- [ ] **Step 2: Add the change-map row** (after "An action's keys")

```markdown
| A shortcut's look | `DESIGN.md` `components.keycap`, `spacing.keycap`, `typography.roles.keycap` | every keycap (`ui::keycap::keycap`): rows, footer, header, panel, links, notice | `keycap` tests (`keys`, `keycap_width`); `entry_list` slot test; the shots |
```

- [ ] **Step 3: Validate the tokens and regenerate the theme**

Run: `mise run design:validate && mise run design:theme`
Expected: `kb (DESIGN.md) pass`; `git status` shows no change to `crates/kb-app/assets/themes/kb.json` (the theme reads
colours only), or a regenerated file to commit.

- [ ] **Step 4: Shoot the screens and check the slot values**

Run: `cargo build --release && mise run design:shots` (needs Accessibility and Screen Recording for the terminal, and
⌥K free).

Open `tools/design/shots/` and check, adjusting a token in DESIGN.md and re-shooting if any fails:

1. `*-01-list.png`: every row's key column lines up; `↵` sits where `⌘ n` sits on the other rows.
2. The footer on a bookmark and on a command: three keys, caps in the same places (`label_slot`; raise it if
   "Configure" on the failed-refresh screen is wider than its slot and pushes its caps).
3. `*-07-filter.png` and the list: the filter pill the same width with "All Entries" and with a filter (raise
   `pill_min_width` if the "All Entries" state is wider than 160).
4. The action panel: caps right-aligned, three-key shortcuts readable.
5. The link pill: outlined, unfilled, on the selected row too.

- [ ] **Step 5: Run the gate**

Run: `hk fix --all && mise run gate`
Expected: all steps pass (nextest, clippy, release build, hk with ast-grep's `no-literal-design`, deny, docs).

- [ ] **Step 6: Commit**

```bash
git add DESIGN.md tools/design/CHANGE-MAP.md tools/design/shots crates/kb-app/assets/themes/kb.json
git commit -m "docs(design): Record one cap per key and the still footer"
```

(Leave `kb.json` out of `git add` if it did not change.)

- [ ] **Step 7: Check by hand**

On the packaged app (`mise run package`, then open `kb.app`): move the selection through a bookmark and a command
and watch the footer and the row chips; open the filter and pick a type; open ⌘K. If a non-Retina display is
available, look at 9 pt `⌘` and `⇧` there; report what you see (the owner chose 9 pt knowing this risk).

---

## Spec coverage

| Spec                                          | Task                                               |
| --------------------------------------------- | -------------------------------------------------- |
| D1 one cap per key, 20 × 20, word 28, 2 apart | 1, 2                                               |
| D2 Inter 9 / 400                              | 1                                                  |
| D3 row slot                                   | 1, 3                                               |
| D4 outlined link pill                         | 1, 4                                               |
| D5 footer label slots                         | 1, 5                                               |
| D6 tabular digits                             | 1, 5, 7                                            |
| D7 filter pill least width                    | 1, 6                                               |
| D8 tokens only                                | 1–7 (no literals); 8 (ast-grep in the gate)        |
| D9 no ⇧⌘↵ in the footer                       | 5                                                  |
| Tests and validation                          | 1–5 (unit, UI); 8 (validate, shots, gate, by hand) |
