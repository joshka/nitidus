//! The key reference overlay.

use ratatui::prelude::*;
use ratatui::widgets::{Clear, Widget};

use super::theme;

pub const KEYS: &[(&str, &[(&str, &str)])] = &[
    (
        "Move around",
        &[
            ("j k", "next / previous"),
            ("g G", "first / last"),
            ("tab h l", "switch pane"),
            ("enter", "open"),
            ("esc", "back / clear"),
            ("n p", "next / prev (reader)"),
        ],
    ),
    (
        "Messages",
        &[
            ("s f", "toggle read / flag"),
            ("a d", "archive / delete"),
            ("m y", "move / copy to…"),
            ("x V", "mark / mark all"),
            ("T", "thread view"),
        ],
    ),
    (
        "Write",
        &[
            ("c", "compose"),
            ("r R", "reply / reply all"),
            ("F", "forward"),
            ("e", "edit draft"),
        ],
    ),
    (
        "Read",
        &[
            ("o S", "open / save attachments"),
            ("L B", "links / open in browser"),
            ("H", "message source"),
        ],
    ),
    (
        "Mailboxes",
        &[
            ("/", "search"),
            ("ctrl-r", "refresh"),
            ("n R D", "new / rename / delete"),
            ("A", "switch account"),
            ("q", "quit"),
        ],
    ),
];

pub struct Help;

impl Widget for Help {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        let rows: u16 = KEYS.iter().map(|(_, k)| k.len() as u16 + 2).sum::<u16>() + 2;
        let w = 46.min(area.width);
        let rect = Rect {
            x: area.x + (area.width - w) / 2,
            y: area.y + area.height.saturating_sub(rows) / 2,
            width: w,
            height: rows.min(area.height),
        };
        Clear.render(rect, buf);
        buf.set_style(rect, t.base(t.header));
        let inner = rect.inner(Margin::new(3, 1));
        let mut y = inner.y;
        for (section, keys) in KEYS {
            if y >= inner.bottom() {
                break;
            }
            Line::styled(
                *section,
                Style::new().fg(t.accent).add_modifier(Modifier::BOLD),
            )
            .render(
                Rect {
                    y,
                    height: 1,
                    ..inner
                },
                buf,
            );
            y += 1;
            for (k, what) in *keys {
                if y >= inner.bottom() {
                    break;
                }
                Line::from(vec![
                    Span::styled(
                        format!("{k:>8}"),
                        Style::new().fg(t.fg).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("   {what}"), Style::new().fg(t.dim)),
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
            y += 1;
        }
    }
}
