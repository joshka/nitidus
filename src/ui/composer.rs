//! The review screen a draft lands on after the editor: what will be sent,
//! and the keys to send, save, edit or attach.

use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Widget, Wrap};

use super::theme;
use crate::mail::{ComposeKind, Draft};

pub struct Composer<'a> {
    pub draft: &'a Draft,
    pub scroll: u16,
}

impl Widget for Composer<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        let d = self.draft;
        buf.set_style(area, t.base(t.reader));
        let kind = match &d.kind {
            ComposeKind::New => "New message",
            ComposeKind::Reply { all: false, .. } => "Reply",
            ComposeKind::Reply { all: true, .. } => "Reply all",
            ComposeKind::Forward { .. } => "Forward",
            ComposeKind::Draft { .. } => "Draft",
        };
        let mut rows: Vec<(&str, String)> = vec![];
        if let Some(from) = &d.from {
            rows.push(("From", from.clone()));
        }
        rows.push(("To", d.to.join(", ")));
        if !d.cc.is_empty() {
            rows.push(("Cc", d.cc.join(", ")));
        }
        if !d.bcc.is_empty() {
            rows.push(("Bcc", d.bcc.join(", ")));
        }
        let files: Vec<String> = d
            .carried
            .iter()
            .cloned()
            .chain(d.attach.iter().map(|p| {
                p.file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_default()
            }))
            .collect();

        let card_h = 4 + rows.len() as u16 + u16::from(!files.is_empty()) + 1;
        let card = Rect {
            height: card_h.min(area.height),
            ..area
        };
        buf.set_style(card, Style::new().bg(t.header));
        let inner = card.inner(Margin::new(3, 1));
        Line::from(vec![
            Span::styled(
                format!(" {kind} "),
                Style::new()
                    .fg(t.bar)
                    .bg(t.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                if d.subject.is_empty() {
                    "(no subject)".to_string()
                } else {
                    d.subject.clone()
                },
                Style::new().fg(t.fg).add_modifier(Modifier::BOLD),
            ),
        ])
        .render(Rect { height: 1, ..inner }, buf);
        let mut y = inner.y + 2;
        for (k, v) in rows {
            let missing = v.is_empty();
            Line::from(vec![
                Span::styled(format!("{k:>5}  "), Style::new().fg(t.faint)),
                Span::styled(
                    if missing { "(none)".into() } else { v },
                    Style::new().fg(if missing {
                        t.error
                    } else if k == "To" {
                        t.accent
                    } else {
                        t.dim
                    }),
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
        if !files.is_empty() {
            let mut spans = vec![Span::styled("       ", Style::new())];
            for f in files {
                spans.push(Span::styled(
                    format!(" ⎘ {f} "),
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

        let body = Rect {
            y: card.bottom() + 1,
            height: area.height.saturating_sub(card_h + 1),
            ..area
        }
        .inner(Margin::new(3, 0));
        let body = Rect {
            width: body.width.min(84),
            ..body
        };
        let lines: Vec<Line> = d
            .body
            .lines()
            .map(|l| {
                let quoted = l.trim_start().starts_with('>');
                Line::styled(
                    l.to_string(),
                    Style::new().fg(if quoted { t.dim } else { t.fg }),
                )
            })
            .collect();
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0))
            .render(body, buf);
    }
}
