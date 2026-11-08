//! Searching the index: one page of entries for a query and a filter.

use rusqlite::{Connection, params_from_iter};

use crate::query::fts_prefix_query;
use crate::row::IndexRow;

/// Entries per page, as Raycast pages a list.
pub const PAGE_SIZE: usize = 50;

/// The entry types and tags a query is narrowed to: any selected type, every selected tag.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterState {
    /// Selected entry types.
    pub types: Vec<String>,
    /// Selected tags.
    pub tags: Vec<String>,
}

/// The search text, filter and page window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    /// What the user typed.
    pub text: String,
    /// The filter, if any.
    pub filter: FilterState,
    /// Page size.
    pub limit: usize,
    /// Rows to skip.
    pub offset: usize,
}

/// A statement and its parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchSql {
    /// The SQL.
    pub sql: String,
    /// Its parameters, in order.
    pub params: Vec<rusqlite::types::Value>,
}

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

/// The search statement: FTS rank order with text, title order without.
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
    let Filtered {
        from, filter, params, ..
    } = filtered(query);
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

/// The entry types in the index, sorted: the filter's type choices.
///
/// # Errors
///
/// An SQLite error.
pub fn entry_types(connection: &Connection) -> rusqlite::Result<Vec<String>> {
    column(connection, "SELECT DISTINCT type FROM entries ORDER BY type ASC")
}

/// The tags in the index, sorted: the filter's tag choices.
///
/// # Errors
///
/// An SQLite error.
pub fn tags(connection: &Connection) -> rusqlite::Result<Vec<String>> {
    column(connection, "SELECT DISTINCT tag FROM entry_tags ORDER BY tag ASC")
}

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

fn column(connection: &Connection, sql: &str) -> rusqlite::Result<Vec<String>> {
    connection
        .prepare_cached(sql)?
        .query_map([], |row| row.get(0))?
        .collect()
}

/// One page of rows for a query.
///
/// # Errors
///
/// An SQLite error.
pub fn search(connection: &Connection, query: &SearchQuery) -> rusqlite::Result<Vec<IndexRow>> {
    let SearchSql { sql, params } = build_search_sql(query);
    let mut statement = connection.prepare_cached(&sql)?;
    let rows = statement.query_map(params_from_iter(params), |row| {
        Ok(IndexRow {
            id: row.get("id")?,
            title: row.get("title")?,
            description: row.get("description")?,
            entry_type: row.get("type")?,
            notes: row.get("notes")?,
            notes_markdown: row.get("notes_markdown")?,
            primary_content: row.get("primary_content")?,
            links_json: row.get("links_json")?,
            source_file: row.get("source_file")?,
            source_rel_path: row.get("source_rel_path")?,
            raw_yaml: row.get("raw_yaml")?,
        })
    })?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(text: &str, types: &[&str], tags: &[&str]) -> SearchQuery {
        SearchQuery {
            text: text.into(),
            filter: FilterState {
                types: types.iter().map(|s| (*s).into()).collect(),
                tags: tags.iter().map(|s| (*s).into()).collect(),
            },
            limit: 25,
            offset: 50,
        }
    }

    #[test]
    fn orders_by_rank_from_the_fts_match_with_text() {
        let sql = build_search_sql(&query("docker", &["commands"], &["infra"]));
        assert!(
            sql.sql
                .contains("FROM entries_fts(?) fts CROSS JOIN entries e ON e.rowid = fts.rowid")
        );
        assert!(
            sql.sql.contains(
                "WHERE e.type IN (?) AND EXISTS (SELECT 1 FROM entry_tags WHERE entry_id = e.id AND tag = ?)"
            )
        );
        assert!(sql.sql.ends_with("ORDER BY fts.rank LIMIT ? OFFSET ?"));
        assert_eq!(sql.params.len(), 5);
    }

    #[test]
    fn orders_by_title_without_text() {
        let sql = build_search_sql(&query("  ", &[], &[]));
        assert_eq!(
            sql.sql,
            "SELECT e.* FROM entries e ORDER BY e.title ASC LIMIT ? OFFSET ?"
        );
        assert_eq!(sql.params.len(), 2);
    }

    #[test]
    fn counts_with_the_page_query_s_filter_and_no_window() {
        let sql = build_count_sql(&query("docker", &["commands"], &["infra"]));
        assert_eq!(
            sql.sql,
            "SELECT COUNT(*) FROM entries_fts(?) fts CROSS JOIN entries e ON e.rowid = fts.rowid WHERE e.type IN (?) AND EXISTS (SELECT 1 FROM entry_tags WHERE entry_id = e.id AND tag = ?)"
        );
        assert_eq!(sql.params.len(), 3);
        assert_eq!(
            build_count_sql(&query("", &[], &[])).sql,
            "SELECT COUNT(*) FROM entries e"
        );
    }

    #[test]
    fn lists_only_the_tags_of_the_chosen_types() {
        let dir = tempfile::tempdir().unwrap();
        let (sources, database) = (dir.path().join("s"), dir.path().join("kbx.db"));
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
}
