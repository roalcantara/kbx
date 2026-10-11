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
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/docs/experiment/methodology.md#L1:1"
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
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/docs/experiment/results/archive/2026-10-10-pre-polish/README.md#L1:1"
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

- [ ] bQVp **-** Run the Visual Pass (docs/experiment/visual-pass.md) on v0.4.0, compact and expanded; file failures
  - **Priority**: high
  - **Created**: 2026-10-11 09:30
  - **Tags**: app, plan7

- [ ] Mw5p **-** Automate the Visual Pass's 'To automate next' list (docs/experiment/visual-pass.md): Title Case labels; every key shown in the popups and footer runs its action; Esc clears search text; the filter resets on restart; the size holds after hiding; ⌘⇧C, ⌘O and ⌘⇧, run their actions
  - **Comments**: yMdG
  - **Priority**: high
  - **Created**: 2026-10-11 12:51
  - **Tags**: app, visual-pass

- [ ] Xvn5 **-** Review ~/Work/knowledge/docs/tags.proposed.toml (keep, edit, uncomment), save it as docs/tags.toml, git init and commit the knowledge folder, then run mise run tags:apply and tags:apply --write
  - **Priority**: high
  - **Created**: 2026-10-11 13:53
  - **Tags**: tags

- [ ] bVkg **-** Regenerate skills.yml from dots' script/catalog_global_skills.py with the reviewed docs/skills.tags.toml (type skills, no skills tag, 105 tags), after the tag review (Xvn5); review and commit the script in dots
  - **Priority**: high
  - **Created**: 2026-10-11 15:37
  - **Tags**: tags

- [ ] XQgR **-** Tag icons: brainstorm the second sub-project (render tinted in a fixed box as Raycast's tintColor; fill gaps from Simple Icons and Lucide), after the tags are clean
  - **Priority**: medium
  - **Created**: 2026-10-11 13:53
  - **Tags**: app, tags

- [ ] Bdaz **-** Verify Raycast's T3 count (parity.md): if its filter dropdown stays open while picking, Raycast is 4 keys, not 5
  - **Comments**: 9Cpo
  - **Priority**: medium
  - **Created**: 2026-10-11 09:30
  - **Tags**: plan7, report

- [ ] Towo **-** Decide how Raycast's memory run starts, so it is measured in the same state as kb (which memory.sh relaunches)
  - **Comments**: HGSp
  - **Priority**: high
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/bench/probes/memory.sh#L36:1"
  - **Tags**: bench, decision, next

- [ ] pzxe **-** Plan 5 T9: re-run mise run bench:session after the polish, both apps in one sitting
  - **Comments**: TbBw
  - **Priority**: high
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/docs/superpowers/plans/2026-10-09-kb-native-05-mvp-compliance-and-report.md#L1:1"
  - **Tags**: bench, plan5

- [ ] mMjd **-** Plan 5 T10: time Raycast's keystroke → list by hand at 120 fps (10 runs), record it with manual-keystroke.sh, commit the results
  - **Priority**: medium
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/bench/probes/manual-keystroke.sh#L1:1"
  - **Tags**: bench, plan5

- [ ] NRYN **-** Plan 5 T11: mise run report, write findings.md and the README Findings section, regenerate and commit the report
  - **Priority**: medium
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/docs/experiment/methodology.md#L1:1"
  - **Tags**: plan5, report

- [ ] 2pmT **-** Plan 5 T12: close the plan and merge feat/plan5-measurement-session into main
  - **Priority**: medium
  - **Created**: 2026-10-10 02:29
  - **Tags**: git, plan5

- [ ] Y0HE **-** Purge the benchmark kit (tools/bench/README.md, Purge) once the Plan 5 report is merged
  - **Comments**: 6165
  - **Priority**: medium
  - **Created**: 2026-10-11 12:51
  - **Tags**: bench

- [ ] qlkb **-** Type-check tools/graph/graph.ts in hk: it needs TypeScript and Bun's types outside the benchmark kit (checked once with the kit's tsc, strict, 2026-10-10)
  - **Priority**: low
  - **Created**: 2026-10-10 07:17
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/graph/graph.ts#L1:1"
  - **Tags**: graph, quality, tools

- [ ] r3TH **-** Port tools/design/sample.py and the design kit's .mjs scripts to Bun, inline in tools/design/tasks.toml (AGENTS.md)
  - **Priority**: low
  - **Created**: 2026-10-11 13:53
  - **Tags**: tools

- [ ] niEv **-** The notes pane is invisible to VoiceOver: GPUI Kit's TextView sets no accessibility role (0.7.1)
  - **Comments**: qC0Z
  - **Priority**: low
  - **Created**: 2026-10-10 02:29
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/crates/kb-app/src/notes.rs#L1:1"
  - **Tags**: a11y, upstream

- [ ] EsTc **-** Drop the vendored block 0.1.6 patch once GPUI Kit no longer depends on block (objc2/block2 migration)
  - **Priority**: low
  - **Created**: 2026-10-10 02:42
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/crates/third-party/README.md#L1:1"
  - **Tags**: deps, upstream

- [ ] BuLS **-** Run Graphify's semantic pass (mise run graph build graphify --force) once API keys are in fnox
  - **Priority**: low
  - **Created**: 2026-10-10 06:58
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/tools/graph/README.md#L1:1"
  - **Tags**: graph, tools

- [ ] KDYD **-** Evaluate moving tool configs (mise, hk, dprint, typos, ls-lint, jscpd, gitleaks, mdschema, commitlint, cchk, taplo, ast-grep) into .config/
  - **Comments**: JOrA
  - **Priority**: low
  - **Created**: 2026-10-10 03:42
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/README.md#L1:1"
  - **Tags**: config, layout

- [ ] Sdkb **-** Sign kb.app with a Developer ID and notarize it, so macOS opens it without asking
  - **Priority**: low
  - **Created**: 2026-10-10 04:46
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/docs/adr/0006-packaging.md#L1:1"
  - **Tags**: release, signing

- [ ] enBz **-** Release a universal binary (Apple silicon and Intel): today's zip is arm64 only
  - **Priority**: low
  - **Created**: 2026-10-10 04:46
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/mise.toml#L1:1"
  - **Tags**: release

- [ ] zfJD **-** GitHub Pages: a one-page site (a screenshot and the feature list); the repo is public, so Pages is available
  - **Comments**: toCs
  - **Priority**: low
  - **Created**: 2026-10-10 04:46
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/docs/adr/0007-releases.md#L1:1"
  - **Tags**: decision, pages, release

- [ ] DTN3 **-** A Linux build of kb (GPUI supports Linux; needs its system libraries on the runner)
  - **Priority**: low
  - **Created**: 2026-10-10 04:46
  - **Position**: "file:///Users/roalcantara/Work/rust/kb/.github/workflows/ci.yml#L1:1"
  - **Tags**: linux, release

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

- TbBw **-** About 23 minutes hands-off. The first run is archived in docs/experiment/results/archive/2026-10-10-pre-polish/ with why.
  - **Task**: pzxe

- ouCi **-** Saves about 5 of the session's 23 hands-off minutes without changing any measurement.
  - **Task**: oCJR

- S3cM **-** The three probes hard-code the same deeplink the variable holds.
  - **Task**: EYAZ

- qC0Z **-** The only unchecked item of docs/experiment/compliance.md. Upstream fix; until then the row label carries the description.
  - **Task**: niEv

- JOrA **-** Checked 2026-10-10 in a scratch worktree: mise finds .config/mise.toml; cargo deny fell back to its defaults (it reads deny.toml, .deny.toml or .cargo/deny.toml); dprint, typos, taplo and ast-grep were inconclusive. Cargo/rustc configs (Cargo.toml, rust-toolchain.toml, rustfmt.toml, clippy.toml, deny.toml) stay at the root, as in every surveyed repo. Survey: uv, ruff, rust-analyzer, Zed and Servo keep a .config/ folder.
  - **Task**: KDYD

- toCs **-** GitHub Free publishes Pages only from public repositories (checked 2026-10-10). Then: a deploy job in .github/workflows/ci.yml after release (actions/upload-pages-artifact + actions/deploy-pages), from a static page in docs/.
  - **Task**: zfJD

- dW09 **-** Superseded by Plan 6 (design foundation) and Plan 7 (components): docs/superpowers/specs/2026-10-10-ui-polish-design.md.
  - **Task**: hU1X

- j6rR **-** Decision 2026-10-11: no. The code graphs are a local aid for agents (tools/graph/README.md); a published report would add CI time and a page nobody reads.
  - **Task**: 3mTj

- EYc7 **-** Decision 2026-10-11: purge it, after the report (Y0HE). The measurements, results and report stay in docs/experiment.
  - **Task**: jYHt

- gMCt **-** Closed 2026-10-11, won't do: the benchmark kit is purged after the report (Y0HE); five minutes of one remaining session do not justify the change.
  - **Task**: oCJR

- nLHf **-** Closed 2026-10-11, won't do: nothing reads the variable, and it goes with the benchmark kit (Y0HE).
  - **Task**: EYAZ

- yMdG **-** Each item moves from the Visual Pass's manual checks to its 'Already automated' appendix as its test lands, so the manual pass keeps shrinking to what only a person can judge.
  - **Task**: Mw5p

- 6165 **-** Decided in jYHt. It goes after Plan 5's report so the session and the report can still run; tools/bench/README.md lists the steps.
  - **Task**: Y0HE

- 9Cpo **-** Before Plan 5's report (NRYN): the report compares both apps' task costs, so Raycast's T3 count must be measured, not assumed.
  - **Task**: Bdaz

