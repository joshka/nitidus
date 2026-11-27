//! The mailbox list, a tinted column on the left.

use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::theme;
use crate::mail::Mailbox;

#[derive(Default)]
pub struct Sidebar {
    pub mailboxes: Vec<Mailbox>,
    pub selected: usize,
    pub focused: bool,
    pub account: String,
    /// Unread count for the open mailbox, the only one we know cheaply.
    pub unread: Option<(usize, usize)>,
}

impl Sidebar {
    pub fn set_mailboxes(&mut self, mut mailboxes: Vec<Mailbox>) {
        mailboxes.sort_by_key(|m| (rank(m), m.label().to_lowercase()));
        self.mailboxes = mailboxes;
        self.selected = self.selected.min(self.mailboxes.len().saturating_sub(1));
    }
    pub fn current(&self) -> Option<&Mailbox> {
        self.mailboxes.get(self.selected)
    }
    pub fn find(&self, role: &str) -> Option<&Mailbox> {
        self.mailboxes
            .iter()
            .find(|m| m.role.as_deref() == Some(role) || m.label().eq_ignore_ascii_case(role))
    }
    pub fn select_name(&mut self, name: &str) {
        if let Some(i) = self
            .mailboxes
            .iter()
            .position(|m| m.label().eq_ignore_ascii_case(name) || m.id == name)
        {
            self.selected = i;
        }
    }
    pub fn select_role(&mut self, role: &str) {
        if let Some(i) = self.mailboxes.iter().position(|m| self::role(m) == role) {
            self.selected = i;
        }
    }
    pub fn move_by(&mut self, delta: isize) {
        let len = self.mailboxes.len() as isize;
        if len > 0 {
            self.selected = (self.selected as isize + delta).clamp(0, len - 1) as usize;
        }
    }
}

fn rank(m: &Mailbox) -> u8 {
    match role(m) {
        "inbox" => 0,
        "drafts" => 1,
        "sent" => 2,
        "archive" => 3,
        "junk" => 5,
        "trash" => 6,
        _ => 4,
    }
}

fn role(m: &Mailbox) -> &str {
    match m.role.as_deref() {
        Some(r) => r,
        None if m.label().eq_ignore_ascii_case("inbox") => "inbox",
        None if m.label().eq_ignore_ascii_case("archive") => "archive",
        None => "",
    }
}

fn icon(m: &Mailbox) -> &'static str {
    match role(m) {
        "inbox" => "↓",
        "drafts" => "✎",
        "sent" => "↑",
        "archive" => "▤",
        "junk" => "!",
        "trash" => "✕",
        _ => "▫",
    }
}

impl Widget for &Sidebar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        buf.set_style(area, t.base(t.sidebar));
        let inner = area.inner(Margin::new(1, 1));
        let mut y = inner.y;
        Line::from(vec![Span::styled(
            " MAILBOXES",
            Style::new().fg(t.faint).add_modifier(Modifier::BOLD),
        )])
        .render(
            Rect {
                y,
                height: 1,
                ..inner
            },
            buf,
        );
        y += 2;
        for (i, m) in self.mailboxes.iter().enumerate() {
            if y >= inner.bottom() {
                break;
            }
            let row = Rect {
                y,
                height: 1,
                ..inner
            };
            let selected = i == self.selected;
            let bg = match (selected, self.focused) {
                (true, true) => t.selected,
                (true, false) => t.selected_dim,
                _ => t.sidebar,
            };
            buf.set_style(row, Style::new().bg(bg));
            let name_style = if selected {
                Style::new().fg(t.fg).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(t.dim)
            };
            let marker = if selected && self.focused { "▌" } else { " " };
            Line::from(vec![
                Span::styled(marker, Style::new().fg(t.accent)),
                Span::raw(icon(m)),
                Span::raw(" "),
                Span::styled(m.label().to_string(), name_style),
            ])
            .render(row, buf);
            if let (true, Some((unread, _))) = (selected, self.unread)
                && unread > 0
            {
                Line::styled(
                    format!("{unread} "),
                    Style::new().fg(t.unread).add_modifier(Modifier::BOLD),
                )
                .right_aligned()
                .render(row, buf);
            }
            y += 1;
        }
        if inner.height > 2 {
            let foot = Rect {
                y: inner.bottom() - 1,
                height: 1,
                ..inner
            };
            Line::from(vec![
                Span::styled(" ● ", Style::new().fg(t.ok)),
                Span::styled(self.account.clone(), Style::new().fg(t.dim)),
            ])
            .render(foot, buf);
        }
    }
}
