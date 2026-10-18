# Skills entry and scoped refresh — design

- Status: approved
- Date: 2026-10-18
- Task: Rkpg - Add Skills Entry Type and Source

## Goal

kb loads the global Skills catalog beside its YAML knowledge base. Every record
from `skills ls --global --agent universal --json` becomes a searchable
`skills` entry, and users can refresh YAML and Skills independently from the
existing More… action panel.

The feature uses the existing `entries`, `entries_fts` and `entry_tags` tables.
It adds one column, `entries.source`, so a refresh can replace one source's
rows. It does not change the YAML source format.

## Decisions

**D1. The existing Refresh action remains the default.** Refresh (⌘R)
continues to refresh all sources. A new **Refresh More…** submenu appears
under **More… → Administration**, after Refresh. It contains exactly **Refresh
YAML** and **Refresh Skills**; it has no **All** item.

**D2. Skills use the existing entry model.** Skills have entry type `skills`,
use the existing index and notes Markdown, and share the existing search,
filter, detail, split-view and refresh infrastructure.

**D3. The Skills key is the slash-prefixed name.** A record with
`name: testing-codegen` has key and title `/testing-codegen` and is copied as
`/testing-codegen`. Duplicate names are a source validation error, not two
entries.

**D4. Source adapters remain separate.** YAML discovery and parsing, and the
Skills command and its JSON parsing, are separate source adapters behind the
sync layer. The sync layer owns source selection, transactions, cancellation,
reporting and index replacement.

**D5. A source failure preserves that source's prior entries.** A command
failure, invalid JSON, an invalid record, duplicate names or a cancellation
does not delete or partially replace the previous Skills entries. The same
rule applies to YAML entries during a YAML refresh.

**D6. The source URL is a link, never a fabricated one.** A record's
`sourceUrl`, when present, becomes the entry's only link, so **Links…** opens
or copies it. kb never derives a URL from a Skill name or repository guess.
Edit (⌘O) opens the local `SKILL.md`.

**D7. The command executable is configurable and found without a shell.**
`skills_command` in `~/.config/kb/config.toml` names the executable or its
path. The application always supplies the fixed arguments
`ls --global --agent universal --json`.

**D8. Rows record their source.** `entries.source` is `yaml` or `skills`. A
scoped refresh deletes and inserts only rows of its source.

## Scope

In scope:

- The `entries.source` column and its migration.
- Parse the global Skills JSON record shape: `name`, `path`, `scope`,
  `agents`, `source`, `sourceUrl` and `sourceType`.
- Map every valid record to a `skills` entry.
- Read the description from each `SKILL.md` front matter; render the body
  through the existing `file://` notes behaviour.
- Add the `skills` catalog type, an optional catalog hero, and the
  **Refresh More…** submenu with its two actions.
- Source selection for the panel and for `kb --sync`.
- Per-source sections in the refresh report and the `kb --sync` output.
- Preserve source data on failure and cancellation.
- Focused domain, application, CLI and action tests.

Out of scope:

- A Skills-specific table.
- A project-local Skills source.
- Changes to the external `skills` CLI.
- Editing Skill documents inside kb.
- Tags on Skills entries (neither `scope`, always `global`, nor the 45 agent
  names make useful tags).
- Searching a Skill's body; search is revised in later work.
- A timeout on the Skills command; Stop covers a hung command.
- New keyboard shortcuts for the scoped refresh actions.
- Restricting YAML to known top-level keys and removing `skills.yml`: both
  are separate pending work.

## Schema and migration

```sql
ALTER TABLE entries ADD COLUMN source TEXT NOT NULL DEFAULT 'yaml';
CREATE INDEX idx_entries_source ON entries(source);
```

`schema.sql` creates `entries` with the column and sets `user_version` to 1
for a new index. An existing
index is migrated once, guarded by `PRAGMA user_version` (0 → 1): every row it
holds came from YAML, so the `yaml` default is correct and nothing is
discarded. The comment calling the schema the TypeScript engine's, statement
for statement, is removed: the Raycast schema no longer constrains it. The
goldens test selects named columns and is unaffected.

## Skills source

### Finding the command

When `skills_command` is a path (it contains `/`), kb runs it, `~/` expanded.
Otherwise kb looks for that name (default `skills`) as an executable in
`PATH`, then in `/opt/homebrew/bin`, `/usr/local/bin` and `~/.local/bin`, and
uses the first found. The fallback
folders exist because an app opened from Finder gets
`PATH=/usr/bin:/bin:/usr/sbin:/sbin`.

| Situation                                           | Skills outcome                      |
| --------------------------------------------------- | ----------------------------------- |
| Not set, no `skills` found                          | Skipped: _not configured_; no error |
| Set, cannot be run                                  | Failed; prior entries kept          |
| Exits non-zero                                      | Failed; prior entries kept          |
| Exits 0, stdout not one valid JSON array of records | Failed; prior entries kept          |
| Exits 0, valid JSON                                 | Replaced                            |

stdout must be one complete JSON document; anything else in it fails the
source. stderr is not a failure: each non-empty stderr line is reported as a
Skills warning (the CLI writes, for example, `⚠ Skipped …/recon/SKILL.md —
YAML parse error …` there and still exits 0).

The command takes about 8 s for 438 skills. While it runs, the refresh screen
shows **Running skills…**. Cancelling kills the child process; nothing is
committed for Skills.

### Records

A record is valid when `name` and `path` are non-empty strings; `scope`,
`agents`, `source`, `sourceUrl` and `sourceType` are optional and unknown
fields are ignored. An invalid record or a duplicate `name` fails the whole
Skills source (D5).

### Entry mapping

| Entry field       | Value                                                                  |
| ----------------- | ---------------------------------------------------------------------- |
| `entry_type`      | `skills`                                                               |
| key, `title`      | `/<name>`                                                              |
| `primary_content` | `/<name>`                                                              |
| `description`     | `SKILL.md` front matter `description`, else none                       |
| `notes`           | `file://<path>/SKILL.md`                                               |
| `notes_markdown`  | Compiled by the existing pipeline, front matter stripped               |
| `links`           | `[sourceUrl]` when present, else none                                  |
| `tags`            | none                                                                   |
| `source_file`     | `<path>/SKILL.md` (absolute)                                           |
| `source_rel_path` | `<name>/SKILL.md`                                                      |
| `id`              | `<name>/SKILL.md#skills#/<name>` (the usual `<rel path>#<type>#<key>`) |
| `raw_yaml`        | The record, as YAML                                                    |
| `source` (row)    | `skills`                                                               |

`path` is the Skill's directory, as the CLI reports it. A YAML id's path ends
in `.yml` or `.yaml`, so Skills and YAML ids cannot collide.

A missing `SKILL.md` or unreadable front matter is a warning for that record;
the record is still indexed with an empty description, and the notes render
nothing.

Full-text search matches the title and description. The `notes` column holds
the `file://` reference, as for YAML `file://` notes, so a Skill's body is not
searched.

## Sync behaviour

The sync API takes a source selection:

- `all`: YAML, then Skills. The default for Refresh and for `kb --sync`.
- `yaml`: YAML only.
- `skills`: Skills only.

Each selected source runs and commits in its own transaction:
`DELETE FROM entries WHERE source = ?` (tags follow through
`ON DELETE CASCADE`; the FTS rows through the delete trigger), then the
inserts. In `all`, YAML commits before the Skills command starts, so a Skills
failure or a cancellation during it keeps the fresh YAML entries, and a YAML
failure does not stop Skills.

Cancellation is checked between YAML files, while the Skills command runs,
between Skills records, and before each commit. A cancelled source commits
nothing; sources committed before it stay.

## Refresh report and progress

The report has one section per refreshed source; a scoped refresh shows only
its source.

- **YAML:** the per-file rows, as today.
- **Skills:** one row: `438 entries`, _failed: <reason>_ (with "The previous
  Skills entries are unchanged."), or _not configured_; then one warning row
  per stderr line and per record warning.

The summary line totals both sources. Progress reads
`{done} of {total} files · {file}` during YAML and **Running skills…** during
the Skills command.

`kb --sync` prints the same: per source, its failures and warnings, then one
line; then the summary.

## Catalog and actions

```toml
[types.skills]
label = "Skills"
icon = "notebook-text"
tint = "yellow"
primary = "copy"
```

`hero` becomes optional: a type without one shows no hero. Skills has no
secondary action; **Links…** carries `sourceUrl`. `notebook-text` must be a
bundled icon (the icon test checks it).

The More… page keeps its layout. **Administration** lists Edit, Refresh,
**Refresh More…**, Configure. **Refresh More…** opens a submenu, with the `‹`
header of the existing submenus, listing **Refresh YAML** (`refresh-yaml`) and
**Refresh Skills** (`refresh-skills`). The root's search finds both, marked
with their submenu. Labels live in the catalog like every other action.

The `no_entries` message names Skills with the other types.

## CLI and configuration

```text
kb --sync [--source all|yaml|skills]
```

`--source` is valid only with `--sync`; its default is `all`. Any other value,
or `--source` without `--sync`, is a usage error.

```toml
skills_command = "skills"
```

The value is an executable name or path. kb appends no user-configurable
arguments; it always runs `<command> ls --global --agent universal --json`.

## Documentation

CONTEXT.md updates Entry type (five types), Source file (or a Skills record),
Index (the `source` column), Sync (sources, selection) and Sync report
(per-source sections, warnings), and adds **Skills source**. The catalog and
CLI help use `Skills` and `Refresh More…` consistently.

## Tests and validation

Skills has no TypeScript counterpart, so its tests use hand-written
expectations; the goldens rule still covers YAML, which must keep matching.

- Parse valid Skills JSON; ignore unknown fields.
- Fail on malformed JSON, non-JSON stdout, a non-zero exit, an invalid record
  and duplicate names, keeping prior Skills rows each time.
- Report stderr lines as warnings without failing.
- Report _not configured_ when no command is set or found.
- Find the command in `PATH`, then in the fallback folders.
- Map a record: key, title, primary, description from front matter, notes,
  link, source file, rel path, id, source.
- Index a record whose `SKILL.md` is missing, with a warning.
- Kill the command on cancel; commit nothing for Skills.
- Refresh YAML without changing Skills rows, and Skills without changing YAML
  rows.
- In `all`, keep fresh YAML rows when Skills fails or is cancelled.
- Migrate an index at `user_version` 0, keeping its rows as `yaml`.
- Parse `--source` values and reject invalid ones and `--source` alone.
- Expose the `skills` catalog type, the optional hero and the Refresh More…
  submenu, including from the root's search.

Run focused tests for the changed crates first. Finish with:

```text
mise run gate
```

The gate keeps its existing commands and thresholds.
