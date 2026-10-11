//! An entry's notes, description, links and provenance as Markdown.
//!
//! The output matches the TypeScript engine byte for byte. That engine builds a list of blocks
//! and renders them with `json2md`; this module keeps the same blocks and renders the subset of
//! `json2md` they use. No crate implements `json2md`, and the subset is small.

use std::cell::Cell;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use saphyr::YamlOwned;

use crate::entry::Entry;
use crate::yaml::{as_str, to_json};

/// One `json2md` block.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Block {
    /// Inserted as it is.
    Text(String),
    /// `![](source "title")`
    Image { title: &'static str, source: String },
    /// A fenced code block.
    Code { language: String, content: String },
    /// Every line prefixed with `> `.
    Blockquote(String),
    /// `### text`
    H3(&'static str),
    /// ` - item` per item.
    List(Vec<String>),
    /// A horizontal rule between notes, when one came from a `file://` reference.
    Rule,
}

/// Aliases the TS engine maps to a canonical fence language; any other language is kept.
fn code_language(language: &str) -> &str {
    match language {
        "sh" | "shell" | "bash" | "zsh" => "bash",
        "ts" | "typescript" => "typescript",
        "js" | "javascript" => "javascript",
        "yaml" | "yml" | "dockercompose" => "yaml",
        "json" | "jsonc" => "json",
        "py" | "python" => "python",
        other => other,
    }
}

/// The `YouTube` video id of a watch, embed or share URL.
///
/// ```
/// use kb_domain::markdown::youtube_video_id;
///
/// assert_eq!(youtube_video_id("https://youtu.be/aqz-KE-bpKQ?t=3"), Some("aqz-KE-bpKQ"));
/// assert_eq!(youtube_video_id("https://example.com"), None);
/// ```
#[must_use]
pub fn youtube_video_id(url: &str) -> Option<&str> {
    let start = ["youtube.com/watch?v=", "youtube.com/embed/", "youtu.be/"]
        .iter()
        .find_map(|marker| url.find(marker).map(|at| at + marker.len()))?;
    let rest = &url[start..];
    let end = rest
        .find(|c: char| matches!(c, '&' | '?' | '/') || c.is_whitespace())
        .unwrap_or(rest.len());
    (end > 0).then(|| &rest[..end])
}

/// Where `file://` notes are read from, and whether any was.
struct NoteFiles<'a> {
    read: &'a dyn Fn(&Path) -> io::Result<String>,
    source_file: &'a str,
    included: Cell<bool>,
}

impl NoteFiles<'_> {
    /// `text`, or the content of the file it names with `file://` (without its front matter, for Markdown). An
    /// unreadable file says so in the notes rather than failing the sync.
    fn resolve(&self, text: &str, markdown: bool) -> String {
        let reference = text.trim();
        if !reference.starts_with(FILE_SCHEME) {
            return text.to_owned();
        }
        self.included.set(true);
        match (self.read)(&note_path(reference, self.source_file)) {
            Ok(content) if markdown => strip_front_matter(&content).to_owned(),
            Ok(content) => content,
            Err(error) => format!("*Couldn’t read `{reference}`: {error}*"),
        }
    }
}

/// The scheme that makes a note a reference to a file whose content becomes the note (kb only; the Raycast extension
/// shows the reference as written).
const FILE_SCHEME: &str = "file://";

/// The file a `file://` note names: `~/` from the home folder, an absolute path as written, any other path beside the
/// source file.
fn note_path(reference: &str, source_file: &str) -> PathBuf {
    let path = reference.trim().trim_start_matches(FILE_SCHEME);
    if let Some(rest) = path.strip_prefix("~/") {
        return std::env::home_dir().map_or_else(|| PathBuf::from(path), |home| home.join(rest));
    }
    let path = Path::new(path);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    Path::new(source_file)
        .parent()
        .map_or_else(|| path.to_path_buf(), |dir| dir.join(path))
}

/// Markdown without a leading YAML front matter block (`---` … `---`), as a skill's `SKILL.md` has.
fn strip_front_matter(content: &str) -> &str {
    let Some(rest) = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
    else {
        return content;
    };
    rest.find("\n---")
        .map_or(content, |end| rest[end + 4..].trim_start_matches(['\r', '\n']))
}

/// One `format: content` notes block.
fn format_block(format: &str, content: &YamlOwned, files: &NoteFiles) -> Block {
    let format = format.trim().to_lowercase();
    let text = as_str(content).map_or_else(
        || serde_json::to_string_pretty(&to_json(content)).unwrap_or_default(),
        str::to_owned,
    );
    let text = files.resolve(&text, matches!(format.as_str(), "md" | "markdown"));
    match format.as_str() {
        "md" | "markdown" => Block::Text(text),
        "mermaid" => Block::Image {
            title: "Mermaid Diagram",
            source: format!(
                "https://mermaid.ink/img/{}?type=png&bgColor=fff",
                URL_SAFE_NO_PAD.encode(&text)
            ),
        },
        "svg" => Block::Image {
            title: "SVG Image",
            source: format!("data:image/svg+xml;base64,{}", STANDARD.encode(&text)),
        },
        "img" => Block::Image {
            title: "Image",
            source: text,
        },
        language => Block::Code {
            language: code_language(language).to_owned(),
            content: text,
        },
    }
}

fn map_blocks(node: &YamlOwned, files: &NoteFiles) -> Vec<Block> {
    match node {
        YamlOwned::Mapping(pairs) => pairs
            .iter()
            .map(|(k, v)| format_block(&crate::yaml::scalar_text(k).unwrap_or_default(), v, files))
            .collect(),
        _ => Vec::new(),
    }
}

/// The notes as blocks: a string, a list of strings and `format: content` maps, or one map. When any note is a
/// `file://` reference, the notes are set apart from one another by a rule.
fn notes_blocks(notes: &YamlOwned, files: &NoteFiles) -> Vec<Block> {
    let blocks: Vec<Block> = match notes {
        YamlOwned::Sequence(items) => items
            .iter()
            .flat_map(|item| match as_str(item) {
                Some(text) => vec![Block::Text(files.resolve(text, true))],
                None => map_blocks(item, files),
            })
            .collect(),
        YamlOwned::Mapping(_) => map_blocks(notes, files),
        _ => as_str(notes)
            .map(|text| vec![Block::Text(files.resolve(text, true))])
            .unwrap_or_default(),
    };
    if !files.included.get() {
        return blocks;
    }
    let mut ruled = Vec::with_capacity(blocks.len() * 2);
    for (index, block) in blocks.into_iter().enumerate() {
        if index > 0 {
            ruled.push(Block::Rule);
        }
        ruled.push(block);
    }
    ruled
}

/// Whether JavaScript would treat the notes as truthy (`if (entry.notes)`).
fn has_notes(notes: Option<&YamlOwned>) -> bool {
    notes.is_some_and(|n| as_str(n).is_none_or(|s| !s.is_empty()))
}

/// Compiles an entry's notes, description, links and source into Markdown, reading `file://` notes from disk.
#[must_use]
pub fn build_entry_markdown(entry: &Entry) -> String {
    build_entry_markdown_with(entry, &|path| fs::read_to_string(path))
}

/// [`build_entry_markdown`], reading `file://` notes with `read`.
fn build_entry_markdown_with(entry: &Entry, read: &dyn Fn(&Path) -> io::Result<String>) -> String {
    let files = NoteFiles {
        read,
        source_file: &entry.source_file,
        included: Cell::new(false),
    };
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
    if has_notes(entry.notes.as_ref()) {
        blocks.extend(
            entry
                .notes
                .as_ref()
                .map(|notes| notes_blocks(notes, &files))
                .unwrap_or_default(),
        );
    }
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

    blocks.iter().map(render).collect::<Vec<_>>().join("\n")
}

/// `json2md` of one block. Every typed block ends with a newline; a text block does not.
fn render(block: &Block) -> String {
    match block {
        Block::Text(text) => text.clone(),
        Block::Image { title, source } => format!("![]({source} \"{title}\")\n"),
        Block::Code { language, content } => format!("```{language}\n{content}\n```\n"),
        Block::Blockquote(text) => format!("{}\n", prefix_lines(text, "> ")),
        Block::H3(text) => format!("### {text}\n"),
        // A blank line first: a `---` right under text would make that text a heading.
        Block::Rule => "\n---\n".to_owned(),
        Block::List(items) => {
            let mut body = String::new();
            for item in items {
                body.push_str("\n - ");
                body.push_str(&text_format(&indent_rest(item, 4)));
            }
            body.push('\n');
            body
        }
    }
}

/// `indento(text, 1, prefix)`: `text.replace(/^/gm, prefix)`, a prefix at the start and after
/// every JavaScript line terminator.
fn prefix_lines(text: &str, prefix: &str) -> String {
    let mut out = String::from(prefix);
    for c in text.chars() {
        out.push(c);
        if matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
            out.push_str(prefix);
        }
    }
    out
}

/// `json2md`'s `indent(content, spaces, true)`: every line but the first indented.
fn indent_rest(text: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    let mut lines = text.split('\n');
    let mut out = lines.next().unwrap_or_default().to_owned();
    for line in lines {
        out.push('\n');
        out.push_str(&pad);
        out.push_str(line);
    }
    out
}

/// `json2md`'s `parseTextFormat`: HTML emphasis tags as Markdown markers.
fn text_format(text: &str) -> String {
    let mut out = text.to_owned();
    for (tag, marker) in [
        ("strong", "**"),
        ("bold", "**"),
        ("em", "*"),
        ("italic", "*"),
        ("u", "_"),
        ("strike", "~~"),
    ] {
        out = replace_tag(&out, tag, marker);
    }
    out
}

/// Replaces `<tag>` and `</tag>`, in any case, with `marker`.
fn replace_tag(text: &str, tag: &str, marker: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        let rest = &lower[i..];
        let close = rest.starts_with("</");
        let name_at = if close { 2 } else { 1 };
        if rest.starts_with('<') && rest[name_at..].starts_with(tag) && rest[name_at + tag.len()..].starts_with('>') {
            out.push_str(marker);
            i += name_at + tag.len() + 1;
        } else {
            let c = text[i..].chars().next().unwrap_or_default();
            out.push(c);
            i += c.len_utf8();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_youtube_ids() {
        assert_eq!(
            youtube_video_id("https://www.youtube.com/watch?v=aqz-KE-bpKQ&t=1"),
            Some("aqz-KE-bpKQ")
        );
        assert_eq!(youtube_video_id("https://youtu.be/wNw9fKErhdg"), Some("wNw9fKErhdg"));
        assert_eq!(youtube_video_id("https://example.com"), None);
    }

    #[test]
    fn prefixes_after_every_javascript_line_terminator() {
        assert_eq!(prefix_lines("a\nb\r\nc", "> "), "> a\n> b\r> \n> c");
    }

    #[test]
    fn rewrites_emphasis_tags_in_list_items() {
        assert_eq!(text_format("<STRONG>a</strong> <em>b</em>"), "**a** *b*");
    }

    #[test]
    fn indents_every_line_but_the_first() {
        assert_eq!(indent_rest("a\nb\nc", 4), "a\n    b\n    c");
    }

    fn notes_entry(notes: &str) -> Entry {
        let raw = crate::yaml::load(&format!("desc: D\nnotes:\n{notes}"))
            .unwrap()
            .unwrap();
        crate::entry::normalize_entry("/kb/skills.yml", "skills.yml", "commands", "skills add x", &raw)
    }

    fn reader(files: &[(&str, &str)]) -> impl Fn(&Path) -> io::Result<String> {
        let files: Vec<(String, String)> = files.iter().map(|(p, c)| ((*p).to_owned(), (*c).to_owned())).collect();
        move |path: &Path| {
            files
                .iter()
                .find(|(p, _)| path.ends_with(p))
                .map(|(_, c)| c.clone())
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))
        }
    }

    #[test]
    fn includes_a_note_file_between_rules_without_its_front_matter() {
        let entry = notes_entry("  - Run it first.\n  - md: file://~/skills/x/SKILL.md\n");
        let read = reader(&[("skills/x/SKILL.md", "---\nname: x\n---\n# X\n\nBody.\n")]);
        let markdown = build_entry_markdown_with(&entry, &read);
        assert!(markdown.contains("Run it first.\n\n---\n\n# X\n\nBody."), "{markdown}");
        assert!(!markdown.contains("name: x"), "front matter dropped: {markdown}");
        assert!(!markdown.contains("file://"), "{markdown}");
    }

    #[test]
    fn resolves_note_files_from_home_or_beside_the_source_file() {
        assert_eq!(
            note_path("file://notes/a.md", "/kb/skills.yml"),
            Path::new("/kb/notes/a.md")
        );
        assert_eq!(note_path("file:///abs/a.md", "/kb/skills.yml"), Path::new("/abs/a.md"));
        assert!(note_path("file://~/a.md", "/kb/skills.yml").ends_with("a.md"));
        assert!(!note_path("file://~/a.md", "/kb/skills.yml").starts_with("~"));
    }

    #[test]
    fn says_so_when_a_note_file_cannot_be_read() {
        let entry = notes_entry("  - file://missing.md\n");
        let markdown = build_entry_markdown_with(&entry, &reader(&[]));
        assert!(
            markdown.contains("*Couldn’t read `file://missing.md`: no such file*"),
            "{markdown}"
        );
    }

    #[test]
    fn leaves_notes_without_files_as_they_were() {
        let entry = notes_entry("  - one\n  - two\n");
        assert_eq!(
            build_entry_markdown_with(&entry, &reader(&[])),
            build_entry_markdown(&entry)
        );
        assert!(!build_entry_markdown(&entry).contains("---"));
    }
}
