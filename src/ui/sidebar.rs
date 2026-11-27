//! The mailbox list, a tinted column on the left with the account at its foot.

use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::layout::{Emphasis, PaneFocus, below, line};
use super::movement::{Movement, Navigate};
use super::theme;
use crate::mail::{Mailbox, Role};

/// The sidebar's state.
#[derive(Default)]
pub struct Sidebar {
    /// The mailboxes, sorted by role then name.
    mailboxes: Vec<Mailbox>,

    /// The index of the open mailbox.
    selected: usize,

    /// Whether keys move the sidebar's selection.
    pub focus: PaneFocus,

    /// The account's name, shown at the foot.
    pub account: String,

    /// The open mailbox's unread count; other mailboxes' counts aren't known cheaply.
    pub unread: usize,
}

impl Sidebar {
    /// A sidebar for `account`, waiting for its mailboxes.
    pub fn new(account: String) -> Self {
        Self {
            account,
            ..Self::default()
        }
    }

    /// Replaces the mailboxes, keeping the selection on the same mailbox when it is still there.
    pub fn set_mailboxes(&mut self, mut mailboxes: Vec<Mailbox>) {
        let open = self.current().map(|mailbox| mailbox.id.clone());
        mailboxes.sort_by_key(|mailbox| (mailbox.role(), mailbox.label().to_lowercase()));
        self.mailboxes = mailboxes;
        self.selected = 0;
        match open {
            Some(id) => self.select_named(&id),
            None => self.select_role(Role::Inbox),
        }
    }

    /// The open mailbox.
    pub fn current(&self) -> Option<&Mailbox> {
        self.mailboxes.get(self.selected)
    }

    /// Every mailbox, in sidebar order.
    pub fn mailboxes(&self) -> &[Mailbox] {
        &self.mailboxes
    }

    /// The first mailbox playing `role`.
    pub fn find(&self, role: Role) -> Option<&Mailbox> {
        self.mailboxes.iter().find(|mailbox| mailbox.role() == role)
    }

    /// Opens the mailbox `name` names, by id or label, if there is one.
    pub fn select_named(&mut self, name: &str) {
        if let Some(index) = self
            .mailboxes
            .iter()
            .position(|mailbox| mailbox.is_named(name))
        {
            self.selected = index;
        }
    }

    /// Opens the first mailbox playing `role`, if there is one.
    fn select_role(&mut self, role: Role) {
        if let Some(index) = self
            .mailboxes
            .iter()
            .position(|mailbox| mailbox.role() == role)
        {
            self.selected = index;
        }
    }
}

impl Navigate for Sidebar {
    fn navigate(&mut self, movement: Movement) -> bool {
        let before = self.selected;
        self.selected = movement.index(self.selected, self.mailboxes.len());
        self.selected != before
    }
}

/// The glyph beside a mailbox of `role`.
fn icon(role: Role) -> &'static str {
    match role {
        Role::Inbox => "↓",
        Role::Drafts => "✎",
        Role::Sent => "↑",
        Role::Archive => "▤",
        Role::Junk => "!",
        Role::Trash => "✕",
        Role::Other => "▫",
    }
}

impl Widget for &Sidebar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        buf.set_style(area, t.base(t.sidebar));
        let inner = area.inner(Margin::new(1, 1));
        let heading = Style::new().fg(t.faint).add_modifier(Modifier::BOLD);
        Line::styled(" MAILBOXES", heading).render(line(inner, 0), buf);
        let list = below(inner, 2);
        for (row, (index, mailbox)) in (0..list.height).zip(self.mailboxes.iter().enumerate()) {
            self.render_row(
                mailbox,
                Emphasis::of_row(index, self.selected, self.focus),
                line(list, row),
                buf,
            );
        }
        self.render_account(line(inner, inner.height.saturating_sub(1)), buf);
    }
}

impl Sidebar {
    /// One mailbox's row; the open one is filled, with its unread count on the right.
    fn render_row(&self, mailbox: &Mailbox, emphasis: Emphasis, area: Rect, buf: &mut Buffer) {
        let t = theme();
        buf.set_style(area, Style::new().bg(emphasis.background(t.sidebar)));
        let (edge, name) = match emphasis {
            Emphasis::Focused => ("▌", Style::new().fg(t.fg).add_modifier(Modifier::BOLD)),
            Emphasis::Unfocused => (" ", Style::new().fg(t.fg).add_modifier(Modifier::BOLD)),
            Emphasis::Plain => (" ", Style::new().fg(t.dim)),
        };
        let label = vec![
            Span::styled(edge, Style::new().fg(t.accent)),
            Span::raw(format!("{} ", icon(mailbox.role()))),
            Span::styled(mailbox.label().to_owned(), name),
        ];
        Line::from(label).render(area, buf);
        if emphasis != Emphasis::Plain && self.unread > 0 {
            let count = Style::new().fg(t.unread).add_modifier(Modifier::BOLD);
            Line::styled(format!("{} ", self.unread), count)
                .right_aligned()
                .render(area, buf);
        }
    }

    /// The account's name with a status dot.
    fn render_account(&self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        let account = vec![
            Span::styled(" ● ", Style::new().fg(t.ok)),
            Span::styled(self.account.clone(), Style::new().fg(t.dim)),
        ];
        Line::from(account).render(area, buf);
    }
}
