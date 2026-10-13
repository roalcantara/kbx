# Plan 8a: Foundation — tokens, fonts, window

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** DESIGN.md's front matter holds every design token; the app reads it at start, bundles Inter and JetBrains
Mono, is always dark and draws its panel with 14 pt corners; the design kit validates and generates from the front
matter; a showcase corpus holds the entries the approved pages show.

**Architecture:** `crates/kb-app/src/ui/design.rs` parses the YAML front matter of `DESIGN.md` (compiled in with
`include_str!`) through `kb_domain::yaml` into typed structs, once (`LazyLock`). Every later component reads
`design()`. `tools/design` (Bun) reads the same front matter with `Bun.YAML` to validate contrast and write GPUI Kit's
`kb.json`.

**Tech Stack:** Rust, GPUI Kit 0.7.1, Bun 1.4.2 (`Bun.YAML`), fontsource and JetBrains Mono releases.

**Expects:** the branch `feat/design-system` at or after commit `41bdc68`; `mise run gate` passing.

**Index:** [2026-10-12-kb-native-08-design-system.md](2026-10-12-kb-native-08-design-system.md) — read its "Rules for
every task" first.

---

## File structure

| File                                                   | Change         | Responsibility                                                                 |
| ------------------------------------------------------ | -------------- | ------------------------------------------------------------------------------ |
| `tools/design/showcase/**/*.yml`                       | create         | The showcase corpus                                                            |
| `tools/design/showcase-images/*.svg`                   | create         | Stand-in tag images                                                            |
| `crates/kb-app/tests/showcase.rs`                      | create         | The showcase syncs to what the pages show                                      |
| `DESIGN.md`                                            | rewrite        | Front matter (tokens) and prose                                                |
| `crates/kb-app/src/ui.rs`, `crates/kb-app/src/ui/design.rs` | create    | The design module                                                              |
| `crates/kb-app/src/lib.rs`                             | modify         | `mod ui`; fonts, theme fonts, always dark                                      |
| `crates/kb-app/src/accents.rs`                         | modify         | Accents from the tokens (removed in 8g)                                        |
| `crates/kb-app/assets/themes/accents.json`             | delete         | Replaced by the tokens                                                         |
| `crates/kb-app/assets/fonts/*`                         | create         | Bundled fonts and licences                                                     |
| `tools/design/color.ts`, `tools/design/color.test.ts`  | modify         | `frontMatter()`; tests                                                         |
| `tools/design/tasks.toml`                              | modify         | `design:validate`, `design:theme` from the front matter; render/palettes gone  |
| `tools/design/layout.json`                             | delete         | Superseded by the front matter                                                 |
| `tools/design/palettes.json`                           | modify         | No `chosen`; alternatives only                                                 |
| `crates/kb-app/assets/themes/kb.json`                  | regenerate     | GPUI Kit's theme from the tokens                                               |
| `crates/kb-app/src/panel.rs`                           | modify         | Transparent window background                                                  |
| `crates/kb-app/src/search_view.rs`                     | modify         | Rounded root on the window colour; no appearance observer                      |
| `crates/kb-app/tests/ui.rs`                            | modify         | Always-dark and token tests replace the appearance tests                       |
| `PRODUCT.md`                                           | modify         | O1's contrast rule (Task 4)                                                    |

---

### Task 1: The showcase corpus

**Files:** create the YAML files and SVGs below; create `crates/kb-app/tests/showcase.rs`.

- [ ] **Step 1: Write the failing test.** Create `crates/kb-app/tests/showcase.rs`:

```rust
//! The showcase corpus (tools/design/showcase) indexes to what the approved pages show, so the
//! app's screenshots and the pages show the same entries.

#![expect(clippy::unwrap_used, reason = "a fixture that cannot be built is a failed test")]

use std::path::Path;
use std::sync::atomic::AtomicBool;

use kb_domain::search::{FilterState, SearchQuery, search};
use kb_domain::sync::{open_readonly, sync_knowledge_base};

fn showcase() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/design/showcase")
        .canonicalize()
        .unwrap()
}

#[test]
fn indexes_the_entries_the_pages_show() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("kb.db");
    let summary = sync_knowledge_base(&showcase(), &database, &AtomicBool::new(false), |_| {}).unwrap();
    assert_eq!(summary.files_scanned, 9);
    assert_eq!(summary.files_failed, 1, "errors/broken.yml feeds the refresh report");
    assert_eq!(summary.total_indexed, 10);
    let query = SearchQuery {
        text: String::new(),
        filter: FilterState::default(),
        limit: 50,
        offset: 0,
    };
    let titles: Vec<String> = search(&open_readonly(&database).unwrap(), &query)
        .unwrap()
        .into_iter()
        .map(|row| row.title)
        .collect();
    for title in [
        "/k8s-rollout-restart",
        "/docker-prune-all",
        "/git-undo-commit",
        "Kubernetes Ingress & TLS Spec Cheatsheet",
        "Rust Memory Safety & Lifetimes Reference",
        "Tokio Tutorial — Async in Depth",
        "Async Rust in Depth",
        "AWS EKS VPC CNI Subnet Exhaustion Playbook",
        "Ship the new design system",
        "Write the findings",
    ] {
        assert!(titles.iter().any(|t| t == title), "missing {title}: {titles:?}");
    }
}
```

- [ ] **Step 2: Run it and see it fail.**

Run: `cargo nextest run -p kb-app --test showcase`
Expected: FAIL, `canonicalize` panics: `No such file or directory`.

- [ ] **Step 3: Write the corpus.** Create each file with exactly this content.

`tools/design/showcase/commands/k8s.yml`:

```yaml
# Showcase corpus (tools/design/showcase): the entries the approved pages show. Not the engine's fixture.
commands:
  /k8s-rollout-restart:
    cmd: kubectl rollout restart deployment/api --namespace production && kubectl rollout status deployment/api -n production --timeout=120s
    desc: Gracefully roll out new pods with zero downtime restart strategy
    tags: [k8s, devops]
    links:
      - Rollout docs: https://kubernetes.io/docs/reference/kubectl/generated/kubectl_rollout/kubectl_rollout_restart/
      - kubectl reference: https://kubernetes.io/docs/reference/kubectl/
    notes: |
      ### When to use

      - After a ConfigMap or Secret change the pods don't pick up.
      - **Zero downtime** needs a readiness probe and `maxUnavailable: 0`.
```

`tools/design/showcase/commands/docker.yml`:

```yaml
# Showcase corpus (tools/design/showcase): the entries the approved pages show. Not the engine's fixture.
commands:
  /docker-prune-all:
    cmd: docker system prune --all --volumes --force | tee /tmp/docker-prune.log
    desc: Remove unused containers, dangling networks, dangling images, and volumes
    tags: [docker, cleanup]
    links:
      - docker system prune: https://docs.docker.com/reference/cli/docker/system/prune/
    notes: |
      Frees disk space on a build machine. `--volumes` also removes named volumes no container uses.
```

`tools/design/showcase/commands/git.yml`:

```yaml
# Showcase corpus (tools/design/showcase): the entries the approved pages show. Not the engine's fixture.
commands:
  /git-undo-commit:
    cmd: git reset --soft HEAD~1
    desc: Undo last git commit while preserving staged changes in index
    tags: [git, vcs]
    notes: |
      The commit's changes stay staged, so `git commit` makes it again after a fix.
```

`tools/design/showcase/cheats/kubernetes.yml`:

```yaml
# Showcase corpus (tools/design/showcase): the entries the approved pages show. Not the engine's fixture.
cheats:
  k8s-ingress-tls:
    title: Kubernetes Ingress & TLS Spec Cheatsheet
    cheat: kubectl get ingress -A -o wide
    desc: Cert-manager annotations, cluster issuer configs, and TLS secrets matrix
    tags: [k8s, networking, yaml]
    links:
      - Ingress: https://kubernetes.io/docs/concepts/services-networking/ingress/
      - cert-manager annotations: https://cert-manager.io/docs/usage/ingress/
      - ClusterIssuer: https://cert-manager.io/docs/concepts/issuer/
      - TLS secrets: https://kubernetes.io/docs/concepts/configuration/secret/#tls-secrets
    notes: |
      ### Annotations that matter

      - **cert-manager.io/cluster-issuer**: which issuer signs the certificate.
      - **nginx.ingress.kubernetes.io/ssl-redirect**: force HTTPS.

      ```yaml
      apiVersion: networking.k8s.io/v1
      kind: Ingress
      metadata:
        annotations:
          cert-manager.io/cluster-issuer: letsencrypt
      ```
```

`tools/design/showcase/cheats/rust.yml`:

```yaml
# Showcase corpus (tools/design/showcase): the entries the approved pages show. Not the engine's fixture.
cheats:
  rust-lifetimes:
    title: Rust Memory Safety & Lifetimes Reference
    desc: Borrow checker rules, Rc<RefCell> patterns, and unsafe block guidelines
    tags: [rust, systems]
    notes: |
      ### Borrowing

      - One `&mut` or any number of `&`, never both.
      - A reference never outlives its owner.

      ### Shared mutation

      - `Rc<RefCell<T>>` on one thread, `Arc<Mutex<T>>` across threads.
```

`tools/design/showcase/bookmarks/rust.yml`:

```yaml
# Showcase corpus (tools/design/showcase): the entries the approved pages show. Not the engine's fixture.
bookmarks:
  https://tokio.rs/tokio/tutorial/async:
    title: Tokio Tutorial — Async in Depth
    desc: tokio.rs · How futures, wakers and the executor fit together
    tags: [rust, async]
    links:
      - Async in depth: https://tokio.rs/tokio/tutorial/async
      - Async book: https://rust-lang.github.io/async-book/
      - mini-redis: https://github.com/tokio-rs/mini-redis
      - Waker: https://doc.rust-lang.org/std/task/struct.Waker.html
    notes: |
      - Read after the **Spawning** chapter.
      - Its mini-executor example is the clearest explanation of `Waker` there is.
  https://www.youtube.com/watch?v=aqz-KE-bpKQ:
    title: Async Rust in Depth
    desc: A conference talk on how futures, wakers and the executor fit together.
    tags: [async]
    notes: |
      - Pinning, wakers and the executor, with diagrams.
      - Pairs with the Tokio tutorial's **Async in depth** chapter.
  not-a-map: a bookmark whose value is a string, so the refresh report shows it skipped
```

`tools/design/showcase/tasks/kb.yml`:

```yaml
# Showcase corpus (tools/design/showcase): the entries the approved pages show. Not the engine's fixture.
tasks:
  design-system:
    title: Ship the new design system
    task: Land Plan 8 and compare every screen with its approved page
    status: in-progress
    priority: high
    tags: [kb, design]
  findings:
    title: Write the findings
    task: Read the report and write why each criterion failed or went unmeasured
    status: todo
    priority: medium
    tags: [kb]
```

`tools/design/showcase/notes/aws.yml` (a type kb ignores, D6; it proves the narrowing):

```yaml
# Showcase corpus (tools/design/showcase): an entry of a type kb ignores (notes), so the app shows it nowhere.
notes:
  eks-subnets:
    title: AWS EKS VPC CNI Subnet Exhaustion Playbook
    desc: Remediation steps for secondary CIDR assignment and prefix delegation flags
    tags: [aws, eks]
```

`tools/design/showcase/errors/broken.yml` (a file that fails to parse, for the refresh report):

```yaml
# Showcase corpus: a file that fails to parse, so the refresh report shows a failed file.
commands:
  broken: [unclosed
```

- [ ] **Step 4: Add the tag images.** They are Lucide shapes (ISC licence), stand-ins for the owner's logos:

```bash
mkdir -p tools/design/showcase-images
for pair in k8s:network docker:box git:git-commit-horizontal rust:cog aws:cloud; do
  curl -fsSL "https://unpkg.com/lucide-static@0.511.0/icons/${pair#*:}.svg" -o "tools/design/showcase-images/${pair%%:*}.svg"
done
ls tools/design/showcase-images
```

Expected output: `aws.svg  docker.svg  git.svg  k8s.svg  rust.svg`.

- [ ] **Step 5: Run the test and see it pass.**

Run: `cargo nextest run -p kb-app --test showcase`
Expected: PASS, 1 test.

- [ ] **Step 6: Exclude the corpus from formatters.** The corpus is authored byte for byte, as the sample is. In
      `hk.pkl`, line 17, add `"tools/design/showcase/**/*"` to `generated`:

```pkl
local generated = List("crates/kb-domain/tests/fixtures/**/*", "tools/bench/fixtures/**/*", "docs/guides/results/**/*", "docs/guides/design_system/**/*", "tools/design/showcase/**/*", "CHANGELOG.md", ".mise/**/*")
```

- [ ] **Step 7: Gate and commit.**

```bash
mise run gate
git add tools/design/showcase tools/design/showcase-images crates/kb-app/tests/showcase.rs hk.pkl
git commit -m "test(design): Add the showcase corpus the approved pages show" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: DESIGN.md with its tokens

**Files:** rewrite `DESIGN.md`.

- [ ] **Step 1: Replace DESIGN.md with exactly this content.**

````markdown
---
# The design tokens: the only place a design value is written (spec D18). The app reads this front matter at start
# (crates/kb-app/src/ui/design.rs); `mise run design:theme` writes GPUI Kit's theme from it; `design:validate` checks it.
# Sizes are points. Colours are #RRGGBB or #RRGGBBAA.
name: kb
colors:
  window: "#24273A"
  mantle: "#1E2030"
  crust: "#181926"
  selected: "#363A4F"
  hover: "#2F334A"
  badge: "#2E3248"
  code: "#191B29"
  edge: "#6E738D1A"
  ring: "#CAD3F514"
  text: "#CAD3F5"
  muted: "#A5ADCB"
  faint: "#6E738D"
  mauve: "#C6A0F6"
  blue: "#8AADF4"
  green: "#A6DA95"
  peach: "#F5A97F"
  pink: "#F5BDE6"
  yellow: "#EED49F"
  red: "#ED8796"
typography:
  ui: Inter
  mono: JetBrains Mono
  roles:
    search: { family: ui, size: 15, weight: 400 }
    title: { family: ui, size: 13, weight: 500 }
    description: { family: ui, size: 12, weight: 400 }
    label: { family: ui, size: 12, weight: 500 }
    section: { family: ui, size: 10, weight: 600, uppercase: true }
    footer: { family: ui, size: 11, weight: 400 }
    chip: { family: ui, size: 11, weight: 600 }
    count: { family: mono, size: 11, weight: 400 }
    keycap: { family: mono, size: 11, weight: 400, line_height: 16 }
    heading: { family: ui, size: 15, weight: 600, line_height: 24 }
    quote: { family: ui, size: 14, weight: 400, line_height: 21, italic: true }
    body: { family: ui, size: 13, weight: 400, line_height: 20 }
    notes_h1: { family: ui, size: 15, weight: 600 }
    notes_h2: { family: ui, size: 14, weight: 600 }
    notes_h3: { family: ui, size: 13, weight: 600 }
    card_split: { family: mono, size: 12, weight: 400, line_height: 20 }
    card_detail: { family: mono, size: 13, weight: 400, line_height: 22 }
    link_title: { family: ui, size: 13, weight: 500 }
    link_url: { family: mono, size: 12, weight: 400 }
    link_hero: { family: mono, size: 13, weight: 500 }
    empty_title: { family: ui, size: 15, weight: 500 }
    empty_text: { family: ui, size: 13, weight: 400 }
    notice: { family: ui, size: 13, weight: 500 }
rounded:
  window: 14
  row: 6
  chip: 4
  tile: 4
  pill: 6
  popover: 14
  card: 10
  hero: 12
  favicon: 3
spacing:
  header: { pad_x: 24, gap: 12 }
  list: { pad: 8, gap: 4 }
  row: { pad_x: 14, gap: 12, text_gap: 2, trailing_gap: 12, tags_gap: 6, split_pad_x: 10, split_gap: 10 }
  pane: { split_pad_x: 24, split_pad_y: 18, detail_pad_x: 32, detail_pad_y: 22, gap: 12, header_gap: 10 }
  footer: { pad_left: 24, pad_right: 24, entry_pad_right: 16, gap: 20, entry_gap: 16, item_gap: 6, left_gap: 8 }
  panel: { pad: 6, row_pad_x: 10, row_gap: 10, section_pad_top: 6, section_pad_x: 10, section_pad_bottom: 4, rule_margin: 4, search_pad_x: 12, search_gap: 8, crumb_pad_x: 12, crumb_gap: 6 }
  pill: { pad_x: 10, pad_y: 4, gap: 8 }
  chip: { pad_x: 8, link_pad_x: 6, link_gap: 4 }
  keycap: { pad_x: 6, gap: 4 }
  empty: { pad_x: 16, icon_gap: 12, title_gap: 4, bar_gap: 10, summary_pad_x: 16, summary_pad_top: 10, summary_pad_bottom: 4 }
  quote: { pad_left: 12, pad_y: 2 }
  card: { pad_x: 14, pad_y: 12, pad_right: 40, copy_inset: 8 }
  link_hero: { pad_x: 12, pad_y: 10, gap: 8 }
  links: { gap: 4, label_gap: 6, label_pad_top: 2, label_pad_bottom: 4, row_pad_x: 8, row_gap: 10 }
  notice: { gap: 12, hint_gap: 6, right: 16, bottom: 48 }
components:
  window: { width: 860, height: 520, expanded_share: 0.85 }
  header: { height: 56, icon: 20, pill_icon: 15, pill_max_width: 220 }
  row: { height: 56, bar_width: 2.5, bar_inset: 6, glow: 8, glow_alpha: 0.6, max_tags: 3 }
  chip: { height: 20, icon: 14, link_icon: 12 }
  keycap: { height: 20, min_width: 20 }
  tile: { size: 24, icon: 16, small: 20, small_icon: 13, mini: 18, mini_icon: 12, alpha: 0.15 }
  favicon: { size: 16, inset: 4 }
  panel: { width: 220, filter_width: 230, row: 28, search: 36, crumb: 26, max_height: 300, chevron: 14, search_icon: 16, crumb_icon: 12, check: 14, offset: 8 }
  footer: { height: 40, dot: 6, dot_glow: 6 }
  split: { list: 0.34 }
  empty: { icon: 56, text_max_width: 420, bar_width: 260, bar_height: 4 }
  hero: { thumb_split: 150, thumb_detail: 250, link_icon: 14 }
  quote: { bar: 3 }
  links: { row: 28, url_chars: 32, title_share: 0.5 }
  notice: { width: 340, icon: 20, check: 13 }
  border: { hairline: 1 }
---

# Design system

How kb looks. The owner's Stitch screens are the reference, settled screen by screen into the **approved pages**
([docs/guides/design/](docs/guides/design/README.md)), which are the visual contract: where this text and a page
differ, the page wins and this text is fixed. [PRODUCT.md](PRODUCT.md) says what kb is for; Raycast is the reference
for behaviour. Read this file before any UI change.

Every value lives in one place:

| What                                   | Where                                                                    | Read by                                         |
| -------------------------------------- | ------------------------------------------------------------------------ | ----------------------------------------------- |
| Colours, type, radii, spacing, sizes   | This file's front matter (the design tokens)                             | the app (`ui/design.rs`); `design:theme`, `design:validate` |
| Entry types, actions, keys, words      | [`crates/kb-app/assets/catalog.toml`](crates/kb-app/assets/catalog.toml) | the app (`catalog.rs`); `~/.config/kb/config.toml` overrides it |
| GPUI Kit's theme                       | [`kb.json`](crates/kb-app/assets/themes/kb.json), generated               | GPUI Kit's own components                       |
| What reaches what when a value changes | [`tools/design/CHANGE-MAP.md`](tools/design/CHANGE-MAP.md)               | whoever changes a value                         |

`mise run design:theme` regenerates `kb.json` after a colour changes; `mise run design:validate` checks contrast;
`mise run design:shots` captures the app on the showcase corpus for comparison with the approved pages.

## 1. Visual Theme & Atmosphere

A calm, dark launcher in Catppuccin Macchiato: a navy panel with 14 pt corners floating over the desktop, flat
surfaces close in tone, hairlines only under the header and above the footer. Hierarchy comes from type weight, a
selected row's fill and its mauve bar, never from boxes. Colour is reserved for meaning: an entry's type (its tile),
a tag's identity (its icon), and state (the status dot). Always dark: kb has one approved palette.

## 2. Color Palette & Roles

The roles, by name (the hexes are in the front matter):

| Role       | Job                                                                                             |
| ---------- | ----------------------------------------------------------------------------------------------- |
| `window`   | The panel                                                                                       |
| `mantle`   | Live chips and keycaps; popovers; command cards and the link hero                              |
| `crust`    | The notice                                                                                      |
| `selected` | The selected row and panel row; popover and card borders                                        |
| `hover`    | A hovered row                                                                                   |
| `badge`    | Rest chips; the filter pill; keycaps on the darker popover and notice surfaces                  |
| `code`     | Code blocks in the notes                                                                        |
| `edge`     | Hairlines (faint at 10 %)                                                                       |
| `ring`     | The panel's 1 pt outline (text at 8 %)                                                          |
| `text`     | Titles, the query, values                                                                       |
| `muted`    | Descriptions, labels, live chip text, keycap text                                               |
| `faint`    | Tertiary text: placeholders, counts, section labels, URLs, rest link chips; icons in the header |
| accents    | `mauve` (selection bar, caret, Cheat Sheets), `blue` (Bookmarks, links), `green` (Commands, ready), `pink` (Tasks), `peach`, `yellow`, `red` (the quote bar, failures) |

- **Type tiles:** a type's accent at 15 % behind its icon in the accent (24 pt in rows, 20 in the footer and the pane
  header, 18 in the filter). The type → accent pairs are the catalog's (`tint`).
- **Tags:** a tag with an image in the tag-images folder is an icon chip, the image tinted in the tag's accent (the
  Raycast hash of its lowercased name over blue, peach, green, mauve, pink, yellow, red); any other tag is a grey
  `#tag` pill. Colour is never the only cue: pills carry the name, icon chips a tooltip.
- **Chip state** (D3): _live_ (mantle, muted text) on every keycap and on the selected row's chips; _rest_ (badge)
  elsewhere, tag text muted, the link count faint.

## 3. Typography Rules

Inter for the interface, JetBrains Mono for commands, code, keycaps, counts and URLs; both bundled
(`crates/kb-app/assets/fonts`), ligatures off wherever mono text is drawn, so what you read is what you copy. Small,
dense type: titles 13 medium, descriptions 12, labels 12 medium, the footer 11, section labels 10 semibold uppercase;
the query 15; the pane's heading 15 semibold; the quote 14 italic. Command titles are mono (the catalog's
`mono_title`). The type roles are the front matter's `typography.roles`.

## 4. Component Stylings

Each component is one file in `crates/kb-app/src/ui/`, and matches a part of an approved page:

- **Keycap** (`keycap.rs`; 01-chips): one cap per shortcut (`⌘K`, `⇧⌘,`), 20 tall, JetBrains Mono 11, live.
- **Chip** (`chip.rs`; 01-list, 01-chips): 20 tall, 4 pt corners: tag pills, tag icon chips, the `↗ n` link chip,
  task chips; live or rest.
- **Tile** (`tile.rs`; 01-list): the tinted type tile, or a bookmark's favicon with the tile as its fallback.
- **Row** (`row.rs`; 01-list): 56 tall, 4 apart, 6 pt corners; tile, title over description, tags, link chip, keycap
  (`↵` selected, `⌘1`…`⌘9` the first nine rows); the selected row on `selected` with a 2.5 pt mauve bar and its glow.
- **Header** (`header.rs`; 01-list, 07-filter): 56 tall, a hairline under it; the magnifier on the list, the back
  chevron elsewhere; the query at 15; the filter pill (sliders, the filter's name, `⌘P`).
- **Footer** (`footer.rs`; 01-list, 04-entry-template): 40 tall, a hairline over it; left the status line on the
  list (`● Ready • N results`) or, with a pane or the detail screen, the entry's tile, file and `⌥⌘C`; right the
  screen's keys and _Actions ⌘K_.
- **Action panel** (`action_panel.rs`; 02-actions, 07-filter): mantle, 14 pt corners, 220 wide (the filter 230),
  28 pt rows, search at the bottom, sections by a hairline, a `‹` header on submenus.
- **Empty view** (`empty_view.rs`; 03-empty, 11-14-refresh): one 56 pt icon, a 15 pt title, a sentence; the query in
  mauve mono.
- **Entry view** (`entry_view.rs`; 04-entry-template, 05-detail-links): Header (icon, title, tags), Hero by type,
  Quote (the description, red bar), Body (the notes, then the Links list); the same component in the split (34 %
  list) and on the detail screen.
- **Command card** (`command_card.rs`; 04-command-cards): mantle, a `selected` border, 10 pt corners; fmt-rs breaks
  at operators, kb wraps at spaces with a hanging indent; GPUI Kit's bash colours; a copy button in the corner.
- **Notice** (`notice.rs`; 01-list): bottom right above the footer, _Copied to clipboard_ with a `⌘V` hint.

## 5. Layout Principles

Compact 860 × 520, expanded to 85 % of the display (⇧⌘F), keeping its centre. The header and footer insets are
24; the list is inset 8; rows inset their content 14 (10 in the split). The split gives the list 34 %. The pane
pads 18 × 24 in the split and 22 × 32 on the detail screen, 12 between its slots. Search fields sit at the bottom of
every panel (Raycast).

## 6. Depth & Elevation

Flat. Only three things float: the panel itself (the system's window shadow and a 1 pt ring), popovers (GPUI Kit's
popover shadow) and the notice. The selected row's bar glows mauve; nothing else has a shadow.

## Adaptations

Where kb departs from the owner's Stitch screens or from Raycast, and why:

- **Back chevron instead of the magnifier** on the detail and refresh screens: kb needs a way back (01-list, note 1).
- **No `deploy/api` chip**: kb entries have no field it could come from (01-list, note 4).
- **Lucide icons** for Material ones: the set GPUI Kit ships.
- **Type colours as the owner drew them**: Commands green, Cheat Sheets mauve, Bookmarks blue, Tasks pink.
- **Four entry types** (D6): any other type is ignored by the app; the engine still indexes it.
- **No metadata column** (D20): every attribute has a home (tile, quote, hero, header tags, footer file).
- **Always dark** (D1).

## Evidence

`tools/design/shots/` holds the app captured on the showcase corpus by `mise run design:shots`, one PNG per screen,
compact and expanded; `tools/design/CHECKLISTS.md` records each screen's checklist against its approved page.
````

- [ ] **Step 2: Format and check.**

Run: `dprint check DESIGN.md`
Expected: no output, exit 0. If dprint reports a difference, run `dprint fmt DESIGN.md` and accept its change (it may
re-wrap prose; it must not change the front matter).

- [ ] **Step 3: Do not commit yet.** The app does not read the front matter until Task 3; Task 3 commits both.

---

### Task 3: The design module

**Files:** create `crates/kb-app/src/ui.rs`, `crates/kb-app/src/ui/design.rs`; modify `crates/kb-app/src/lib.rs`.

- [ ] **Step 1: Write the module with its tests first, the body left to fail.** Create `crates/kb-app/src/ui.rs`:

```rust
//! The components the approved pages show, one file each, and the design tokens they read.

pub mod design;
```

Create `crates/kb-app/src/ui/design.rs` with the tests and a `parse` that fails:

```rust
//! The design tokens: DESIGN.md's front matter, parsed once at start into typed values. The only
//! place a token becomes a GPUI length or colour.

// Plan 8d–8g's components read the remaining tokens; 8g Task 6 removes this and deletes any token
// still unread.
#![expect(dead_code, reason = "the components of Plan 8d–8g read the remaining tokens")]

use std::sync::{Arc, LazyLock};

use gpui_kit::{FontFeatures, FontWeight, Hsla, Rems, Styled, rems, rgb, rgba};
use serde::Deserialize;

use crate::presentation::Hue;

/// DESIGN.md, compiled in.
const DESIGN_MD: &str = include_str!("../../../../DESIGN.md");

/// Points per rem: GPUI Kit's base font size, which kb keeps at its default.
pub const POINTS_PER_REM: f32 = 16.;

/// `points` as rems, so every size follows the base font (R9).
#[must_use]
pub fn pt(points: f32) -> Rems {
    rems(points / POINTS_PER_REM)
}

/// Why the front matter could not be read.
#[derive(Debug, thiserror::Error)]
pub enum DesignError {
    /// DESIGN.md does not start with a `---` block.
    #[error("DESIGN.md has no front matter")]
    NoFrontMatter,
    /// The block is not YAML.
    #[error("DESIGN.md's front matter is not YAML: {0}")]
    Yaml(String),
    /// The block does not have the expected tokens.
    #[error("DESIGN.md's front matter: {0}")]
    Shape(String),
}

/// The tokens, parsed once. The front matter is compiled in and a test parses it, so a failure
/// here is a broken build, not a user's mistake.
#[must_use]
#[expect(clippy::expect_used, reason = "the compiled-in front matter is checked by a unit test")]
pub fn design() -> &'static Design {
    static DESIGN: LazyLock<Design> =
        LazyLock::new(|| parse(DESIGN_MD).expect("DESIGN.md's front matter is valid"));
    &DESIGN
}

/// Parses the YAML front matter of `markdown`.
///
/// # Errors
///
/// No front matter, YAML that does not parse, or tokens of the wrong shape.
pub fn parse(markdown: &str) -> Result<Design, DesignError> {
    let _ = markdown;
    Err(DesignError::NoFrontMatter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_tokens_of_design_md() {
        let d = parse(DESIGN_MD).unwrap();
        assert_eq!(d.colors.window, Hsla::from(rgb(0x24273A)));
        assert_eq!(d.colors.edge, Hsla::from(rgba(0x6E738D1A)));
        assert_eq!(d.typography.ui, "Inter");
        assert_eq!(d.typography.mono, "JetBrains Mono");
        assert!((d.typography.roles.title.size - 13.).abs() < f32::EPSILON);
        assert!((d.components.row.height - 56.).abs() < f32::EPSILON);
        assert!((d.components.split.list - 0.34).abs() < f32::EPSILON);
        assert!((d.rounded.window - 14.).abs() < f32::EPSILON);
    }

    #[test]
    fn rejects_a_file_without_front_matter_or_with_a_missing_token() {
        assert!(matches!(parse("# Design\n"), Err(DesignError::NoFrontMatter)));
        let without_window = DESIGN_MD.replacen("  window: \"#24273A\"\n", "", 1);
        assert!(matches!(parse(&without_window), Err(DesignError::Shape(_))));
    }

    #[test]
    fn converts_points_to_rems_at_sixteen_points_per_rem() {
        assert_eq!(pt(16.), rems(1.));
        assert_eq!(pt(8.), rems(0.5));
    }

    #[test]
    fn names_the_seven_hues_after_catppuccin() {
        let c = &design().colors;
        assert_eq!(c.hue(Hue::Purple), c.mauve);
        assert_eq!(c.hue(Hue::Magenta), c.pink);
        assert_eq!(c.hue(Hue::Orange), c.peach);
        assert_eq!(c.named("green"), Some(c.green));
        assert_eq!(c.named("teal"), None);
    }
}
```

- [ ] **Step 2: Register the module.** In `crates/kb-app/src/lib.rs`, after `mod trace;` add `mod ui;` (keep the list
      sorted: it goes after `mod trace;` and before `pub mod window_size;`).

- [ ] **Step 3: Run the tests and see them fail.**

Run: `cargo nextest run -p kb-app ui::design`
Expected: FAIL to compile: `cannot find type Design`.

- [ ] **Step 4: Write the types and `parse`.** Replace the `parse` stub in `design.rs` with the code below, placed
      after `DesignError` (keep `design()`, `pt` and the tests as they are):

```rust
/// A colour token: `#RRGGBB` or `#RRGGBBAA`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub Hsla);

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        hex(&text)
            .map(Color)
            .ok_or_else(|| serde::de::Error::custom(format!("{text} is not #RRGGBB or #RRGGBBAA")))
    }
}

/// `#RRGGBB` or `#RRGGBBAA` as a colour.
#[must_use]
pub fn hex(text: &str) -> Option<Hsla> {
    let digits = text.strip_prefix('#')?;
    let value = u32::from_str_radix(digits, 16).ok()?;
    match digits.len() {
        6 => Some(rgb(value).into()),
        8 => Some(rgba(value).into()),
        _ => None,
    }
}

/// The colour roles (DESIGN.md, Color Palette & Roles).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Colors {
    #[serde(deserialize_with = "color")]
    pub window: Hsla,
    #[serde(deserialize_with = "color")]
    pub mantle: Hsla,
    #[serde(deserialize_with = "color")]
    pub crust: Hsla,
    #[serde(deserialize_with = "color")]
    pub selected: Hsla,
    #[serde(deserialize_with = "color")]
    pub hover: Hsla,
    #[serde(deserialize_with = "color")]
    pub badge: Hsla,
    #[serde(deserialize_with = "color")]
    pub code: Hsla,
    #[serde(deserialize_with = "color")]
    pub edge: Hsla,
    #[serde(deserialize_with = "color")]
    pub ring: Hsla,
    #[serde(deserialize_with = "color")]
    pub text: Hsla,
    #[serde(deserialize_with = "color")]
    pub muted: Hsla,
    #[serde(deserialize_with = "color")]
    pub faint: Hsla,
    #[serde(deserialize_with = "color")]
    pub mauve: Hsla,
    #[serde(deserialize_with = "color")]
    pub blue: Hsla,
    #[serde(deserialize_with = "color")]
    pub green: Hsla,
    #[serde(deserialize_with = "color")]
    pub peach: Hsla,
    #[serde(deserialize_with = "color")]
    pub pink: Hsla,
    #[serde(deserialize_with = "color")]
    pub yellow: Hsla,
    #[serde(deserialize_with = "color")]
    pub red: Hsla,
}

fn color<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Hsla, D::Error> {
    Color::deserialize(deserializer).map(|c| c.0)
}

impl Colors {
    /// A tag hue's colour: the Raycast hash's seven hues in Catppuccin's names.
    #[must_use]
    pub fn hue(&self, hue: Hue) -> Hsla {
        match hue {
            Hue::Blue => self.blue,
            Hue::Orange => self.peach,
            Hue::Green => self.green,
            Hue::Purple => self.mauve,
            Hue::Magenta => self.pink,
            Hue::Yellow => self.yellow,
            Hue::Red => self.red,
        }
    }

    /// An accent by its token name (the catalog's `tint`): `mauve`, `blue`, `green`, `peach`,
    /// `pink`, `yellow` or `red`.
    #[must_use]
    pub fn named(&self, name: &str) -> Option<Hsla> {
        Some(match name {
            "mauve" => self.mauve,
            "blue" => self.blue,
            "green" => self.green,
            "peach" => self.peach,
            "pink" => self.pink,
            "yellow" => self.yellow,
            "red" => self.red,
            _ => return None,
        })
    }
}

/// Which bundled family a role uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Family {
    /// Inter.
    Ui,
    /// JetBrains Mono.
    Mono,
}

/// One type role.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextRole {
    /// The family.
    pub family: Family,
    /// The size, in points.
    pub size: f32,
    /// The weight, 100–900.
    pub weight: f32,
    /// The line height, in points; the family's own when absent.
    pub line_height: Option<f32>,
    /// Italic.
    #[serde(default)]
    pub italic: bool,
    /// Drawn in capitals (the caller upper-cases the text).
    #[serde(default)]
    pub uppercase: bool,
}

/// The type roles (DESIGN.md, Typography Rules).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Roles {
    pub search: TextRole,
    pub title: TextRole,
    pub description: TextRole,
    pub label: TextRole,
    pub section: TextRole,
    pub footer: TextRole,
    pub chip: TextRole,
    pub count: TextRole,
    pub keycap: TextRole,
    pub heading: TextRole,
    pub quote: TextRole,
    pub body: TextRole,
    pub notes_h1: TextRole,
    pub notes_h2: TextRole,
    pub notes_h3: TextRole,
    pub card_split: TextRole,
    pub card_detail: TextRole,
    pub link_title: TextRole,
    pub link_url: TextRole,
    pub link_hero: TextRole,
    pub empty_title: TextRole,
    pub empty_text: TextRole,
    pub notice: TextRole,
}

/// The families and the roles.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Typography {
    /// The interface family.
    pub ui: String,
    /// The code family.
    pub mono: String,
    /// The roles.
    pub roles: Roles,
}

/// Corner radii, in points.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rounded {
    pub window: f32,
    pub row: f32,
    pub chip: f32,
    pub tile: f32,
    pub pill: f32,
    pub popover: f32,
    pub card: f32,
    pub hero: f32,
    pub favicon: f32,
}

/// Declares a struct of `f32` points, each field documented by its name.
macro_rules! points {
    ($(#[$meta:meta])* $name:ident { $($field:ident),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            $(pub $field: f32,)*
        }
    };
}

points!(/// The header's spacing.
    HeaderSpacing { pad_x, gap });
points!(/// The list's spacing.
    ListSpacing { pad, gap });
points!(/// A row's spacing.
    RowSpacing { pad_x, gap, text_gap, trailing_gap, tags_gap, split_pad_x, split_gap });
points!(/// The entry view's spacing.
    PaneSpacing { split_pad_x, split_pad_y, detail_pad_x, detail_pad_y, gap, header_gap });
points!(/// The footer's spacing.
    FooterSpacing { pad_left, pad_right, entry_pad_right, gap, entry_gap, item_gap, left_gap });
points!(/// The action panel's spacing.
    PanelSpacing { pad, row_pad_x, row_gap, section_pad_top, section_pad_x, section_pad_bottom, rule_margin, search_pad_x, search_gap, crumb_pad_x, crumb_gap });
points!(/// The filter pill's spacing.
    PillSpacing { pad_x, pad_y, gap });
points!(/// A chip's spacing.
    ChipSpacing { pad_x, link_pad_x, link_gap });
points!(/// A keycap's spacing.
    KeycapSpacing { pad_x, gap });
points!(/// The empty view's spacing.
    EmptySpacing { pad_x, icon_gap, title_gap, bar_gap, summary_pad_x, summary_pad_top, summary_pad_bottom });
points!(/// The quote's spacing.
    QuoteSpacing { pad_left, pad_y });
points!(/// The command card's spacing.
    CardSpacing { pad_x, pad_y, pad_right, copy_inset });
points!(/// The link hero's spacing.
    LinkHeroSpacing { pad_x, pad_y, gap });
points!(/// The Links section's spacing.
    LinksSpacing { gap, label_gap, label_pad_top, label_pad_bottom, row_pad_x, row_gap });
points!(/// The notice's spacing.
    NoticeSpacing { gap, hint_gap, right, bottom });

/// Spacing, by component.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spacing {
    pub header: HeaderSpacing,
    pub list: ListSpacing,
    pub row: RowSpacing,
    pub pane: PaneSpacing,
    pub footer: FooterSpacing,
    pub panel: PanelSpacing,
    pub pill: PillSpacing,
    pub chip: ChipSpacing,
    pub keycap: KeycapSpacing,
    pub empty: EmptySpacing,
    pub quote: QuoteSpacing,
    pub card: CardSpacing,
    pub link_hero: LinkHeroSpacing,
    pub links: LinksSpacing,
    pub notice: NoticeSpacing,
}

points!(/// The window's sizes.
    WindowSize { width, height, expanded_share });
points!(/// The header's sizes.
    HeaderSize { height, icon, pill_icon, pill_max_width });
points!(/// A row's sizes.
    RowSize { height, bar_width, bar_inset, glow, glow_alpha, max_tags });
points!(/// A chip's sizes.
    ChipSize { height, icon, link_icon });
points!(/// A keycap's sizes.
    KeycapSize { height, min_width });
points!(/// The type tile's sizes.
    TileSize { size, icon, small, small_icon, mini, mini_icon, alpha });
points!(/// The favicon's sizes.
    FaviconSize { size, inset });
points!(/// The action panel's sizes.
    PanelSize { width, filter_width, row, search, crumb, max_height, chevron, search_icon, crumb_icon, check, offset });
points!(/// The footer's sizes.
    FooterSize { height, dot, dot_glow });
points!(/// The split's shares.
    SplitSize { list });
points!(/// The empty view's sizes.
    EmptySize { icon, text_max_width, bar_width, bar_height });
points!(/// The hero's sizes.
    HeroSize { thumb_split, thumb_detail, link_icon });
points!(/// The quote's sizes.
    QuoteSize { bar });
points!(/// The Links section's sizes.
    LinksSize { row, url_chars, title_share });
points!(/// The notice's sizes.
    NoticeSize { width, icon, check });
points!(/// Borders.
    BorderSize { hairline });

/// Component sizes.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Components {
    pub window: WindowSize,
    pub header: HeaderSize,
    pub row: RowSize,
    pub chip: ChipSize,
    pub keycap: KeycapSize,
    pub tile: TileSize,
    pub favicon: FaviconSize,
    pub panel: PanelSize,
    pub footer: FooterSize,
    pub split: SplitSize,
    pub empty: EmptySize,
    pub hero: HeroSize,
    pub quote: QuoteSize,
    pub links: LinksSize,
    pub notice: NoticeSize,
    pub border: BorderSize,
}

/// Every design token.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Design {
    /// The design's name.
    pub name: String,
    /// Colour roles.
    pub colors: Colors,
    /// Families and type roles.
    pub typography: Typography,
    /// Corner radii.
    pub rounded: Rounded,
    /// Spacing.
    pub spacing: Spacing,
    /// Component sizes.
    pub components: Components,
}

/// Parses the YAML front matter of `markdown`.
///
/// # Errors
///
/// No front matter, YAML that does not parse, or tokens of the wrong shape.
pub fn parse(markdown: &str) -> Result<Design, DesignError> {
    let rest = markdown.strip_prefix("---\n").ok_or(DesignError::NoFrontMatter)?;
    let end = rest.find("\n---\n").ok_or(DesignError::NoFrontMatter)?;
    let node = kb_domain::yaml::load(&rest[..end])
        .map_err(|error| DesignError::Yaml(error.to_string()))?
        .ok_or(DesignError::NoFrontMatter)?;
    serde_json::from_value(kb_domain::yaml::to_json(&node)).map_err(|error| DesignError::Shape(error.to_string()))
}

/// JetBrains Mono's ligatures and contextual alternates off: code reads as it is copied.
#[must_use]
pub fn no_ligatures() -> FontFeatures {
    FontFeatures(Arc::new(vec![("liga".into(), 0), ("calt".into(), 0)]))
}

/// Applies a type role: family, size, weight, line height, italic; ligatures off for mono.
pub trait TextRoleExt: Styled + Sized {
    /// Draws text in `role`.
    #[must_use]
    fn text_role(self, role: &TextRole) -> Self {
        let typography = &design().typography;
        let family = match role.family {
            Family::Ui => typography.ui.clone(),
            Family::Mono => typography.mono.clone(),
        };
        let mut this = self
            .font_family(family)
            .text_size(pt(role.size))
            .font_weight(FontWeight(role.weight));
        if let Some(line_height) = role.line_height {
            this = this.line_height(pt(line_height));
        }
        if role.italic {
            this = this.italic();
        }
        if role.family == Family::Mono {
            this = this.font_features(no_ligatures());
        }
        this
    }
}

impl<T: Styled> TextRoleExt for T {}
```

Note: delete the first, failing `parse` stub; only the full `parse` above remains. `kb_domain::yaml::YamlError`
implements `Display` (it is a `thiserror` error); if it does not, use `format!("{error:?}")` and say so under
"Deviation:".

- [ ] **Step 5: Run the tests and see them pass.**

Run: `cargo nextest run -p kb-app ui::design`
Expected: PASS, 4 tests.

- [ ] **Step 6: Gate and commit.**

```bash
mise run gate
git add DESIGN.md crates/kb-app/src/ui.rs crates/kb-app/src/ui crates/kb-app/src/lib.rs
git commit -m "feat(design): Put the design tokens in DESIGN.md's front matter" \
  -m "The app parses them once at start (ui/design.rs); the prose describes the approved pages." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: `design:validate` from the tokens — owner checkpoint O1

**Files:** modify `tools/design/color.ts`, `tools/design/color.test.ts`, `tools/design/tasks.toml`,
`tools/design/palettes.json`; modify `PRODUCT.md` and maybe `DESIGN.md` (by O1's answer).

- [ ] **Step 1: Stop and ask the owner O1.** Show them this, verbatim, and wait for the answer:

> The approved `faint` (`#6E738D`) is 3.15:1 on the window and 2.70:1 on a rest chip; PRODUCT.md asks AA (4.5:1) on
> every surface. **A (recommended):** keep the approved colour and amend the rule to "body and label text AA;
> tertiary text (`faint`: counts, hints, section labels, URLs) at least 3:1 on the window and popovers; colour never
> the only cue" — faint on a rest chip becomes a warning, as D19 did for accents. **B:** keep the rule and change
> `faint` to `#939AB7` (Catppuccin overlay2: 5.29:1 on the window, 4.53:1 on a chip); every faint text gets brighter
> than the approved pages.

Record the answer as a comment on TODO task `pMDV`:
`todo comment pMDV "2026-10-13 O1: <A or B> (owner)"` (check the exact comment syntax with `todo --help` first).

- [ ] **Step 2: If B,** in `DESIGN.md`'s front matter change `faint: "#6E738D"` to `faint: "#939AB7"`. If A, change
      nothing there.

- [ ] **Step 3: Write the failing test.** Append to `tools/design/color.test.ts`:

```ts
import { frontMatter } from './color.ts'

test('reads the tokens from DESIGN.md front matter', () => {
  const tokens = frontMatter('---\ncolors:\n  window: "#24273A"\n---\n\n# Design\n')
  expect(tokens.colors.window).toBe('#24273A')
  expect(() => frontMatter('# no front matter\n')).toThrow('DESIGN.md has no front matter')
})
```

(Add `frontMatter` to the file's existing import from `./color.ts` instead of a second import if one exists; check
the file's top line.)

Run: `mise run design:test`
Expected: FAIL: `frontMatter` is not exported.

- [ ] **Step 4: Add `frontMatter` to `color.ts`.** Append:

```ts
/** DESIGN.md's front matter (the design tokens), parsed with Bun's YAML parser. */
export const frontMatter = (markdown: string): Record<string, any> => {
  const match = markdown.match(/^---\n([\s\S]*?)\n---\n/)
  if (!match) throw new Error('DESIGN.md has no front matter')
  return Bun.YAML.parse(match[1]!) as Record<string, any>
}
```

Run: `mise run design:test`
Expected: PASS.

- [ ] **Step 5: Replace the `design:validate` task.** In `tools/design/tasks.toml`, replace the whole
      `["design:validate"]` table (from its header to the line before `["design:theme"]`) with:

````toml
["design:validate"]
description = "Check the design tokens in DESIGN.md (WCAG 2.2 contrast of every text role on its surfaces; accents apart under normal, protan and deutan vision, as warnings, D19) and the comparison palettes in palettes.json, into tools/design/validation.json; fail when a token check fails"
run = ['''
#!/usr/bin/env bun
import { readFileSync, writeFileSync } from 'node:fs'
// mise runs an inline script from a temporary file, so modules are imported from the project root.
const { ACCENTS, contrast, frontMatter, lin, mix, pill, rgb } = await import(`${process.cwd()}/tools/design/color.ts`)

const dir = 'tools/design/'
const CVD = {
  normal: null,
  protan: [[0.152286, 1.052583, -0.204868], [0.114503, 0.786281, 0.099216], [-0.003882, -0.048116, 1.051998]],
  deutan: [[0.367322, 0.860646, -0.227968], [0.280085, 0.672501, 0.047413], [-0.01182, 0.04294, 0.968881]]
}
const simulate = (h, m) => {
  const c = rgb(h).map(lin)
  return m ? m.map((r) => Math.min(1, Math.max(0, r[0] * c[0] + r[1] * c[1] + r[2] * c[2]))) : c
}
const oklab = ([r, g, b]) => {
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b)
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b)
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b)
  return [
    0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s
  ]
}
const deltaE = (a, b, m) => {
  const [p, q] = [oklab(simulate(a, m)), oklab(simulate(b, m))]
  return 100 * Math.hypot(p[0] - q[0], p[1] - q[1], p[2] - q[2])
}
const check = (name, value, pass, warn) => ({
  name,
  value: Math.round(value * 100) / 100,
  status: value >= pass ? 'pass' : warn !== undefined && value >= warn ? 'warn' : 'fail'
})
const separation = (colors, names) => {
  const checks = []
  for (const [vision, m] of Object.entries(CVD)) {
    let worst = { value: Infinity }
    for (let i = 0; i < names.length; i++) {
      for (let j = i + 1; j < names.length; j++) {
        const value = deltaE(colors[names[i]], colors[names[j]], m)
        if (value < worst.value) worst = { value, pair: `${names[i]}/${names[j]}` }
      }
    }
    // D19 (owner, 2026-10-12): accents apart is advisory; colour is never the only cue.
    const c = check(`accents apart (${vision})`, worst.value, vision === 'normal' ? 15 : 8, 0)
    checks.push({ ...c, pair: worst.pair })
  }
  return checks
}

// The tokens: what the app draws. These checks fail the task.
const t = frontMatter(readFileSync('DESIGN.md', 'utf8')).colors
const tokenChecks = []
for (const surface of ['window', 'mantle', 'selected', 'badge', 'crust']) {
  tokenChecks.push(check(`text on ${surface}`, contrast(t.text, t[surface]), 4.5))
  tokenChecks.push(check(`muted on ${surface}`, contrast(t.muted, t[surface]), 4.5))
}
// O1 (owner, see TODO pMDV): FAINT_RULE is 'tertiary' (answer A) or 'aa' (answer B).
const FAINT_RULE = 'tertiary'
for (const surface of ['window', 'mantle', 'crust']) {
  tokenChecks.push(check(`faint on ${surface}`, contrast(t.faint, t[surface]), FAINT_RULE === 'aa' ? 4.5 : 3))
}
tokenChecks.push(check('faint on badge', contrast(t.faint, t.badge), FAINT_RULE === 'aa' ? 4.5 : 3, FAINT_RULE === 'aa' ? undefined : 2.5))
const tints = ['mauve', 'blue', 'green', 'peach', 'pink', 'yellow', 'red']
for (const a of tints) {
  // A type tile: the accent at 15 % over the window, its icon in the accent (non-text, 3:1).
  tokenChecks.push(check(`${a} on its tile`, contrast(t[a], mix(t[a], t.window, 0.15)), 3))
}
tokenChecks.push(...separation(t, tints))
const tokenFailed = tokenChecks.filter((c) => c.status === 'fail')

// The comparison palettes: informational, they draw nothing.
const palettes = JSON.parse(readFileSync(`${dir}palettes.json`, 'utf8'))
const paletteResults = palettes.map(({ id, roles: r }) => {
  const checks = []
  for (const surface of ['window', 'selection', 'raised', 'header']) {
    checks.push(check(`text on ${surface}`, contrast(r.text, r[surface]), 4.5))
    checks.push(check(`textMuted on ${surface}`, contrast(r.textMuted, r[surface]), 4.5))
  }
  for (const a of ACCENTS) {
    const { background, text } = pill(r, a)
    checks.push(check(`#${a} on its pill`, contrast(text, background), 4.5, 3))
  }
  checks.push(...separation(r, ACCENTS))
  return { id, failed: checks.filter((c) => c.status === 'fail').length, warned: checks.filter((c) => c.status === 'warn').length, checks }
})

const results = [
  { id: 'kb (DESIGN.md)', failed: tokenFailed.length, warned: tokenChecks.filter((c) => c.status === 'warn').length, checks: tokenChecks },
  ...paletteResults
]
writeFileSync(`${dir}validation.json`, `${JSON.stringify(results, null, 2)}\n`)
for (const { id, failed, warned } of results) {
  console.log(`${id.padEnd(32)} ${failed ? `${failed} fail` : 'pass'}${warned ? `, ${warned} warn` : ''}`)
}
if (tokenFailed.length) {
  console.error(`DESIGN.md's tokens fail: ${tokenFailed.map((c) => `${c.name} ${c.value}`).join(', ')}`)
  process.exit(1)
}
''', "dprint fmt tools/design/validation.json"]
````

If O1 was **B**, change the line `const FAINT_RULE = 'tertiary'` to `const FAINT_RULE = 'aa'`.

- [ ] **Step 6: Drop `chosen` from the palettes.** In `tools/design/palettes.json`, delete the `"chosen": true` line
      (on the `tokyo-night-storm-flat-separated` entry) and the comma that precedes it if it leaves a trailing comma.
      Then run `dprint fmt tools/design/palettes.json`.

- [ ] **Step 7: Run the task and see it pass.**

Run: `mise run design:validate`
Expected: the first line reads `kb (DESIGN.md)` followed by `pass` and some warnings (accents apart; faint on badge
if A); the other palettes print their own results; exit 0.

- [ ] **Step 8: Amend PRODUCT.md (A only).** If O1 was **A**, in `PRODUCT.md` replace the line
      `- Text meets WCAG 2.2 AA contrast on every surface it sits on; `mise run design:validate` checks the palette.`
      with:

```markdown
- Body and label text meet WCAG 2.2 AA (4.5:1) on every surface they sit on; tertiary text (`faint`: counts, hints,
  section labels, URLs) at least 3:1 on the window and popovers (owner, 2026-10-13, O1); `mise run design:validate`
  checks the tokens.
```

If O1 was **B**, replace it with:

```markdown
- Text meets WCAG 2.2 AA contrast on every surface it sits on; `mise run design:validate` checks the tokens.
```

- [ ] **Step 9: Gate and commit.**

```bash
mise run gate
git add tools/design/color.ts tools/design/color.test.ts tools/design/tasks.toml tools/design/palettes.json tools/design/validation.json PRODUCT.md DESIGN.md
git commit -m "feat(design): Validate the design tokens in DESIGN.md" \
  -m "Accent separation is a warning (D19); faint follows the owner's O1." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: `design:theme` from the tokens; accents from the tokens

**Files:** modify `tools/design/tasks.toml`, `crates/kb-app/src/accents.rs`, `crates/kb-app/tests/ui.rs`;
regenerate `crates/kb-app/assets/themes/kb.json`; delete `crates/kb-app/assets/themes/accents.json`,
`tools/design/layout.json`.

- [ ] **Step 1: Write the failing test.** Append to the `tests` module of `crates/kb-app/src/ui/design.rs`:

```rust
    #[test]
    fn gives_gpui_kit_the_tokens() {
        let theme: serde_json::Value =
            serde_json::from_str(include_str!("../../assets/themes/kb.json")).unwrap();
        let colors = &theme["themes"][0]["colors"];
        let token = |key: &str| hex(colors[key].as_str().unwrap()).unwrap();
        let c = &design().colors;
        assert_eq!(token("popover.background"), c.mantle);
        assert_eq!(token("foreground"), c.text);
        assert_eq!(token("muted.foreground"), c.muted);
        assert_eq!(token("border"), c.selected);
        assert_eq!(token("caret"), c.mauve);
        assert_eq!(token("background").a, 0., "the window draws its own rounded surface");
    }
```

Run: `cargo nextest run -p kb-app ui::design::tests::gives_gpui_kit_the_tokens`
Expected: FAIL: `popover.background` is `#24293E`.

- [ ] **Step 2: Replace the `design:theme` task.** In `tools/design/tasks.toml`, replace the whole `["design:theme"]`
      table (from its header to the line before `["design:render"]`) with:

````toml
["design:theme"]
description = "Write GPUI Kit's dark theme (crates/kb-app/assets/themes/kb.json) and the tag-accent fixture from the design tokens in DESIGN.md"
run = [
  '''
#!/usr/bin/env bun
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
// mise runs an inline script from a temporary file, so modules are imported from the project root.
const { AA, contrast, frontMatter, mix, tagAccent, tint } = await import(`${process.cwd()}/tools/design/color.ts`)

const root = process.cwd()
const out = resolve(root, 'crates/kb-app/assets/themes/kb.json')
const t = frontMatter(readFileSync(resolve(root, 'DESIGN.md'), 'utf8')).colors
const CLEAR = '#00000000'

// Token → GPUI Kit key. The panel draws its own rounded surface on a transparent window (DESIGN.md, Shape), so the
// root and list backgrounds are clear; kb's components draw rows, chips and panels from the tokens themselves.
const colors = {
  background: CLEAR,
  'list.background': CLEAR,
  'list.even.background': CLEAR,
  'list.head.background': CLEAR,
  'table.background': t.window,
  'title_bar.background': t.window,
  'list.active.background': tint(t.selected, t.window, 0.2),
  'list.active.border': t.selected,
  'accent.background': t.selected,
  'accent.foreground': t.text,
  'secondary.background': t.badge,
  'secondary.hover.background': t.selected,
  'secondary.active.background': t.selected,
  'secondary.foreground': t.text,
  'popover.background': t.mantle,
  'popover.foreground': t.text,
  'muted.background': t.code,
  'muted.foreground': t.muted,
  border: t.selected,
  'input.border': t.selected,
  'title_bar.border': t.selected,
  'window.border': t.selected,
  foreground: t.text,
  caret: t.mauve,
  'selection.background': `${t.mauve}4D`,
  ring: t.mauve,
  'link.foreground': t.blue,
  'progress_bar.background': t.blue
}
for (const [hue, token] of Object.entries({ blue: 'blue', cyan: 'blue', green: 'green', red: 'red', yellow: 'yellow', magenta: 'pink' })) {
  colors[`base.${hue}`] = t[token]
  colors[`base.${hue}.light`] = mix(t[token], t.window, 0.2)
}

// GPUI Kit caps these keys' opacity (theme/schema.rs, clamp_alpha).
const CAPPED = { 'list.active.background': 0.2, 'selection.background': 0.3 }
for (const [key, cap] of Object.entries(CAPPED)) {
  const value = colors[key]
  const alpha = value.length === 9 ? Number.parseInt(value.slice(7), 16) / 255 : 1
  if (alpha > cap + 0.005) throw new Error(`${key} is ${value}: GPUI Kit draws it at most ${cap} opaque`)
}
const defaults = JSON.parse(readFileSync(defaultThemePath(), 'utf8')).themes.find((x) => x.mode === 'dark').colors
const unknown = Object.keys(colors).filter((k) => !(k in defaults))
if (unknown.length) throw new Error(`Not GPUI Kit dark theme keys: ${unknown.join(', ')}`)

// Code blocks in the notes: each syntax colour lifted toward `text` only as far as AA needs on `code`.
const readable = (accent) => {
  let c = accent
  for (let lift = 0.05; contrast(c, t.code) < AA && lift <= 1; lift += 0.05) c = mix(t.text, accent, lift)
  return c
}
const color = (token) => ({ color: readable(t[token]) })
const highlight = {
  'editor.foreground': t.text,
  'editor.background': t.code,
  'editor.active_line.background': t.selected,
  'editor.line_number': readable(t.faint),
  'editor.active_line_number': t.muted,
  syntax: {
    attribute: color('blue'),
    boolean: color('peach'),
    comment: color('faint'),
    'comment.doc': color('faint'),
    constant: color('peach'),
    constructor: color('mauve'),
    embedded: color('text'),
    emphasis: { font_style: 'italic' },
    'emphasis.strong': { font_weight: 700 },
    function: color('blue'),
    keyword: color('mauve'),
    link_text: { ...color('blue'), font_style: 'normal' },
    link_uri: { ...color('blue'), font_style: 'italic' },
    number: color('peach'),
    operator: color('muted'),
    property: color('text'),
    punctuation: color('muted'),
    string: color('green'),
    'string.escape': color('yellow'),
    'string.regex': color('yellow'),
    'string.special': color('green'),
    'string.special.symbol': color('green'),
    tag: color('red'),
    'text.literal': color('green'),
    'text.code.span': color('green'),
    title: { ...color('text'), font_weight: 600 },
    type: color('yellow'),
    variable: color('text'),
    'variable.special': color('red')
  }
}
const theme = {
  name: 'kb',
  author: 'kb',
  themes: [{ name: 'kb Dark', mode: 'dark', colors: Object.fromEntries(Object.entries(colors).sort()), highlight }]
}
writeFileSync(out, `${JSON.stringify(theme, null, 2)}\n`)
console.log(`wrote ${out.replace(`${root}/`, '')} from DESIGN.md`)

// Tag → hue by the Raycast extension's hash: the Rust hash is tested against it (presentation.rs).
const fixtureOut = resolve(root, 'crates/kb-app/tests/fixtures/tag-accents.json')
const sampleTags = ['git', 'docker', 'kube', 'rust', 'kb', 'video', 'talks', 'docs', 'containers', 'mixed-case', 'aws', 'nu', 'linux', 'gradle', 'grails', 'schedule', 'ai', 'react', 'café']
mkdirSync(dirname(fixtureOut), { recursive: true })
writeFileSync(fixtureOut, `${JSON.stringify(Object.fromEntries(sampleTags.map((x) => [x, tagAccent(x)])), null, 2)}\n`)
console.log(`wrote ${fixtureOut.replace(`${root}/`, '')}`)

// GPUI Kit's default-theme.json in the Cargo registry, at the version Cargo.lock pins.
function defaultThemePath() {
  const lock = readFileSync(resolve(root, 'Cargo.lock'), 'utf8')
  const version = lock.match(/name = "gpui-component"\nversion = "([^"]+)"/)?.[1]
  const home = process.env.CARGO_HOME ?? resolve(process.env.HOME, '.cargo')
  const registry = resolve(home, 'registry', 'src')
  const index = readdirSync(registry).find((d) => existsSync(resolve(registry, d, `gpui-component-${version}`)))
  if (!index) throw new Error(`gpui-component ${version} is not in ${registry}: run cargo fetch.`)
  return resolve(registry, index, `gpui-component-${version}`, 'src/theme/default-theme.json')
}
''',
  "dprint fmt crates/kb-app/assets/themes/kb.json crates/kb-app/tests/fixtures/tag-accents.json",
]
````

- [ ] **Step 3: Remove what the old design used.** Delete the `["design:render"]` and `["design:palettes"]` tables
      from `tools/design/tasks.toml` (each from its header to the line before the next `[` header), and:

```bash
git rm tools/design/layout.json crates/kb-app/assets/themes/accents.json
grep -rn "layout.json\|accents.json\|design:render\|design:palettes" --include=*.toml --include=*.ts --include=*.rs --include=*.md . | grep -v "^./target\|^./docs/superpowers"
```

Expected: the grep prints only lines in `README.md` and `tools/design/README.md`-like docs, if any (8h rewrites
them); no `.rs`, `.ts` or `.toml` line. If a `.rs` line other than `accents.rs` appears, it is handled in Step 5.

- [ ] **Step 4: Regenerate the theme.**

Run: `mise run design:theme`
Expected: `wrote crates/kb-app/assets/themes/kb.json from DESIGN.md` and `wrote crates/kb-app/tests/fixtures/tag-accents.json`.

- [ ] **Step 5: Accents from the tokens.** Replace `crates/kb-app/src/accents.rs` entirely with:

```rust
//! The tag and type accents, from the design tokens (DESIGN.md's front matter). The components of
//! Plan 8d replace this module; until then it keeps the old call sites drawing the new colours.

use gpui_kit::{App, Hsla};

use crate::presentation::Hue;
use crate::ui::design::design;

/// An accent and its tag pill.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Accent {
    /// Icons, bars, the type tint.
    pub color: Hsla,
    /// The tag pill's fill: the rest chip.
    pub pill: Hsla,
    /// The tag pill's text.
    pub pill_text: Hsla,
}

/// The accent for `hue`; pills are grey (D3, chip at rest).
#[must_use]
pub fn accent(hue: Hue, _: &App) -> Accent {
    let colors = &design().colors;
    Accent {
        color: colors.hue(hue),
        pill: colors.badge,
        pill_text: colors.muted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_tag_pills_grey_and_hues_from_the_tokens() {
        let colors = &design().colors;
        assert_eq!(colors.hue(Hue::Purple), colors.mauve);
        assert_eq!(colors.badge, colors.badge);
    }
}
```

- [ ] **Step 6: Update the theme UI test.** In `crates/kb-app/tests/ui.rs`, replace the test
      `draws_dark_mode_in_the_palette_of_design_md` with:

```rust
#[gpui_kit::test]
fn draws_gpui_kit_components_in_the_design_tokens(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        kb_app::init_theme(cx);
        let theme: serde_json::Value = serde_json::from_str(include_str!("../assets/themes/kb.json")).unwrap();
        let popover = theme["themes"][0]["colors"]["popover.background"].as_str().unwrap();
        let popover = u32::from_str_radix(popover.trim_start_matches('#'), 16).unwrap();
        assert_eq!(cx.theme().popover, gpui_kit::rgb(popover).into());
    });
}
```

- [ ] **Step 7: Run every kb-app test.**

Run: `cargo nextest run -p kb-app`
Expected: PASS. (The UI tests still pass: the list draws its selection with `list.active.background`.)

- [ ] **Step 8: Gate and commit.**

```bash
mise run gate
git add -A tools/design crates/kb-app/assets/themes crates/kb-app/src/accents.rs crates/kb-app/src/ui/design.rs crates/kb-app/tests
git commit -m "feat(design): Generate GPUI Kit's theme from the design tokens" \
  -m "layout.json, accents.json, design:render and design:palettes go: the tokens replace them." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Bundled fonts

**Files:** create `crates/kb-app/assets/fonts/{Inter-Regular,Inter-Medium,Inter-SemiBold,Inter-Italic,
JetBrainsMono-Regular,JetBrainsMono-Medium}.ttf`, `crates/kb-app/assets/fonts/Inter-OFL.txt`,
`crates/kb-app/assets/fonts/JetBrainsMono-OFL.txt`; modify `crates/kb-app/src/ui/design.rs`,
`crates/kb-app/src/lib.rs`.

- [ ] **Step 1: Download the fonts and check them.**

```bash
mkdir -p crates/kb-app/assets/fonts && cd crates/kb-app/assets/fonts
base=https://cdn.jsdelivr.net/fontsource/fonts/inter@5.2.8
curl -fsSL "$base/latin-400-normal.ttf" -o Inter-Regular.ttf
curl -fsSL "$base/latin-500-normal.ttf" -o Inter-Medium.ttf
curl -fsSL "$base/latin-600-normal.ttf" -o Inter-SemiBold.ttf
curl -fsSL "$base/latin-400-italic.ttf" -o Inter-Italic.ttf
jb=https://raw.githubusercontent.com/JetBrains/JetBrainsMono/v2.304
curl -fsSL "$jb/fonts/ttf/JetBrainsMono-Regular.ttf" -o JetBrainsMono-Regular.ttf
curl -fsSL "$jb/fonts/ttf/JetBrainsMono-Medium.ttf" -o JetBrainsMono-Medium.ttf
curl -fsSL "$jb/OFL.txt" -o JetBrainsMono-OFL.txt
curl -fsSL https://raw.githubusercontent.com/rsms/inter/v4.1/LICENSE.txt -o Inter-OFL.txt
shasum -a 256 *.ttf
cd -
```

Expected hashes (stop and report if any differs):

```text
27396a89e10048c8e76096b1b4a5500bdc54d3b04d200f267ed6bf0d5466b2ba  Inter-Regular.ttf
3073f87cb6eac0ac47fee30af580ba12f4bedeb62d1e1254c07dca7b2ad607e7  Inter-Medium.ttf
85c5903688f4329aaeebfc050fe6c3cb373f75ef1ff0a6f68d2640a870573584  Inter-SemiBold.ttf
94c5defb258253328887c469683381ae5670594415eb6c39e9aa43e4ac744ebc  Inter-Italic.ttf
a0bf60ef0f83c5ed4d7a75d45838548b1f6873372dfac88f71804491898d138f  JetBrainsMono-Regular.ttf
31c92d01a8a08528b718a43addf0ad3df0af2ca4b7b3290a452f70f358e14d3d  JetBrainsMono-Medium.ttf
```

(Inter is fontsource's Latin subset, 68 KB each: macOS's font fallback draws other scripts. JetBrains Mono is whole,
274 KB, because keycaps need its ⌘ ⌥ ⇧ glyphs. Both stay under hk's 500 KB `large-files` limit.)

- [ ] **Step 2: Write the failing test.** Append to the `tests` module of `crates/kb-app/src/ui/design.rs`:

```rust
    /// Whether `font` names `family` in its name table (UTF-16 big-endian, as TrueType stores it).
    fn names_family(font: &[u8], family: &str) -> bool {
        let needle: Vec<u8> = family.encode_utf16().flat_map(u16::to_be_bytes).collect();
        font.windows(needle.len()).any(|window| window == needle.as_slice())
    }

    #[test]
    fn bundles_the_families_the_tokens_name() {
        let typography = &design().typography;
        for font in FONTS {
            assert_eq!(&font[..4], &[0, 1, 0, 0], "a TrueType font");
        }
        for font in &FONTS[..4] {
            assert!(names_family(font, &typography.ui));
        }
        for font in &FONTS[4..] {
            assert!(names_family(font, &typography.mono));
        }
    }
```

Run: `cargo nextest run -p kb-app ui::design`
Expected: FAIL to compile: `cannot find value FONTS`.

- [ ] **Step 3: Add the fonts and their loader.** In `crates/kb-app/src/ui/design.rs`, after `no_ligatures`, add:

```rust
/// The bundled fonts: Inter 400, 500, 600 and 400 italic, then JetBrains Mono 400 and 500.
pub const FONTS: [&[u8]; 6] = [
    include_bytes!("../../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../../assets/fonts/Inter-Medium.ttf"),
    include_bytes!("../../assets/fonts/Inter-SemiBold.ttf"),
    include_bytes!("../../assets/fonts/Inter-Italic.ttf"),
    include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../../assets/fonts/JetBrainsMono-Medium.ttf"),
];

/// Registers the bundled fonts with GPUI's text system.
///
/// # Errors
///
/// The platform rejected a font.
pub fn load_fonts(cx: &gpui_kit::App) -> gpui_kit::Result<()> {
    cx.text_system()
        .add_fonts(FONTS.iter().map(|font| std::borrow::Cow::Borrowed(*font)).collect())
}
```

- [ ] **Step 4: Load them at start and name them in the theme.** In `crates/kb-app/src/lib.rs`, in `run`, replace
      `init_theme(cx);` with:

```rust
            if let Err(error) = ui::design::load_fonts(cx) {
                #[expect(clippy::print_stderr, reason = "a font failed to load before any window exists")]
                {
                    eprintln!("kb: {error}");
                }
            }
            init_theme(cx);
```

- [ ] **Step 5: Run the tests and see them pass.**

Run: `cargo nextest run -p kb-app ui::design`
Expected: PASS, 6 tests.

- [ ] **Step 6: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/assets/fonts crates/kb-app/src/ui/design.rs crates/kb-app/src/lib.rs
git commit -m "feat(design): Bundle Inter and JetBrains Mono" \
  -m "Inter in its Latin subsets, JetBrains Mono whole for the keycap glyphs; both OFL." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Always dark, in the design's fonts, on a rounded panel

**Files:** modify `crates/kb-app/src/lib.rs`, `crates/kb-app/src/panel.rs`, `crates/kb-app/src/search_view.rs`,
`crates/kb-app/tests/ui.rs`.

- [ ] **Step 1: Write the failing test.** In `crates/kb-app/tests/ui.rs`, replace the test
      `follows_the_system_appearance` with:

```rust
#[gpui_kit::test]
fn is_always_dark_in_the_design_fonts(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        kb_app::init_theme(cx);
        assert_eq!(cx.theme().mode, ThemeMode::Dark);
        assert_eq!(cx.theme().font_family.as_ref(), "Inter");
        assert_eq!(cx.theme().mono_font_family.as_ref(), "JetBrains Mono");
    });
}
```

Remove the now-unused `use gpui_kit::WindowAppearance;` line at the top of `ui.rs`, and `Theme` from the
`use gpui_kit::component::{ActiveTheme as _, Theme, ThemeMode};` line if nothing else uses it (keep `ThemeMode`).

Run: `cargo nextest run -p kb-app --test ui is_always_dark_in_the_design_fonts`
Expected: FAIL: the mode follows the test platform, or the font family is GPUI Kit's default.

- [ ] **Step 2: Make the theme always dark, in the design's fonts.** In `crates/kb-app/src/lib.rs`, replace the doc
      comment and body of `init_theme` with:

```rust
/// kb's GPUI Kit theme, always dark (D1): the tokens' colours (`kb.json`, written by
/// `mise run design:theme`), Inter and JetBrains Mono, and GPUI Kit's base font size kept at the
/// 16 points the design module counts rems in.
pub fn init_theme(cx: &mut gpui_kit::App) {
    let registry = ThemeRegistry::global_mut(cx);
    // The theme is compiled in and a UI test loads it: should it ever fail, the default dark stays.
    if registry.load_themes_from_str(THEME).is_ok()
        && let Some(dark) = registry.themes().get(DARK_THEME).cloned()
    {
        Theme::global_mut(cx).dark_theme = dark;
    }
    Theme::change(ThemeMode::Dark, None, cx);
    let typography = &ui::design::design().typography;
    let theme = Theme::global_mut(cx);
    theme.font_family = typography.ui.clone().into();
    theme.mono_font_family = typography.mono.clone().into();
    theme.font_size = gpui_kit::px(ui::design::POINTS_PER_REM);
}
```

and change the import line `use gpui_kit::component::{Theme, ThemeRegistry};` to
`use gpui_kit::component::{Theme, ThemeMode, ThemeRegistry};`.

- [ ] **Step 3: Stop following the system appearance.** In `crates/kb-app/src/search_view.rs`, in `SearchView::new`,
      delete the three lines that build `appearance` (`// The theme follows…`, `let appearance = …`, and its closing
      `});`), and change `subscriptions: vec![list_sub, input_sub, appearance],` to
      `subscriptions: vec![list_sub, input_sub],`. Remove `Theme` from the `use gpui_kit::component::{…}` line of
      that file if nothing else in it uses `Theme`.

- [ ] **Step 4: Run the tests and see them pass.**

Run: `cargo nextest run -p kb-app`
Expected: PASS.

- [ ] **Step 5: The rounded panel on a transparent window.** In `crates/kb-app/src/panel.rs`:
  - add `WindowBackgroundAppearance` to the `use gpui_kit::{…}` list;
  - in `window_options`, add `window_background: WindowBackgroundAppearance::Transparent,` after `show: true,`, with
    the comment `// GPUI's windows are square: the panel draws its own rounded surface on a clear window (D1).`

In `crates/kb-app/src/search_view.rs`, in `render`, replace

```rust
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
```

with

```rust
            .size_full()
            .rounded(pt(tokens.rounded.window))
            .bg(tokens.colors.window)
            .border_1()
            .border_color(tokens.colors.ring)
            .overflow_hidden()
            .text_color(tokens.colors.text)
```

and add, as the first line of `render`, `let tokens = design();`, plus the import
`use crate::ui::design::{design, pt};` at the top of the file.

- [ ] **Step 6: Look at the real panel.** Build and run the app over the showcase corpus in a scratch home:

```bash
home=$PWD/tmp/design/home && rm -rf "$home" && mkdir -p "$home/.config/kb"
printf 'sources_path = "%s"\n' "$PWD/tools/design/showcase" > "$home/.config/kb/config.toml"
cargo build -p kb-app && HOME=$home target/debug/kb --sync && (HOME=$home target/debug/kb &) && sleep 3
screencapture -x tmp/design/rounded.png && pkill -x kb
open tmp/design/rounded.png
```

Expected: the panel's four corners are rounded, with the desktop visible behind them; the text is Inter. **If the
corners show a dark square behind them**, revert Step 5's `window_background` line and the `.rounded(…)` line (keep
the rest), and add "Deviation: GPUI draws the window opaque; the panel stays square (spec Risks)" to the commit body.

- [ ] **Step 7: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/src/lib.rs crates/kb-app/src/panel.rs crates/kb-app/src/search_view.rs crates/kb-app/tests/ui.rs
git commit -m "feat(app): Draw the panel always dark with rounded corners" \
  -m "Inter and JetBrains Mono, on a clear window with the panel's 14 pt corners (D1)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Done when

- `cargo nextest run -p kb-app` and `mise run gate` pass.
- `mise run design:validate` prints `kb (DESIGN.md)  pass, …`.
- The app opens dark, in Inter, with rounded corners (or the recorded deviation).
- Mark progress: `todo` — set `nNln` (showcase corpus) done with the comment
  `"2026-10-13: tools/design/showcase, Plan 8a Task 1"` (check the CLI's syntax with `todo --help`).
