# Plan 8c: Engine additions and the four entry types

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** the engine can count a query's matches, list the tags of chosen types, and give an entry's notes without
the description, command, links and source (spec §5); the app shows only Bookmarks, Cheat Sheets, Commands and
Tasks in the list, the filter and the counts (D6), while the index and the TypeScript goldens stay as they are.

**Architecture:** `kb-domain/src/search.rs` shares one FROM/WHERE builder between the page query and a new
`count`; `tags_for_types` joins `entry_tags` to `entries`. `kb-domain/src/markdown.rs` splits the Markdown builder
into its head, notes and tail blocks so `notes_body` can slice the stored `notes_markdown` exactly. `EntryList`
narrows every query to the index's types the catalog names.

**Tech Stack:** Rust, rusqlite (FTS5), the TypeScript goldens in `crates/kb-domain/tests/fixtures/sample-goldens`.

**Expects:** Plans 8a and 8b done (`Catalog::default()`, `SearchView::new(config, catalog, window, cx)`).

**Index:** [2026-10-12-kb-native-08-design-system.md](2026-10-12-kb-native-08-design-system.md).

---

## File structure

| File                                  | Change | Responsibility                                                  |
| ------------------------------------- | ------ | --------------------------------------------------------------- |
| `crates/kb-domain/src/search.rs`      | modify | `count`, `build_count_sql`, `tags_for_types`                    |
| `crates/kb-domain/tests/goldens.rs`   | modify | `count` against the goldens' full result sets                   |
| `crates/kb-domain/src/markdown.rs`    | modify | `notes_body`                                                    |
| `crates/kb-app/src/entry_list.rs`     | modify | The four types; the true count                                  |
| `crates/kb-app/src/search_view.rs`    | modify | Passes the catalog's types to the list                          |
| `crates/kb-app/src/search_view/overlays.rs` | modify | Filter choices from the visible types and their tags     |
| `crates/kb-app/tests/ui.rs`           | modify | Expectations over the sample corpus's four types                |

---

### Task 1: `search::count`

**Files:** modify `crates/kb-domain/src/search.rs`, `crates/kb-domain/tests/goldens.rs`.

- [ ] **Step 1: Write the failing tests.** Append to the `tests` module of `search.rs`:

```rust
    #[test]
    fn counts_with_the_page_query_s_filter_and_no_window() {
        let sql = build_count_sql(&query("docker", &["commands"], &["infra"]));
        assert_eq!(
            sql.sql,
            "SELECT COUNT(*) FROM entries_fts(?) fts CROSS JOIN entries e ON e.rowid = fts.rowid WHERE e.type IN (?) AND EXISTS (SELECT 1 FROM entry_tags WHERE entry_id = e.id AND tag = ?)"
        );
        assert_eq!(sql.params.len(), 3);
        assert_eq!(build_count_sql(&query("", &[], &[])).sql, "SELECT COUNT(*) FROM entries e");
    }
```

Append to `crates/kb-domain/tests/goldens.rs`:

```rust
#[test]
fn counts_what_the_typescript_engine_returns() {
    let f = fixture();
    let connection = open_database(&f.database.path().join("kb.db")).unwrap();
    let golden: Vec<GoldenQuery> =
        serde_json::from_str(&fs::read_to_string(f.goldens.join("queries.json")).unwrap()).unwrap();
    // A golden holds the whole result set only when the page did not cut it.
    for q in golden.into_iter().filter(|q| q.offset == 0 && q.results.len() < PAGE_SIZE) {
        let count = kb_domain::search::count(
            &connection,
            &SearchQuery {
                text: q.text.clone(),
                filter: FilterState {
                    types: q.types,
                    tags: q.tags,
                },
                limit: PAGE_SIZE,
                offset: 0,
            },
        )
        .unwrap();
        assert_eq!(count, q.results.len(), "{}: count", q.id);
    }
}
```

Run: `cargo nextest run -p kb-domain`
Expected: FAIL to compile: no function `build_count_sql`, no function `count`.

- [ ] **Step 2: Share the filter between the page and the count.** In `search.rs`, replace `build_search_sql` with:

```rust
/// The FROM clause, the WHERE clause and their parameters, shared by a page and its count.
struct Filtered {
    from: &'static str,
    filter: String,
    params: Vec<rusqlite::types::Value>,
    fts: bool,
}

fn filtered(query: &SearchQuery) -> Filtered {
    use rusqlite::types::Value;
    let fts = fts_prefix_query(&query.text);
    let mut conditions = Vec::new();
    let mut params: Vec<Value> = Vec::new();
    if !fts.is_empty() {
        params.push(Value::Text(fts.clone()));
    }
    if !query.filter.types.is_empty() {
        conditions.push(format!(
            "e.type IN ({})",
            vec!["?"; query.filter.types.len()].join(", ")
        ));
        params.extend(query.filter.types.iter().cloned().map(Value::Text));
    }
    for tag in &query.filter.tags {
        conditions.push("EXISTS (SELECT 1 FROM entry_tags WHERE entry_id = e.id AND tag = ?)".to_owned());
        params.push(Value::Text(tag.clone()));
    }
    let filter = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };
    Filtered {
        from: if fts.is_empty() {
            "entries e"
        } else {
            "entries_fts(?) fts CROSS JOIN entries e ON e.rowid = fts.rowid"
        },
        filter,
        params,
        fts: !fts.is_empty(),
    }
}

/// The statement the TypeScript engine runs: FTS rank order with text, title order without.
/// `CROSS JOIN` pins the FTS match as the outer loop on every SQLite version.
#[must_use]
pub fn build_search_sql(query: &SearchQuery) -> SearchSql {
    use rusqlite::types::Value;
    let Filtered {
        from,
        filter,
        mut params,
        fts,
    } = filtered(query);
    let limit = i64::try_from(query.limit).unwrap_or(i64::MAX);
    let offset = i64::try_from(query.offset).unwrap_or(i64::MAX);
    params.extend([Value::Integer(limit), Value::Integer(offset)]);
    let order = if fts { "fts.rank" } else { "e.title ASC" };
    SearchSql {
        sql: format!("SELECT e.* FROM {from}{filter} ORDER BY {order} LIMIT ? OFFSET ?"),
        params,
    }
}

/// How many entries `query` matches, whatever its page window: the status line's count.
#[must_use]
pub fn build_count_sql(query: &SearchQuery) -> SearchSql {
    let Filtered { from, filter, params, .. } = filtered(query);
    SearchSql {
        sql: format!("SELECT COUNT(*) FROM {from}{filter}"),
        params,
    }
}

/// How many entries `query` matches.
///
/// # Errors
///
/// An SQLite error.
pub fn count(connection: &Connection, query: &SearchQuery) -> rusqlite::Result<usize> {
    let SearchSql { sql, params } = build_count_sql(query);
    let count: i64 = connection
        .prepare_cached(&sql)?
        .query_row(params_from_iter(params), |row| row.get(0))?;
    Ok(usize::try_from(count).unwrap_or(0))
}
```

- [ ] **Step 3: Run the tests and see them pass.**

Run: `cargo nextest run -p kb-domain`
Expected: PASS, including `orders_by_rank_from_the_fts_match_with_text`, `orders_by_title_without_text`,
`queries_return_what_the_typescript_engine_returns` and the two new tests.

- [ ] **Step 4: Gate and commit.**

```bash
mise run gate
git add crates/kb-domain/src/search.rs crates/kb-domain/tests/goldens.rs
git commit -m "feat(domain): Count a query's matches" \
  -m "The page and the count share one FROM/WHERE builder; tested against the goldens." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: `search::tags_for_types`

**Files:** modify `crates/kb-domain/src/search.rs`.

- [ ] **Step 1: Write the failing test.** Append to the `tests` module of `search.rs`:

```rust
    #[test]
    fn lists_only_the_tags_of_the_chosen_types() {
        let dir = tempfile::tempdir().unwrap();
        let (sources, database) = (dir.path().join("s"), dir.path().join("kb.db"));
        std::fs::create_dir(&sources).unwrap();
        std::fs::write(
            sources.join("a.yml"),
            "commands:\n  ls:\n    tags: [shell]\nnotes:\n  n:\n    tags: [diary]\n",
        )
        .unwrap();
        crate::sync::sync_knowledge_base(&sources, &database, &std::sync::atomic::AtomicBool::new(false), |_| {})
            .unwrap();
        let connection = crate::sync::open_readonly(&database).unwrap();
        assert_eq!(tags_for_types(&connection, &["commands".into()]).unwrap(), ["shell"]);
        assert_eq!(tags_for_types(&connection, &[]).unwrap(), ["diary", "shell"]);
    }
```

Run: `cargo nextest run -p kb-domain search`
Expected: FAIL to compile: no function `tags_for_types`.

- [ ] **Step 2: Write it.** Add after `tags` in `search.rs`:

```rust
/// The tags of the entries of `types`, sorted; every tag when `types` is empty. The app's filter
/// offers only the tags of the types it shows.
///
/// # Errors
///
/// An SQLite error.
pub fn tags_for_types(connection: &Connection, types: &[String]) -> rusqlite::Result<Vec<String>> {
    if types.is_empty() {
        return tags(connection);
    }
    let sql = format!(
        "SELECT DISTINCT t.tag FROM entry_tags t JOIN entries e ON e.id = t.entry_id WHERE e.type IN ({}) ORDER BY t.tag ASC",
        vec!["?"; types.len()].join(", ")
    );
    connection
        .prepare_cached(&sql)?
        .query_map(params_from_iter(types), |row| row.get(0))?
        .collect()
}
```

- [ ] **Step 3: Run the tests and see them pass.**

Run: `cargo nextest run -p kb-domain search`
Expected: PASS. (`tempfile` is already a dev-dependency of kb-domain.)

- [ ] **Step 4: Gate and commit.**

```bash
mise run gate
git add crates/kb-domain/src/search.rs
git commit -m "feat(domain): List the tags of chosen entry types" \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: `markdown::notes_body`

**Files:** modify `crates/kb-domain/src/markdown.rs`.

- [ ] **Step 1: Write the failing tests.** Append to the `tests` module of `markdown.rs`:

```rust
    fn compiled(entry_type: &str, key: &str, source: &str) -> Entry {
        let raw = crate::yaml::load(source).unwrap().unwrap();
        let mut entry = crate::entry::normalize_entry("/kb/f.yml", "f.yml", entry_type, key, &raw);
        entry.notes_markdown = build_entry_markdown(&entry);
        entry
    }

    #[test]
    fn keeps_only_the_notes_for_the_body() {
        let command = compiled(
            "commands",
            "docker ps -a",
            "desc: List them\nnotes: Add `--format` for names.\nlinks: [{Docs: https://d.example}]\n",
        );
        assert_eq!(notes_body(&command), "Add `--format` for names.");
        let video = compiled(
            "bookmarks",
            "https://youtu.be/aqz-KE-bpKQ",
            "desc: A talk\nnotes:\n  - One.\n  - md: Two.\n",
        );
        assert_eq!(notes_body(&video), "One.\nTwo.");
    }

    #[test]
    fn has_no_body_without_notes() {
        let bare = compiled("tasks", "t", "desc: Only a description\n");
        assert_eq!(notes_body(&bare), "");
        let mut untouched = compiled("tasks", "t", "notes: hi\n");
        untouched.notes_markdown = "something else entirely".into();
        assert_eq!(notes_body(&untouched), "something else entirely", "an unexpected shape stays whole");
    }
```

Run: `cargo nextest run -p kb-domain markdown`
Expected: FAIL to compile: no function `notes_body`.

- [ ] **Step 2: Split the builder and slice the body.** In `markdown.rs`, replace `build_entry_markdown_with` with the
      three functions below and add `notes_body` after `build_entry_markdown`:

```rust
/// The blocks before the notes: a YouTube bookmark's thumbnail, a command's code block, the
/// description quote.
fn head_blocks(entry: &Entry) -> Vec<Block> {
    let entry_type = entry.entry_type.to_lowercase();
    let mut blocks = Vec::new();
    if entry_type.contains("bookmark")
        && let Some(id) = youtube_video_id(&entry.primary_content)
    {
        blocks.push(Block::Image {
            title: "YouTube Thumbnail",
            source: format!("https://img.youtube.com/vi/{id}/maxresdefault.jpg"),
        });
    }
    if entry_type.contains("command") {
        blocks.push(Block::Code {
            language: "bash".into(),
            content: entry.primary_content.clone(),
        });
    }
    if let Some(description) = entry.description.as_deref().map(str::trim).filter(|d| !d.is_empty()) {
        blocks.push(Block::Blockquote(description.to_owned()));
    }
    blocks
}

/// The blocks after the notes: the links and the source line.
fn tail_blocks(entry: &Entry) -> Vec<Block> {
    let mut blocks = Vec::new();
    if !entry.links.is_empty() {
        blocks.push(Block::H3("LINKS"));
        blocks.push(Block::List(
            entry
                .links
                .iter()
                .map(|l| {
                    format!(
                        "[{}]({})",
                        l.title.as_deref().filter(|t| !t.is_empty()).unwrap_or(&l.url),
                        l.url
                    )
                })
                .collect(),
        ));
    }
    blocks.push(Block::Text(format!("\n\n*Source: `{}`*", entry.source_rel_path)));
    blocks
}

fn join(blocks: &[Block]) -> String {
    blocks.iter().map(render).collect::<Vec<_>>().join("\n")
}

/// [`build_entry_markdown`], reading `file://` notes with `read`.
fn build_entry_markdown_with(entry: &Entry, read: &dyn Fn(&Path) -> io::Result<String>) -> String {
    let files = NoteFiles {
        read,
        source_file: &entry.source_file,
        included: Cell::new(false),
    };
    let mut blocks = head_blocks(entry);
    if has_notes(entry.notes.as_ref()) {
        blocks.extend(
            entry
                .notes
                .as_ref()
                .map(|notes| notes_blocks(notes, &files))
                .unwrap_or_default(),
        );
    }
    blocks.extend(tail_blocks(entry));
    join(&blocks)
}
```

```rust
/// The notes alone, for the entry view's Body (spec §5): the entry's stored `notes_markdown`
/// without its head (thumbnail, command, description) and tail (links, source), which the entry
/// view draws itself. The head and tail are rebuilt from the entry exactly as the builder wrote
/// them, so the notes are never compiled again (no `file://` read). Markdown of another shape
/// stays whole.
#[must_use]
pub fn notes_body(entry: &Entry) -> &str {
    let markdown = entry.notes_markdown.as_str();
    let head = join(&head_blocks(entry));
    let tail = join(&tail_blocks(entry));
    let rest = if head.is_empty() {
        markdown
    } else {
        match markdown.strip_prefix(head.as_str()).and_then(|rest| rest.strip_prefix('\n')) {
            Some(rest) => rest,
            None => return markdown,
        }
    };
    if rest == tail {
        return "";
    }
    rest.strip_suffix(tail.as_str())
        .and_then(|body| body.strip_suffix('\n'))
        .unwrap_or(markdown)
        .trim_end_matches('\n')
}
```

(`build_entry_markdown` keeps calling `build_entry_markdown_with`; the goldens prove the output is byte-identical.)

- [ ] **Step 3: Run the tests and see them pass.**

Run: `cargo nextest run -p kb-domain`
Expected: PASS, including `rows_match_the_typescript_engine` (the stored Markdown is unchanged). If
`keeps_only_the_notes_for_the_body` fails only on a trailing newline, the `trim_end_matches('\n')` above is the
intended behaviour: fix the expectation's whitespace, never the slicing.

- [ ] **Step 4: Gate and commit.**

```bash
mise run gate
git add crates/kb-domain/src/markdown.rs
git commit -m "feat(domain): Give an entry's notes without its head and tail" \
  -m "notes_body slices the stored Markdown; notes_markdown and the goldens are unchanged (spec §5)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: The app shows four entry types, with a true count

**Files:** modify `crates/kb-app/src/entry_list.rs`, `crates/kb-app/src/search_view.rs`,
`crates/kb-app/src/search_view/overlays.rs`, `crates/kb-app/tests/ui.rs`.

- [ ] **Step 1: Write the failing UI test.** Append to `crates/kb-app/tests/ui.rs`:

```rust
#[gpui_kit::test]
fn shows_only_the_four_entry_types_and_counts_them(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (window, view) = open(cx, sample_config(home.path()));
    let all = titles(&view, cx);
    assert_eq!(all.len(), 8, "the sample's 12 entries less 3 notes and 1 snippet: {all:?}");
    assert!(!all.iter().any(|t| t == "Café ☕ notes" || t == "An entry of an unknown type"));
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
```

Run: `cargo nextest run -p kb-app --test ui shows_only_the_four_entry_types_and_counts_them`
Expected: FAIL to compile: no method `count` / `visible_types`.

- [ ] **Step 2: Narrow the list.** In `crates/kb-app/src/entry_list.rs`:

  - change the `kb_domain::search` import to `use kb_domain::search::{FilterState, PAGE_SIZE, SearchQuery, count, entry_types, search};`;
  - add three fields to `EntryList` after `filter: FilterState,`:

```rust
    /// The catalog's entry types (D6); other types never show.
    known_types: Vec<String>,
    /// The index's types the catalog knows: what an empty type filter means.
    visible_types: Vec<String>,
    /// How many entries the query and filter match, whatever the page.
    count: usize,
```

  - in `EntryList::new`, add `known_types: Vec::new(), visible_types: Vec::new(), count: 0,` to the struct literal;
  - add this builder after `with_tag_images`:

```rust
    /// Shows only `types` (the catalog's), as the sources write them, lowercase.
    #[must_use]
    pub fn with_types(mut self, types: impl IntoIterator<Item = String>) -> Self {
        self.known_types = types.into_iter().collect();
        self.refresh();
        self
    }

    /// The index's types the list shows, sorted: the filter's type choices.
    #[must_use]
    pub fn visible_types(&self) -> &[String] {
        &self.visible_types
    }

    /// How many entries match the query and filter (the status line).
    #[must_use]
    pub fn count(&self) -> usize {
        self.count
    }
```

  - replace `refresh` with:

```rust
    /// Runs the query for the current text, filter and page window, over the visible types only.
    fn refresh(&mut self) {
        let Some(connection) = &self.connection else {
            self.entries.clear();
            self.visible_types.clear();
            self.count = 0;
            return;
        };
        self.visible_types = entry_types(connection)
            .unwrap_or_default()
            .into_iter()
            .filter(|t| self.known_types.contains(&t.trim().to_lowercase()))
            .collect();
        let types = if self.filter.types.is_empty() {
            self.visible_types.clone()
        } else {
            self.filter.types.clone()
        };
        if types.is_empty() {
            self.entries.clear();
            self.count = 0;
            return;
        }
        let query = SearchQuery {
            text: self.query.clone(),
            filter: FilterState {
                types,
                tags: self.filter.tags.clone(),
            },
            limit: self.limit,
            offset: 0,
        };
        // A failed query keeps the previous results on screen rather than emptying the list.
        if let Ok(rows) = search(connection, &query) {
            self.entries = rows.iter().map(row_to_entry).collect();
        }
        if let Ok(matched) = count(connection, &query) {
            self.count = matched;
        }
    }
```

- [ ] **Step 3: Give the list the catalog's types.** In `search_view.rs`, `SearchView::new`, change the list
      construction to:

```rust
        let types: Vec<String> = catalog.types.keys().cloned().collect();
        let list = cx.new(|cx| {
            ListState::new(
                EntryList::new(connection).with_tag_images(tag_images).with_types(types),
                window,
                cx,
            )
            .searchable(false)
        });
```

(This must sit before `catalog` moves into `Rc::new`; place `let types = …` right after `let tag_images = …`.)

- [ ] **Step 4: Offer only the visible types and their tags in the filter.** In
      `crates/kb-app/src/search_view/overlays.rs`, replace the body of `filter_sections` with:

```rust
        let delegate = self.list.read(cx).delegate();
        let Some(connection) = delegate.connection() else {
            return Vec::new();
        };
        let types = delegate.visible_types().to_vec();
        // A failed read offers no choices rather than stale ones; the list shows the index.
        match tags_for_types(connection, &types) {
            Ok(tags) => filter_groups(&types, &tags, delegate.filter()),
            Err(_) => Vec::new(),
        }
```

and change `use kb_domain::search::{FilterState, entry_types, tags};` to
`use kb_domain::search::{FilterState, tags_for_types};`.

- [ ] **Step 5: Update the sample-corpus expectations.** In `crates/kb-app/tests/ui.rs`:
  - `lists_entries_by_title_and_narrows_them_as_the_user_types`: `assert_eq!(titles(&view, cx).len(), 12);` becomes
    `assert_eq!(titles(&view, cx).len(), 8);`;
  - `runs_one_query_after_typing_settles`: `assert_eq!(titles(&view, cx).len(), 12, "nothing runs before the debounce elapses");`
    becomes `assert_eq!(titles(&view, cx).len(), 8, "nothing runs before the debounce elapses");`.

- [ ] **Step 6: Run every kb-app test.**

Run: `cargo nextest run -p kb-app`
Expected: PASS.

- [ ] **Step 7: Gate and commit.**

```bash
mise run gate
git add crates/kb-app/src/entry_list.rs crates/kb-app/src/search_view.rs crates/kb-app/src/search_view/overlays.rs crates/kb-app/tests/ui.rs
git commit -m "feat(app): Show only Bookmarks, Cheat Sheets, Commands and Tasks" \
  -m "The app narrows every query to the catalog's types and counts the matches; the index keeps every type (D6)." \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Done when

- `cargo nextest run --workspace` passes, goldens included; `mise run gate` passes.
- `todo`: set `7lTb` (narrow entry types) done with the comment `"2026-10-13: app-side, Plan 8c Task 4 (Bo3f option a)"`.
