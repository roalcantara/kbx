# AGENTS.md

## Project overview

kbx: a native macOS panel, opened by a hotkey, that searches a personal knowledge base of YAML source files. Rust on
GPUI Kit, inspired by Raycast.

## Repository map

The layout and the rule for each folder: [README, "Project Structure"](README.md#project-structure).

- `crates/kbx-domain/`: the engine; no UI dependency; `tests/fixtures/` holds the sample corpus and its goldens
- `crates/kbx-app/`: the GPUI Kit application
- `crates/third-party/`: patched external crates, outside the workspace
- `tools/graph/`: code graphs for agents (CRG, Gortex, Graphify); use them before a broad search: tools/graph/README.md
- `tools/tags/`: the tag tools (suggest, apply, check) for the knowledge folder's tags; rules in
  `docs/doctrine/tag-conventions.doctrine.md`
- `docs/`: ADRs and guides; `docs/doctrine/`: principles to load when a change matches their
  `applies_to`
- `PRODUCT.md`: what kbx is for, its users and principles; `DESIGN.md`: its visual design. Read DESIGN.md before any
  UI change
- `crates/kbx-app/assets/catalog.toml`: what kbx offers — entry types, actions, keys, words — overridable from
  `~/.config/kbx/config.toml`
- `tools/design/`: the design kit: tokens validated (`design:validate`), GPUI Kit's theme generated (`design:theme`), the showcase corpus, screenshots (`design:shots`) and the change map (`CHANGE-MAP.md`)
- `TODO.md`: done and pending work, kept with the `todo` CLI (pinned in `mise.toml`; usage in its REF comments)
  Done tasks come first, pending ones after, in the order to do them; a decision or a won't-do is a comment on its
  task, with the date.

## Core engineering rules

- `kbx-domain` must never depend on `gpui*` or `kbx-app`, directly or transitively. The `domain-purity` hk step
  enforces it; do not bypass it.
- Follow `~/.agents/skills/rust-best-practices`, and for UI work read the GPUI Kit Coding Guides
  (`~/.agents/skills/gpui-kit`) and Design Guides (`~/.agents/skills/gpui-kit-design-guides`) before deciding.
- Never commit real personal data: the sources directory stays outside the repository; tests and screenshots use the
  sample and showcase corpora.
- Keep quality gates and thresholds unchanged unless the owner approves the change.
- Scripts that stay in `tools/` are TypeScript run by Bun, written inline in the tool's mise task file
  (`run = '''#!/usr/bin/env bun …'''`). A module file exists only for logic with unit tests (`bun test` beside it).
  npm packages are imported with a pinned version (`import x from "pkg@1.2.3"`), which `bun run` installs on first
  use: no `package.json`. `bun test` does not, so a tested module imports none. No Python or Node-only scripts.

## Naming conventions

- Rust files and modules snake_case, one concept per file; crate directories kebab-case.
- No `mod.rs` + sibling file for the same module; prefer `module.rs` with `module/` children.
- Use the domain vocabulary in [CONTEXT.md](CONTEXT.md).

## Setup

```bash
mise run setup
```

## Validation

```bash
mise run gate
```

It runs, in order, stopping at the first failure: `cargo nextest run`, `cargo clippy -D warnings`,
`cargo build --release`, `hk check --all`, `cargo deny check`, `cargo doc -D warnings`.

## Testing expectations

- Write the failing test first. Tests sit beside the code in `#[cfg(test)]` modules; integration tests in
  `crates/*/tests/`.
- Engine behaviour is checked against the goldens; a change to a golden is a behaviour change, reviewed by hand.

## Commits

Conventional Commits with a sentence-case subject, at most 72 characters.

Stage your own files with `git add <paths>`, then run `git commit`: never `git commit <paths>` (the pre-commit hook's
fixes would leave the index stale) and never `git commit -a` (other agents may have work in progress). To format
before a gate run, use `hk fix --all`; `hk check --all` and `mise run gate` only check.
