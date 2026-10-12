---
name: Tokyo Storm Palette
colors:
  surface: '#0e1224'
  surface-dim: '#0e1224'
  surface-bright: '#34384c'
  surface-container-lowest: '#090d1f'
  surface-container-low: '#161b2d'
  surface-container: '#1b1f31'
  surface-container-high: '#25293c'
  surface-container-highest: '#303447'
  on-surface: '#dee1fb'
  on-surface-variant: '#c0c7d1'
  inverse-surface: '#dee1fb'
  inverse-on-surface: '#2b2f43'
  outline: '#8a919a'
  outline-variant: '#40484f'
  surface-tint: '#92ccff'
  primary: '#a3d3ff'
  on-primary: '#003351'
  primary-container: '#70b9f4'
  on-primary-container: '#004970'
  inverse-primary: '#006397'
  secondary: '#d9b9ff'
  on-secondary: '#460085'
  secondary-container: '#6214b0'
  on-secondary-container: '#cca3ff'
  tertiary: '#3ce8ac'
  on-tertiary: '#003826'
  tertiary-container: '#00cb92'
  on-tertiary-container: '#004f37'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#cce5ff'
  primary-fixed-dim: '#92ccff'
  on-primary-fixed: '#001d31'
  on-primary-fixed-variant: '#004b73'
  secondary-fixed: '#eedbff'
  secondary-fixed-dim: '#d9b9ff'
  on-secondary-fixed: '#2a0054'
  on-secondary-fixed-variant: '#6214b0'
  tertiary-fixed: '#58fdbf'
  tertiary-fixed-dim: '#2ee0a4'
  on-tertiary-fixed: '#002114'
  on-tertiary-fixed-variant: '#005138'
  background: '#0e1224'
  on-background: '#dee1fb'
  surface-variant: '#303447'
  surface-window: '#23273A'
  surface-header: '#24293E'
  surface-raised: '#24293E'
  surface-selection: '#2A3047'
  border-divider: '#353D5A'
  text-primary: '#CBD4FF'
  text-muted: '#B2BADB'
  text-faint: '#7A86B6'
  accent-blue: '#70B9F4'
  accent-green: '#30E1A5'
  accent-orange: '#F5A95D'
  accent-purple: '#A362F2'
  accent-magenta: '#E927AB'
  accent-yellow: '#D9E753'
  accent-red: '#E04F50'
typography:
  headline-lg:
    fontFamily: Inter
    fontSize: 24px
    fontWeight: '600'
    lineHeight: 32px
    letterSpacing: -0.015em
  headline-md:
    fontFamily: Inter
    fontSize: 18px
    fontWeight: '400'
    lineHeight: 26px
    letterSpacing: -0.01em
  headline-sm:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: '600'
    lineHeight: 24px
    letterSpacing: -0.005em
  body-lg:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  body-md:
    fontFamily: Inter
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 18px
  body-sm:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 16px
  label-md:
    fontFamily: Inter
    fontSize: 13px
    fontWeight: '500'
    lineHeight: 18px
  label-sm:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '500'
    lineHeight: 16px
  label-xs:
    fontFamily: Inter
    fontSize: 11px
    fontWeight: '600'
    lineHeight: 14px
    letterSpacing: 0.02em
  code-sm:
    fontFamily: JetBrains Mono
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 18px
rounded:
  sm: 0.25rem
  DEFAULT: 0.5rem
  md: 0.75rem
  lg: 1rem
  xl: 1.5rem
  full: 9999px
spacing:
  gutter: 1px
  margin: 0.5rem
  space-xs: 0.25rem
  space-sm: 0.5rem
  space-md: 0.75rem
  space-lg: 1rem
  space-xl: 1.5rem
---

## Brand & Style

This design system delivers a focused, keyboard-first desktop command palette tailored for searching and managing structured personal YAML knowledge bases. Inspired directly by native macOS productivity tools such as Raycast, the experience balances speed, spatial restraint, and low cognitive load.

The visual direction combines flat surfaces with meticulous micro-contrast. Rejecting gratuitous blur shaders or heavy translucent glassmorphism in favor of deterministic rendering performance, the system relies on flat, tone-adjacent dark layers derived from an expertly tuned Tokyo Night Storm palette. Hierarchy emerges strictly through spacing discipline, deliberate weight shifts in system typography, and inset interaction states. Accent colors are never ornamental; they serve as semantic identifiers for data types, categorical tags, and state cues. The floating window feels native, purposeful, and calm against any desktop background.

## Colors

The color system is engineered specifically for dark desktop environments using the `tokyo-night-storm-flat-separated` chromatic schema. Surfaces remain deliberately flat and closely aligned in luminance, preventing visual noise while eliminating window-level blur overhead.

### Semantic Surface Roles

- `surface-window` (`#23273A`): Base canvas for the main window, item list, and detail panes.
- `surface-header` (`#24293E`): Top command and search chrome.
- `surface-raised` (`#24293E`): Floating popovers, dropdown action menus, and metadata containers.
- `surface-selection` (`#2A3047`): Inset active row highlight, pressed elements, and active keyboard target indicators.
- `border-divider` (`#353D5A`): Hairline column dividers and structural containment outlines.

### Text & Contrast Hierarchy

- `text-primary` (`#CBD4FF`): Default readable foreground for command input, active item titles, and primary data values.
- `text-muted` (`#B2BADB`): Secondary descriptions, shortcut legends, and metadata labels.
- `text-faint` (`#7A86B6`): Search placeholder text, unselected iconography, and subtle supplementary hints.

### Type and Tag Semantics

Accents are assigned deterministically to core knowledge base entities:

- `bookmark` → `accent-purple` (`#A362F2`)
- `command` → `accent-green` (`#30E1A5`)
- `cheat` → `accent-blue` (`#70B9F4`)
- `note` → `accent-orange` (`#F5A95D`)
- `task` → `accent-magenta` (`#E927AB`)

Dynamic tags hash consistently into the standard 7-accent cycle (`blue`, `orange`, `green`, `purple`, `magenta`, `yellow`, `red`). Tag pill backgrounds blend 20% accent color over `surface-window`. Tag text is tinted with the native accent and lifted toward `text-primary` in 5% increments to guarantee a strict WCAG AA contrast ratio of at least 4.5:1.

## Typography

Typography prioritizes rapid legibility, dense information scanning, and alignment with Apple system conventions (SF Pro / Inter system fallback, with JetBrains Mono for embedded code blocks).

### Type Roles & Applications

- **Search Header (`headline-md`)**: 18px Regular. Delivers immediate focus within the main query bar.
- **Result Row Title (`body-lg`)**: 14px Regular in `text-primary`. Primary list identifier.
- **Row Descriptions (`body-sm`)**: 12px Regular in `text-muted`. Subordinate explanation beneath the title.
- **Section Headers (`label-xs`)**: 11px Semibold uppercase in `text-muted` for popover categories and grouping splits.
- **Metadata Labels & Values (`body-md` / `label-md`)**: 13px Regular and Medium for side-by-side key-value pairs.
- **Keycaps & Badges (`label-sm`)**: 12px Medium. Monospaced modifier sequences (`⌘`, `↵`, `K`) rendered within visual badges.
- **Detail Note Headings (`headline-lg`, `headline-sm`)**: 24px and 16px Semibold for Markdown content rendering.
- **Code Annotations (`code-sm`)**: 12px JetBrains Mono for shell commands, parameters, and YAML path keys.

### Casing Conventions

All commands, menu items, and control titles use Title Case ("Show Details", "Clear All Filters", "Open in Browser") matching desktop launcher standards. User content, validation messages, and system status notices use sentence case.

## Layout & Spacing

The layout is anchored to a strict 4pt (0.25rem) spacing rhythm. Whitespace defines groupings before dividers are introduced.

### Base Rhythm Scale

- `space-xs` (4px / 0.25rem): Tight spacing, such as keycap glyph separations and internal badge padding.
- `space-sm` (8px / 0.5rem): Compact spacing, panel outer insets, popover padding, and row selection X-insets.
- `space-md` (12px / 0.75rem): Standard internal element gaps, header control spacing, and icon-to-label separation.
- `space-lg` (16px / 1rem): Standard content padding, row horizontal insets, and detail section gaps.
- `space-xl` (24px / 1.5rem): Header horizontal insets and detail view outer content padding.

### Panel & Screen Layout

- **Window Modes**: Compact window defaults to 860×520pt. Expanded view scales to approximately 85% of screen real estate. The window is centered on screen.
- **Split Pane Ratio**: Asymmetric two-column presentation: Master list spans 34% width, Detail preview pane occupies the remaining 66%. The panes are separated by a 1px vertical hairline divider (`gutter`).
- **Vertical Rhythm Axis**: Interactive entry rows and the header search bar share an identical 56px height. The leading glyph axis of search back buttons and list row icons are strictly locked to the same 24px vertical alignment column.

## Elevation & Depth

Visual hierarchy avoids heavy blurs or saturated colored glows, opting for structured flat layers, subtle surface shifts, and micro-borders.

### Surface Stacking

1. **Base Canvas (`surface-window`, `#23273A`)**: The resting application plane for lists and markdown detail screens.
2. **Structural Chrome (`surface-header`, `#24293E`)**: Houses the primary input field. Integrated flush without drop shadows.
3. **Selected Element Highlight (`surface-selection`, `#2A3047`)**: Floats within list items as an inset pill (8px horizontal inset, 2px vertical inset) to indicate focus without covering the entire row bounds.
4. **Floating Popovers (`surface-raised`, `#24293E`)**: Elevated contextual menus, filter popouts, and action sheets.

### Shadows & Boundaries

- **Hairline Dividers**: A 1px border colored with `border-divider` (`#353D5A`) separates the 34/66 master-detail split pane and popover headers.
- **Popover Elevation**: Raised popovers share the header's base tone and achieve visual separation using a sharp 1px outer stroke (`#353D5A`) coupled with a dual-tier neutral drop shadow:
  - Ambient: `0 4px 12px -2px rgba(10, 12, 18, 0.45)`
  - Key: `0 16px 32px -4px rgba(8, 10, 16, 0.65)`

## Shapes

The design system uses a rounded geometry level (Level 2), calibrated for modern native desktop utility windows.

### Corner Radius Hierarchy

- **Window Panel**: 28px (`rounded-3xl` / `1.75rem`) — Generous rounded corners matching native macOS floating panels.
- **Popovers & Overlays**: 16px (`rounded-xl` / `1rem`) — Compact floating action panels.
- **List Selection Pills & Media**: 12px (`rounded-lg` / `0.75rem`) — Soft interactive selection highlights and embedded preview cards.
- **Action Menu Rows**: 8px (`rounded-md` / `0.5rem`) — Target items inside the actions popover.
- **Keycaps**: 6px (`rounded` / `0.375rem`) — Individual keyboard shortcut blocks.
- **Tag Pills**: 4px (`rounded-sm` / `0.25rem`) — Compact metadata indicators.

## Components

### Header & Command Search Bar

- **Dimensions**: Fixed 56px height, 24px horizontal padding.
- **Layout**: Left-aligned back chevron (24px fixed column), followed by a 12px gap to the search input text. No horizontal divider separates the header from the results list below.
- **Right Accessory**: Filter trigger displaying current selection (e.g., "All Entries", "Bookmarks, Cheats") accompanied by a funnel glyph and disclosure chevron.

### List Rows

- **Dimensions**: 56px height with 16px horizontal content padding.
- **Selection State**: A background fill using `surface-selection` inset by 8px horizontally and 2px vertically, finished with a 12px border radius. Non-selected items have an entirely transparent background.
- **Content Structure**: A fixed 24px icon column holding the semantic type icon or remote favicon, followed by a 12px gap to the content column. The content column stacks the primary title (`text-primary`, 14px) over the description (`text-muted`, 12px) with a 4px vertical gap.
- **Trailing Accessories**: Up to 3 metadata tag pills, an overflow badge (`+N`), and a link counter with an external link glyph.

### Tag Pills

- **Geometry**: 4px corner radius, 10px horizontal padding, 4px vertical padding, 8px gap between pills.
- **Styling**: Solid background composed of 20% accent color blended over `surface-window`. No border stroke. Text is tinted with the accent color lifted toward `text-primary` to satisfy WCAG AA contrast (4.5:1).
- **Tag Images**: When a local SVG asset is detected, the text pill is replaced by a standardized 16×16px icon slot displaying the brand glyph.

### Keycaps & Footer

- **Footer**: Flush along the bottom edge of the panel with zero background fill. Left side displays the active entry path; right side displays available actions ("Show Details ↵", "Actions ⌘K").
- **Keycap Badges**: Rendered individually (`⌘` and `K` are separate elements). Background set to `surface-selection` (`#2A3047`), 6px border radius, 4px inner horizontal padding, 2px vertical padding, with a 4px gap between adjacent keys.

### Actions Popover & Filter Dropdown

- **Geometry**: Anchored directly above the footer's action button or beneath the filter trigger. Max-height of 200px before scrolling. Outer corner radius of 16px.
- **Background**: `surface-raised` with a 1px `#353D5A` border and two-tier drop shadow.
- **Action Row**: 40px height, 8px horizontal content padding, 8px corner radius. Features the action title left-aligned and its keycap combo right-aligned. Hover and keyboard selection triggers a `surface-selection` highlight.
- **Filter Menu**: Remains open across multiple selections. Active states display checkmark glyphs (`✓`), while unselected items display subtle circular rings (`○`).

### Metadata Card & Detail Pane

- **Split Pane**: List (34%) and Detail (66%) separated by a 1px vertical hairline divider in `border-divider`.
- **Metadata Card**: Table layout inside the detail pane with 12px content insets. Labels sit left-aligned in `text-muted` (13px), values right-aligned in `text-primary` (13px). URLs feature a trailing diagonal arrow icon (`↗`).
- **Markdown Notes**: Rendered with Semibold headings, standard body text in `text-primary`, and pull quotes highlighted with a 3px vertical `accent-red` left accent bar. Code blocks sit on an inset container 2% darker than `surface-window`, with monospaced text colored via semantic syntax tokens.
