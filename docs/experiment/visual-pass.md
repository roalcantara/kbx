# Visual Pass

What the tests cannot see, checked by hand in the running app until each check is automated. Every check verifies a
rule that lives elsewhere and cites it; the doctrine
[verify-what-users-see](../doctrine/verify-what-users-see.doctrine.md) says where each kind of rule lives.

**Last run:** none yet.

## How to run

1. `mise run show`, with the system in dark appearance and your own sources indexed (⌘R once).
2. Go through the phases in order, compact first; phase 8 repeats the layout checks in expanded (⌘⇧F).
3. Tick what holds. Leave a failure unticked with ` — <what was seen>` after it, and file it in TODO.md.
4. Update **Last run** (date, kb version, sizes) and commit the ticks: `docs: Run the Visual Pass on <version>`.
5. Before the next run, untick everything: `sed -i '' 's/^- \[x\] \*\*VP-/- [ ] **VP-/' docs/experiment/visual-pass.md`.

## Tags

How each check is verified, and the route to automating it:

- **[test]** a UI or unit test already proves it; the test is named. A manual run only confirms the app as built.
- **[probe]** scriptable from outside: the CGEvent `probe` (`tools/bench/probes`) drives keys; a screenshot or the
  window list shows the result.
- **[pixel]** needs an image compared with a reference capture.
- **[eye]** human judgement: balance, legibility, calm.

## Sources

`DESIGN §n` is a DESIGN.md section; `PRODUCT` is PRODUCT.md; `Keys` is the README's key table; `Sn`, `An`, `Fn`, `Ln`
and `Tn` are rows of [parity.md](parity.md); `UI-nn` is a UI comparison finding, mapped in DESIGN.md's appendix;
`compliance` is [compliance.md](compliance.md); `CONTEXT` is CONTEXT.md.

## 0. Global

- [ ] **VP-0.01** [pixel] THE panel SHALL draw each surface in its palette role: window, header, selection, raised,
      divider (DESIGN §2).
- [ ] **VP-0.02** [eye] THE panel SHALL keep colour for meaning only: type tints, tags, state (DESIGN §1, §2).
- [ ] **VP-0.03** [pixel] THE panel SHALL tint each type and each tag with its accent, the same tag the same accent on
      every screen (DESIGN §2 Type tints, Tag accents).
- [ ] **VP-0.04** [eye] THE panel SHALL keep every text legible on its surface, tag pills included (DESIGN §2 Pills;
      PRODUCT Accessibility & Inclusion).
- [ ] **VP-0.05** [eye] THE panel SHALL draw each text element in its size, weight and colour (DESIGN §3).
- [ ] **VP-0.06** [eye] THE panel SHALL label commands, buttons and sections in Title Case and write messages as
      sentences (DESIGN §3; A11; compliance).
- [ ] **VP-0.07** [eye] THE panel SHALL space and inset equivalent content alike on every screen, with a divider only
      at a pane or section boundary (DESIGN §5).
- [ ] **VP-0.08** [eye] THE panel SHALL be flat and opaque, overlays separated by their edge and shadow (DESIGN §6).
- [ ] **VP-0.09** [eye] THE panel SHALL name nothing by colour alone: a tag shows its `#name` or tooltip, a type its
      icon (PRODUCT Accessibility & Inclusion; compliance).
- [ ] **VP-0.10** [eye] THE panel SHALL use CONTEXT's words in every label and message (CONTEXT).

## 1. Open and hide

- [ ] **VP-1.01** [probe] WHEN ⌥K is pressed anywhere THE panel SHALL appear centred on the screen with the query
      field focused (Keys; L01; CONTEXT Hotkey).
- [ ] **VP-1.02** [probe] WHEN the panel opens THE list SHALL show the entries with the first selected and no split
      pane (S02, S09).
- [ ] **VP-1.03** [probe] WHEN another window takes focus THE panel SHALL hide (CONTEXT Panel; L01).
- [ ] **VP-1.04** [test] WHEN Esc is pressed with a query THE panel SHALL clear it, and with none, hide (Keys;
      L01; `opens_the_detail_screen_on_enter_and_returns_on_escape`).
- [ ] **VP-1.05** [probe] WHEN ⌥K is pressed again after hiding THE panel SHALL reappear as it was left (L01).
- [ ] **VP-1.06** [probe] WHEN ⌘Q is pressed THE app SHALL quit (Keys; L02).

## 2. Search and list

- [ ] **VP-2.01** [test] WHEN the user types THE list SHALL narrow to matching entries once typing settles, ranked
      (S01, S02; `lists_entries_by_title_and_narrows_them_as_the_user_types`, `runs_one_query_after_typing_settles`).
- [ ] **VP-2.02** [eye] WHILE a query runs THE previous results SHALL stay on screen and typing SHALL never stall
      (S04; PRODUCT principle 4).
- [ ] **VP-2.03** [pixel] THE query's first character SHALL start where row titles start, and the back chevron sit on
      the row-icon axis (DESIGN §4 Header, §5).
- [ ] **VP-2.04** [eye] THE header SHALL show the query field's placeholder, the filter button with its funnel, the
      current filter and a closed chevron, and no divider below (DESIGN §4 Header; UI-02).
- [ ] **VP-2.05** [pixel] THE selected row SHALL be an inset, rounded fill, never a full-width bar (DESIGN §4 List row;
      UI-03).
- [ ] **VP-2.06** [eye] THE list SHALL show on each row its tinted type icon, or a bookmark's favicon with the globe as fallback
      (DESIGN §4 List row; S05; UI-21).
- [ ] **VP-2.07** [eye] THE list SHALL show on each row a regular-weight title and a muted description of the same size
      (DESIGN §3, §4 List row; S06; UI-04).
- [ ] **VP-2.08** [eye] THE list SHALL end each row with at most three tags, each a filled pill or its image, then `+N`, then the
      link count (DESIGN §4 List row, Tag pill; S07; UI-05, UI-06).
- [ ] **VP-2.09** [eye] WHEN a title or description is longer than its row THE row SHALL truncate it, the tags still
      in view (DESIGN §4 List row).
- [ ] **VP-2.10** [test] WHEN ↑ or ↓ is pressed THE selection SHALL move one row and stay in view (Keys;
      `moves_the_selection_with_the_arrow_keys`).
- [ ] **VP-2.11** [test] WHEN a new query returns THE first result SHALL be selected
      (`selects_the_first_result_of_a_new_query`).
- [ ] **VP-2.12** [probe] WHEN the selection reaches the end of a page THE next page SHALL load (S03).
- [ ] **VP-2.13** [eye] IF nothing matches THE list SHALL say so, and IF there is no index, offer ⌘R (S08).

## 3. Split pane

- [ ] **VP-3.01** [test] WHEN → is pressed with the caret at the query's end THE split pane SHALL open, and WHEN ← is
      pressed the same way, close (Keys; S09; A06;
      `shows_details_with_right_at_the_end_of_the_query_and_hides_them_with_left`).
- [ ] **VP-3.02** [test] WHEN → or ← is pressed mid-query THE caret SHALL move and the pane stay as it is (DESIGN
      Adaptations; `moves_the_caret_with_right_and_left_inside_the_query`).
- [ ] **VP-3.03** [test] WHEN ⌘D is pressed THE split pane SHALL toggle (DESIGN Adaptations;
      `opens_on_the_list_alone_and_toggles_the_split_pane_with_cmd_d`).
- [ ] **VP-3.04** [pixel] WHILE the pane is open THE list SHALL take about a third of the width and the pane the rest,
      one divider between (DESIGN §4 Split pane; UI-08).
- [ ] **VP-3.05** [eye] WHILE the pane is open THE rows SHALL show their titles without tags or descriptions crowding
      them (DESIGN §4 List row, Split pane).
- [ ] **VP-3.06** [eye] WHEN the selection moves THE pane SHALL show the newly selected entry (S09).
- [ ] **VP-3.07** [eye] THE pane SHALL show the notes, or the "no notes" placeholder, above the metadata card (S09,
      S12; DESIGN §4 Notes).
- [ ] **VP-3.08** [pixel] THE description SHALL be a quote with the red bar (DESIGN §4 Notes; UI-10).
- [ ] **VP-3.09** [eye] THE notes SHALL render headings, lists, tables, highlighted code and images (S12).
- [ ] **VP-3.10** [eye] WHEN a YouTube bookmark is selected THE pane SHALL show its thumbnail first (S12; DESIGN §4
      Notes; UI-20).
- [ ] **VP-3.11** [eye] THE metadata card SHALL list its rows in order, labels left and values right, alternate rows
      shaded, Type and Tags as pills, the source with a file icon (S10; DESIGN §4 Metadata card; UI-09).
- [ ] **VP-3.12** [probe] WHEN a URL in the card is clicked THE browser SHALL open it, and the row show ↗ (DESIGN §4
      Metadata card; UI-22).
- [ ] **VP-3.13** [eye] WHEN the notes are longer than the pane THE notes SHALL scroll, the card staying reachable
      (DESIGN §4 Split pane).

## 4. Detail screen

- [ ] **VP-4.01** [test] WHEN ↩ is pressed in the list THE detail screen SHALL open without running the primary action
      (Keys; A02; S11; `opens_details_with_enter_in_the_list_without_running_the_primary_action`).
- [ ] **VP-4.02** [test] THE header SHALL show only the back chevron (DESIGN §4 Detail screen, Header; UI-11;
      `shows_only_the_back_chevron_in_the_detail_screen_header`).
- [ ] **VP-4.03** [eye] THE screen SHALL start with the notes, without a title block, the thumbnail first for a YouTube
      bookmark (DESIGN §4 Detail screen; UI-12, UI-23).
- [ ] **VP-4.04** [test] WHEN → or ← is pressed THE metadata column SHALL show or hide on the right, about a third wide
      (Keys; A06; DESIGN §4 Detail screen;
      `shows_and_hides_the_metadata_with_right_and_left_on_the_detail_screen`).
- [ ] **VP-4.05** [test] WHEN ↩ is pressed THE primary action SHALL run (Keys; A04;
      `runs_the_primary_action_with_enter_on_the_detail_screen`).
- [ ] **VP-4.06** [test] WHEN Esc is pressed THE list SHALL return with the query and selection kept (Keys; L01;
      `opens_the_detail_screen_on_enter_and_returns_on_escape`).

## 5. Footer

- [ ] **VP-5.01** [eye] THE footer SHALL be two floating pills over the panel's bottom edge, with no full-width bar or
      divider (DESIGN §4 Footer; UI-07).
- [ ] **VP-5.02** [eye] THE left pill SHALL name the context: the extension in the list, the entry (with its tinted
      icon) on the detail screen, the rebuild while it runs (DESIGN §4 Footer; S13).
- [ ] **VP-5.03** [eye] THE right pills SHALL show the primary action of the screen and Actions, each key its own
      keycap (DESIGN §4 Footer; S13).
- [ ] **VP-5.04** [test] WHEN a footer pill is clicked THE same action as its key SHALL run
      (`the_action_bar_opens_details_and_runs_the_primary_action`).

## 6. Actions popover

- [ ] **VP-6.01** [probe] WHEN ⌘K is pressed THE popover SHALL open above the footer's Actions button, opening upwards,
      with its search focused (Keys; A01; DESIGN §4 Actions popover; UI-13).
- [ ] **VP-6.02** [eye] THE popover SHALL list the actions in Raycast's order, sectioned, the list's first two rows
      Show Details and the primary action (A01–A03; DESIGN §4 Actions popover; UI-15).
- [ ] **VP-6.03** [eye] THE popover SHALL show on each row its keys, one keycap per key, and those keys SHALL run it from the list
      (A01; Keys; UI-14).
- [ ] **VP-6.04** [eye] THE search field SHALL sit at the bottom, and typing SHALL narrow the rows (DESIGN §4 Actions
      popover; UI-16).
- [ ] **VP-6.05** [test] WHEN Copy… is chosen THE popover SHALL show its page in place, and its rows copy (A09; UI-16;
      `copies_the_title_from_the_copy_page_of_the_actions`).
- [ ] **VP-6.06** [eye] WHEN Links is chosen on an entry with links THE popover SHALL show, per link, Open and Copy URL
      (A08).
- [ ] **VP-6.07** [test] WHEN Esc is pressed on a page THE main page SHALL return, and on the main page the popover
      close, the screen untouched (`leaves_a_page_of_the_actions_then_the_actions_with_escape`).
- [ ] **VP-6.08** [eye] THE popover SHALL be separated from the panel by its edge and shadow, not a lighter fill
      (DESIGN §6).
- [ ] **VP-6.09** [eye] WHEN an action runs THE popover SHALL close and the keyboard return to the screen (DESIGN §4
      Actions popover).
- [ ] **VP-6.10** [eye] WHILE the detail screen shows THE popover SHALL lead with the primary action on ↩ and offer Show/Hide
      Metadata (A04, A06).

## 7. Filter dropdown

- [ ] **VP-7.01** [probe] WHEN ⌘P is pressed THE dropdown SHALL open under the filter button, its chevron turned up,
      its search focused (Keys; F01; DESIGN §4 Filter dropdown, Header; UI-17).
- [ ] **VP-7.02** [eye] THE dropdown SHALL list Types and Tags, each choice marked ✓ or ○ (F01; DESIGN §4 Filter
      dropdown; UI-18).
- [ ] **VP-7.03** [test] WHEN a choice is picked THE list SHALL refilter at once and THE dropdown SHALL stay open with
      a fresh search (F02; T3; UI-18; `task_t3_filters_by_a_type_and_a_tag_in_one_visit`).
- [ ] **VP-7.04** [eye] WHILE a filter is active THE dropdown SHALL lead with the count of filters and Clear All
      Filters, and the button SHALL name the filter (F01; DESIGN §4 Filter dropdown; UI-19).
- [ ] **VP-7.05** [probe] WHEN Esc or ⌘P is pressed THE dropdown SHALL close, the filter kept and the keyboard back in
      the query (Keys; DESIGN §4 Filter dropdown).
- [ ] **VP-7.06** [probe] WHEN the app is relaunched THE filter SHALL be reset (F03).

## 8. Window size

- [ ] **VP-8.01** [test] WHEN ⌘⇧F is pressed THE panel SHALL switch between compact and expanded, and back
      (Keys; DESIGN §5; CONTEXT Size toggle; `toggles_the_window_size_with_cmd_shift_f`).
- [ ] **VP-8.02** [probe] WHEN the size changes THE panel SHALL keep its centre on the screen (DESIGN §5, Adaptations).
- [ ] **VP-8.03** [eye] WHILE expanded THE panel SHALL satisfy VP-2.03 to VP-2.09, VP-3.04, VP-3.05, VP-3.07,
      VP-3.11, VP-4.03, VP-4.04, VP-5.01 to VP-5.03, VP-6.01, VP-6.08 and VP-7.01 (DESIGN §5).
- [ ] **VP-8.04** [eye] WHILE expanded THE extra room SHALL go to content (longer titles, notes, the pane), not to
      wider gaps (DESIGN §5).
- [ ] **VP-8.05** [probe] WHEN the panel hides and reopens THE size SHALL hold for the session (CONTEXT Size toggle).

## 9. Rebuild

- [ ] **VP-9.01** [test] WHEN ⌘R is pressed THE rebuild screen SHALL replace the list and the rebuild start (Keys;
      A10; CONTEXT Sync; `task_t5_rebuilds_the_index_with_one_chord`).
- [ ] **VP-9.02** [eye] WHILE the rebuild runs THE screen SHALL show its progress, and Esc SHALL cancel it, the
      previous index kept (CONTEXT Sync).
- [ ] **VP-9.03** [eye] WHEN the rebuild ends THE screen SHALL show the sync report per source file, failures with
      their reason (CONTEXT Sync report).
- [ ] **VP-9.04** [eye] WHEN the rebuild ends THE list SHALL show the new index after Esc (CONTEXT Sync).

## 10. Keyboard map

- [ ] **VP-10.01** [test] WHEN ⌘↩ is pressed in the list THE primary action SHALL run (Keys; A03;
      `task_t1_opens_a_bookmark_with_a_query_and_one_chord`, `task_t2_copies_a_command_with_a_query_and_one_chord`).
- [ ] **VP-10.02** [probe] WHEN ⌘⇧C is pressed on a bookmark THE URL SHALL be copied (Keys; A05).
- [ ] **VP-10.03** [probe] WHEN ⌘O is pressed THE entry's source file SHALL open (Keys; A07).
- [ ] **VP-10.04** [probe] WHEN ⌘⇧, is pressed THE preferences file SHALL open, created from its template when missing
      (Keys; A10).
- [ ] **VP-10.05** [eye] THE keys every overlay and pill shows SHALL be the keys that run them (Keys; A01).

## Summary

| Phase              | Checks | test | probe | pixel | eye |
| ------------------ | ------ | ---- | ----- | ----- | --- |
| 0. Global          | 10     | 0    | 0     | 2     | 8   |
| 1. Open and hide   | 6      | 1    | 5     | 0     | 0   |
| 2. Search and list | 13     | 3    | 1     | 2     | 7   |
| 3. Split pane      | 13     | 3    | 1     | 2     | 7   |
| 4. Detail screen   | 6      | 5    | 0     | 0     | 1   |
| 5. Footer          | 4      | 1    | 0     | 0     | 3   |
| 6. Actions popover | 10     | 2    | 1     | 0     | 7   |
| 7. Filter dropdown | 6      | 1    | 3     | 0     | 2   |
| 8. Window size     | 5      | 1    | 2     | 0     | 2   |
| 9. Rebuild         | 4      | 1    | 0     | 0     | 3   |
| 10. Keyboard map   | 5      | 1    | 3     | 0     | 1   |
