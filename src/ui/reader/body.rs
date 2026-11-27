//! A message body as styled lines: hard-wrapped prose rejoined, quotes set off with a bar,
//! headings bold, the signature faint.

use ratatui::prelude::*;

use crate::ui::theme;

/// A line at least this long is taken as hard-wrapped, so the next joins it.
const HARD_WRAP: usize = 60;

/// The deepest quote level drawn with its own bar.
const MAX_QUOTE_DEPTH: usize = 4;

/// Joins hard-wrapped prose so narrow panes wrap cleanly; quotes, lists, indented lines and short
/// lines stay as they are.
pub fn reflow(body: &str) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let mut out = String::with_capacity(body.len());
    for (index, current) in lines.iter().enumerate() {
        out.push_str(current.trim_end());
        let next_is_prose = lines.get(index + 1).is_some_and(|next| is_prose(next));
        let joins = current.len() >= HARD_WRAP && is_prose(current) && next_is_prose;
        out.push(if joins { ' ' } else { '\n' });
    }
    out
}

/// Whether a line is plain prose: not blank, indented, quoted, a list item or a heading.
fn is_prose(line: &str) -> bool {
    let first = line.chars().next();
    let marker =
        first.is_some_and(|c| matches!(c, '>' | '-' | '*' | '#' | '|' | ' ') || c.is_ascii_digit());
    !line.trim().is_empty() && !marker
}

/// The body's lines, styled; everything after a `-- ` line is the signature.
pub fn body_lines(body: &str) -> Vec<Line<'static>> {
    let mut part = Part::Text;
    let mut style_line = |text: &str| {
        if text == "-- " || text == "--" {
            part = Part::Signature;
        }
        styled(text, part)
    };
    body.lines().map(&mut style_line).collect()
}

/// Which part of the body a line is in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    /// The message itself.
    Text,

    /// The signature, from the `-- ` line on.
    Signature,
}

/// One line: faint in the signature, a bar per quote level, bold for a heading.
fn styled(text: &str, part: Part) -> Line<'static> {
    let t = theme();
    let depth = quote_depth(text);
    if part == Part::Signature {
        Line::styled(text.to_owned(), Style::new().fg(t.faint))
    } else if depth > 0 {
        quoted(text, depth)
    } else if text.starts_with("# ") || text.starts_with("## ") {
        Line::styled(
            text.to_owned(),
            Style::new().fg(t.accent).add_modifier(Modifier::BOLD),
        )
    } else {
        Line::styled(text.to_owned(), Style::new().fg(t.fg))
    }
}

/// How many `>` open the line, spaces between them allowed.
fn quote_depth(text: &str) -> usize {
    text.chars()
        .take_while(|&c| c == '>' || c == ' ')
        .filter(|&c| c == '>')
        .count()
}

/// A quoted line: a faint bar per level, then the text in dim italics.
fn quoted(text: &str, depth: usize) -> Line<'static> {
    let t = theme();
    let bars = Span::styled(
        "┃ ".repeat(depth.min(MAX_QUOTE_DEPTH)),
        Style::new().fg(t.faint),
    );
    let text = text.trim_start_matches(['>', ' ']).to_owned();
    Line::from(vec![
        bars,
        Span::styled(text, Style::new().fg(t.dim).add_modifier(Modifier::ITALIC)),
    ])
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use insta::assert_snapshot;
    use ratatui::widgets::{Paragraph, Widget};
    use rstest::rstest;

    use super::*;
    use crate::ui::test_support::render;

    #[test]
    fn reflow_joins_prose_but_keeps_quotes_and_short_lines() {
        let long = format!("{} {}", "a".repeat(30), "b".repeat(35));
        let body = format!(
            "Hi,\n\n{long}\nnext line\n> quoted line that is long enough to be hard wrapped ok\n> more\nAda"
        );

        let out = reflow(&body);

        let expected = format!(
            "Hi,\n\n{long} next line\n> quoted line that is long enough to be hard wrapped ok\n> more\nAda\n"
        );
        assert_eq!(out, expected);
    }

    #[rstest]
    #[case::plain("hello", 0)]
    #[case::one("> hello", 1)]
    #[case::nested_with_spaces("> > hello", 2)]
    #[case::not_at_the_start("a > b", 0)]
    fn quote_depth_counts_leading_markers(#[case] text: &str, #[case] depth: usize) {
        assert_eq!(quote_depth(text), depth);
    }

    #[test]
    fn quotes_get_bars_and_the_signature_starts_at_its_delimiter() {
        let lines = body_lines(indoc! {"
            # Notes
            Hi
            > asked
            > > before
            -- 
            Ada"});

        let screen = render(24, 6, |area, buf| Paragraph::new(lines).render(area, buf));

        assert_snapshot!(screen, @"
        # Notes
        Hi
        ┃ asked
        ┃ ┃ before
        --
        Ada
        ");
    }
}
