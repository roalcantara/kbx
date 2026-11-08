//! Finds the Skills command without a shell: an app opened from Finder gets
//! `PATH=/usr/bin:/bin:/usr/sbin:/sbin`, so standard install folders are searched after `PATH`.
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::config::expand_home;

/// The command's name when `skills_command` is not set.
pub const DEFAULT_SKILLS_COMMAND: &str = "skills";

/// Searched after `PATH`, in order; `~/` is the home folder.
const INSTALL_DIRS: [&str; 3] = ["/opt/homebrew/bin", "/usr/local/bin", "~/.local/bin"];

/// The Skills command to run: a set path as is, `~/` expanded; a
/// set name where it is found, else the name itself (running it then fails and says why); with no
/// setting, `skills` where it is found, else `None` (not configured).
#[must_use]
pub fn skills_command(setting: Option<&str>, path_var: Option<&OsStr>, home: &Path) -> Option<PathBuf> {
    let dirs = search_dirs(path_var, home);
    match setting.map(str::trim).filter(|s| !s.is_empty()) {
        Some(path) if path.contains('/') => Some(expand_home(path, home)),
        Some(name) => Some(find_executable(name, &dirs).unwrap_or_else(|| PathBuf::from(name))),
        None => find_executable(DEFAULT_SKILLS_COMMAND, &dirs),
    }
}

/// The program `name` names: a path as is (`~/` expanded); a bare name where `PATH` or the install
/// folders have it, else the name itself (spawning it then fails and says why).
#[must_use]
pub fn find_command(name: &str, path_var: Option<&OsStr>, home: &Path) -> PathBuf {
    if name.contains('/') {
        return expand_home(name, home);
    }
    find_executable(name, &search_dirs(path_var, home)).unwrap_or_else(|| PathBuf::from(name))
}

/// `PATH`'s folders, then [`INSTALL_DIRS`].
fn search_dirs(path_var: Option<&OsStr>, home: &Path) -> Vec<PathBuf> {
    path_var
        .map(|path| std::env::split_paths(path).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .chain(INSTALL_DIRS.iter().map(|dir| expand_home(dir, home)))
        .collect()
}

/// The first `dir/name` that is an executable file.
fn find_executable(name: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt as _;
    dirs.iter().map(|dir| dir.join(name)).find(|path| {
        path.metadata()
            .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt as _;

    use super::*;

    fn executable(dir: &Path, name: &str) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[test]
    fn searches_path_then_the_install_folders() {
        let home = Path::new("/h");
        let dirs = search_dirs(Some(std::ffi::OsStr::new("/a:/b")), home);
        assert_eq!(
            dirs,
            ["/a", "/b", "/opt/homebrew/bin", "/usr/local/bin", "/h/.local/bin"].map(PathBuf::from)
        );
    }

    #[test]
    fn finds_the_first_executable_and_skips_plain_files() {
        let root = tempfile::tempdir().unwrap();
        let (first, second) = (root.path().join("first"), root.path().join("second"));
        fs::create_dir_all(&first).unwrap();
        fs::write(first.join("skills"), "not executable").unwrap();
        let found = executable(&second, "skills");
        assert_eq!(find_executable("skills", &[first, second]), Some(found));
        assert_eq!(find_executable("skills", &[root.path().join("none")]), None);
    }

    #[test]
    fn runs_a_set_path_and_keeps_a_set_name_it_cannot_find() {
        let home = Path::new("/h");
        assert_eq!(
            skills_command(Some("~/bin/skills"), None, home),
            Some(PathBuf::from("/h/bin/skills"))
        );
        assert_eq!(
            skills_command(Some("kb-test-no-such-command"), None, home),
            Some(PathBuf::from("kb-test-no-such-command"))
        );
    }

    #[test]
    fn finds_a_command_by_name_or_keeps_what_it_was_given() {
        let home = tempfile::tempdir().unwrap();
        let my_editor = executable(&home.path().join(".local/bin"), "my-editor");
        assert_eq!(find_command("my-editor", None, home.path()), my_editor);
        assert_eq!(
            find_command("~/bin/code", None, home.path()),
            home.path().join("bin/code")
        );
        assert_eq!(
            find_command("kb-test-no-such", None, home.path()),
            PathBuf::from("kb-test-no-such")
        );
    }
}
