# Plan 8h: Screenshots, the review against the approved pages, the documents

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** prove every screen against its approved page with screenshots of the real app on the showcase corpus
(spec §8–9), compact and expanded; move the approved pages into `docs/guides/design/` (replacing the Stitch export);
update every document of spec §10; write `tools/design/CHANGE-MAP.md` (§11); close the Plan 8 tasks.

**Architecture:** `mise run design:shots` (Bun, inline in `tools/design/tasks.toml`, as the repository's rule for
`tools/` scripts says) builds the release app, runs it in a scratch home over the showcase corpus, drives it with the
bench kit's `probe` (CGEvent), and captures the window by its id with `screencapture -l`. A filled-in
`tools/design/CHECKLISTS.md` records each screen's checklist against its page; the owner signs it off.

**Tech Stack:** Bun 1.4.2, Swift (`probe`), `screencapture`, mise, the `todo` CLI.

**Expects:** Plans 8a–8g done; `tmp/bin/probe` buildable with `mise run probes`; the Mac allows the terminal
Accessibility (key events) and Screen Recording (window capture); ⌥K not taken by another app.

**Index:** [2026-10-12-kb-native-08-design-system.md](2026-10-12-kb-native-08-design-system.md).

---

### Task 1: Owner checkpoint O2 — screenshots and the 500 KB limit

- [ ] **Step 1: Stop and ask the owner**, verbatim:

> `design:shots` commits PNG screenshots at 2× (1720 × 1040 compact, larger expanded). A dense screen can pass hk's
> `large-files` limit of 500 KB. **A (recommended):** exclude `tools/design/shots/**` from that one step — the limit
> guards against accidental binaries, and these are deliberate evidence. **B:** keep the limit and capture at 1×,
> losing detail the review checks (2.5 pt bars, 1 pt hairlines).

Record the answer on TODO task `y3ob`: `todo add --comment "2026-10-13 O2: <A or B> (owner)" --task-id y3ob`.

- [ ] **Step 2: If A,** in `hk.pkl`, change `["large-files"] = Builtins.check_added_large_files` to:

```pkl
["large-files"] = (Builtins.check_added_large_files) {
    // design:shots commits deliberate 2× screenshots (owner, 2026-10-13, O2).
    exclude = List("tools/design/shots/**")
}
```

If B, change nothing here; Task 3's script passes `--scale 1` (it captures at the screen's scale otherwise, and
resizes to 1× with `sips -Z` when the flag is set).

---

### Task 2: `probe window-id`

**Files:** modify `tools/bench/probes/probe.swift`.

- [ ] **Step 1: Add the command.** In the `switch args[1]`, before `default:`, add:

```swift
case "window-id":
    // The window number of the owner's largest on-screen window, for `screencapture -l`.
    let ids = visibleWindows(args[2])
        .sorted { (($0[kCGWindowBounds as String] as? [String: CGFloat])?["Width"] ?? 0) > (($1[kCGWindowBounds as String] as? [String: CGFloat])?["Width"] ?? 0) }
        .compactMap { $0[kCGWindowNumber as String] as? Int }
    guard let id = ids.first else { fputs("no window of \(args[2])\n", stderr); exit(1) }
    print(id)
```

and add `//   probe window-id <owner>                     the window number of the app's largest window` to the usage
comment at the top.

- [ ] **Step 2: Build and try it.**

```bash
mise run probes
open -a Finder && sleep 1 && tmp/bin/probe window-id Finder
```

Expected: a number.

- [ ] **Step 3: Commit.**

```bash
mise run gate
git add tools/bench/probes/probe.swift
git commit -m "feat(bench): Add probe window-id for window captures" \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: `mise run design:shots`

**Files:** modify `tools/design/tasks.toml`; create `tools/design/shots/` (by running it).

- [ ] **Step 1: Add the task.** Append to `tools/design/tasks.toml`:

```toml
["design:shots"]
description = "Capture kb on the showcase corpus, every screen, compact and expanded, into tools/design/shots (needs Accessibility and Screen Recording for the terminal, and ⌥K free)"
depends = ["probes"]
usage = 'flag "--scale <n>" help="1 to save at 1× (owner O2, option B); the screen scale by default"'
run = '''
#!/usr/bin/env bun
import { existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

const root = process.cwd()
const probe = resolve(root, 'tmp/bin/probe')
const kb = resolve(root, 'target/release/kb')
const out = resolve(root, 'tools/design/shots')
const scale = process.env.usage_scale

const run = (cmd: string[], env: Record<string, string> = {}) => {
  const r = Bun.spawnSync(cmd, { env: { ...process.env, ...env }, stdout: 'pipe', stderr: 'pipe' })
  if (r.exitCode !== 0) throw new Error(`${cmd.join(' ')}: ${r.stderr.toString()}`)
  return r.stdout.toString().trim()
}
const sleep = (ms: number) => Bun.sleepSync(ms)
// Virtual key codes (US layout): Return 36, Escape 53, Right 124, K 40, P 35, R 15, F 3, A 0.
const key = (code: number, mods = '') => run([probe, 'key', String(code), ...(mods ? [mods] : [])])
const type = (text: string) => run([probe, 'type', text])

const home = (name: string, withIndex: boolean) => {
  const dir = resolve(root, 'tmp/design', name)
  rmSync(dir, { recursive: true, force: true })
  mkdirSync(resolve(dir, '.config/kb'), { recursive: true })
  writeFileSync(
    resolve(dir, '.config/kb/config.toml'),
    `sources_path = "${resolve(root, 'tools/design/showcase')}"\ntag_images_path = "${resolve(root, 'tools/design/showcase-images')}"\n`
  )
  if (withIndex) run([kb, '--sync'], { HOME: dir })
  return dir
}
const launch = (dir: string) => {
  Bun.spawnSync(['pkill', '-x', 'kb'])
  Bun.spawn([kb], { env: { ...process.env, HOME: dir }, stdout: 'ignore', stderr: 'ignore' })
  sleep(3000)
  key(40, 'opt') // ⌥K shows the panel
  sleep(800)
}
const shot = (name: string) => {
  sleep(700)
  const id = run([probe, 'window-id', 'kb'])
  const file = resolve(out, `${name}.png`)
  run(['screencapture', '-x', '-o', '-l', id, file])
  if (scale === '1') run(['sips', '-Z', '1040', file])
  console.log(`shot ${name}`)
}
const clear = () => { key(53); key(53); sleep(300) } // Esc closes any overlay, then clears the query

run(['cargo', 'build', '--release', '-p', 'kb-app'])
if (!existsSync(probe)) throw new Error(`need ${probe}; run mise run probes`)
mkdirSync(out, { recursive: true })

const screens = (size: string) => {
  shot(`${size}-01-list`)
  key(40, 'cmd'); shot(`${size}-02-actions`)
  type('More'); key(36); shot(`${size}-02-more`); clear()
  type('/terraform-plan'); sleep(400); shot(`${size}-03-empty`); clear()
  type('k8s'); sleep(400); key(124); shot(`${size}-04-split-command`); clear()
  type('k8s'); sleep(400); key(36); shot(`${size}-05-detail-command`); key(53)
  type('tokio'); sleep(400); key(36); shot(`${size}-05-detail-bookmark-links`); key(53)
  type('youtube'); sleep(400); key(124); shot(`${size}-04-split-bookmark-hero`); clear()
  type('ingress'); sleep(400); key(36); shot(`${size}-06-detail-cheat`); key(53)
  key(35, 'cmd'); shot(`${size}-07-filter`); clear()
  type('docker'); sleep(400); key(36, 'cmd'); shot(`${size}-17-notice`); clear()
  key(15, 'cmd'); sleep(200); shot(`${size}-11-refreshing`); sleep(2500); shot(`${size}-12-report`); key(36)
}

const indexed = home('shots-home', true)
launch(indexed)
screens('compact')
key(3, 'shift+cmd'); sleep(800) // ⇧⌘F: expanded
screens('expanded')
key(3, 'shift+cmd')

launch(home('shots-noindex', false))
shot('compact-10-no-index')
Bun.spawnSync(['pkill', '-x', 'kb'])
console.log(`wrote ${out}`)
'''
```

(`11-refreshing` is caught only if the showcase sync is slower than 200 ms; when it is not, the shot shows the report,
and the checklist notes that the running state was checked by eye instead. `13-stopped` and `14-failed` are not
driven: stopping needs Esc inside the first moments of a sync, and a failure needs an unreadable sources folder — check
them by eye with `mv tools/design/showcase /tmp/x` before ⌘R, and move it back.)

- [ ] **Step 2: Run it.**

Run: `mise run design:shots` (add `-- --scale 1` if O2 was B)
Expected: `shot …` lines for 24 compact and expanded screens plus `compact-10-no-index`, and `wrote …/tools/design/shots`.
If a capture fails with "no window of kb", grant the terminal Screen Recording in System Settings → Privacy & Security
and rerun; if the keys do nothing, grant Accessibility.

- [ ] **Step 3: Commit the shots.**

```bash
git add tools/design/tasks.toml tools/design/shots
git commit -m "feat(design): Capture every screen on the showcase corpus" \
  -m "mise run design:shots: compact and expanded, by window id (spec §8)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: The checklists, signed off by the owner

**Files:** create `tools/design/CHECKLISTS.md`.

- [ ] **Step 1: Write the checklists.** Create `tools/design/CHECKLISTS.md` with one section per screen of spec §9 —
      List (01), Action panel (02), Empty and no index (03, 10), Split and detail (04–06), Filter (07), Refresh
      (11–14), Notice (17) — each item of §9 as a row:

```markdown
## List (01) — `shots/compact-01-list.png`, `shots/expanded-01-list.png` vs `docs/guides/design/01-list.html`

| Item                                                 | Compact | Expanded | Note |
| ---------------------------------------------------- | ------- | -------- | ---- |
| Magnifier, query in 15 pt, the filter pill with `⌘P` |         |          |      |
| …one row per item of spec §9, copied verbatim…       |         |          |      |
```

Copy every item of spec §9 as its own row (split §9's sentences at their semicolons). Then look at each shot beside
its page and fill each cell with ✓ or ✗; for each ✗, write what differs in "Note".

- [ ] **Step 2: Fix or record each difference.** For each ✗: fix it if it is a value (a token) or a component bug,
      then re-run `mise run design:shots` for that screen; if it cannot be fixed (a GPUI limit), record it in DESIGN.md's
      "Adaptations" with its reason and mark the cell "deviation".

- [ ] **Step 3: Stop and ask the owner** to review `tools/design/CHECKLISTS.md` with the shots open: "Every screen
      against its approved page — approve, or name what to change." Record the answer under a `## Sign-off` section
      at the end of the file, dated.

- [ ] **Step 4: Commit.**

```bash
mise run gate
git add tools/design/CHECKLISTS.md tools/design/shots DESIGN.md crates
git commit -m "docs(design): Check every screen against its approved page" \
  -m "Spec §9 checklists, filled in from the shots; signed off by the owner." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: The approved pages replace the Stitch export

**Files:** move `docs/superpowers/brainstorm/2026-10-12-design-system/*` to `docs/guides/design/`; modify the links;
modify `tools/design/tasks.toml` (`design:prototypes`), `hk.pkl`.

- [ ] **Step 1: Move them.**

```bash
git rm -r -q docs/guides/design
git mv docs/superpowers/brainstorm/2026-10-12-design-system docs/guides/design
grep -rln "brainstorm/2026-10-12-design-system\|guides/design/[0-9][0-9]-[a-z-]*/code.html\|docs/guides/design_system" --include='*.md' --include='*.toml' --include='*.pkl' . | grep -v "^./target"
```

In every file the grep lists (expect the spec, TODO.md, DESIGN.md, `hk.pkl`, `tools/design/tasks.toml`), replace
`docs/superpowers/brainstorm/2026-10-12-design-system` with `docs/guides/design` (adjusting relative paths: from
`docs/superpowers/specs/`, the page folder is `../../guides/design/`); in `hk.pkl`'s `generated` list, replace
`"docs/guides/design_system/**/*"` with `"docs/guides/design/**/*"`.

- [ ] **Step 2: `design:prototypes` renders the approved pages.** In `tools/design/tasks.toml`, replace the
      `["design:prototypes"]` task's description with
      `"Render each approved page (docs/guides/design/*.html) to tmp/design/pages/<page>.png at 2×, in headless Chrome, for side-by-side review with the shots"`,
      and in its script: `const DIR = resolve('docs/guides/design')`; the screen list becomes
      `readdirSync(DIR).filter((f) => f.endsWith('.html'))`; the output becomes
      `resolve('tmp/design/pages', screen.replace('.html', '.png'))` (create the folder); the `--window-size` becomes
      `1400,1100` (the pages are wider than the panel).

- [ ] **Step 3: Run and commit.**

```bash
mise run design:prototypes
mise run gate
git add -A docs/guides/design docs/superpowers TODO.md DESIGN.md hk.pkl tools/design/tasks.toml
git commit -m "docs(design): Make the approved pages the design guide" \
  -m "They replace the Stitch export; design:prototypes renders them for review." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: The documents (spec §10)

**Files:** modify `PRODUCT.md`, `CONTEXT.md`, `AGENTS.md`, `README.md`, `docs/guides/parity.md`,
`docs/guides/compliance.md`.

- [ ] **Step 1: CONTEXT.md.** In the main table:
  - **Entry type:** meaning → "The top-level YAML key an entry sits under. The app shows four — Bookmarks, Cheat
    Sheets, Commands, Tasks (the catalog's `types`); the engine keeps every key." Lives in →
    `crates/kb-domain/src/entry.rs`; labels, icons and tints in `crates/kb-app/assets/catalog.toml`.
  - **Palette:** replace the row with **Action panel** — "The ⌘K list of actions for the screen and the selected
    entry, search at the bottom: the root, Links… and More…" — `crates/kb-app/src/ui/action_panel.rs`,
    `crates/kb-app/src/actions.rs`.
  - **Colour palette:** meaning → "The colour roles and accents of the design (DESIGN.md's front matter);
    `palettes.json` keeps alternatives for comparison. Never just 'palette'." Lives in → `DESIGN.md`,
    `tools/design/palettes.json`, `crates/kb-app/assets/themes/kb.json`.
  - **Sync:** append "Its action is named Refresh (⌘R)."
  - Add rows, in this order after **Palette**'s replacement: **Catalog**, **Design token**, **Screen keys**,
    **Status line**, **Command card**, **Notice**, with the meanings and locations of the index's "Words this plan
    adds" table and these locations: `crates/kb-app/src/catalog.rs` and `assets/catalog.toml`; `DESIGN.md` front
    matter and `crates/kb-app/src/ui/design.rs`; `catalog.toml` `[screens.*]`; `crates/kb-app/src/ui/footer.rs`;
    `crates/kb-app/src/ui/command_card.rs`; `crates/kb-app/src/ui/notice.rs`.
  - Remove the "_(Plan N)_" marker sentence if no location carries one any more.

- [ ] **Step 2: PRODUCT.md.** Principle 2 becomes: "**Copy Raycast's behaviour, follow the approved design.** Raycast
      is the reference for behaviour; the approved pages (`docs/guides/design/`) are the reference for the look; every
      departure is recorded in DESIGN.md's Adaptations." Add to Boundaries: "Always dark: one approved palette."

- [ ] **Step 3: AGENTS.md.** In the repository map, change the `tools/design/` line to: "`tools/design/`: the design
      kit: tokens validated (`design:validate`), GPUI Kit's theme generated (`design:theme`), the showcase corpus,
      screenshots (`design:shots`) and the change map (`CHANGE-MAP.md`)", and add after the `PRODUCT.md` line:
      "`crates/kb-app/assets/catalog.toml`: what kb offers — entry types, actions, keys, words — overridable from
      `~/.config/kb/config.toml`".

- [ ] **Step 4: README.md.** In "Project Structure", add `tools/design/showcase/` (the corpus the screenshots use)
      and `crates/kb-app/assets/catalog.toml`; anywhere the README shows the panel, point at
      `tools/design/shots/compact-01-list.png`. Run `mdschema check --schema tools/mdschema/<readme schema>` through
      hk (`hk check --step docs`).

- [ ] **Step 5: parity.md and compliance.md.** In `docs/guides/parity.md`, update each row whose keys or labels
      changed: ↵ Details, ⌘↵ the entry's primary, ⇧⌘↵ its secondary (bookmarks: Copy); ⌘D removed (→ and ← show and
      hide the split); the Copy… page replaced by More…; Copy URL ⇧⌘C replaced by ⇧⌘↵; Quit removed from the panel
      (⌘Q stays); the metadata column removed (D20). In `docs/guides/compliance.md`:
  - "follows the system appearance" → "is always dark (D1): one approved palette; a deliberate departure from GPUI
    Kit's appearance guidance";
  - "Quit kb in the action palette" → "⌘Q quits; the action panel has no Quit row (D9)";
  - the Title Case examples → "Open", "Copy", "Resize", "Links…", "More…", "Clear All";
  - "type badges carry … the type in the metadata" → "type tiles carry a glyph and the type's name in the row's
    accessibility label";
  - "text sizes are in the rem scale" stays true (R9): add "(design tokens in points, applied as rems by
    `ui::design::pt`)".

- [ ] **Step 6: Commit.**

```bash
mise run gate
git add PRODUCT.md CONTEXT.md AGENTS.md README.md docs/guides/parity.md docs/guides/compliance.md
git commit -m "docs: Describe the new design system everywhere" \
  -m "CONTEXT, PRODUCT, AGENTS, README, parity and compliance (spec §10)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: The change map, then close Plan 8

**Files:** create `tools/design/CHANGE-MAP.md`; modify `TODO.md`.

- [ ] **Step 1: Write the change map from the actual diff.** Run
      `git diff --stat 41bdc68..HEAD -- crates tools DESIGN.md` and create `tools/design/CHANGE-MAP.md`:

```markdown
# Change map

What a change reaches, and what catches a miss. Written from Plan 8's diff (`41bdc68..<HEAD>`).

| To change          | Edit                                                   | It reaches                                                               | What catches a miss                                                         |
| ------------------ | ------------------------------------------------------ | ------------------------------------------------------------------------ | --------------------------------------------------------------------------- |
| A colour           | `DESIGN.md` front matter `colors`                      | every component (`design().colors`); `mise run design:theme` → `kb.json` | `design:validate` (contrast); `ui::design` test `gives_gpui_kit_the_tokens` |
| A type role        | `DESIGN.md` `typography.roles`                         | components using `text_role(…)`                                          | `ui::design` tests; the shots                                               |
| A size or spacing  | `DESIGN.md` `rounded`, `spacing`, `components`         | the component of the same name                                           | the shots; ast-grep `no-literal-design` keeps it out of views               |
| A label or message | `crates/kb-app/assets/catalog.toml` (or `config.toml`) | the action panel, footer, empty view, notices                            | `catalog` tests                                                             |
| An action's keys   | `catalog.toml` `[actions.<id>] keys`                   | `keys::key_bindings`, keycaps in rows, footer and panel                  | `catalog` validation (parses, bound once); `keys` tests                     |
| A screen's keys    | `catalog.toml` `[screens.<screen>]`                    | `SearchView::ladder`, the footer, the panel's root                       | `catalog` validation; UI tests                                              |
| An entry type      | `catalog.toml` `[types.<key>]`                         | the list, the filter, the counts, the entry view's hero                  | `catalog` validation (icon bundled, tint a token); UI tests                 |
| A component        | `crates/kb-app/src/ui/<component>.rs`                  | every screen that composes it                                            | its unit tests; the UI tests; the shots and `CHECKLISTS.md`                 |
```

Then check each row against the diff: every file the diff touched under `crates/kb-app/src/ui/`, `catalog.toml`
and `DESIGN.md` must appear in a row; add a row for any that does not.

- [ ] **Step 2: Close the tasks.**

```bash
todo status pMDV --set done
todo status VHnE --set done
todo status y3ob --set done
todo add --comment "2026-10-13: Plan 8 executed; shots and checklists in tools/design; change map written" --task-id y3ob
```

(If `VHnE`, the plan review, was not done as its own step, set it done with the comment "reviewed through
execution" — or leave it open if the owner wants a separate review; ask.)

- [ ] **Step 3: Commit.**

```bash
mise run gate
git add tools/design/CHANGE-MAP.md TODO.md
git commit -m "docs(design): Map what each design change reaches" \
  -m "Closes Plan 8 (spec §11)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Done when

- The spec's acceptance criteria 1–8 hold: shots and signed-off checklists (1), the ast-grep rule and the tokens (2),
  `design:validate` (3), the catalog tests (4), the four types and the goldens (5), the count, ⌘1–⌘9 and the notice
  (6), the documents and the change map (7), the gate (8).
