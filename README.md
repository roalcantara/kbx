# kbx

## Project Name and Description

kbx is a macOS panel, opened from anywhere with ⌥K, that searches a personal knowledge base: bookmarks,
cheat sheets, commands, tasks and agent skills kept in plain YAML files. Type a few letters, press Return, and the
entry opens or is copied. It is written in Rust on [GPUI Kit](https://gpui-kit.com), indexes with SQLite FTS5, and
is inspired by Raycast.

**Who it's for.** Keyboard-first developers on macOS who keep their own commands, cheat sheets, bookmarks, notes and
tasks, and want them one keystroke away from any app instead of in a browser tab or a terminal history. I use it
dozens of times a day over my own knowledge base; the repository ships a made-up sample corpus for tests and
screenshots.

## Technology Stack

- Rust 1.99 (edition 2024), Cargo workspace
- GPUI Kit for the UI
- SQLite with FTS5 through `rusqlite`
- YAML through saphyr; JSON through serde_json
- mise for tool versions and tasks; hk for git hooks and quality steps; Bun for the scripts in `tools/`

## Project Architecture

| Crate / folder      | Role                                                                                        |
| ------------------- | ------------------------------------------------------------------------------------------- |
| `crates/kbx-domain` | The engine: entries, source files, notes Markdown, the SQLite index and its queries. No UI. |
| `crates/kbx-app`    | The GPUI Kit application: a resident panel opened by a global hotkey.                       |

`kbx-domain` never depends on a UI crate; the `domain-purity` hk step enforces it. See [CONTEXT.md](CONTEXT.md)
for the domain vocabulary and [docs/adr](docs/adr) for the decisions.

## Getting Started

**Quickstart.**

```bash
mise run setup        # tools, crates, git hooks
mise run install      # build kbx.app and copy it to ~/Applications
open ~/Applications/kbx.app
```

Then press **⌥K** anywhere to open the panel, and **⌘R** once to build the index from your sources.

| Key   | Does                                                 |
| ----- | ---------------------------------------------------- |
| type  | search titles, descriptions and notes                |
| ↑ ↓   | move the selection; on an entry, scroll its notes    |
| ⌘↑ ⌘↓ | on an entry, scroll its notes a page                 |
| ↩     | show the entry; on the entry, run its main action    |
| ⌘↩    | run the main action: open a bookmark, copy a command |
| ⌘K    | all actions for the selected entry                   |
| ⌘P    | filter by type and tag                               |
| → ←   | show or hide the details pane (at the query's end)   |
| ⌘R    | rebuild the index                                    |
| ⌘⇧C   | copy a bookmark's URL                                |
| ⌘O    | open the entry's source file                         |
| ⌘⇧,   | open the preferences file                            |
| ⌘⇧F   | switch between the compact and expanded window       |
| Esc   | close a popover, go back, then hide the panel        |
| ⌘Q    | quit kbx                                             |

kbx has no Dock icon: it lives behind ⌥K until you quit it. To start it at login, add `~/Applications/kbx.app` in
System Settings › General › Login Items. If another app holds ⌥K, kbx says so when it opens.

The sources are `~/.config/kodexb/sources` unless `~/.config/kbx/config.toml` says otherwise:

```toml
sources_path = "~/Work/knowledge"
tag_images_path = "~/.config/kbx/images" # <tag>.svg draws #tag
```

The index lives in `~/Library/Application Support/kbx/kbx.db`; ⌘R rebuilds it from the sources.

**Working on it.**

```bash
mise run gate         # every quality gate, stopping at the first failure
mise run app          # build and start the panel from the terminal (then ⌥K); --trace logs its latencies
```

`mise run package` builds `target/packager/kbx.app` and signs it ad hoc, which is enough for this machine. To give it
to someone else, sign it with a Developer ID and notarize it (`codesign --options runtime`, `xcrun notarytool`);
Gatekeeper refuses an ad hoc signature from another machine.

## Project Structure

Three folders at the root, each named by a widely used convention; the `root-layout` hk step fails on a fourth.

```text
crates/                     Rust code: our crates, and patched third-party ones
  kbx-domain/                  the engine; tests/fixtures/ holds the sample corpus and its expected outputs
  kbx-app/                     the GPUI Kit app (assets/catalog.toml: entry types, actions, keys, words)
  third-party/                patched external crates, outside the workspace (see its README)
docs/                       adr/, doctrine/, guides/
tools/                      developer tooling
  graph/                      code graphs for agents: CRG, Gortex, Graphify (see its README)
  design/                     the design kit: palettes, validation, theme, showcase corpus, screenshots
  size/                       the release binary's size budget and its check
  ast-grep/ icon/             lint rules, the app icon's source
```

- **Where things go.** Rust code in `crates/`; prose in `docs/`; anything that helps build, check or measure in
  `tools/`, one folder per tool. Test data lives with the crate whose contract it is (`<crate>/tests/fixtures/`).
- **Product and design** are described at the root: [PRODUCT.md](PRODUCT.md) (what kbx is for) and
  [DESIGN.md](DESIGN.md) (how it looks; read it before any UI change).
- **Root files** are the configs each tool looks for there. Moving tool configs into `.config/` is a later decision
  (`TODO.md`).
- **Hidden folders** are owned by tools: `.github/` (CI, ADR 0007) and `.mise/` (the npm and pipx tools' locks that
  `mise.lock` points to, written by `mise lock`). `.agents/` and `.claude/` hold agent wiring and skills.
- **Local state** (`target/`, `tmp/`, code graphs, agent worktrees) is gitignored; generated output goes to `tmp/`.

## Key Features

- The kbx search experience — FTS search, filters, split pane, rendered notes with highlighted code, actions,
  sync — as a native panel opened by ⌥K.
- Themes: base24 colour schemes, previewed as you move through them.
- A catalog (`crates/kbx-app/assets/catalog.toml`) of entry types, actions, keys and words, overridable from
  `~/.config/kbx/config.toml`.

## Development Workflow

| Command                                               | Does                                                                           |
| ----------------------------------------------------- | ------------------------------------------------------------------------------ |
| `mise run setup`                                      | install everything                                                             |
| `mise run graph setup`                                | initialize the code graphs for agents (CRG, Gortex, Graphify)                  |
| `mise doctor project`                                 | verify toolchain, code graph health and secrets                                |
| `mise run gate`                                       | all gates in order; `--fast` runs tests and clippy only                        |
| `mise run package`                                    | build and sign `kbx.app`; `install` copies it to ~/Applications                |
| `mise run show`                                       | (re)start kbx in the background with the panel showing, without ⌥K             |
| `mise run tags:suggest` / `tags:apply` / `tags:check` | propose, apply and check tag merges in your sources (tag conventions doctrine) |
| `mise run ci`                                         | what CI checks: the gate on macOS, `ci:lint` and `ci:test` elsewhere           |
| `mise run release -- --dry-run --no-ci`               | preview the next release (`GITHUB_TOKEN=$(gh auth token)`)                     |
| `hk check --all`                                      | every baseline step over the whole repository                                  |

Commits follow Conventional Commits with a sentence-case subject; hk checks the message.

**Releases.** Every push to `main` is checked on GitHub Actions, and one with a `feat`, `fix` or `perf` commit is released:
semantic-release picks the version, updates [CHANGELOG.md](CHANGELOG.md), tags it and publishes a
[GitHub Release](https://github.com/roalcantara/kbx/releases) with `kbx.app` for Apple silicon. The app is signed ad hoc:
open it the first time with right-click → Open. How and why: [ADR 0007](docs/adr/0007-releases.md).

## Coding Standards

- The Rust best practices in `~/.agents/skills/rust-best-practices`, and the GPUI Kit Coding and Design Guides.
- Typed errors (`thiserror`) in libraries; no `unwrap`, `expect`, printing or `process::exit` outside tests and
  `main`.
- One concept per file, named after it; a folder carries context.
- Use the vocabulary in [CONTEXT.md](CONTEXT.md).

## Testing

```bash
cargo nextest run --workspace   # Rust tests
bun test tools/                  # the tested modules of the TS tools
```

Golden tests run the engine over the sample corpus (`crates/kbx-domain/tests/fixtures/sample`) and compare it with
the committed expected outputs in `sample-goldens`, byte for byte.

## Contributing

Every change passes `mise run gate`.

**Roadmap.** The work ahead lives in [TODO.md](TODO.md), pending tasks after the done ones. The next steps:

- **Daily use:** paste an entry straight into the app it was opened from; run a command entry in the terminal;
  frecency sorting; search by keywords and tags.
- **Notes:** render fenced code with line numbers and copy; fix bold text in the notes body.
- **Adoption** ([ADR 0008](docs/adr/0008-kbx-stays-its-own-code.md)): import navi, cheat and pet files and Atuin's
  shell history; bundle tldr pages so kbx is useful on first launch.
- **Distribution:** a Developer ID signature and notarization, a universal binary, a Linux build.

## License

[MIT](LICENSE). The patched crates in `crates/third-party/` keep their own licenses: `block` is MIT
([its LICENSE](crates/third-party/block/LICENSE)) and `gpui-pre-macos` is Apache-2.0
([its LICENSE](crates/third-party/gpui-pre-macos/LICENSE-APACHE)).
