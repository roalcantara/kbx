# Change map

What a change reaches, and what catches a miss. Written from Plan 8's diff (`41bdc68..<HEAD>`).

| To change          | Edit                                                                         | It reaches                                                                      | What catches a miss                                                         |
| ------------------ | ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| A colour           | `DESIGN.md` front matter `colors`                                            | every component (`design().colors`); `mise run design:theme` → `kb.json`        | `design:validate` (contrast); `ui::design` test `gives_gpui_kit_the_tokens` |
| A type role        | `DESIGN.md` `typography.roles`                                               | components using `text_role(…)`                                                 | `ui::design` tests; the shots                                               |
| A size or spacing  | `DESIGN.md` `rounded`, `spacing`, `components`                               | the component of the same name                                                  | the shots; ast-grep `no-literal-design` keeps it out of views               |
| A label or message | `crates/kb-app/assets/catalog.toml` (or `config.toml`)                       | the action panel, footer, empty view, notices                                   | `catalog` tests                                                             |
| An action's keys   | `catalog.toml` `[actions.<id>] keys`                                         | `keys::key_bindings`, keycaps in rows, footer and panel                         | `catalog` validation (parses, bound once); `keys` tests                     |
| A shortcut's look  | `DESIGN.md` `components.keycap`, `spacing.keycap`, `typography.roles.keycap` | every keycap (`ui::keycap::keycap`): rows, footer, header, panel, links, notice | `keycap` tests (`keys`, `keycap_width`); `entry_list` slot test; the shots  |
| A screen's keys    | `catalog.toml` `[screens.<screen>]`                                          | `SearchView::ladder`, the footer, the panel's root                              | `catalog` validation; UI tests                                              |
| An entry type      | `catalog.toml` `[types.<key>]`                                               | the list, the filter, the counts, the entry view's hero                         | `catalog` validation (icon bundled, tint a token); UI tests                 |
| A component        | `crates/kb-app/src/ui/<component>.rs`                                        | every screen that composes it                                                   | its unit tests; the UI tests; the shots and `CHECKLISTS.md`                 |
