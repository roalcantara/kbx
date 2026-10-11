# Visual Pass

A manual check of kb, done by a person at the keyboard, in about 20 minutes. It covers only what automated tests
cannot see: how kb looks and how it feels to use. What tests already prove is listed in the appendix, so nobody checks
it twice.

**Last run:** none yet.

## Before you start

1. Install the latest `kb.app` from the [GitHub Releases](https://github.com/roalcantara/kbx/releases), or run
   `mise run show` in the repository.
2. Set macOS to the dark appearance (System Settings › Appearance › Dark).
3. Press **⌥K** (Option+K) to open kb, then **⌘R** to index your entries. Wait for the report, then press **Esc**.
4. Your data should hold at least: a bookmark, a YouTube video bookmark, a command, an entry with tags, an entry with
   links and an entry with notes. When it has no such entry, skip the check that needs it and say so.

## The parts of the panel

| Name              | Where                                                       |
| ----------------- | ----------------------------------------------------------- |
| **search field**  | top, after the `‹` back arrow                               |
| **filter button** | top right, with a funnel icon and "All Entries"             |
| **list**          | the rows below the search field                             |
| **details pane**  | the right part of the panel, when shown                     |
| **footer**        | the bottom edge: a pill on the left, two pills on the right |
| **Actions popup** | the menu ⌘K opens                                           |
| **Filter popup**  | the menu ⌘P opens                                           |
| **detail screen** | one entry full width, opened with ↩                         |

## How to record

Tick each check that holds. When one fails, leave it unticked and write what you saw after it
(`— the row touches the panel's edge`). When done, fill in **Last run** (date, kb version) and commit:
`docs: Run the Visual Pass on <version>`. To reset for the next run:
`sed -i '' 's/^- \[x\] \*\*VP/- [ ] **VP/' docs/experiment/visual-pass.md`.

Each check reads: WHILE <the situation>, WHEN <you do this>, THE <part> SHALL <what you should see>. One check, one
thing to look at.

## VP1. Opening and closing

AS a user,
I want to interact with **KB**
SO THAT I can efficiently search and manage my knowledge base

- [ ] **VP101** WHEN `⌥K` is pressed, THEN **KB** SHALL appear in the middle of the currently focused screen.
- [ ] **VP102** WHEN **KB** appears, THEN the search field SHALL have a blinking cursor, focused and ready for typing.
- [ ] **VP103** WHEN **KB** appears, THEN list SHALL show Bookmarks, Cheat Sheets with the first row selected and highlighted.
- [ ] **VP104** WHEN another app's window is focused, THEN **KB** SHALL disappear.
- [ ] **VP105** WHEN `Esc` is pressed and the search field is empty, THEN **KB** SHALL disappear.

## VP2. Searching and the list

As a user,
I want to search and view items in the list
SO THAT I can quickly find and interact with the content I need.

- [ ] **VP201** WHILE you type a word, THE letters SHALL appear at once, without waiting for results.
- [ ] **VP202** WHILE you type, THE previous rows SHALL stay visible until the new ones arrive (no blank flash).
- [ ] **VP203** WHILE the list shows, THE first letter you type SHALL line up with the first letter of the row titles below it.
- [ ] **VP204** WHILE the list shows, THE `‹` back arrow SHALL line up with the row icons below it.
- [ ] **VP205** WHILE the list shows, THE line between the search field and the list SHALL NOT exist.
- [ ] **VP206** WHILE a row is highlighted, THE highlight SHALL have rounded corners and SHALL NOT touch the panel's left or right edge.
- [ ] **VP207** WHILE the list shows, THE row titles SHALL be in regular weight, not bold.
- [ ] **VP208** WHILE the list shows, THE row description SHALL be the same size as the title, in a dimmer colour.
- [ ] **VP209** WHILE a command row shows, THE icon at its left SHALL be a green terminal icon.
- [ ] **VP210** WHILE a bookmark row shows, THE icon at its left SHALL be the website's own icon, or a globe while it loads or when the site has none.
- [ ] **VP211** WHILE a row with tags shows, THE tags SHALL appear at its right end as coloured `#name` pills, or as a small picture for tags that have one.
- [ ] **VP212** WHILE a row with links shows, THE link icon and the number of links SHALL appear after its tags.
- [ ] **VP213** WHILE a title is too long for its row, THE title SHALL end in `…` and the tags SHALL stay visible.
- [ ] **VP214** WHEN you type text that matches nothing, THE list SHALL say that no entries match.

## VP3. The details pane

- [ ] **VP301** WHILE the cursor is at the end of your search text, WHEN you press →, THE details pane SHALL open on the right.
- [ ] **VP302** WHILE the details pane is open, THE list SHALL take about one third of the panel's width.
- [ ] **VP303** WHILE the details pane is open, WHEN you press ↓, THE details pane SHALL show the newly highlighted entry.
- [ ] **VP304** WHILE an entry with a description is highlighted, THE description SHALL show as a quote with a red bar at its left.
- [ ] **VP305** WHILE an entry with notes is highlighted, THE notes SHALL show headings, lists and code with coloured syntax.
- [ ] **VP306** WHILE a YouTube video bookmark is highlighted, THE video's thumbnail SHALL show at the top of the pane.
- [ ] **VP307** WHILE any entry is highlighted, THE box below the notes SHALL list its details with labels on the left, values on the right and every other row shaded.
- [ ] **VP308** WHILE an entry with tags is highlighted, THE tags in that box SHALL have the same colours as in its list row.
- [ ] **VP309** WHILE a bookmark is highlighted, WHEN you click its URL in that box, THE browser SHALL open that page.
- [ ] **VP310** WHILE an entry's notes are longer than the pane, WHEN you scroll over the pane, THE notes SHALL scroll.
- [ ] **VP311** WHILE the details pane is open and the cursor is at the end of your search text, WHEN you press ←, THE details pane SHALL close.

## VP4. The detail screen

- [ ] **VP401** WHILE an entry with notes is highlighted, WHEN you press ↩, THE entry's notes SHALL fill the panel.
- [ ] **VP402** WHILE the detail screen shows, THE top of the panel SHALL show only the `‹` back arrow: no search field, no filter button.
- [ ] **VP403** WHILE the detail screen shows, THE notes SHALL start at the top, without the entry's title above them.
- [ ] **VP404** WHILE the detail screen shows, WHEN you press →, THE entry's details SHALL appear in a column on the right.
- [ ] **VP405** WHILE the detail screen shows, THE footer's left pill SHALL show the entry's icon and title.
- [ ] **VP406** WHILE the detail screen shows, WHEN you press Esc, THE list SHALL return with the same search text and the same row highlighted.

## VP5. The footer

- [ ] **VP501** WHILE the list shows, THE footer SHALL be separate rounded pills floating above the bottom edge, not a full-width bar.
- [ ] **VP502** WHILE the list shows, THE right pills SHALL read "Show Details" with a `↵` key and "Actions" with separate `⌘` and `K` keys.

## VP6. The Actions popup

- [ ] **VP601** WHEN you press ⌘K, THE Actions popup SHALL open just above the footer's Actions pill, aligned to its right.
- [ ] **VP602** WHILE the Actions popup is open, THE search field SHALL sit at its bottom.
- [ ] **VP603** WHILE the Actions popup is open, THE first two actions SHALL be "Show Details" and the entry's main action (for a bookmark, "Open in Browser").
- [ ] **VP604** WHILE the Actions popup is open, THE actions that have a shortcut SHALL show its keys, one key per box.
- [ ] **VP605** WHILE the Actions popup is open, THE popup SHALL stand out from the panel by its border and shadow.
- [ ] **VP606** WHILE the Actions popup is open, WHEN you type `copy`, THE popup SHALL list only actions containing "copy".
- [ ] **VP607** WHILE an entry with links is highlighted and the Actions popup is open, WHEN you choose "Links", THE popup SHALL list, for each link, "Open" and "Copy URL".
- [ ] **VP608** WHILE the Actions popup is open, WHEN you choose "Open Source File", THE popup SHALL close and the entry's file SHALL open in your editor.

## VP7. The Filter popup

- [ ] **VP701** WHEN you press ⌘P, THE Filter popup SHALL open just below the filter button, aligned to its right.
- [ ] **VP702** WHILE the Filter popup is open, THE filter button's arrow SHALL point up.
- [ ] **VP703** WHILE the Filter popup is open, THE entry types and tags SHALL each show an empty circle (○).
- [ ] **VP704** WHILE the Filter popup is open, WHEN you choose a type, THE popup SHALL stay open and that type SHALL show a filled circle with a tick.
- [ ] **VP705** WHILE a filter is active, THE top of the Filter popup SHALL show how many filters are on (for example "1 Filter") and "Clear All Filters".
- [ ] **VP706** WHILE a filter is active, WHEN you press Esc, THE filter button SHALL show the filter's name instead of "All Entries".

## VP8. The expanded window

- [ ] **VP801** WHEN you press ⌘⇧F, THE panel SHALL grow to most of the screen and stay centred.
- [ ] **VP802** WHILE the panel is expanded, WHEN you go through sections 2 to 7 again, THE checks SHALL hold as they did in the small window.
- [ ] **VP803** WHILE the panel is expanded, THE extra room SHALL go to longer titles and notes, not to wider empty gaps.
- [ ] **VP804** WHILE the panel is expanded, WHEN you press ⌘⇧F, THE panel SHALL shrink back and stay centred.

## VP9. Rebuilding the index

- [ ] **VP901** WHEN you press ⌘R, THE panel SHALL show the rebuild's progress.
- [ ] **VP902** WHEN the rebuild ends, THE panel SHALL list each of your source files with its number of entries, and the reason for any file that failed.
- [ ] **VP903** WHILE the rebuild report shows, WHEN you press Esc, THE list SHALL return.

## Appendix: already automated

Automated tests prove these on every push (`mise run gate`, CI). They are not part of the manual pass.

| What                                                                                 | Proved by                                                                                                                                     |
| ------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------- |
| Typing narrows the list once typing settles; a new query selects its first result    | `lists_entries_by_title_and_narrows_them_as_the_user_types`, `runs_one_query_after_typing_settles`, `selects_the_first_result_of_a_new_query` |
| ↑ ↓ move the selection                                                               | `moves_the_selection_with_the_arrow_keys`                                                                                                     |
| On an entry, ↓ ↑ scroll its notes and ⌘↓ ⌘↑ a page, within their ends                | `scrolls_the_detail_screen_with_the_arrow_keys`, `notes::tests::scrolls_within_the_notes_and_stops_at_their_ends`                             |
| → / ← mid-text move the cursor, not the pane; ⌘D toggles the pane                    | `moves_the_caret_with_right_and_left_inside_the_query`, `opens_on_the_list_alone_and_toggles_the_split_pane_with_cmd_d`                       |
| ↩ opens the detail screen without running the main action; ↩ there runs it           | `opens_details_with_enter_in_the_list_without_running_the_primary_action`, `runs_the_primary_action_with_enter_on_the_detail_screen`          |
| → / ← toggle the metadata on the detail screen; its header shows only the back arrow | `shows_and_hides_the_metadata_with_right_and_left_on_the_detail_screen`, `shows_only_the_back_chevron_in_the_detail_screen_header`            |
| ⌘↩ runs the main action (opens a bookmark, copies a command)                         | `task_t1_…`, `task_t2_…`, `copies_a_command_with_its_primary_action`                                                                          |
| The footer pills run the same actions as their keys                                  | `the_action_bar_opens_details_and_runs_the_primary_action`                                                                                    |
| Copy… opens a page of copy actions; Esc leaves the page, then the popup              | `copies_the_title_from_the_copy_page_of_the_actions`, `leaves_a_page_of_the_actions_then_the_actions_with_escape`                             |
| Picking filters keeps the Filter popup open and narrows the list                     | `task_t3_filters_by_a_type_and_a_tag_in_one_visit`                                                                                            |
| ⌘⇧F switches the window size; ⌘R starts a rebuild                                    | `toggles_the_window_size_with_cmd_shift_f`, `task_t5_rebuilds_the_index_with_one_chord`                                                       |
| The dark theme uses DESIGN.md's palette; every text meets WCAG AA contrast           | `draws_dark_mode_in_the_palette_of_design_md`; `tools/design/validate.mjs`                                                                    |
| Tags take the same colour as in the Raycast extension; types their tint              | `hashes_tags_as_the_raycast_extension_does`, `tints_each_type_as_raycast`                                                                     |
| Rows show at most three tags, then `+N`                                              | `collapses_tags_after_three`                                                                                                                  |

## Appendix: to automate next

Rules that only a reviewer could check today, and that tests can prove (TODO):

- Every command, button and section label is in Title Case; messages are sentences.
- Every key shown in the Actions popup and the footer is the key that runs that action.
- Esc with search text clears it; the filter resets when kb restarts; the window size holds after kb hides.
- ⌘⇧C copies a bookmark's URL; ⌘O opens the source file; ⌘⇧, opens the preferences file.
- Later, with reference screenshots: the colours of each surface, the row and header alignment, the split width.

- [ ] **VP106** WHILE **KB** is open, THE desktop behind it SHALL NOT show through it.
