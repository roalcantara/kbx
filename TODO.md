<!-- todo-project: kb · CLI docs: https://rayanbo.github.io/todo/documentation · usage examples live in the REF* comments, the only place the `todo` CLI keeps them -->

# Tasks

- [x] 6GH4 **-** Plan 5 T1: kb --sync rebuilds the index without a window (39e962b)
  - **Priority**: high
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/crates/kb-app/src/cli.rs#L1:1"
  - **Tags**: app, plan5

- [x] 0TZ2 **-** Plan 5 T2: bench:home, and the kb probes run under its synced home over the private corpus (ed2eeae)
  - **Priority**: high
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/bench/probes/kb-home.sh#L1:1"
  - **Tags**: bench, plan5

- [x] oEGj **-** Plan 5 T3: the Raycast latency run requires KB_RAYCAST_HOTKEY, kb Search's own hotkey (81ec9f3)
  - **Priority**: high
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/bench/probes/latency.sh#L1:1"
  - **Tags**: bench, plan5

- [x] VkWL **-** Plan 5 T4: manual-keystroke.sh records Raycast's hand-timed keystroke run, and the report shows it (cd45960)
  - **Priority**: medium
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/bench/probes/manual-keystroke.sh#L1:1"
  - **Tags**: plan5, report

- [x] mjTU **-** Plan 5 T5: maintainability adds dependency depth, coverage, duplication and clippy warnings (7224357)
  - **Priority**: medium
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/bench/probes/maint.sh#L1:1"
  - **Tags**: plan5, report

- [x] pL64 **-** Plan 5 T6: the report links findings.md from its verdict (7076c6f)
  - **Priority**: medium
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/bench/report/src/report.rs#L1:1"
  - **Tags**: plan5, report

- [x] nHvS **-** Plan 5 T7: bench:cpu, bench:size, bench:session and a preflight (68f678d)
  - **Priority**: high
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/bench/probes/preflight.sh#L1:1"
  - **Tags**: bench, plan5

- [x] bt8n **-** Plan 5 T8: methodology Session and Maintainability sections, and the spec's report amendment (44fedfa)
  - **Priority**: medium
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/docs/guides/methodology.md#L1:1"
  - **Tags**: docs, plan5

- [x] Kkuw **-** kb opens on the list alone, as Raycast's kb Search does; ⌘D shows the split pane (1cc355b)
  - **Comments**: Wgal
  - **Priority**: high
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/crates/kb-app/src/search_view.rs#L99:1"
  - **Tags**: app, parity

- [x] JJ7v **-** Raycast probes stay inside kb Search: clear the field after typing, Esc until the window closes (18be397)
  - **Comments**: z4MO
  - **Priority**: high
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/bench/probes/drive-queries.sh#L1:1"
  - **Tags**: bench

- [x] AmRh **-** Archive the pre-polish tier 2 and 3 runs (a0ee1aa)
  - **Comments**: 7Tbq
  - **Priority**: medium
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/docs/guides/results/archive/2026-10-10-pre-polish/README.md#L1:1"
  - **Tags**: bench, results

- [x] hU1X **-** Visual polish: bring kb's colours, typography and spacing up to Raycast's, with a spec, a plan, side-by-side screenshots and the GPUI Kit Design Guides
  - **Comments**: lh5P,dW09
  - **Priority**: high
  - **Created**: 2026-10-10 02:29
  - **Tags**: next, parity, ui

- [x] oCJR **-** Move bench:maint out of bench:session: it presses no keys, so it need not be in the hands-off window
  - **Comments**: ouCi,gMCt
  - **Priority**: low
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/bench/tasks.toml#L1:1"
  - **Tags**: bench

- [x] EYAZ **-** Read KB_RAYCAST_DEEP_LINK in memory.sh, action.sh and cpu-ts.sh, or drop it from mise.local.toml
  - **Comments**: S3cM,nLHf
  - **Priority**: low
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/bench/probes/memory.sh#L13:1"
  - **Tags**: bench

- [x] tquG **-** Repository layout: three root folders (crates, docs, tools), the benchmark kit isolated in tools/bench, a root-layout hk guard
  - **Priority**: medium
  - **Created**: 2026-10-10 03:42
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/docs/superpowers/specs/2026-10-10-repository-layout-design.md#L1:1"
  - **Tags**: layout, tools

- [x] jYHt **-** Decide whether to keep the benchmark kit after the report, or purge it (tools/bench/README.md, Purge)
  - **Comments**: EYc7
  - **Priority**: low
  - **Created**: 2026-10-10 03:42
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/bench/README.md#L1:1"
  - **Tags**: bench, decision

- [x] jyHv **-** CI and releases: lint and test on Linux for every PR and push to main; semantic-release cuts releases with CHANGELOG.md and kb.app (ADR 0007); tools installed from mise.lock
  - **Priority**: medium
  - **Created**: 2026-10-10 04:46
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/.github/workflows/ci.yml#L1:1"
  - **Tags**: ci, release

- [x] TbiD **-** Code graph tools for agents: CRG, Gortex, Graphify
  - **Created**: 2026-10-10 06:58
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/graph/README.md#L1:1"
  - **Tags**: graph, tools

- [x] 3mTj **-** Decide whether CI should publish a graph report
  - **Comments**: j6rR
  - **Priority**: low
  - **Created**: 2026-10-10 06:58
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/graph/README.md#L1:1"
  - **Tags**: graph, tools

- [x] dN8d **-** Plan 6: design foundation: measure Raycast, pick a validated palette (comparison page), PRODUCT.md, DESIGN.md, apply the theme (68de3a8)
  - **Priority**: high
  - **Created**: 2026-10-10 10:30
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/docs/superpowers/plans/2026-10-10-kb-native-06-design-foundation.md#L1:1"
  - **Tags**: design, next, ui

- [x] eWka **-** Plan 7: components from DESIGN.md: anchored overlays, rows, split pane, footer, detail, media, ⌘⇧F size toggle; parity A01 and S12 (feat/components)
  - **Priority**: high
  - **Created**: 2026-10-10 10:30
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/docs/superpowers/specs/2026-10-10-ui-polish-design.md#L1:1"
  - **Tags**: design, ui

- [x] BAsb **-** Plan 7 T1: seven accents from accents.json; Raycast's type tints and tag hash, tested against it (fdf3b3c)
  - **Priority**: medium
  - **Created**: 2026-10-11 09:29
  - **Tags**: app, plan7

- [x] HIFy **-** Plan 7 T2: Raycast's Title Case labels (858cd98)
  - **Priority**: medium
  - **Created**: 2026-10-11 09:29
  - **Tags**: app, plan7

- [x] YVq7 **-** Plan 7 T3: → / ← toggle the split pane and the metadata, caret-aware; ⌘D alias (37ab079)
  - **Priority**: medium
  - **Created**: 2026-10-11 09:29
  - **Tags**: app, plan7

- [x] 56dX **-** Plan 7 T4: ⌘⇧F compact/expanded window, centred via patched gpui-pre-macos (809a72c)
  - **Priority**: medium
  - **Created**: 2026-10-11 09:30
  - **Tags**: app, plan7

- [x] l5jK **-** Plan 7 T5: header as Raycast's: chevron on the icon axis, funnel filter button (ce8207e)
  - **Priority**: medium
  - **Created**: 2026-10-11 09:30
  - **Tags**: app, plan7

- [x] boCl **-** Plan 7 T6: rows with favicons, tag images (tag_images_path, mise run images), +N, link count (6167d0b)
  - **Priority**: medium
  - **Created**: 2026-10-11 09:30
  - **Tags**: app, plan7

- [x] UJWP **-** Plan 7 T7: remote images through curl (f399dc5); metadata card, ⅓ split, red description bar (f737e7e)
  - **Priority**: medium
  - **Created**: 2026-10-11 09:30
  - **Tags**: app, plan7

- [x] DLzs **-** Plan 7 T8: footer as floating pills, one keycap per key (9ce42e7)
  - **Priority**: medium
  - **Created**: 2026-10-11 09:30
  - **Tags**: app, plan7

- [x] ZNHd **-** Plan 7 T9: actions popover anchored to the footer, keycaps, Links and Copy… pages (e7b65b3)
  - **Priority**: medium
  - **Created**: 2026-10-11 09:30
  - **Tags**: app, plan7

- [x] Al5b **-** Plan 7 T10: filter dropdown anchored and kept open; T3 in one visit (639c023)
  - **Priority**: medium
  - **Created**: 2026-10-11 09:30
  - **Tags**: app, plan7

- [x] Xvn5 **-** Review ~/Work/knowledge/docs/tags.proposed.toml (keep, edit, uncomment), save it as docs/tags.toml, git init and commit the knowledge folder, then run mise run tags:apply and tags:apply --write
  - **Comments**: HjbD
  - **Priority**: high
  - **Created**: 2026-10-11 13:53
  - **Tags**: tags

- [x] XQgR **-** Tag icons: brainstorm the second sub-project (render tinted in a fixed box as Raycast's tintColor; fill gaps from Simple Icons and Lucide), after the tags are clean
  - **Comments**: ubm7
  - **Priority**: medium
  - **Created**: 2026-10-11 13:53
  - **Tags**: app, tags

- [x] r3TH **-** Port tools/design/sample.py and the design kit's .mjs scripts to Bun, inline in tools/design/tasks.toml (AGENTS.md)
  - **Comments**: gHB7
  - **Priority**: low
  - **Created**: 2026-10-11 13:53
  - **Tags**: tools

- [x] niEv **-** The notes pane is invisible to VoiceOver: GPUI Kit's TextView sets no accessibility role (0.7.1)
  - **Comments**: qC0Z,ITyZ
  - **Priority**: low
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/crates/kb-app/src/notes.rs#L1:1"
  - **Tags**: a11y, upstream

- [x] zfJD **-** GitHub Pages: a one-page site (a screenshot and the feature list); the repo is public, so Pages is available
  - **Comments**: toCs,Uj0P
  - **Priority**: low
  - **Created**: 2026-10-10 04:46
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/docs/adr/0007-releases.md#L1:1"
  - **Tags**: decision, pages, release

- [x] elwx **-** Plan 8, 01 List in the companion: the owner's screen as reference, every chip and keycap 20 pt, one keycap component, chip state (live, rest) sets fill and text; approved as v5
  - **Comments**: jBhm
  - **Priority**: high
  - **Created**: 2026-10-12 17:19
  - **Tags**: design, plan8

- [x] OKAo **-** Plan 8, 02 Actions in the companion: the 01 rules on the actions popover; owner approves
  - **Comments**: 3kaC,QQca,303s,1xKt,ARsG,jm7r
  - **Priority**: high
  - **Created**: 2026-10-12 17:19
  - **Tags**: design, plan8

- [x] w5fY **-** Plan 8, 03 Empty in the companion: the no-match state; owner approves
  - **Comments**: QbWV,GxRT
  - **Priority**: high
  - **Created**: 2026-10-12 17:19
  - **Tags**: design, plan8

- [x] n4kx **-** Plan 8, 10 No index in the companion; owner approves
  - **Comments**: knER
  - **Priority**: medium
  - **Created**: 2026-10-12 17:19
  - **Tags**: design, plan8

- [x] Z64Y **-** Plan 8, 04 Split in the companion: list and split pane; owner approves
  - **Comments**: oVNz,xpAF,QU41,dNCU,nkjz,cObJ
  - **Priority**: high
  - **Created**: 2026-10-12 17:19
  - **Tags**: design, plan8

- [x] sTbE **-** Plan 8, 05-06 Detail in the companion: drawn from the rules (no Stitch screen), with and without metadata; owner approves
  - **Comments**: m0LI
  - **Priority**: high
  - **Created**: 2026-10-12 17:19
  - **Tags**: design, plan8

- [x] kzv4 **-** Plan 8, 07 Filter in the companion: drawn from the rules (no Stitch screen); owner approves
  - **Comments**: qUan,8liG
  - **Priority**: high
  - **Created**: 2026-10-12 17:19
  - **Tags**: design, plan8

- [x] 3XFj **-** Plan 8, 08-09 Actions pages in the companion: Links and Copy; owner approves
  - **Comments**: 1AzQ,y0NF
  - **Priority**: high
  - **Created**: 2026-10-12 17:19
  - **Tags**: design, plan8

- [x] prlL **-** Plan 8, 11-14 Rebuild in the companion: running, report, stopped, failed; owner approves
  - **Comments**: 9SZI
  - **Priority**: medium
  - **Created**: 2026-10-12 17:19
  - **Tags**: design, plan8

- [x] ER8Q **-** Plan 8, 15-16 Bookmark media in the companion: thumbnail and favicon, split and detail; owner approves
  - **Comments**: RQzn
  - **Priority**: medium
  - **Created**: 2026-10-12 17:19
  - **Tags**: design, plan8

- [x] r58y **-** Plan 8: rebuild the prototypes in docs/guides/design from the approved companion screens, on the showcase corpus (replaces the 0fcaddc set)
  - **Comments**: dPmR
  - **Priority**: high
  - **Created**: 2026-10-12 17:19
  - **Tags**: design, plan8

- [ ] nNln **-** Plan 8: add a realistic showcase corpus to the repo (committed YAML) that the prototypes and design:shots both use
  - **Comments**: Vx4Z
  - **Priority**: high
  - **Created**: 2026-10-12 17:19
  - **Tags**: design, plan8

- [x] 7lTb **-** Narrow the entry types to Bookmarks, Cheat Sheets, Commands and Tasks across the app; any other type is ignored (owner, 2026-10-12)
  - **Comments**: Bo3f,28mD
  - **Priority**: high
  - **Created**: 2026-10-12 23:09
  - **Tags**: app, plan8

- [x] RKHF **-** Plan 8: catalog: crates/kb-app/assets/catalog.toml (entry types: label, icon, tint, primary, secondary; actions: label, keys; screens: ↵ ⌘↵ ⇧⌘↵; messages), deep-merged under ~/.config/kb/config.toml (same keys win), validated at start with a notice and per-key fallback; colours and sizes stay in DESIGN.md; Configure (⇧⌘,) creates the file with the defaults as comments
  - **Comments**: IF60

- [x] KhLt **-** Plan 8: rewrite the spec around the prototypes: the owner's screens canonical, the rules as tokens, a checklist per screen, the ui components, the change map
  - **Comments**: LGMU,ixvo,oSwX

- [x] pMDV **-** Plan 8: write the plan from the spec
  - **Comments**: gvTO,HuqS

- [x] VHnE **-** Plan 8: review the plan until no findings remain

- [~] y3ob **-** Plan 8: execute the plan (another agent), then verify the app against the prototypes
  - **Comments**: CT5F,HZc2,mmdD

- [ ] SeKS **-** Paste: hide kb and paste the entry into the app it came from (Raycast's Paste to Active App; needs Accessibility); becomes the Command and Cheat primary, Copy their secondary (one catalog change)

- [ ] bVkg **-** Review and commit dots' script/catalog_global_skills.py (writes cheats tagged skills from docs/skills.tags.toml; default output overwrites skills.yml, so use --output); skills.yml's tags already match the map

- [ ] Bdaz **-** Verify Raycast's T3 count (parity.md): if its filter dropdown stays open while picking, Raycast is 4 keys, not 5

- [ ] Towo **-** Decide how Raycast's memory run starts, so it is measured in the same state as kb (which memory.sh relaunches)

- [ ] pzxe **-** Plan 5 T9: re-run mise run bench:session after the polish, both apps in one sitting

- [ ] mMjd **-** Plan 5 T10: time Raycast's keystroke → list by hand at 120 fps (10 runs), record it with manual-keystroke.sh, commit the results

- [ ] NRYN **-** Plan 5 T11: mise run report, write findings.md and the README Findings section, regenerate and commit the report

- [ ] 2pmT **-** Plan 5 T12: close the plan and merge feat/plan5-measurement-session into main

- [ ] Y0HE **-** Purge the benchmark kit (tools/bench/README.md, Purge) once the Plan 5 report is merged

- [ ] qlkb **-** Type-check tools/graph/graph.ts in hk: it needs TypeScript and Bun's types outside the benchmark kit (checked once with the kit's tsc, strict, 2026-10-10)

- [ ] EsTc **-** Drop the vendored block 0.1.6 patch once GPUI Kit no longer depends on block (objc2/block2 migration)

- [ ] BuLS **-** Run Graphify's semantic pass (mise run graph build graphify --force) once API keys are in fnox

- [ ] KDYD **-** Evaluate moving tool configs (mise, hk, dprint, typos, ls-lint, jscpd, gitleaks, mdschema, commitlint, cchk, taplo, ast-grep) into .config/

- [ ] Sdkb **-** Sign kb.app with a Developer ID and notarize it, so macOS opens it without asking

- [ ] enBz **-** Release a universal binary (Apple silicon and Intel): today's zip is arm64 only

- [ ] DTN3 **-** A Linux build of kb (GPUI supports Linux; needs its system libraries on the runner)

- [ ] MOB4 **-** Plan 8, 17 Toast: restyle the notice (the owner is not happy with it); copy feedback lands meanwhile

- [ ] 3g6H **-** Execute/Run Command: run a command entry in the terminal (proposed in iriskb.augment and iriskb.cursor; executes text from the source files, so decide its safety first)

- [ ] rPyR **-** Notes code blocks: render fenced code in the Body with GPUI Kit's read-only code Editor (line numbers, Clipboard copy), as dbflux's SQL preview does, instead of TextView's code block; needs the notes Markdown split around code blocks
  - **Comments**: lsQ5

- [ ] miT3 **-** Watch Nohrs (noh-rs/nohrs, Raycast-style GPUI launcher, MIT): when its plugin host ships (0.3.0), check whether kb-domain could run as a Nohrs plugin

- [ ] Th4B **-** Idea: read massCode's Markdown vault as a second kind of source beside the YAML, so massCode can be the editor and kb the launcher (owner-approved idea, 2026-10-12; only if a GUI editor is wanted)
  - **Priority**: low
  - **Created**: 2026-10-12 23:33
  - **Tags**: idea

- [ ] CKx2 **-** Engine: accept every link shape the corpus uses: a bare URL (links: https://…) and a top-level title: url map, besides the two list forms; both engines drop them today (2 entries in the private corpus lose their links). After the experiment's report, since it changes kb-domain's output against the TS goldens; until then the sync report could flag them
  - **Priority**: medium
  - **Created**: 2026-10-13 00:30
  - **Tags**: domain

- [ ] ZF8t **-** Adoption: import navi, cheat and pet cheat files into kb (navi's library crate, Apache-2.0); after the experiment's report (ADR 0008)
  - **Priority**: medium
  - **Created**: 2026-10-13 14:50
  - **Tags**: adoption, engine

- [ ] E78j **-** Adoption: import shell history from Atuin's SQLite database as command entries; after the experiment's report (ADR 0008)
  - **Priority**: low
  - **Created**: 2026-10-13 14:50
  - **Tags**: adoption, engine

- [ ] uAa3 **-** Adoption: bundle tldr pages as built-in content so kb is useful on first launch (CC BY 4.0 attribution); after the experiment's report (ADR 0008)
  - **Priority**: medium
  - **Created**: 2026-10-13 14:50
  - **Tags**: adoption, content

- [ ] 8qUa **-** Notes body: adopt zorite-markdown =0.10.0 (MIT, on kb's gpui-pre 0.3 line) in ui/entry_view.rs in place of TextView: MarkdownStyle from the design tokens (notes roles, code background), code highlighting through on_highlight with GPUI Kit's SyntaxHighlighter, links through cx.open_url; owner choice 2026-10-13 (Plan 8f Task 5)
  - **Created**: 2026-10-13 19:52

- [ ] 4hQn **-** Notes bold: **bold** renders regular in the notes body (with TextView and zorite-markdown alike): kb bundles Inter Regular, Medium, SemiBold and Italic but no Bold; bundle Inter Bold or map bold to SemiBold
  - **Created**: 2026-10-13 19:52

- [x] Z5GP **-** Action panel and filter take no keyboard in the running app: after ⌘K (and ⌘P), ↓ and typing reach nothing and ↵ runs the list's Details; the filter's footer keeps the list's keys. UI tests pass (the harness routes keys differently). Found by design:shots, 2026-10-14; More… is not captured until it is fixed
  - **Comments**: aSat
  - **Priority**: high
  - **Created**: 2026-10-14 02:53

# Actors

# Comments

- REF1 **-** Source and docs: https://github.com/RayanBO/todo · https://rayanbo.github.io/todo/documentation (pinned in mise.toml as cargo:https://github.com/RayanBO/todo/)

- REF2 **-** List: `todo list --tasks` (the pinned v0.2.9 filters with --tag, --priority, --search, --overdue, e.g. `todo list --tasks --tag next`)

- REF3 **-** Add: `todo add --task "Measure X" --priority high --tag bench,next` (--tag, not --tags, in v0.2.9)

- REF4 **-** Status: `todo status a1b2 --set done` (todo, en-cours, done, bloqued)

- REF5 **-** Keep notes as comments (`todo add --comment "..." --task-id <id>`): the CLI rewrites this file and drops front matter, HTML comments other than the todo-project line, and free-text lines under a task

- Wgal **-** The first session measured kb with the notes pane open against Raycast's bare list: more work per keystroke, more memory. Parity row S09 now states the default.
  - **Task**: Kkuw

- z4MO **-** Found on screen by the owner: ⌫ in Raycast's empty search field pops to its root search, so all 100 memory-run queries went there.
  - **Task**: JJ7v

- 7Tbq **-** Tier 1 stays in results/: the engine is unchanged and Plan 5 does not re-run it.
  - **Task**: AmRh

- lh5P **-** Owner, 2026-10-10: kb resembles Raycast but its colours, typography and spacing look off. Measure after it: a bundled font or window translucency changes memory, size and first-frame time; colours and spacing do not.
  - **Task**: hU1X

- HGSp **-** A rerun right after the session's CPU, latency and ⌘↩ runs read 505 MiB idle against 178 MiB earlier. Relaunching Raycast in memory.sh, as cpu-ts.sh does, was proposed and set aside for now. Owner: run Raycast's memory after the polish.
  - **Task**: Towo

- TbBw **-** About 23 minutes hands-off. The first run is archived in docs/guides/results/archive/2026-10-10-pre-polish/ with why.
  - **Task**: pzxe

- ouCi **-** Saves about 5 of the session's 23 hands-off minutes without changing any measurement.
  - **Task**: oCJR

- S3cM **-** The three probes hard-code the same deeplink the variable holds.
  - **Task**: EYAZ

- qC0Z **-** The only unchecked item of docs/guides/compliance.md. Upstream fix; until then the row label carries the description.
  - **Task**: niEv

- JOrA **-** Checked 2026-10-10 in a scratch worktree: mise finds .config/mise.toml; cargo deny fell back to its defaults (it reads deny.toml, .deny.toml or .cargo/deny.toml); dprint, typos, taplo and ast-grep were inconclusive. Cargo/rustc configs (Cargo.toml, rust-toolchain.toml, rustfmt.toml, clippy.toml, deny.toml) stay at the root, as in every surveyed repo. Survey: uv, ruff, rust-analyzer, Zed and Servo keep a .config/ folder.
  - **Task**: KDYD

- toCs **-** GitHub Free publishes Pages only from public repositories (checked 2026-10-10). Then: a deploy job in .github/workflows/ci.yml after release (actions/upload-pages-artifact + actions/deploy-pages), from a static page in docs/.
  - **Task**: zfJD

- dW09 **-** Superseded by Plan 6 (design foundation) and Plan 7 (components): docs/superpowers/specs/2026-10-10-ui-polish-design.md.
  - **Task**: hU1X

- j6rR **-** Decision 2026-10-11: no. The code graphs are a local aid for agents (tools/graph/README.md); a published report would add CI time and a page nobody reads.
  - **Task**: 3mTj

- EYc7 **-** Decision 2026-10-11: purge it, after the report (Y0HE). The measurements, results and report stay in docs/guides.
  - **Task**: jYHt

- gMCt **-** Closed 2026-10-11, won't do: the benchmark kit is purged after the report (Y0HE); five minutes of one remaining session do not justify the change.
  - **Task**: oCJR

- nLHf **-** Closed 2026-10-11, won't do: nothing reads the variable, and it goes with the benchmark kit (Y0HE).
  - **Task**: EYAZ

- 6165 **-** Decided in jYHt. It goes after Plan 5's report so the session and the report can still run; tools/bench/README.md lists the steps.
  - **Task**: Y0HE

- 9Cpo **-** Before Plan 5's report (NRYN): the report compares both apps' task costs, so Raycast's T3 count must be measured, not assumed.
  - **Task**: Bdaz

- HjbD **-** Done 2026-10-12: 107 merges accepted (Stack Overflow master tags longer than the tag in use, and amazon/google, rejected), applied to 48 files; every tag follows the convention.
  - **Task**: Xvn5

- ubm7 **-** Done 2026-10-12 without a separate brainstorm: the image is a mask in the pill's text colour inside the pill; tags:images fills the folder from docs/tags.images.toml, brands from Simple Icons, concepts from Lucide; 375 of 563 tags have one.
  - **Task**: XQgR

- gHB7 **-** Done 2026-10-12: color.ts (tested) and the design: tasks, inline Bun in tools/design/tasks.toml; the outputs regenerate byte for byte (raycast.json writes 1049 for 1049.0).
  - **Task**: r3TH

- ITyZ **-** Archived 2026-10-12 at the owner's request.
  - **Task**: niEv

- Uj0P **-** Archived 2026-10-12 at the owner's request.
  - **Task**: zfJD

- jBhm **-** 2026-10-12: decided with the owner in the brainstorming companion (v1→v5): their Stitch screens are the reference, not the generated DESIGN.md text; the earlier spec (27918b2) and prototypes (0fcaddc) are superseded. Rules: Catppuccin Macchiato, 14 pt window, hairlines, Inter + JetBrains Mono; every chip and keycap 20 pt; one keycap (one cap per shortcut, mono 11, muted on mantle; the toast's keeps the grey fill); chip live (mantle, muted text) on keycaps and the selected row, rest (grey) elsewhere; magnifier on the list, back chevron elsewhere; no deploy/api chip; tag icons from the tag-images folder; type colours as drawn; Lucide for Material; ⌘V hint; ⌘1–⌘9 row shortcuts built.
  - **Task**: elwx

- 3kaC **-** 2026-10-12: approved as v4 in the companion. One popover, 220 pt, no icons: Details ↵, Copy ⌘↵ (Open on bookmarks: their main action opens the URL; one name whatever the type), More… › (the page of other copies: title, description, notes, YAML), Split ⌘D, Source ⌘O, Toggle Size ⇧⌘F; a hairline; ADMINISTRATION: Reindex ⌘R, Preferences ⇧⌘,. Dropped: Quit kb (⌘Q still quits), Copy YAML Snippet, Configure Filter, Run in Terminal (not now). Footer reads Details ↵. Goal for every popover: the smallest comfortable size, no redundant or rarely used rows.
  - **Task**: OKAo

- QQca **-** 2026-10-12, v5: order Details ↵, Copy ⌘↵, Split ⌘D, Resize ⇧⌘F (was Toggle Size), Source ⌘O, More… ›; a hairline; ADMINISTRATION: Reindex ⌘R, Preferences ⇧⌘,.
  - **Task**: OKAo

- 303s **-** 2026-10-12, v6: the popover's search placeholder reads Search. Simplifications closed.
  - **Task**: OKAo

- 1xKt **-** 2026-10-12, v7 labels (one word each, Raycast's words where it has one): Details ↵, Copy ⌘↵ (Open on bookmarks), Preview ⌘D (the split pane), Resize ⇧⌘F, Edit ⌘O (opens the entry's YAML in the editor), More… ›; ADMINISTRATION: Refresh ⌘R (the sync; CONTEXT.md keeps 'Sync'), Configure ⇧⌘, (Raycast's 'Configure Extension'). The popover is closed.
  - **Task**: OKAo

- ARsG **-** 2026-10-12, final (companion actions-final-v2): Raycast layout, 28 pt rows, search at the bottom, sections by a line. Main panel (218 pt): Details ↵, Copy ⌘↵ (Open on bookmarks), Preview ⌘D, Resize ⇧⌘F, Edit ⌘O, More… ›. More… is a submenu with no search of its own and a ‹ More header: LINKS (one row per link, opens it; only when the entry has links), COPY (URL ⇧⌘C on bookmarks, Title, Description, Notes, Path = the source file's absolute path; rows only when the entry has them), ADMINISTRATION (Refresh ⌘R, Configure ⇧⌘,). The main panel's search also finds More…'s rows, marked 'More'. Raw YAML copy dropped (Path + Edit instead). Shortcuts fire from the list wherever an action is listed.
  - **Task**: OKAo

- jm7r **-** 2026-10-12, final (superseding the earlier v-notes): one action component for every screen, Raycast layout (28 pt rows, search at the bottom, sections by a line, at most two levels). Primary/secondary model: entry types define primary + optional secondary (Bookmark: Open in the default browser / Copy URL; Command, Cheat: Copy, later Paste / Copy; Note, Task, Other: Copy / none); screens map the key ladder ↵, ⌘↵, ⇧⌘↵: List ↵ Details, ⌘↵ entry primary, ⇧⌘↵ entry secondary; Detail ↵ entry primary, ⌘↵ entry secondary. Root panel: screen's keys, Resize ⇧⌘F, Links… › (only with links; each link row = the bookmark pair), More… ›. More… (no search of its own; the root's search finds its rows): COPY Description, Notes, Path (absolute) │ ADMINISTRATION Edit ⌘O, Refresh ⌘R, Configure ⇧⌘,. Dropped: Preview/Info rows and ⌘D (← → toggle them), Title copy (Copy does the job), raw YAML copy. Open: a footer hint for ← → (to settle with 04/05).
  - **Task**: OKAo

- 1AzQ **-** 2026-10-12: scope changed by the 02 decisions: the Links… and More… submenus (Raycast submenus, no search of their own), not the old Links and Copy pages.
  - **Task**: 3XFj

- QbWV **-** 2026-10-12: each screen also gets its ↵ / ⌘↵ pair from the catalog (proposed: no match ↵ Clear Search, ⌘↵ Clear Filters; no index ↵ Refresh, ⌘↵ Configure; filter ↵ Toggle, ⌘↵ Only this; rebuild report ↵ Back, stopped ↵ Retry, failed ↵ Retry, ⌘↵ Configure); confirm each in its companion pass.
  - **Task**: w5fY

- GxRT **-** 2026-10-12, approved v2: Raycast's empty view as is (List.EmptyView): one large search icon, 'No matching entries found', 'No entries match "<query>"' (query in mauve mono; '… with this filter' when one is set); no badge circle, no hint chips, no ⌘N (kb never writes sources). Footer: status 'Ready • 0 results', primary Refresh ↵ (Raycast's Rebuild Index…); Esc clears the query as everywhere. Supersedes the ↵ Clear Search proposal. Builds on the toolkit's Empty already used.
  - **Task**: w5fY

- knER **-** 2026-10-12: folded into 03 (same Raycast empty view): a database icon, 'No entries indexed yet', 'Press Refresh to build the index from your sources.'; status '● No index'; footer Refresh ↵.
  - **Task**: n4kx

- oVNz **-** 2026-10-12, command hero (owner): native card B (mantle box, JetBrains Mono, no window bar, no ❯), lines broken at the operators (&&, ||, |) when the chain is wider than the pane. Buy, don't build: formatting by fmt-rs =0.1.0 (MIT, pure Rust, no deps; tried on real commands; vendor into crates/third-party if abandoned); colours by GPUI Kit's SyntaxHighlighter with the tree-sitter-bash grammar kb already enables (function, constant = flags, string, property = vars, embedded, operator, number, comment; plain values stay text colour) mapped to the design tokens; drawn as GPUI StyledText in a CommandCard component. Used as the hero of commands and of cheats with a snippet; Markdown code in notes keeps TextView. Replaces the fabrix SVG idea (owner's earlier Raycast workaround).
  - **Task**: Z64Y

- xpAF **-** 2026-10-12, pane template (owner, to render next): split and detail share one component; slots Header (title left, tags right), Hero, Summary (description), Body (notes); per-type fill from the catalog (command: no title, hero CommandCard; bookmark: URL title, hero YouTube thumbnail, og:image later; cheat: title, hero snippet card when present; note: title; task: task, hero status+priority badges; other: title); empty slots collapse; no type badge or path in the pane; Links only in the Links… submenu. Footer: left context (status line on the list, [type] file ⌥⌘C copy-path when a pane or detail is open), right the screen's ↵ / ⌘↵ / ⇧⌘↵ actions and Actions ⌘K. Stored notes_markdown stays (TS goldens); kb-domain adds a body-only Markdown function.
  - **Task**: Z64Y

- QU41 **-** 2026-10-12, template v2 (owner): one header row for every entry: [the row's icon: favicon for bookmarks, the type tile otherwise] [header text by type: bookmark URL (link style), cheat title, command description (else title), note title, task text] … tags right; at most 2 lines in the split, whole in detail. Summary = the description unless the header used it (so commands have none). The footer's left always shows the type tile + file + ⌥⌘C. Header text per type lives in the catalog with the actions.
  - **Task**: Z64Y

- dNCU **-** 2026-10-12, template v3 (owner, supersedes v2's per-type header): four fixed slots for every entry, only the hero varies by type. 1 Header: [the row's icon: favicon for bookmarks, type tile otherwise] the title (title field, else the key), one line trimmed, tags right. 2 Hero by type: command → CommandCard; bookmark → thumbnail (YouTube; og:image later) else the URL as a link line (↗ host/path); cheat → snippet card when it has one; task → status + priority badges; note, other → none. 3 Quote: the description (desc or description), whole, red bar; omitted when absent. 4 Body: the notes. Split = the same component, hero capped. Footer left: type tile + file + ⌥⌘C.
  - **Task**: Z64Y

- nkjz **-** 2026-10-12, command card decided after a full-screen GPUI spike (real renders) and a survey (paperoll, probe, pasta, Loungy, markraft, dbflux): the hero is option (b), StyledText + GPUI Kit's bash highlighter, as pasta does: fmt-rs breaks at operators to the card's real column count; kb pre-wraps at spaces with a hanging indent, operators glued, tokens never split; ligatures off; 12 pt in the split, 13 pt in detail; GPUI Kit's Clipboard button in the corner; no line numbers. Register bash with an extended highlight query (dbflux-style, LanguageRegistry::register) so argument values get their own colour; test each token's colour role. Fixes from the spike: notes headings take the type tokens (TextView styles), the quote's italic needs Inter Italic bundled (or drop italic), favicons come from kb's curl client.
  - **Task**: Z64Y

- 8Ylj **-** 2026-10-12 survey: buy a terminal view instead of building output handling: gpui_xterm 0.2.1 (alacritty_terminal on GPUI Kit 0.6; kb is on 0.7.1, check before use) runs the command in an embedded terminal pane; tty7, KazeTerm and Miaominal are GPUI terminal apps on alacritty_terminal for reference. Prior art for runnable commands: Runme (DevOps notebooks from Markdown), Atuin Desktop (runbooks, archived), navi (argument placeholders with suggestion commands).
  - **Task**: 3g6H

- 2A23 **-** 2026-10-12, mechanism-level prior art for showing a command and running it: GPUI API clients on kb's GPUI (probe, Arc, Broquest, GetCat, Setu, Protide: request body in GPUI Kit's Editor, run, response pane, environments/variables); notebooks (marimo, Jupyter: output under the cell); Runme (Markdown code blocks as runnable cells; kb's notes already hold code blocks); navi (argument placeholders with suggestion commands). Read the GPUI API clients first when this task starts.
  - **Task**: 3g6H

- cObJ **-** 2026-10-12: closed by the full-screen GPUI spike (four real screens): template v3, command card (b), footer with the type tile + file + ⌥⌘C and the screen's ↵/⌘↵/⇧⌘↵; 12 pt card in the split, 13 pt in detail; the count leaves the footer while a pane is open (← brings it back). Detail (05–06) is the same template at full width; → shows the metadata column.
  - **Task**: Z64Y

- m0LI **-** 2026-10-12: folded into 04 (same component at full width). Still to settle in the spec: the metadata column's look (→).
  - **Task**: sTbE

- qUan **-** 2026-10-12, approved (brainstorm 07-filter.html): the action component reused under the filter pill; search at the top (rule: the search sits next to the control that opened it); 28 pt rows; TYPES with their tile, TAGS with their icon or #; only chosen rows show ✓; picking keeps it open; with filters set a 'n filters · Clear All' row leads and the pill names the filters (Commands, #k8s; truncated); footer: Toggle ↵ · Only ⌘↵ · Actions ⌘K. Types offered: Bookmarks, Cheat Sheets, Commands, Tasks only (owner).
  - **Task**: kzv4

- Bo3f **-** 2026-10-12, where to ignore them: (a) in the app (catalog lists the four types; list, filter and counts skip the rest) keeps kb-domain and the index identical to the TypeScript engine, so the goldens and the experiment's parity hold; (b) at sync (unknown types skipped, the sync report says so) is cleaner but changes the engine's output, which the TS goldens check (CONTEXT.md: 'Unknown keys are kept, not rejected'). Recommended: (a) in Plan 8, (b) after the experiment's report. Also update: CONTEXT.md (Entry type), the catalog, the type tints (Notes' peach and Other's hash fall away), the template rows for notes, parity.md, and the sample and showcase corpora.
  - **Task**: 7lTb

- 8liG **-** 2026-10-12, correction (owner): the filter's search sits at the bottom, as Raycast's and as the actions panel's (07-filter.html v4); my 'search next to its trigger' rule is withdrawn.
  - **Task**: kzv4

- y0NF **-** 2026-10-12: settled in 02 (02-actions.html): Links… › (only when the entry has links; one row per link; ↵ opens, ⌘↵ copies, the bookmark pair) and More… › (COPY Description, Notes, Path, URL on bookmarks │ ADMINISTRATION Edit, Refresh, Configure); no search of their own, the root's search finds their rows; ‹ header; at most two levels.
  - **Task**: 3XFj

- 9SZI **-** 2026-10-12 (11-14-refresh.html, 01-list v5 + a script, nothing new designed): header back chevron only; running, stopped, failed use 03's empty view (large icon, title, sentence; running adds a thin progress bar and 'n of m files · current file'); the report uses 01's rows, one per file worth a look, failed first (tile ✕ failed, ⚠ skipped entries, ✓ indexed; chip: failed / count), a summary line on top. Keys: running Stop esc; report Back ↵; stopped Retry ↵; failed Retry ↵ · Configure ⌘↵. The action is named Refresh everywhere (Sync in the code).
  - **Task**: prlL

- RQzn **-** 2026-10-12: settled by template v3 (04-entry-template.html): hero = the thumbnail (YouTube now, og:image later), capped in the split, full width in detail; otherwise the URL link line; the favicon leads the header and the list row.
  - **Task**: ER8Q

- dPmR **-** 2026-10-12, won't do as a separate step (owner): the approved pages in docs/guides/design/ are the visual contract; the plan replaces docs/guides/design/ with them and points design:shots at them.
  - **Task**: r58y

- Vx4Z **-** 2026-10-12 (owner): becomes the plan's first task, since both the pages and the app's screenshots need it.
  - **Task**: nNln

- LGMU **-** 2026-10-12, decided before writing the spec: keep evolving kb rather than port massCode's slice (~55k lines of Vue/TS, an editing workspace without a hotkey panel) to GPUI; kb's design is bought from Raycast, the best-tested answer for its job.
  - **Task**: KhLt

- ixvo **-** 2026-10-13, detail links (owner): A1 compact Links list at the end of the Body (05-detail-links.html), no metadata column (every attribute has a home), one line per link (title trimmed with an ellipsis, URL without scheme/www/query/fragment and trimmed in the middle, full text in a tooltip), ⌘1–⌘9 open links, clickable header tags filter. Spec: D11 updated, D20 added, 04–06 checklist updated. Link shapes: only the two list forms are parsed by both engines; the other two queued as an engine task.
  - **Task**: KhLt

- gvTO **-** 2026-10-13 O1: A (owner)
  - **Task**: pMDV

- IF60 **-** 2026-10-13: Plan 8b
  - **Task**: RKHF

- 28mD **-** 2026-10-13: app-side, Plan 8c Task 4 (Bo3f option a)
  - **Task**: 7lTb

- oSwX **-** 2026-10-13: approved by the owner (rewritten spec, D1-D20)
  - **Task**: KhLt

- HuqS **-** 2026-10-13: index and sub-plans 8a-8e written; 8f-8h next
  - **Task**: pMDV

- lsQ5 **-** 2026-10-13: notes body — adopt zorite-markdown (owner, Plan 8f Task 5, screenshots compared); its on_highlight hook may replace this task's Editor approach for code blocks
  - **Task**: rPyR

- CT5F **-** 2026-10-13 O2: A (owner)
  - **Task**: y3ob

- HZc2 **-** 2026-10-13: Plan 8 executed; shots and checklists in tools/design; change map written
  - **Task**: y3ob

- mmdD **-** 2026-10-14: reopened; 8h review found 5 of 13 compact shots show another screen and the checklist ticked them; the sign-off came from a run whose Gemini turns aborted on exhausted quota (owner)
  - **Task**: y3ob

- aSat **-** 2026-10-15: fixed; the popover tracked the panel container, not its search field, so the root's search never had the keyboard; the filter's footer also ignored the open overlay (owner reproduced it by hand; the fix is checked in the More… and filter shots)
  - **Task**: Z5GP

