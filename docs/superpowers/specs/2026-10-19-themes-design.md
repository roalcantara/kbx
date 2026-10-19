# Themes — design

- Status: draft for owner review
- Date: 2026-10-19
- Context: page 19 (`docs/guides/design/19-colours.html`) showed eight colour schemes on the approved list; the owner
  chose Neon Dusk and asked to make all of them, and any others, selectable from the app.

## Goal

kb's colours come from a **theme file**, the way Ghostty's do: themes are files in `~/.config/kb/themes/`, the user
picks one from **More… → Theme…** with a live preview, and the choice is saved in `config.toml` by name. Adding a
theme is dropping a file into the folder; it takes no change to the app.

## Decisions

**D1. Themes are base24 files.** A theme is a tinted-theming base24 (or base16) scheme file, so any of the published
schemes drops in. kb reads YAML schemes (`*.yaml`, `*.yml`), the format that ecosystem publishes:

```yaml
system: "base24"
name: "Neon Dusk"
author: "daniel-felipe (omarchy-neon-dusk-theme), mapped for kb"
variant: "dark"
palette:
  base00: "#16122A"
  # … base01 to base17 (base16 files stop at base0F)
kb:
  # Optional: pins any of kb's 19 roles exactly, over the mapping below.
  mantle: "#0D0A18"
```

A theme's id is its file stem (`neon-dusk`); its label is `name`, else the stem.

**D2. The mapping.** kb derives its 19 colour roles from the palette, using each colour's base24 meaning; base16
files use base24's documented fallbacks (base10 and base11 → base00; base12–base17 → their normal colours):

| kb role    | From                                             |
| ---------- | ------------------------------------------------ |
| `window`   | base00 (default background)                      |
| `badge`    | base01 (lighter background)                      |
| `selected` | base02 (selection background)                    |
| `hover`    | base00 and base02 mixed half and half            |
| `mantle`   | base10 (darker background)                       |
| `code`     | base10                                           |
| `crust`    | base11 (darkest background)                      |
| `text`     | base05 (default foreground)                      |
| `muted`    | base04 (darker foreground)                       |
| `faint`    | base03 (comments)                                |
| `edge`     | base03 at 50 % opacity                           |
| `ring`     | base05 at 8 % opacity                            |
| `red`      | base08                                           |
| `peach`    | base09                                           |
| `yellow`   | base0A                                           |
| `green`    | base0B                                           |
| `pink`     | base0C (base16 has no pink; cyan takes the slot) |
| `blue`     | base0D                                           |
| `mauve`    | base0E (kb's main accent: selection bar, caret)  |

The file's `kb:` section then overrides any role, and `config.toml`'s `[colors]` overrides that (D4).

**D3. The themes folder, seeded.** `~/.config/kb/themes/` (beside `config.toml`). At start kb writes each of its
shipped themes there **if no file of that name exists**, never overwriting one: edits survive upgrades and a deleted
theme comes back. The shipped eight, from page 19, each with a full `kb:` section so they render exactly as
approved: `neon-dusk` (the default), `kb-original` (Catppuccin Macchiato, kb's colours until 2026-10-19), `archwave`,
`azure-glow`, `aetheria`, `monokai`, `sapphire`, `nagai-poolside`. They live in the repo as
`crates/kb-app/assets/themes/*.yaml`, compiled in.

**D4. `config.toml` names the theme.** `theme = "neon-dusk"` (default `neon-dusk` when absent). An optional
`[colors]` table overrides single roles (`mauve = "#FF00AA"`), as `[actions.copy]` overrides the catalog. Saving
writes only the `theme` line, through `toml_edit`, keeping every other setting, comment and blank line.

**D5. Live switching.** Colours stop being fixed at start: the active palette is a global the app can replace.
Replacing it rebuilds GPUI Kit's theme from the palette and redraws the window. The non-colour tokens (type,
spacing, sizes) stay in DESIGN.md and fixed at start.

**D6. kb builds its GPUI Kit theme itself.** The mapping `design:theme` writes into `assets/themes/kb.json` (a
GPUI Kit theme from kb's roles: list, popover, input, selection, code highlighting) moves into Rust (`ui/theme.rs`),
so any theme and any `[colors]` override reach GPUI Kit's own parts too. `kb.json` and the `design:theme` task go; a
test pins the Rust output for Neon Dusk to today's `kb.json`. A highlight GPUI Kit draws translucently (the selected
list row at 20 %, text selection at 30 %) is clamped to the nearest colour it can reach instead of failing.

**D7. The picker.** More… → Administration lists **Theme…** after Refresh More…; it opens a submenu (with the `‹`
header, like Refresh More…):

- one row per theme file, sorted by label; the lead is four dots in the theme's own `mauve`, `blue`, `green` and
  `peach`; ✓ marks the current theme;
- moving the selection **previews** the theme: kb recolours at once;
- ↵ keeps it, saves `theme` to `config.toml` and shows the notice "Theme: <label>";
- Esc, or the panel losing focus, restores the theme kb had when the submenu opened and saves nothing;
- the root's search finds themes by label, marked with their submenu.

**D8. Broken and weak themes.** A file that does not parse, or lacks base00–base0F, is listed dimmed with the reason
as its row's text and cannot be previewed. A theme that fails kb's legibility rules (D9) loads, and choosing it shows
a notice naming the first failure (e.g. "Theme Foo: muted on badge is 3.9:1, below 4.5:1"). A `theme` in
`config.toml` naming no file, or a bad `[colors]` value, becomes a notice and falls back to `neon-dusk`, then to the
compiled-in copy if the folder cannot be read or written.

**D9. Legibility moves into Rust.** The contrast rules `design:validate` applies to DESIGN.md's colours today
(text and muted 4.5:1 on window, mantle, selected, badge, crust; faint 3:1 on window, mantle, crust and 2.5:1 on
badge; each accent 3:1 on its 15 % tile) become a Rust function used by D8 and by a test over every shipped theme.
DESIGN.md's `colors` block is replaced by `theme: neon-dusk`; `design:validate` keeps scoring `palettes.json` and
drops its DESIGN.md colour checks (the accent-separation advisory goes with them; the shipped themes were checked
by hand on page 19).

**D10. kb stays dark-first.** Light (`variant: light`) files are listed and work; kb ships only dark themes, and its
own surfaces assume a dark window (the rules in D9 hold either way).

## Scope

In scope: the theme file reader and mapping, the folder and its seeding, `config.toml`'s `theme` and `[colors]`,
saving with `toml_edit`, the swappable palette, the Rust GPUI Kit theme, the Theme… submenu with preview, notices,
the eight shipped theme files, the legibility function, DESIGN.md, CONTEXT.md, the catalog (`theme` action and
messages), tests.

Out of scope: light-mode design work; importing omarchy `colors.toml` or Ghostty theme files; editing themes inside
kb; per-theme fonts or spacing; watching the folder for changes while kb runs (themes are re-read each time Theme…
opens).

## Data flow

```text
start ─┬─ seed missing shipped themes into ~/.config/kb/themes/
       ├─ read config.toml: theme, [colors]
       └─ load themes/<theme>.yaml → map (D2) → kb: → [colors] → Palette ─┬─ the colours global
                                                                         └─ ui/theme.rs → GPUI Kit theme
Theme… opens → read every theme file → rows
selection moves → load that theme → replace the global (preview)
↵ → save `theme` (toml_edit) → notice      Esc / focus lost → restore the opening palette
```

## Tests and validation

- Mapping: a base16 file and a base24 file map to the D2 roles; `kb:` overrides the mapping; `[colors]` overrides
  `kb:`.
- The Rust GPUI Kit theme for Neon Dusk equals today's `assets/themes/kb.json`, key for key.
- Every shipped theme passes D9's rules; a clamped highlight never fails.
- Seeding writes missing files and leaves existing ones untouched.
- Saving changes only the `theme` line of a `config.toml` with comments and other settings.
- Broken file, missing theme, bad override: each falls back with its notice.
- UI: Theme… lists the themes with ✓ on the current one; moving the selection recolours; Esc restores; ↵ saves and
  shows the notice.
- `mise run gate`. No design shots in this pass (owner, 2026-10-19).
