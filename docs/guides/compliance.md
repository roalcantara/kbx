# Compliance

The checklists kbx is held to, each item with its evidence; an unchecked item says why and what would close it. Sources: the GPUI Kit Coding Guides ("Implementation checklist",
"Theme and styling", "Events, actions, and focus", "Performance rules"), the GPUI Kit Design Guides ("Accessibility
checklist", "Interface language"), and the rust-best-practices quick reference.

## GPUI Kit coding guides

- [x] State and side-effect ownership are explicit: `SearchView` owns the screen and the panes; `EntryList` owns the
      query, filter, page and connection; the sync runs on the background executor and reports through `SyncFinished`
      (`search_view.rs`, `entry_list.rs`, `sync_view.rs`).
- [x] `RenderOnce` versus `Entity<T>` is deliberate: views with state and focus are entities (`SearchView`,
      `SyncView`, the list); the notes pane is a function returning an element (`notes.rs`).
- [x] Repeated elements have stable domain-based ids: the notes `TextView` is keyed by the entry id; sync report rows
      by their path; list rows by the List's own index model, which is virtualized (`notes.rs`, `sync_view.rs`).
- [x] Theme tokens replace visual literals: no hex, `rgb` or `hsla` in the app's code; colours come from the theme
      (`assets/themes/kbx.json`) and the seven accents from `assets/themes/accents.json` (`accents::accent`), both
      generated from DESIGN.md's palette by `mise run design:theme`. **Exception:** component geometry (header,
      rows, footer, popovers, the window sizes) is in points, the values of `tools/design/layout.json` as DESIGN.md
      states them; text sizes stay in the rem scale.
- [x] Keyboard actions, focus and overlays work together: every action has a binding in a context
      (`keys::key_bindings`); the up and down keys are captured from the search field, and → / ← only with the caret
      at its end; the overlays are popovers anchored to their controls, Esc leaves them before the screen, and focus
      returns to the screen after them (`task_t3_filters_by_a_type_and_a_tag_in_one_visit`,
      `leaves_a_page_of_the_actions_then_the_actions_with_escape`).
- [x] Loading, empty, error and cancellation paths are represented: empty states for no index and no match
      (`EntryList::render_empty`); rebuild stopped and failed states (`SyncStatus`); unreadable preferences and a taken
      hotkey become notifications (`main.rs`, `panel::open`); a failed filter read too (`search_view.rs`).
- [x] Long data sets use a virtualized component: `List`, paged 50 at a time like Raycast (`EntryList::load_more`).
- [x] Public API additions preserve direction and encapsulation: `kbx-domain` has no UI dependency (hk
      `domain-purity`); `kbx-app` exposes only what `main` and its UI tests use (`lib.rs`).
- [x] Tests prove behaviour at the right layer: domain units and goldens; presentation units; UI tests that press keys
      in a real window (`tests/ui.rs`).
- [x] No state mutation or `notify` in `render`; entities and subscriptions are built once in `SearchView::new`.
- [x] Root is the first level of the window and `gpui_kit::init` runs once before any view (`lib.rs::run`).
- [x] The app is always dark (D1): one approved palette; a deliberate departure from GPUI Kit's appearance guidance.
- [x] Async work is cancellable: the debounce task is dropped by the next query; a rebuild stops on Esc and rolls back
      (`entry_list::perform_search`, `SyncView::cancel`).

## GPUI Kit design guides

- [x] Every action is reachable and operable by keyboard: the action panel lists every action of the selection,
      and each has a shortcut (`actions::root_page`, `actions::more_page`).
- [x] Focus order follows the task, and focus returns after overlays (see the coding-guides item on focus).
- [x] Icon-only controls have names and tooltips: the back button has an accessibility label and the "Back (Esc)"
      tooltip; the footer buttons carry their labels and key hints (`search_view::render_header`, `render_footer`).
- [x] Status is not told by colour alone: type tiles carry a glyph and the type's name in the row's accessibility label;
      tags carry their `#name`; rebuild states have a title and text.
- [x] Copy follows Raycast's Title Case, with an ellipsis only on commands that open more choices or ask for more input:
      "Open", "Copy", "Resize", "Links…", "More…", "Clear All"
      ([DESIGN.md, Adaptations](../../DESIGN.md#adaptations); an owner-approved deviation from the GPUI Kit Design
      Guides) (`crates/kbx-app/assets/catalog.toml`). Messages stay sentences.
- [x] Errors read as what happened and what to do: "Couldn’t register ⌥K: … Another app may be using it."
- [x] An app-level Quit command exists: ⌘Q quits; the action panel has no Quit row (D9).
- [x] Content remains usable at larger text: text sizes are in the rem scale and grow with the base font (design tokens in points, applied as rems by `ui::design::pt`); the split is
      a share of the window (34% list), and ⌘⇧F expands the window when the compact size is too small.
- [x] Assistive technology can read the controls and the list: GPUI exposes them through AccessKit once a client
      asks. The back, filter and footer buttons and the search field carry their names; each list row is named by its
      visible text — title, type, description, tags (`presentation::row_label`, `names_a_row_by_its_visible_text`).
      Checked in the macOS accessibility tree of the packaged app; the test platform builds no tree, so no UI test can.
- [ ] Assistive technology can read the notes. GPUI Kit's `TextView` sets no accessibility role (0.7.1), so the notes
      pane is invisible to VoiceOver. Upstream: a role on `TextView`; until then the row label carries the description.

## Rust best practices

- [x] Fallible operations return `Result`; no `unwrap` or `expect` outside tests (`clippy::unwrap_used` and
      `expect_used` are denied by the gate, `clippy.toml` allows them only in tests).
- [x] `thiserror` for every error type (`SyncError`, `ConfigError`, `PanelError`); no `anyhow`.
- [x] `#[expect(lint, reason = …)]` instead of `#[allow]`: no `allow` attribute remains in the crates.
- [x] Clippy `pedantic` with `-D warnings` on all targets, in the gate (`mise run gate`).
- [x] Public items are documented: `missing_docs` on both library crates, an error under the gate's `-D warnings`;
      `cargo doc` with `-D warnings`.
- [x] Doc tests show the public API: the crate example syncs and searches; the query and id helpers have examples
      (`kbx_domain`, `query.rs`, `entry.rs`, `markdown.rs`); the gate runs them (`cargo test --doc`).
- [x] Tests are named for the behaviour they prove (`opens_details_with_enter_in_the_list_without_running_the_primary_action`).
- [x] The release build uses LTO (`profile.release`).
- [x] No `TODO` without an issue: none in the crates.
- [x] No unsafe code: `unsafe_code = "forbid"` for the workspace.
- [x] Static dispatch by default; `dyn` only where values are heterogeneous: the action panel's row callback
      (`ui::action_panel::OnRow`).

## Repository conventions

- [x] Tool versions pinned in `mise.toml`, including the packager and the line counter.
- [x] Dependencies audited by `cargo deny`; exceptions recorded with reasons (ADR 0005).
- [x] The release binary stays within its size budget (`tools/size/budget`, `mise run size`).
- [x] A signed, installable `.app` is built by one task (`mise run package`); the bundle id is shared with the app
      identity (`kbx_app::APP_ID`).
