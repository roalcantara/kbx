//! `kbx --sync`: the built binary rebuilds the index under `HOME` without opening a window.

#![expect(clippy::unwrap_used, reason = "a fixture that cannot be built is a failed test")]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn sync_under(home: &Path, sources: &Path, extra: &[&str]) -> Output {
    use std::os::unix::fs::PermissionsExt as _;
    fs::create_dir_all(home.join(".config/kbx")).unwrap();
    let skills = home.join("skills");
    fs::write(&skills, "#!/bin/sh\necho '[]'\n").unwrap();
    fs::set_permissions(&skills, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(
        home.join(".config/kbx/config.toml"),
        format!(
            "sources_path = \"{}\"\nskills_command = \"{}\"\n",
            sources.display(),
            skills.display()
        ),
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_kbx"))
        .arg("--sync")
        .args(extra)
        .env("HOME", home)
        .output()
        .unwrap()
}

#[test]
fn syncs_the_configured_sources_into_the_data_directory() {
    let home = tempfile::tempdir().unwrap();
    let sample = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../kbx-domain/tests/fixtures/sample")
        .canonicalize()
        .unwrap();

    let output = sync_under(home.path(), &sample, &[]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(stderr.contains("kbx: yaml: indexed "), "{stderr}");
    assert!(home.path().join("Library/Application Support/kbx/kbx.db").is_file());
}

#[test]
fn fails_when_the_sources_directory_is_missing() {
    let home = tempfile::tempdir().unwrap();

    let output = sync_under(home.path(), &home.path().join("nowhere"), &[]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("cannot read the sources directory"), "{stderr}");
}

#[test]
fn refreshes_one_source_and_rejects_an_unknown_one() {
    let home = tempfile::tempdir().unwrap();

    let output = sync_under(home.path(), &home.path().join("nowhere"), &["--source", "skills"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(stderr.contains("kbx: skills: indexed 0 entries"), "{stderr}");
    assert!(!stderr.contains("kbx: yaml:"), "{stderr}");

    let output = sync_under(home.path(), &home.path().join("nowhere"), &["--source", "both"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage: kbx"));
}
