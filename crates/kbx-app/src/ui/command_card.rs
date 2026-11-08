//! D13's command card (04-command-cards): a shell command in a monospace box, broken at its
//! operators by fmt-rs, the rest wrapped at spaces with a hanging indent, coloured by GPUI Kit's
//! bash highlighter, with a copy button. It takes plain GPUI values only, no kbx types, so it can
//! become a crate of its own.

use std::ops::Range;
use std::rc::Rc;

use gpui_kit::base::input::{HighlightStyleResolver, Rope};
use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::highlighter::SyntaxHighlighter;
use gpui_kit::{
    App, ElementId, FontFeatures, FontWeight, HighlightStyle, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, Rems, RenderOnce, SharedString, Styled as _, StyledText, Window, div,
};

/// A wrapped command: `text` with the line breaks inserted, and `breaks`, the byte offsets in the
/// unwrapped text where a space became a break, with the text that replaced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrapped {
    /// The text to draw.
    pub text: String,
    /// (offset of the replaced space in the input, its replacement).
    pub breaks: Vec<(usize, String)>,
}

/// Wraps `formatted` (fmt-rs's output) to `columns`: a word moves to the next line, indented four
/// more than its line, when it would pass the column; a token is never split, and `&&`, `||`, `|`,
/// `;` and `\` stay with the word before them.
#[must_use]
pub fn wrap(formatted: &str, columns: usize) -> Wrapped {
    let mut breaks: Vec<(usize, String)> = Vec::new();
    let mut line_start = 0;
    for line in formatted.split('\n') {
        let indent = line.len() - line.trim_start().len();
        let hang = " ".repeat(indent + 4);
        let mut words: Vec<(usize, usize)> = Vec::new();
        let mut offset = line_start;
        for (ix, part) in line.split(' ').enumerate() {
            if ix > 0 {
                offset += 1;
            }
            if !part.is_empty() {
                let glued = matches!(part, "&&" | "||" | "|" | ";" | "\\");
                match words.last_mut() {
                    Some(last) if glued => last.1 = offset + part.len() - last.0,
                    _ => words.push((offset, part.len())),
                }
            }
            offset += part.len();
        }
        let mut column = 0;
        for (ix, (start, len)) in words.iter().enumerate() {
            let width = formatted[*start..start + len].chars().count();
            if ix == 0 {
                column = (start - line_start) + width;
            } else if column + 1 + width > columns && column > indent {
                breaks.push((start - 1, format!("\n{hang}")));
                column = hang.len() + width;
            } else {
                column += 1 + width;
            }
        }
        line_start += line.len() + 1;
    }
    let mut text = String::with_capacity(formatted.len() + breaks.len() * 8);
    let mut last = 0;
    for (at, replacement) in &breaks {
        text.push_str(&formatted[last..*at]);
        text.push_str(replacement);
        last = at + 1;
    }
    text.push_str(&formatted[last..]);
    Wrapped { text, breaks }
}

/// Moves `offset`, a byte offset in the unwrapped text, to the wrapped text.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "shift is kept for callers that colour the unwrapped text")
)]
#[must_use]
pub fn shift(offset: usize, breaks: &[(usize, String)]) -> usize {
    offset
        + breaks
            .iter()
            .filter(|(at, _)| *at < offset)
            .map(|(_, replacement)| replacement.len() - 1)
            .sum::<usize>()
}

/// fmt-rs's formatting of `command` at `columns`, then [`wrap`]: what the card draws.
#[must_use]
pub fn layout(command: &str, columns: usize) -> Wrapped {
    let formatted = fmt_rs::format::format_command(command, isize::try_from(columns).unwrap_or(isize::MAX));
    wrap(formatted.trim_end(), columns)
}

/// The colour of each bash highlight the card shows; plain words keep the text colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyntaxColors {
    /// Commands (`function`), drawn medium.
    pub function: Hsla,
    /// Flags and numbers (`constant`, `number`).
    pub constant: Hsla,
    /// Strings.
    pub string: Hsla,
    /// Variables and substitutions (`property`, `embedded`, `variable`).
    pub variable: Hsla,
    /// Keywords.
    pub keyword: Hsla,
    /// Operators, punctuation and comments.
    pub punctuation: Hsla,
}

impl HighlightStyleResolver for SyntaxColors {
    fn style(&self, name: &str) -> Option<HighlightStyle> {
        let (color, medium) = match name.split('.').next().unwrap_or(name) {
            "function" => (self.function, true),
            "constant" | "number" => (self.constant, false),
            "string" => (self.string, false),
            "property" | "embedded" | "variable" => (self.variable, false),
            "keyword" => (self.keyword, false),
            "operator" | "punctuation" | "comment" => (self.punctuation, false),
            _ => return None,
        };
        Some(HighlightStyle {
            color: Some(color),
            font_weight: medium.then_some(FontWeight::MEDIUM),
            ..HighlightStyle::default()
        })
    }
}

/// How a card looks: plain GPUI values, so the card knows nothing of kbx.
#[derive(Debug, Clone)]
pub struct CardStyle {
    /// The box.
    pub background: Hsla,
    /// Its 1 pt border.
    pub border: Hsla,
    /// Plain words.
    pub text: Hsla,
    /// Highlight colours.
    pub syntax: SyntaxColors,
    /// The monospace family.
    pub family: SharedString,
    /// Ligatures off.
    pub features: FontFeatures,
    /// Text size and line height.
    pub size: Rems,
    /// Line height.
    pub line_height: Rems,
    /// Corner radius.
    pub radius: Rems,
    /// Padding: left, top and bottom, right (room for the copy button).
    pub pad_x: Rems,
    /// Top and bottom padding.
    pub pad_y: Rems,
    /// Right padding.
    pub pad_right: Rems,
    /// The copy button's inset from the top-right corner.
    pub copy_inset: Rems,
    /// The copy button's tooltip.
    pub copy_label: SharedString,
}

/// A callback after the card copied its command.
pub type OnCopied = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// A command card.
#[derive(IntoElement)]
pub struct CommandCard {
    id: ElementId,
    command: SharedString,
    columns: usize,
    style: CardStyle,
    on_copied: Option<OnCopied>,
}
impl CommandCard {
    /// A card `id` showing `command` wrapped to `columns`.
    #[must_use]
    pub fn new(id: impl Into<ElementId>, command: impl Into<SharedString>, columns: usize, style: CardStyle) -> Self {
        Self {
            id: id.into(),
            command: command.into(),
            columns,
            style,
            on_copied: None,
        }
    }

    /// Runs `handler` after the copy button copies the command (the copy notice, D17).
    #[must_use]
    pub fn on_copied(mut self, handler: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_copied = Some(std::rc::Rc::new(handler));
        self
    }
}

impl RenderOnce for CommandCard {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let s = self.style;
        let wrapped = layout(&self.command, self.columns);
        let formatted_len = wrapped.text.len();
        let rope = Rope::from_str(&wrapped.text);
        let mut highlighter = SyntaxHighlighter::new("bash");
        highlighter.update(None, &rope, None);
        let styles: Vec<(Range<usize>, HighlightStyle)> = highlighter.styles(&(0..formatted_len), &s.syntax);
        let mut copy = Clipboard::new(ElementId::Name(format!("{:?}-copy", self.id).into()))
            .value(self.command.clone())
            .tooltip(s.copy_label.clone());
        if let Some(handler) = self.on_copied {
            copy = copy.on_copied(move |value, window, cx| handler(value, window, cx));
        }
        div()
            .id(self.id)
            .relative()
            .w_full()
            .pl(s.pad_x)
            .py(s.pad_y)
            .pr(s.pad_right)
            .rounded(s.radius)
            .bg(s.background)
            .border_1()
            .border_color(s.border)
            .font_family(s.family)
            .font_features(s.features)
            .text_size(s.size)
            .line_height(s.line_height)
            .text_color(s.text)
            .child(StyledText::new(wrapped.text).with_highlights(styles))
            .child(div().absolute().top(s.copy_inset).right(s.copy_inset).child(copy))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    const LONG: &str = "kubectl rollout restart deployment/api --namespace production --timeout=120s --wait";

    #[test]
    fn wraps_at_spaces_with_a_hanging_indent_and_never_splits_a_token() {
        let wrapped = wrap(LONG, 40);
        let lines: Vec<&str> = wrapped.text.lines().collect();
        assert!(lines.len() > 1, "{lines:?}");
        for line in &lines[1..] {
            assert!(line.starts_with("    "), "continuations hang by four: {line:?}");
        }
        for line in &lines {
            let trimmed = line.trim_start();
            assert!(
                trimmed.chars().count() <= 40 || !trimmed.contains(' '),
                "{line:?} passes 40"
            );
        }
        let tokens: Vec<&str> = LONG.split(' ').collect();
        let rebuilt: Vec<&str> = wrapped.text.split_whitespace().collect();
        assert_eq!(rebuilt, tokens, "every token intact, in order");
    }

    #[test]
    fn keeps_operators_with_the_word_before_them() {
        let wrapped = wrap("cd /a/very/long/path/to/a/project && make build-everything", 30);
        for line in wrapped.text.lines() {
            assert!(!line.trim_start().starts_with("&&"), "{:?}", wrapped.text);
        }
    }

    #[test]
    fn leaves_a_short_command_alone() {
        assert_eq!(wrap("git status", 40).text, "git status");
    }

    #[test]
    fn moves_offsets_past_inserted_breaks() {
        let breaks = vec![(5, "\n    ".to_owned())];
        assert_eq!(shift(3, &breaks), 3);
        assert_eq!(shift(6, &breaks), 6 + 4);
    }

    #[test]
    fn breaks_a_chain_at_its_operators_before_wrapping() {
        let text = layout("make build && make test && make deploy-to-production-now", 30).text;
        assert!(text.lines().count() >= 2, "{text}");
        assert!(text.lines().all(|l| !l.trim_start().starts_with("&&")), "{text}");
    }
}
