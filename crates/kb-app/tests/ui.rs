//! UI integration tests: the production `SearchView` in a headless window over the committed
//! sample corpus, driven by real key events.

#![expect(clippy::unwrap_used, reason = "a fixture that cannot be built is a failed test")]

use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use gpui_kit::component::{ActiveTheme as _, ThemeMode};
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, Bounds, Entity, Point, TestAppContext, Window, WindowBounds, WindowOptions,
    px, size,
};
use kb_app::catalog::Catalog;
use kb_app::config::Config;
use kb_app::entry_list::DEBOUNCE;
use kb_app::keys::key_bindings;
use kb_app::search_view::SearchView;
use kb_app::window_size::WindowSize;

/// A config whose sources are the sample corpus and whose index is already built.
fn sample_config(home: &Path) -> Config {
    let sample = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../kb-domain/tests/fixtures/sample")
        .canonicalize()
        .unwrap();
    let preferences = home.join(".config/kb/config.toml");
    fs::create_dir_all(preferences.parent().unwrap()).unwrap();
    // A Skills command that lists no skills, so a refresh never runs the machine's own `skills`.
    let skills = home.join("skills");
    fs::write(&skills, "#!/bin/sh\necho '[]'\n").unwrap();
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&skills, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fs::write(
        &preferences,
        format!(
            "sources_path = \"{}\"\nskills_command = \"{}\"\n",
            sample.display(),
            skills.display()
        ),
    )
    .unwrap();
    let config = Config::load(home, &home.join("data")).unwrap();
    kb_domain::sync::sync_knowledge_base(config.sources_dir(), config.database(), &AtomicBool::new(false), |_| {})
        .unwrap();
    config
}

fn open(cx: &mut TestAppContext, config: Config) -> (AnyWindowHandle, Entity<SearchView>) {
    cx.update(gpui_kit::init);
    cx.update(|cx| cx.bind_keys(key_bindings(&Catalog::default())));
    cx.update(|cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                Point::default(),
                size(px(860.), px(520.)),
            ))),
            ..WindowOptions::default()
        };
        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| SearchView::new(config, Catalog::default(), window, cx))
        })
        .unwrap()
    })
}

fn selected_title(view: &Entity<SearchView>, cx: &mut TestAppContext) -> Option<String> {
    cx.read(|cx| {
        view.read(cx)
            .list()
            .read(cx)
            .delegate()
            .selected_entry()
            .map(|e| e.title.clone())
    })
}

fn titles(view: &Entity<SearchView>, cx: &mut TestAppContext) -> Vec<String> {
    cx.read(|cx| {
        view.read(cx)
            .list()
            .read(cx)
            .delegate()
            .entries()
            .iter()
            .map(|e| e.title.clone())
            .collect()
    })
}

#[gpui_kit::test]
fn is_always_dark_in_the_design_fonts(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        kb_app::init_theme(cx);
        assert_eq!(cx.theme().mode, ThemeMode::Dark);
        assert_eq!(cx.theme().font_family.as_ref(), "Inter");
        assert_eq!(cx.theme().mono_font_family.as_ref(), "JetBrains Mono");
    });
}

#[gpui_kit::test]
fn draws_gpui_kit_components_in_the_design_tokens(cx: &mut gpui_kit::TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        kb_app::init_theme(cx);
        let theme: serde_json::Value = serde_json::from_str(include_str!("../assets/themes/kb.json")).unwrap();
        let popover = theme["themes"][0]["colors"]["popover.background"].as_str().unwrap();
        let popover = u32::from_str_radix(popover.trim_start_matches('#'), 16).unwrap();
        assert_eq!(cx.theme().popover, gpui_kit::rgb(popover).into());
    });
}

/// Renders a frame, then acts on the window.
fn act(cx: &mut TestAppContext, window: AnyWindowHandle, action: impl FnOnce(&mut Window, &mut App)) {
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        action(window, cx);
    })
    .unwrap();
}

/// Types `text` at the focused element.
fn type_text(cx: &mut TestAppContext, window: AnyWindowHandle, text: &'static str) {
    act(cx, window, |window, cx| window.input(text, cx));
}

/// Presses the named key.
fn press(cx: &mut TestAppContext, window: AnyWindowHandle, key: &'static str) {
    act(cx, window, |window, cx| window.press(key, cx));
}

/// Clicks the element with this id.
fn click(cx: &mut TestAppContext, window: AnyWindowHandle, id: &'static str) {
    act(cx, window, |window, cx| window.click(id, cx));
}

/// Opens the filter (⌘P) and lets it render.
fn open_filter(cx: &mut TestAppContext, window: AnyWindowHandle) {
    press(cx, window, "cmd-p");
    cx.run_until_parked();
}

/// Types `query` into the open filter and confirms it with `keys` (↵ toggles, ⌘↵ isolates).
fn confirm_filter(cx: &mut TestAppContext, window: AnyWindowHandle, query: &'static str, keys: &'static str) {
    type_text(cx, window, query);
    cx.run_until_parked();
    press(cx, window, keys);
    settle(cx);
}

/// Opens the panel over the sample corpus and searches for `query`. Keep the home alive.
fn open_with_query(
    cx: &mut TestAppContext,
    query: &'static str,
) -> (tempfile::TempDir, AnyWindowHandle, Entity<SearchView>) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    type_text(cx, window, query);
    settle(cx);
    (home, window, view)
}

/// Lets the debounce elapse and every task it releases finish.
fn settle(cx: &mut TestAppContext) {
    cx.executor().advance_clock(DEBOUNCE + Duration::from_millis(10));
    cx.run_until_parked();
}

#[gpui_kit::test]
fn marks_the_selected_row_with_return_and_the_first_nine_with_their_number(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (_window, view) = open(cx, sample_config(home.path()));
    let keys = cx.read(|cx| view.read(cx).list().read(cx).delegate().row_keycaps());
    assert_eq!(keys, ["↵", "⌘2", "⌘3", "⌘4", "⌘5", "⌘6", "⌘7", "⌘8"]);
}

#[gpui_kit::test]
fn lists_entries_by_title_and_narrows_them_as_the_user_types(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    assert_eq!(titles(&view, cx).len(), 8);

    type_text(cx, window, "docker");
    settle(cx);
    assert_eq!(titles(&view, cx), ["docker ps -a"]);
}

#[gpui_kit::test]
fn copies_a_command_with_its_primary_action(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    press(cx, window, "secondary-enter");
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("kubectl get pods -A"));
    assert_eq!(
        cx.read(|cx| view.read(cx).last_notice().map(str::to_owned)).as_deref(),
        Some("Copied to clipboard")
    );
}

#[gpui_kit::test]
fn lists_the_root_actions_for_a_bookmark_with_links(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "example");
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| view.read(cx).panel_labels(cx)),
        [["Details", "Open", "Copy", "Resize", "Links…", "More…"]]
    );
}
#[gpui_kit::test]
fn runs_the_row_matching_cmd_enter_or_shift_cmd_enter_in_the_panel(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "example");
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    // Row 0 is "Details", row 1 is "Open" (caps ⌘↵), row 2 is "Copy" (caps ⇧⌘↵).
    // With row 0 selected, ⌘↵ runs row 1 ("Open"), not row 0.
    press(cx, window, "cmd-enter");
    cx.run_until_parked();
    assert_eq!(
        cx.opened_url().as_deref(),
        Some("https://example.com/docs"),
        "⌘↵ in panel ran the row with ⌘↵ caps (Open)"
    );
    assert!(!cx.read(|cx| view.read(cx).is_actions_open()));

    // Now test ⇧⌘↵: with row 0 selected, ⇧⌘↵ runs row 2 ("Copy").
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    press(cx, window, "shift-cmd-enter");
    cx.run_until_parked();
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(
        copied.as_deref(),
        Some("https://example.com/docs"),
        "⇧⌘↵ in panel ran the row with ⇧⌘↵ caps (Copy)"
    );
    assert!(!cx.read(|cx| view.read(cx).is_actions_open()));
}
#[gpui_kit::test]
fn rebuild_screen_maps_to_refreshing_and_panel_omits_details_without_entry(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));

    // 1. Rebuild screen maps to Screen::Refreshing: footer shows Stop esc
    press(cx, window, "cmd-r");
    assert!(cx.read(|cx| view.read(cx).is_rebuilding()));
    let keys = cx.read(|cx| view.read(cx).footer_keys(cx));
    assert_eq!(keys, ["Stop esc", "Actions ⌘K"]);

    // 2. ⌘K over a rebuild must not offer Details
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    let panel_labels = cx.read(|cx| view.read(cx).panel_labels(cx));
    assert!(
        !panel_labels.iter().flatten().any(|l| l == "Details"),
        "no Details over rebuild: {panel_labels:?}"
    );
    press(cx, window, "escape");
    cx.run_until_parked();

    // 3. Root page when the list is empty (no selection / no matching entry): no Details
    press(cx, window, "escape"); // return from rebuild
    settle(cx);
    type_text(cx, window, "nomatchforthisqueryxyz");
    settle(cx);
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    let empty_labels = cx.read(|cx| view.read(cx).panel_labels(cx));
    assert!(
        !empty_labels.iter().flatten().any(|l| l == "Details"),
        "no Details with empty list: {empty_labels:?}"
    );
}
#[gpui_kit::test]
fn scrolls_selected_panel_row_into_view_when_filter_overflows(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    open_filter(cx, window);
    cx.run_until_parked();
    let initial_offset = cx.read(|cx| view.read(cx).panel_scroll_offset());
    assert_eq!(initial_offset, px(0.));
    for _ in 0..12 {
        press(cx, window, "down");
        cx.run_until_parked();
    }
    let scrolled_offset = cx.read(|cx| view.read(cx).panel_scroll_offset());
    assert!(
        scrolled_offset < px(0.),
        "moving panel down scrolled the list into view (offset < 0): {scrolled_offset:?}"
    );
}

#[gpui_kit::test]
fn copies_the_path_found_from_the_root_s_search(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "rebase");
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    type_text(cx, window, "path");
    cx.run_until_parked();
    press(cx, window, "enter");
    cx.run_until_parked();
    let copied = cx.read_from_clipboard().and_then(|item| item.text()).unwrap();
    assert!(copied.ends_with("cheats.yml"), "{copied}");
    assert!(!cx.read(|cx| view.read(cx).is_actions_open()));
}

#[gpui_kit::test]
fn clears_the_search_then_leaves_more_then_closes_with_escape(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "rebase");
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    type_text(cx, window, "More");
    cx.run_until_parked();
    press(cx, window, "enter");
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| view.read(cx).panel_labels(cx)),
        [
            vec!["Notes", "Path"],
            vec!["Edit", "Refresh", "Refresh More…", "Configure"]
        ],
        "the cheat sheet has no description"
    );
    press(cx, window, "escape");
    cx.run_until_parked();
    assert_eq!(cx.read(|cx| view.read(cx).panel_labels(cx))[0][0], "Details");
    press(cx, window, "escape");
    cx.run_until_parked();
    assert!(!cx.read(|cx| view.read(cx).is_actions_open()));
    assert_eq!(
        titles(&view, cx),
        ["Interactive rebase"],
        "Esc closed the panel, not the query"
    );
}

#[gpui_kit::test]
fn opens_refresh_more_and_returns_to_more_with_escape(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "rebase");
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    type_text(cx, window, "Refresh More");
    cx.run_until_parked();
    press(cx, window, "enter");
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| view.read(cx).panel_labels(cx)),
        [["Refresh YAML", "Refresh Skills"]]
    );
    press(cx, window, "escape");
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| view.read(cx).panel_labels(cx))[1],
        ["Edit", "Refresh", "Refresh More…", "Configure"]
    );
}

#[gpui_kit::test]
fn refreshes_skills_alone_from_the_action_panel(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    type_text(cx, window, "Refresh Skills");
    cx.run_until_parked();
    press(cx, window, "enter");
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| view.read(cx).refresh_selection(cx)),
        Some(kb_domain::selection::Selection::Skills)
    );
}

#[gpui_kit::test]
fn opens_and_copies_a_link_from_links(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "example");
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    type_text(cx, window, "Links…");
    cx.run_until_parked();
    press(cx, window, "enter");
    cx.run_until_parked();
    // Check that links rows show the ⌘↵ keycap from catalog
    let details = cx.read(|cx| view.read(cx).panel_row_details(cx));
    assert_eq!(
        details[0].1[0].1.as_deref(),
        Some("⌘↵"),
        "Links rows show the ⌘↵ keycap from catalog"
    );
    press(cx, window, "cmd-enter");
    cx.run_until_parked();
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("https://example.com/changelog"));
}

#[gpui_kit::test]
fn isolates_a_filter_choice_with_cmd_return(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    open_filter(cx, window);
    confirm_filter(cx, window, "docker", "cmd-enter");
    assert!(!cx.read(|cx| view.read(cx).is_filter_open()), "Only closes the filter");
    let filter = cx.read(|cx| view.read(cx).list().read(cx).delegate().filter().clone());
    assert_eq!((filter.types.len(), filter.tags), (0, vec!["docker".to_owned()]));
}

#[gpui_kit::test]
fn keeps_the_toggled_filter_row_selected(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    open_filter(cx, window);
    // ↵ toggles the first type, adding the Clear All row above it; ↓ ↵ must reach the next type.
    press(cx, window, "enter");
    settle(cx);
    press(cx, window, "down");
    press(cx, window, "enter");
    settle(cx);
    let filter = cx.read(|cx| view.read(cx).list().read(cx).delegate().filter().clone());
    assert_eq!(filter.types.len(), 2, "two types toggled on: {filter:?}");
}

#[gpui_kit::test]
fn opens_the_detail_screen_on_enter_and_returns_on_escape(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    press(cx, window, "enter");
    assert!(cx.read(|cx| view.read(cx).is_showing_detail()));
    press(cx, window, "escape");
    assert!(!cx.read(|cx| view.read(cx).is_showing_detail()));
}

#[gpui_kit::test]
fn scrolls_the_detail_screen_with_the_arrow_keys(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "rebase");
    press(cx, window, "enter");
    // A short window, so the notes overflow it.
    act(cx, window, |window, _| window.resize(size(px(500.), px(140.))));
    act(cx, window, |_, _| {});
    press(cx, window, "cmd-down");
    let paged = cx.read(|cx| view.read(cx).detail_scroll_offset());
    assert!(paged < px(0.), "⌘↓ scrolls down a page: {paged:?}");
    press(cx, window, "up");
    assert!(
        cx.read(|cx| view.read(cx).detail_scroll_offset()) > paged,
        "↑ scrolls back up"
    );
    press(cx, window, "cmd-up");
    assert_eq!(
        cx.read(|cx| view.read(cx).detail_scroll_offset()),
        px(0.),
        "⌘↑ goes back to the top"
    );
}

#[gpui_kit::test]
fn runs_the_primary_action_with_enter_on_the_detail_screen(cx: &mut TestAppContext) {
    let (_home, window, _view) = open_with_query(cx, "explicit");
    press(cx, window, "enter");
    press(cx, window, "enter");
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("./scripts/run.sh --verbose"));
}

#[gpui_kit::test]
fn runs_one_query_after_typing_settles(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    type_text(cx, window, "docker");
    cx.run_until_parked();
    assert_eq!(titles(&view, cx).len(), 8, "nothing runs before the debounce elapses");
    settle(cx);
    assert_eq!(titles(&view, cx), ["docker ps -a"]);
}

#[gpui_kit::test]
fn moves_the_selection_with_the_arrow_keys(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    let all = titles(&view, cx);
    assert_eq!(selected_title(&view, cx).as_ref(), all.first());
    press(cx, window, "down");
    press(cx, window, "down");
    assert_eq!(selected_title(&view, cx).as_ref(), all.get(2));
    press(cx, window, "up");
    assert_eq!(selected_title(&view, cx).as_ref(), all.get(1));
    press(cx, window, "enter");
    assert!(cx.read(|cx| view.read(cx).is_showing_detail()));
    assert_eq!(selected_title(&view, cx).as_ref(), all.get(1));
}

#[gpui_kit::test]
fn selects_the_first_result_of_a_new_query(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    press(cx, window, "down");
    press(cx, window, "down");
    type_text(cx, window, "kubectl");
    settle(cx);
    assert_eq!(selected_title(&view, cx).as_deref(), Some("All pods"));
}

#[gpui_kit::test]
fn opens_on_the_list_alone_and_shows_the_split_pane_with_right(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    assert!(!cx.read(|cx| view.read(cx).is_showing_details()));
    press(cx, window, "right");
    assert!(cx.read(|cx| view.read(cx).is_showing_details()));
}

#[gpui_kit::test]
fn shows_titles_alone_beside_the_split_pane(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    let compact = |cx: &mut TestAppContext| cx.read(|cx| view.read(cx).list().read(cx).delegate().is_compact());
    assert!(!compact(cx));
    press(cx, window, "right");
    assert!(compact(cx));
    press(cx, window, "left");
    assert!(!compact(cx));
}

#[gpui_kit::test]
fn shows_details_with_right_at_the_end_of_the_query_and_hides_them_with_left(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    press(cx, window, "right");
    assert!(cx.read(|cx| view.read(cx).is_showing_details()));
    press(cx, window, "left");
    assert!(!cx.read(|cx| view.read(cx).is_showing_details()));
}

#[gpui_kit::test]
fn moves_the_caret_with_right_and_left_inside_the_query(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    press(cx, window, "left");
    assert!(
        !cx.read(|cx| view.read(cx).is_showing_details()),
        "left with the pane hidden moves the caret"
    );
    press(cx, window, "right");
    assert!(
        !cx.read(|cx| view.read(cx).is_showing_details()),
        "right mid-query moves the caret"
    );
    press(cx, window, "right");
    assert!(
        cx.read(|cx| view.read(cx).is_showing_details()),
        "right at the end shows the pane"
    );
}

#[gpui_kit::test]
fn toggles_the_window_size_with_cmd_shift_f(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    press(cx, window, "cmd-shift-f");
    assert_eq!(cx.read(|cx| view.read(cx).window_size()), WindowSize::Expanded);
    press(cx, window, "cmd-shift-f");
    assert_eq!(cx.read(|cx| view.read(cx).window_size()), WindowSize::Compact);
}

#[gpui_kit::test]
fn shows_the_magnifier_on_the_list_and_only_the_chevron_on_the_detail_screen(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    let parts = cx.read(|cx| view.read(cx).header_parts());
    assert_eq!((parts.back, parts.query, parts.filter), (false, true, true));
    press(cx, window, "enter");
    let parts = cx.read(|cx| view.read(cx).header_parts());
    assert_eq!((parts.back, parts.query, parts.filter), (true, false, false));
}

#[gpui_kit::test]
fn opens_details_with_enter_in_the_list_without_running_the_primary_action(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    cx.write_to_clipboard(gpui_kit::ClipboardItem::new_string("before".into()));
    press(cx, window, "enter");
    assert!(cx.read(|cx| view.read(cx).is_showing_detail()));
    let clipboard = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(clipboard.as_deref(), Some("before"));
}

#[gpui_kit::test]
fn the_action_bar_opens_details_and_runs_the_primary_action(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    click(cx, window, "footer-primary-action");
    assert!(cx.read(|cx| view.read(cx).is_showing_detail()));
    click(cx, window, "footer-primary-action");
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("kubectl get pods -A"));
}

// Task cost (docs/guides/parity.md, "Tasks"): each test presses exactly the keys the matrix
// counts for its task once the panel is shown.

#[gpui_kit::test]
fn task_t1_opens_a_bookmark_with_a_query_and_one_chord(cx: &mut TestAppContext) {
    let (_home, window, _view) = open_with_query(cx, "conference");
    press(cx, window, "secondary-enter");
    assert_eq!(
        cx.opened_url().as_deref(),
        Some("https://www.youtube.com/watch?v=aqz-KE-bpKQ")
    );
}

#[gpui_kit::test]
fn task_t2_copies_a_command_with_a_query_and_one_chord(cx: &mut TestAppContext) {
    let (_home, window, _view) = open_with_query(cx, "docker");
    press(cx, window, "secondary-enter");
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("docker ps -a"));
}

#[gpui_kit::test]
fn task_t3_filters_by_a_type_and_a_tag_in_one_visit(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    open_filter(cx, window);
    for choice in ["Commands", "docker"] {
        confirm_filter(cx, window, choice, "enter");
    }
    assert!(cx.read(|cx| view.read(cx).is_filter_open()), "stays open while picking");
    press(cx, window, "escape");
    assert!(!cx.read(|cx| view.read(cx).is_filter_open()));
    let filter = cx.read(|cx| view.read(cx).list().read(cx).delegate().filter().clone());
    assert_eq!(
        (filter.types, filter.tags),
        (vec!["commands".to_owned()], vec!["docker".to_owned()])
    );
    assert_eq!(titles(&view, cx), ["docker ps -a"]);
}

#[gpui_kit::test]
fn task_t4_reads_a_note_with_a_query_and_return(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "rebase");
    press(cx, window, "enter");
    assert!(cx.read(|cx| view.read(cx).is_showing_detail()));
    assert_eq!(selected_title(&view, cx).as_deref(), Some("Interactive rebase"));
}

#[gpui_kit::test]
fn task_t5_rebuilds_the_index_with_one_chord(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    press(cx, window, "cmd-r");
    assert!(cx.read(|cx| view.read(cx).is_rebuilding()));
}

#[gpui_kit::test]
fn opens_the_actions_and_the_filter_with_a_click(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    click(cx, window, "footer-open-actions");
    assert!(
        cx.read(|cx| view.read(cx).is_actions_open()),
        "a click opens the actions"
    );
    press(cx, window, "escape");
    click(cx, window, "filter-button");
    assert!(cx.read(|cx| view.read(cx).is_filter_open()), "a click opens the filter");
}

#[gpui_kit::test]
fn shows_only_the_four_entry_types_and_counts_them(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    let all = titles(&view, cx);
    assert_eq!(
        all.len(),
        8,
        "the sample's 12 entries less 3 notes and 1 snippet: {all:?}"
    );
    assert!(
        !all.iter()
            .any(|t| t == "Café ☕ notes" || t == "An entry of an unknown type")
    );
    let (count, types) = cx.read(|cx| {
        let list = view.read(cx).list().read(cx);
        (list.delegate().count(), list.delegate().visible_types().to_vec())
    });
    assert_eq!(count, 8);
    assert_eq!(types, ["bookmarks", "cheats", "commands", "tasks"]);
    type_text(cx, window, "café");
    settle(cx);
    assert!(titles(&view, cx).is_empty(), "a note never shows, even when it matches");
}

#[gpui_kit::test]
fn runs_a_row_s_primary_action_with_its_number(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, _view) = open(cx, sample_config(home.path()));
    // The sample's rows by title: A conference talk (a bookmark), All pods (a command), …
    press(cx, window, "cmd-2");
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("kubectl get pods -A"));
    press(cx, window, "cmd-1");
    assert_eq!(
        cx.opened_url().as_deref(),
        Some("https://www.youtube.com/watch?v=aqz-KE-bpKQ")
    );
}

#[gpui_kit::test]
fn shows_the_status_line_and_the_screen_keys_in_the_footer(cx: &mut TestAppContext) {
    let (_home, _window, view) = open_with_query(cx, "conference");
    let (status, keys) = cx.read(|cx| (view.read(cx).status_line(cx), view.read(cx).footer_keys(cx)));
    assert_eq!(status, "Ready • 1 result");
    assert_eq!(keys, ["Details ↵", "Open ⌘↵", "Copy ⇧⌘↵", "Actions ⌘K"]);
}

#[gpui_kit::test]
fn copies_a_bookmark_s_url_with_its_secondary_action(cx: &mut TestAppContext) {
    let (_home, window, _view) = open_with_query(cx, "conference");
    press(cx, window, "shift-cmd-enter");
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("https://www.youtube.com/watch?v=aqz-KE-bpKQ"));
}

#[gpui_kit::test]
fn ignores_right_and_left_on_the_detail_screen(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    press(cx, window, "enter");
    press(cx, window, "right");
    press(cx, window, "left");
    assert!(
        cx.read(|cx| view.read(cx).is_showing_detail()),
        "still on the detail screen"
    );
}

#[gpui_kit::test]
fn opens_a_link_with_its_number_on_the_detail_screen(cx: &mut TestAppContext) {
    let (_home, window, _view) = open_with_query(cx, "example");
    press(cx, window, "enter");
    press(cx, window, "cmd-2");
    assert_eq!(cx.opened_url().as_deref(), Some("https://example.com/guide"));
}

#[gpui_kit::test]
fn filters_the_list_by_a_tag_from_the_detail_screen(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    press(cx, window, "enter");
    click(cx, window, "entry-tag-kube-commands.yml#commands#kubectl get pods -A");
    settle(cx);
    assert!(!cx.read(|cx| view.read(cx).is_showing_detail()), "back on the list");
    let filter = cx.read(|cx| view.read(cx).list().read(cx).delegate().filter().clone());
    assert_eq!(filter.tags, ["kube"]);
}

#[gpui_kit::test]
fn caps_the_header_tags_to_three_in_the_split_pane(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let sources = home.path().join("sources");
    fs::create_dir_all(&sources).unwrap();
    fs::write(
        sources.join("tags.yml"),
        "commands:\n  many-tags:\n    cmd: ls\n    title: Command Title That Stays Visible\n    tags: [t1, t2, t3, t4, t5, t6, t7, t8]\n",
    )
    .unwrap();
    let preferences = home.path().join(".config/kb/config.toml");
    fs::create_dir_all(preferences.parent().unwrap()).unwrap();
    fs::write(&preferences, format!("sources_path = \"{}\"\n", sources.display())).unwrap();
    let config = Config::load(home.path(), &home.path().join("data")).unwrap();
    kb_domain::sync::sync_knowledge_base(config.sources_dir(), config.database(), &AtomicBool::new(false), |_| {})
        .unwrap();
    let (window, view) = open(cx, config);
    press(cx, window, "right");
    assert!(cx.read(|cx| view.read(cx).is_showing_details()));
    act(cx, window, |window, _| {
        assert!(
            window.try_find("entry-tag-t3-tags.yml#commands#many-tags").is_some(),
            "third tag drawn"
        );
        assert!(
            window.try_find("entry-tag-t4-tags.yml#commands#many-tags").is_none(),
            "fourth tag capped"
        );
    });
    click(cx, window, "entry-tag-t1-tags.yml#commands#many-tags");
    settle(cx);
    let filter = cx.read(|cx| view.read(cx).list().read(cx).delegate().filter().clone());
    assert_eq!(filter.tags, ["t1"]);
}

#[gpui_kit::test]
fn offers_refresh_when_nothing_matches(cx: &mut TestAppContext) {
    let (_home, _window, view) = open_with_query(cx, "zzqxj");
    assert_eq!(titles(&view, cx), [] as [String; 0]);
    let keys = cx.read(|cx| view.read(cx).footer_keys(cx));
    assert_eq!(keys, ["Refresh ↵", "Actions ⌘K"]);
    assert_eq!(cx.read(|cx| view.read(cx).status_line(cx)), "Ready • 0 results");
}

#[gpui_kit::test]
fn typing_and_fixing_query_does_not_rebuild_on_immediate_enter(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    type_text(cx, window, "zzqxj");
    settle(cx);
    assert_eq!(titles(&view, cx), [] as [String; 0]);
    press(cx, window, "escape"); // clears the search query
    type_text(cx, window, "docker");
    press(cx, window, "enter");
    assert!(
        !cx.read(|cx| view.read(cx).is_rebuilding()),
        "Enter while debounce is catching up must not trigger Rebuild"
    );
}

#[gpui_kit::test]
fn reports_a_refresh_and_goes_back_with_return(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    press(cx, window, "cmd-r");
    cx.run_until_parked();
    let keys = cx.read(|cx| view.read(cx).footer_keys(cx));
    assert_eq!(
        keys,
        ["Back ↵", "Actions ⌘K"],
        "the report's keys once the sync finished"
    );
    assert!(cx.read(|cx| view.read(cx).header_parts()).back);
    press(cx, window, "enter");
    assert!(
        !cx.read(|cx| view.read(cx).is_rebuilding()),
        "Back ↵ returns to the list"
    );
}

#[gpui_kit::test]
fn moves_one_row_per_arrow_in_the_action_panel(cx: &mut TestAppContext) {
    let (_home, window, view) = open_with_query(cx, "kubectl");
    press(cx, window, "cmd-k");
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| view.read(cx).panel_labels(cx))[0][..2],
        ["Details", "Copy"]
    );
    press(cx, window, "down");
    cx.run_until_parked();
    press(cx, window, "enter");
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| view.read(cx).last_notice().map(str::to_owned)),
        Some("Copied to clipboard".to_owned()),
        "one ↓ selects Copy, the second row"
    );
}

#[gpui_kit::test]
fn shows_the_filter_s_keys_in_the_footer_while_it_is_open(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    open_filter(cx, window);
    let keys = cx.read(|cx| view.read(cx).footer_keys(cx));
    assert!(
        keys[0].starts_with("Toggle") && keys[1].starts_with("Only"),
        "the filter's keys (D7), not the list's: {keys:?}"
    );
}
