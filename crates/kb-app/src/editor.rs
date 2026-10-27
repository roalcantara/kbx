//! The `editor` setting (spec 2026-10-27 D6): a command template that opens a file at a line, e.g.
//! `zed {file}:{line}` or `code -g {file}:{line}:{column}`. The template is split into words first
//! (quotes group; no shell runs), then `{file}`, `{line}` and `{column}` are filled in, so a path with
//! spaces stays one argument. Without a known line, `:{line}` and `:{column}` go with their colon.
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
    let expand_word = |word: String| {
        let mut word = word;
        if line.is_none() {
            word = word.replace(":{line}", "").replace("{line}", "");
        }
        if line.is_none() || column.is_none() {
            word = word.replace(":{column}", "").replace("{column}", "");
        }
        word.replace("{file}", file)
            .replace("{line}", &line.map(|n| n.to_string()).unwrap_or_default())
            .replace("{column}", &column.map(|n| n.to_string()).unwrap_or_default())
    };
    Ok(words(template)?
        .into_iter()
        .map(expand_word)
        .filter(|w| !w.is_empty())
        .collect())
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
        /// The program kb tried.
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
        Ok(_) => Opened::Editor,
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
