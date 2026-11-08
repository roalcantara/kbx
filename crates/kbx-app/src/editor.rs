//! The `editor` setting: a command template that opens a file at a line, e.g.
//! `zed {file}:{line}` or `code -g {file}:{line}:{column}`. The template is split into words first
//! (quotes group; no shell runs), then `{file}`, `{line}` and `{column}` are filled in, so a path with
//! spaces stays one argument. Without a known line, `{line}` and `{column}` go with the punctuation before
//! them (`:{line}`, `+{line}`).
use std::path::Path;

/// Why a template cannot be used.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EditorError {
    /// It names no program.
    #[error("it names no program")]
    Empty,
    /// A quote is never closed.
    #[error("a quote is never closed")]
    Unbalanced,
}

/// Checks `template` once, when config.toml is read.
///
/// # Errors
///
/// The template is empty or has an unclosed quote.
pub fn check(template: &str) -> Result<(), EditorError> {
    words(template).map(|_| ())
}

fn words(template: &str) -> Result<Vec<String>, EditorError> {
    let words = shlex::split(template).ok_or(EditorError::Unbalanced)?;
    if words.is_empty() {
        return Err(EditorError::Empty);
    }
    Ok(words)
}

/// The program and its arguments for `file` at `line`:`column`.
///
/// # Errors
///
/// As [`check`].
pub fn command(
    template: &str,
    file: &str,
    line: Option<usize>,
    column: Option<usize>,
) -> Result<Vec<String>, EditorError> {
    Ok(words(template)?
        .iter()
        .map(|word| fill(word, file, line, line.and(column)))
        .filter(|w| !w.is_empty())
        .collect())
}

/// Fills one word's placeholders in a single pass, so a path that contains `{line}` stays as it is. A
/// placeholder without a value goes with the punctuation before it: `:{line}`, `+{line}`, `@{line}`.
fn fill(word: &str, file: &str, line: Option<usize>, column: Option<usize>) -> String {
    let mut out = String::new();
    let mut rest = word;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        let (placeholder, value) = [
            ("{file}", Some(file.to_owned())),
            ("{line}", line.map(|n| n.to_string())),
            ("{column}", column.map(|n| n.to_string())),
        ]
        .into_iter()
        .find(|(p, _)| rest.starts_with(p))
        .unwrap_or(("{", Some("{".to_owned())));
        match value {
            Some(value) => out.push_str(&value),
            None => {
                if out.ends_with(|c: char| c.is_ascii_punctuation()) {
                    out.pop();
                }
            }
        }
        rest = &rest[placeholder.len()..];
    }
    out.push_str(rest);
    out
}

/// How a file was opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opened {
    /// In the configured editor.
    Editor,
    /// With the system's default app: no editor is set.
    DefaultApp,
    /// With the default app, because the editor could not be run: the program and the OS error.
    Fallback {
        /// The program kbx tried.
        program: String,
        /// Why it could not run.
        error: String,
    },
    /// Not at all: the file is gone.
    Missing,
}

/// Opens `file` in `editor` (the template) at `line`:`column`, else with the default app.
pub fn open(
    editor: Option<&str>,
    file: &Path,
    line: Option<usize>,
    column: Option<usize>,
    home: &Path,
    cx: &gpui_kit::App,
) -> Opened {
    if !file.exists() {
        return Opened::Missing;
    }
    let Some(words) = editor.and_then(|t| command(t, &file.to_string_lossy(), line, column).ok()) else {
        cx.open_with_system(file);
        return Opened::DefaultApp;
    };
    let program = crate::command_path::find_command(&words[0], std::env::var_os("PATH").as_deref(), home);
    match std::process::Command::new(&program).args(&words[1..]).spawn() {
        Ok(mut child) => {
            // Waited on aside, so an editor CLI that exits at once leaves no zombie behind.
            std::thread::spawn(move || child.wait());
            Opened::Editor
        }
        Err(error) => {
            cx.open_with_system(file);
            Opened::Fallback {
                program: program.display().to_string(),
                error: error.to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_the_file_line_and_column() {
        assert_eq!(
            command("zed {file}:{line}:{column}", "/s/a b.yml", Some(4), Some(11)).unwrap(),
            ["zed", "/s/a b.yml:4:11"],
            "a path with a space stays one word"
        );
        assert_eq!(
            command("code -g {file}:{line}", "/s/x.yml", Some(4), None).unwrap(),
            ["code", "-g", "/s/x.yml:4"]
        );
    }

    #[test]
    fn drops_the_line_and_column_it_does_not_know() {
        assert_eq!(
            command("zed {file}:{line}:{column}", "/s/x.yml", None, None).unwrap(),
            ["zed", "/s/x.yml"]
        );
        assert_eq!(
            command("zed {file}:{line}:{column}", "/s/x.yml", Some(4), None).unwrap(),
            ["zed", "/s/x.yml:4"]
        );
        assert_eq!(
            command("subl --wait {file}", "/s/x.yml", Some(4), Some(1)).unwrap(),
            ["subl", "--wait", "/s/x.yml"],
            "a template without line opens the file"
        );
        assert_eq!(
            command("vim +{line} {file}", "/s/x.yml", None, None).unwrap(),
            ["vim", "/s/x.yml"],
            "the prefix goes with the line"
        );
        assert_eq!(
            command("subl {file}@{line}", "/s/x.yml", None, None).unwrap(),
            ["subl", "/s/x.yml"]
        );
    }

    #[test]
    fn leaves_placeholders_inside_the_path_alone() {
        assert_eq!(
            command("zed {file}:{line}", "/s/notes-{line}.yml", Some(4), None).unwrap(),
            ["zed", "/s/notes-{line}.yml:4"]
        );
        assert_eq!(
            command("ed {other} {file}", "/x", None, None).unwrap(),
            ["ed", "{other}", "/x"],
            "an unknown brace stays"
        );
    }

    #[test]
    fn groups_quoted_words_and_rejects_a_broken_template() {
        assert_eq!(
            command("'/Applications/My Editor.app/bin/ed' {file}", "/x", None, None).unwrap(),
            ["/Applications/My Editor.app/bin/ed", "/x"]
        );
        assert_eq!(check(""), Err(EditorError::Empty));
        assert_eq!(check("   "), Err(EditorError::Empty));
        assert_eq!(check("zed '{file}"), Err(EditorError::Unbalanced));
        assert_eq!(check("zed {file}"), Ok(()));
    }
}
