# Refresh report inspection — design

- Status: approved
- Date: 2026-10-27
- Task: 2YnZ (Improve Refresh Results)
- Context: the refresh report lists one row per source file, but its selection is fixed on the first row
  (`sync_view.rs`, `.selected(ix == 0)`), so a failed file's error, excerpt and counts cannot be inspected and the file
  cannot be opened from it. The Raycast command had a per-file detail (parity row Y03); the port deferred it (spec D8).
  Approved in the brainstorming companion (2026-10-27): navigation option **A, like the list**, and the pane for each
  kind of row; the pages are kept as `docs/guides/design/12-report-navigation.html` and `12-report-pane.html`.

## Goal

The report works as the search list does: the user moves through its rows, glances at one in a split pane, opens it
full screen, and acts on it — opens the file in their editor at the failing line, copies its location or error, or
reveals it in Finder — then refreshes again without leaving the report.

## Decisions

**D1. The report navigates like the list.** When a refresh finishes, the report opens with its first row selected
(failed rows sort first). ↑↓ and ⌘↑↓ move the selection; the mouse selects; the list keeps the selection in view. The
summary line and the YAML and Skills section labels show but are never selected. → opens the split (the list 34 %, the
selected row's pane beside it) and ← closes it; the pane follows the selection. ↵ opens the row full screen. Esc goes
back one level: detail → report (the pane as it was) → search list; the back chevron does the same.

**D2. Screen keys follow the list → detail pattern.** Catalog `[screens.report]` changes and
`[screens.report-detail]` is new:

| Key | Report (list or split) | Report detail   |
| --- | ---------------------- | --------------- |
| ↵   | `details`              | `edit`          |
| ⌘↵  | `edit`                 | `copy-location` |
| ⇧⌘↵ | `copy-location`        |                 |

**D3. Row actions.** ⌘K on a report row lists Edit, Copy Location, Copy Error (failed rows only), Reveal in Finder, then
Refresh. Open is the existing `edit` action (⌘O, labelled _Edit_ as on the list), acting on the selected row; three
actions are new, each overridable from `config.toml`:

| Action          | Label            | Keys          |
| --------------- | ---------------- | ------------- |
| `copy-location` | Copy Location    | `cmd-shift-l` |
| `copy-error`    | Copy Error       | `cmd-shift-e` |
| `reveal`        | Reveal in Finder | `cmd-shift-r` |

`cmd-shift-f` stays Resize. Refresh (⌘R) on the report reruns the same selection: after Refresh YAML it refreshes YAML.

**D4. What each action does, by row.**

| Row                 | Edit                                | Copy Location      | Copy Error          | Reveal    |
| ------------------- | ----------------------------------- | ------------------ | ------------------- | --------- |
| Failed file         | the file at the error's line:column | `path:line:column` | the message         | the file  |
| Skipped / indexed   | the file                            | the path           | —                   | the file  |
| Skills warning      | its `SKILL.md`, when it names one   | that path          | —                   | that file |
| Skills failed / off | Configure                           | —                  | the reason (failed) | —         |

Copies show the _Copied to clipboard_ notice. An action with nothing to act on is not offered in ⌘K and its key does
nothing.

**D5. The pane, per row** (approved page `12-report-pane.html`). Header: the row's tile, path (mono) and status chip.
Then:

- **Failed file**: the error as the red-bar quote; the excerpt (two lines either side) on mantle, the failing line
  marked with a red inset bar and tint; facts _Location_ (`line 4:11`), _File_ (the absolute path), _Index_ (_its
  previous entries are kept_).
- **Skipped entries**: a yellow quote (_N entries were skipped: only a map of fields becomes an entry._); the skipped
  keys as a list (`type / key — why`); facts _Entries_ (a pill per type with its count), _File_. No excerpt: the engine
  records no line for a skipped entry.
- **Empty file**: a yellow quote (_No entries_); fact _File_.
- **Indexed file**: facts only, _Entries_ and _File_.
- **Skills failed**: the reason as the red-bar quote; the command kb ran on mantle; facts _Exit_, _Stderr_ (its lines,
  or _(nothing)_), _Index_ (_the previous skills are kept_).
- **Skills warning**: the full warning as a yellow quote; fact _File_ when the warning names a `SKILL.md`.
- **Skills not configured**: today's sentence as the quote; no facts.

The footer shows the selected row's tile, path and ⌥⌘C (Copy Path), as it does for an entry.

**D6. The editor is configurable and opens at the line.** `editor` in `config.toml` is a command template, e.g.
`editor = "zed {file}:{line}"` or `"code -g {file}:{line}:{column}"`. kb fills `{file}`, `{line}` and `{column}`, splits
the result into words (quotes group; no shell runs) and spawns it. Without a known line, `:{line}` and `:{column}`
(with their colon) are dropped. Unset, kb opens the file with the system's default app (`open_with_system`), as Edit does
today. The list's Edit (⌘O) goes through the same code, so entries open in the editor too, at the top of their source
file. The Configure template documents the key.

**D7. Engine additions, for the report only.** In `kb-domain`, nothing new reaches the index:

- `FileReport::location()`: `path:line:column` when the error has a line, else the path.
- A Skills warning becomes `SkillsWarning { message: String, file: Option<String> }`; `skill_entry` sets `file` to the
  `SKILL.md` it could not read; stderr lines have none.
- `SkillsOutcome::Failed` also keeps the command line and its exit status.

**D8. Structure (approach 1).** `crates/kb-app/src/report_list.rs` (new) is a GPUI Kit `ListDelegate` over the report's
rows and section headers; today's `report_rows` / `report_sections` move into it unchanged, and the summary line is its
header. `crates/kb-app/src/ui/report_pane.rs` (new) draws a row's pane for both the split and the detail screen.
`SyncView` holds the `ReportList` and `pane` / `detail` flags and handles ShowDetails, HideDetails, Details and Back as
`SearchView` does; `SyncView::selected_row()` lets `SearchView` run the shared commands and draw the footer.
`SearchView::key_screen` returns `Report` or the new `ReportDetail`. `crates/kb-app/src/editor.rs` (new) parses and runs
the editor template. `SearchView`'s list, split and detail are unchanged.

## Error handling

| Case                                             | Behaviour                                                                                                  |
| ------------------------------------------------ | ---------------------------------------------------------------------------------------------------------- |
| The file was moved or deleted since the refresh  | Edit and Reveal show a notice, _Couldn’t open {file}: not found_; the pane keeps what the report recorded. |
| `editor` is malformed (empty, an unclosed quote) | One start-up notice, as for other ignored config keys; the default app is used.                            |
| The editor cannot be spawned                     | A notice with the OS error; the file opens with the default app.                                           |
| A report without rows                            | The summary line only; →, ↵ and the row actions do nothing.                                                |

## Testing

Tests first, beside the code (AGENTS.md).

- `kb-domain`: `location()` with and without a line; a `SKILL.md` warning carries its file; a failed Skills outcome
  keeps its command and status.
- `kb-app`, without a window: `editor.rs` template filling (with a line, without, with quotes) and the fallback;
  `report_list` row order and which rows select; the report's action page per kind of row; the `report-detail` keys
  and their overrides.
- `kb-app`, GPUI: from a finished report, ↓ → ↵ Esc Esc ends on the search list, with the pane as D1 says.
- By hand: `kb` against the private corpus with one broken file; `mise run design:shots` for report, split and detail.

## Docs and design kit (same change)

- `docs/guides/parity.md` Y03: _Ported_, with kb's split, detail and editor-at-line.
- `DESIGN.md` §4: a **Report pane** component — the yellow quote, the excerpt, the facts — citing
  `12-report-pane.html`.
- `CONTEXT.md`: _Sync report_ mentions the report pane; a new **Editor** term (`editor` in `config.toml`, else the
  default app; `crates/kb-app/src/editor.rs`).
- `tools/design`: the showcase corpus gains a broken file so `design:shots` captures the report's split and detail.
- `docs/guides/design/`: the two approved companion pages, `12-report-navigation.html` and `12-report-pane.html`.

## Scope

In scope: the report's navigation, split and detail (D1, D2); the row actions and their keys (D3, D4); the report pane
(D5); the `editor` setting, also used by the list's Edit (D6); the engine additions (D7); the catalog, docs and design
kit changes below; tests.

Out of scope: typing to filter the report, Copy Report, jumping to a file's entries in search, refreshing a single
file, and opening a skipped entry at its line.
