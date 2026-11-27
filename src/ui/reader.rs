//! The message reader: a tinted header card above the body.

use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Widget, Wrap};

use super::theme;
use crate::mail::{Address, Message};

#[derive(Default)]
pub struct Reader {
    pub message: Option<Message>,
    pub loading: bool,
    pub scroll: u16,
    pub focused: bool,
    /// The raw source, shown instead of the message when `show_raw`.
    pub raw: Option<String>,
    pub show_raw: bool,
}

impl Reader {
    /// Every http(s) and mailto link in the message, in order, deduplicated.
    pub fn links(&self) -> Vec<String> {
        let Some(m) = &self.message else {
            return vec![];
        };
        let text = format!(
            "{}\n{}",
            m.text.as_deref().unwrap_or(""),
            m.html.as_deref().unwrap_or("")
        );
        let mut out: Vec<String> = vec![];
        for start in ["https://", "http://", "mailto:"] {
            let mut rest = text.as_str();
            while let Some(i) = rest.find(start) {
                let tail = &rest[i..];
                let end = tail
                    .find(|c: char| {
                        c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | ')' | ']')
                    })
                    .unwrap_or(tail.len());
                let link = tail[..end].trim_end_matches(['.', ',', ';']).to_string();
                if !out.contains(&link) {
                    out.push(link);
                }
                rest = &tail[end..];
            }
        }
        out
    }

    pub fn clear(&mut self) {
        self.message = None;
        self.raw = None;
        self.show_raw = false;
        self.scroll = 0;
    }

    pub fn scroll_by(&mut self, delta: i32) {
        self.scroll = (i32::from(self.scroll) + delta).max(0) as u16;
    }
}

fn pretty_date(raw: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(raw)
        .or_else(|_| chrono::DateTime::parse_from_rfc2822(raw))
        .map_or_else(
            |_| raw.to_string(),
            |d| {
                d.with_timezone(&chrono::Local)
                    .format("%a, %e %b %Y  %H:%M")
                    .to_string()
            },
        )
}

fn addresses(list: &[Address]) -> String {
    list.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Joins hard-wrapped prose lines so narrow panes wrap cleanly. A line is
/// joined to the next when it looks hard-wrapped (long, plain prose) and
/// the next is plain prose too; quotes, lists and signatures stay as is.
pub fn reflow(body: &str) -> String {
    let plain = |l: &str| {
        let t = l.trim_start();
        !t.is_empty()
            && !t.starts_with(['>', '-', '*', '#', '|'])
            && !l.starts_with(' ')
            && !t.chars().next().is_some_and(|c| c.is_ascii_digit())
    };
    let mut out = String::new();
    let lines: Vec<&str> = body.lines().collect();
    for (i, l) in lines.iter().enumerate() {
        out.push_str(l.trim_end());
        let join = l.len() >= 60 && plain(l) && lines.get(i + 1).is_some_and(|n| plain(n));
        out.push(if join { ' ' } else { '\n' });
    }
    out
}

fn body_lines(body: &str) -> Vec<Line<'static>> {
    let t = theme();
    let mut in_sig = false;
    body.lines()
        .map(|l| {
            if l == "-- " || l == "--" {
                in_sig = true;
            }
            let style = if in_sig {
                Style::new().fg(t.faint)
            } else if l.trim_start().starts_with('>') {
                Style::new().fg(t.dim).add_modifier(Modifier::ITALIC)
            } else if l.starts_with("## ") || l.starts_with("# ") {
                Style::new().fg(t.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(t.fg)
            };
            if l.trim_start().starts_with('>') {
                let rest = l
                    .trim_start()
                    .trim_start_matches('>')
                    .trim_start()
                    .to_string();
                let depth = l.chars().filter(|c| *c == '>').count().min(4);
                Line::from(vec![
                    Span::styled("┃ ".repeat(depth), Style::new().fg(t.faint)),
                    Span::styled(rest, style),
                ])
            } else {
                Line::styled(l.to_string(), style)
            }
        })
        .collect()
}

impl Widget for &mut Reader {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        buf.set_style(area, t.base(t.reader));
        if self.show_raw {
            let inner = area.inner(Margin::new(3, 1));
            let text = self.raw.as_deref().unwrap_or("Loading source…");
            let title = Rect { height: 1, ..inner };
            Line::styled(
                "Message source",
                Style::new().fg(t.accent).add_modifier(Modifier::BOLD),
            )
            .render(title, buf);
            let body = Rect {
                y: inner.y + 2,
                height: inner.height.saturating_sub(2),
                ..inner
            };
            let lines: Vec<Line> = text
                .lines()
                .map(|l| match l.split_once(':') {
                    Some((k, v)) if !k.contains(' ') && !k.is_empty() => Line::from(vec![
                        Span::styled(k.to_string(), Style::new().fg(t.accent)),
                        Span::styled(format!(":{v}"), Style::new().fg(t.dim)),
                    ]),
                    _ => Line::styled(l.to_string(), Style::new().fg(t.dim)),
                })
                .collect();
            let max = (lines.len() as u16).saturating_sub(body.height / 2);
            self.scroll = self.scroll.min(max);
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .scroll((self.scroll, 0))
                .render(body, buf);
            return;
        }
        let Some(m) = &self.message else {
            let msg = if self.loading {
                "Loading…"
            } else {
                "Select a message"
            };
            Line::styled(msg, Style::new().fg(t.faint))
                .centered()
                .render(
                    Rect {
                        y: area.y + area.height / 3,
                        height: 1,
                        ..area
                    },
                    buf,
                );
            return;
        };

        let h = &m.headers;
        let mut meta: Vec<(&str, String)> =
            vec![("From", addresses(&h.from)), ("To", addresses(&h.to))];
        if !h.cc.is_empty() {
            meta.push(("Cc", addresses(&h.cc)));
        }
        meta.push((
            "Date",
            h.date.as_deref().map(pretty_date).unwrap_or_default(),
        ));
        let attachments: Vec<&str> = m.attachments().collect();

        let card_h = 3 + meta.len() as u16 + u16::from(!attachments.is_empty()) + 1;
        let card = Rect {
            height: card_h.min(area.height),
            ..area
        };
        buf.set_style(card, Style::new().bg(t.header));
        let inner = card.inner(Margin::new(3, 1));
        let subject = h.subject.clone().unwrap_or_else(|| "(no subject)".into());
        Line::styled(subject, Style::new().fg(t.fg).add_modifier(Modifier::BOLD))
            .render(Rect { height: 1, ..inner }, buf);
        let mut y = inner.y + 2;
        for (k, v) in meta {
            if y >= inner.bottom() {
                break;
            }
            Line::from(vec![
                Span::styled(format!("{k:>5}  "), Style::new().fg(t.faint)),
                Span::styled(
                    v,
                    Style::new().fg(if k == "From" { t.accent } else { t.dim }),
                ),
            ])
            .render(
                Rect {
                    y,
                    height: 1,
                    ..inner
                },
                buf,
            );
            y += 1;
        }
        if !attachments.is_empty() && y < inner.bottom() {
            let mut spans = vec![Span::styled("       ", Style::new())];
            for a in attachments {
                spans.push(Span::styled(
                    format!(" ⎘ {a} "),
                    Style::new().fg(t.fg).bg(t.chip),
                ));
                spans.push(Span::raw(" "));
            }
            Line::from(spans).render(
                Rect {
                    y,
                    height: 1,
                    ..inner
                },
                buf,
            );
        }

        let body_area = Rect {
            y: card.bottom() + 1,
            height: area.height.saturating_sub(card_h + 1),
            ..area
        }
        .inner(Margin::new(3, 0));
        // A readable measure: mail is mostly hard-wrapped near 72-78 columns.
        let body_area = Rect {
            width: body_area.width.min(84),
            ..body_area
        };
        let text = m.body(body_area.width as usize);
        let lines = body_lines(&reflow(&text));
        let max = (lines.len() as u16).saturating_sub(body_area.height / 2);
        self.scroll = self.scroll.min(max);
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0))
            .render(body_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::reflow;

    #[test]
    fn reflow_joins_prose_but_keeps_quotes_and_short_lines() {
        let long = "a".repeat(30) + " " + &"b".repeat(35);
        let body = format!(
            "Hi,\n\n{long}\nnext line\n> quoted line that is long enough to be hard wrapped ok\n> more\nAda"
        );
        let out = reflow(&body);
        assert!(out.contains(&format!("{long} next line")), "{out}");
        assert!(out.contains("\n> more\n"), "{out}");
        assert!(out.starts_with("Hi,\n\n"));
    }
}
