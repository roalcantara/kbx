//! The showcase corpus (tools/design/showcase) indexes to what the approved pages show, so the
//! app's screenshots and the pages show the same entries.

#![expect(clippy::unwrap_used, reason = "a fixture that cannot be built is a failed test")]

use std::path::Path;
use std::sync::atomic::AtomicBool;

use kbx_domain::search::{FilterState, SearchQuery, search};
use kbx_domain::sync::{open_readonly, sync_knowledge_base};

fn showcase() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/design/showcase")
        .canonicalize()
        .unwrap()
}

#[test]
fn indexes_the_entries_the_pages_show() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("kbx.db");
    let summary = sync_knowledge_base(&showcase(), &database, &AtomicBool::new(false), |_| {}).unwrap();
    assert_eq!(summary.files_scanned, 9);
    assert_eq!(summary.files_failed, 1, "errors/broken.yml feeds the refresh report");
    assert_eq!(summary.total_indexed, 10);
    let query = SearchQuery {
        text: String::new(),
        filter: FilterState::default(),
        limit: 50,
        offset: 0,
    };
    let titles: Vec<String> = search(&open_readonly(&database).unwrap(), &query)
        .unwrap()
        .into_iter()
        .map(|row| row.title)
        .collect();
    for title in [
        "/k8s-rollout-restart",
        "/docker-prune-all",
        "/git-undo-commit",
        "Kubernetes Ingress & TLS Spec Cheatsheet",
        "Rust Memory Safety & Lifetimes Reference",
        "Tokio Tutorial — Async in Depth",
        "Async Rust in Depth",
        "AWS EKS VPC CNI Subnet Exhaustion Playbook",
        "Ship the new design system",
        "Write the findings",
    ] {
        assert!(titles.iter().any(|t| t == title), "missing {title}: {titles:?}");
    }
}
