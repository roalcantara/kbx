# Skills entry and scoped refresh - design

- Status: draft for owner review
- Date: 2026-10-18
- Task: Rkpg - Add Skills Entry Type and Source

## Goal

kb loads the global Skills catalog beside its YAML knowledge base. Every record
from `skills ls --global --agent universal --json` becomes a searchable
`skills` entry, and users can refresh YAML and Skills independently from the
existing More action panel.

The feature uses the existing SQLite `entries` and FTS5 index. It does not add
a Skills-specific table or change the YAML source format.

## Decisions

**D1. The existing Refresh action remains the default.** Refresh continues to
refresh all configured sources. A new **Refresh More...** action appears under
**More -> Administration**. Its submenu contains exactly **Refresh YAML** and
**Refresh Skills**. The submenu has no **All** item.

**D2. Skills use the existing entry model.** Skills have entry type `skills`,
use the existing index and notes Markdown, and share the existing search,
filter, detail, split-view, and refresh-report infrastructure.

**D3. The Skills key is the slash-prefixed name.** A Skills record with
`name: testing-codegen` has key `/testing-codegen` and is copied as
`/testing-codegen`. Skill names are unique in the command output. Duplicate
names are a source validation error, not two entries with different hidden
keys.

**D4. Source adapters remain separate.** YAML discovery/parsing and Skills
command parsing are separate source adapters behind the sync layer. The sync
layer owns source selection, transactions, cancellation, reporting, and index
replacement.

**D5. A source failure preserves that source's prior entries.** Invalid JSON,
command warnings mixed into stdout, command failure, duplicate names, and
cancellation do not delete or partially replace the previous Skills entries.
The same rule applies to YAML entries during a YAML refresh.

**D6. Local and web actions have different targets.** Edit opens the local
`SKILL.md` with the system editor. The browser action opens `sourceUrl` when
present. kb never fabricates a web URL from a Skill name or repository guess.

**D7. The command executable is configurable.** Configuration may override the
Skills executable with `skills_command = "skills"`. The application always
supplies the fixed arguments `ls --global --agent universal --json`.

## Scope

In scope:

- Parse the global Skills JSON record shape: `name`, `path`, `scope`,
  `agents`, `source`, `sourceUrl`, and `sourceType`.
- Map every valid record to a `skills` entry with key `/<name>`.
- Render the local Skill Markdown through the existing notes/file inclusion
  behavior.
- Add the `skills` catalog type and its actions.
- Add source selection to the More action panel and sync CLI.
- Preserve source data on failure and cancellation.
- Add focused domain, application, CLI, and action tests.

Out of scope:

- A new Skills database table.
- A project-local Skills source.
- Changes to the external `skills` CLI.
- Editing Skill documents inside kb.
- Deriving `skills.sh` or GitHub URLs when `sourceUrl` is absent.
- New keyboard shortcuts for the scoped refresh actions.

## Source and entry mapping

The Skills adapter executes the configured executable with the fixed arguments
and parses stdout as one complete JSON document. A warning or any other
non-JSON stdout content makes the complete refresh invalid.

Each returned record maps as follows:

| Skills field | Entry value                                             |
| ------------ | ------------------------------------------------------- |
| `name`       | Entry key `/<name>` and fallback primary content        |
| `path`       | Local source path, normalized to the Skill's `SKILL.md` |
| `scope`      | Metadata used for tags or report context                |
| `agents`     | Metadata used for tags                                  |
| `source`     | Source metadata used for report context                 |
| `sourceUrl`  | Optional browser-action target                          |
| `sourceType` | Source metadata used for report context                 |

The local source path is `~/.agents/skills/<name>/SKILL.md`. The path is
expanded when the application opens or reads it. The adapter must use the
record's `path` to locate the actual local document when it is provided, while
normalizing a directory path to its `SKILL.md` child. The normalized path is
stored in the entry's source-file field.

The entry's notes are the Markdown contents of the local `SKILL.md`, compiled
through the existing Markdown pipeline. A missing local document is reported
for that record, but the record is still indexed with its key, primary
content, source path, and available metadata; its notes Markdown is empty.

The adapter rejects a record without a non-empty `name`, rejects duplicate
names, and rejects invalid JSON. It preserves the existing Skills set when any
whole-source validation fails.

## Sync behavior

The sync API accepts a source selection with these values:

- `all`: refresh YAML and Skills. This is the default for the existing Refresh
  action and for the CLI when `--source` is omitted.
- `yaml`: refresh YAML only.
- `skills`: refresh Skills only.

A scoped refresh changes only the selected source's entries. An `all` refresh
runs both adapters but commits each source as its own transaction so a Skills
failure does not remove successful YAML results, and a YAML failure does not
remove successful Skills results.

A successful source refresh replaces that source's prior entries with the
validated result. An unsuccessful or cancelled source refresh leaves its prior
entries unchanged and adds a source-specific failure to the sync report. The
report identifies YAML or Skills and includes the command or parse failure
reason when available.

The refresh view remains cancellable. Cancellation is checked while collecting
source output, parsing records, reading local Skill files, and writing index
rows. Cancellation does not commit a partial source result.

## Catalog and actions

The catalog adds a `skills` type with the following presentation and actions:

- Label: `Skills`
- Icon: `notebook-text`
- Tint: yellow
- Hero: none
- Primary action: copy `/<name>`
- Secondary action: open `sourceUrl` when present
- Edit action: open the local `SKILL.md` with the system editor

The full Skill Markdown is shown in split and detail views through the existing
notes renderer. The existing action panel keeps its current More layout and
adds **Refresh More...** under **Administration**. Its child actions are
**Refresh YAML** and **Refresh Skills**.

## CLI and configuration

The sync CLI accepts:

```text
--source all|yaml|skills
```

The default is `all`. The application validates the value and reports an
argument error for any other value.

The configuration accepts:

```toml
skills_command = "skills"
```

The value is the executable name or path. The application appends no
user-configurable arguments; it always invokes:

```text
skills ls --global --agent universal --json
```

## Schema and compatibility

No SQLite schema migration is required. The existing entry type and source
fields already support arbitrary entry types and source paths. Existing YAML
entries keep their current identity and behavior.

The context glossary must update the definitions of Entry type, Source file,
Sync, and Sync report to include Skills records and source-specific reports.
The catalog and CLI documentation must use `Skills` and `Refresh More...`
consistently.

## Tests and validation

The implementation adds focused tests for the following behaviors:

- Parse valid Skills JSON records.
- Reject malformed JSON and mixed-output command streams.
- Reject empty names and duplicate names.
- Map records to entry type `skills` with key `/<name>`.
- Copy `/<name>` as the primary action value.
- Resolve the local `SKILL.md` and render its Markdown notes.
- Open `sourceUrl` for the browser action when it exists.
- Preserve Skills entries after command failure, parse failure, duplicate names,
  missing source data, or cancellation.
- Refresh YAML without changing Skills and Skills without changing YAML.
- Refresh both sources independently when the selection is `all`.
- Parse CLI source selections and reject invalid values.
- Expose the catalog type and More submenu actions.

Run focused tests for the changed crates first. Finish with:

```text
mise run gate
```

The gate must continue to run with its existing commands and thresholds.
