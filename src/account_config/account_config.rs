use std::{borrow::Cow, ops::ControlFlow};

use crossterm::event::{KeyCode, KeyEvent};
use directories::ProjectDirs;
use ratatui::{
    layout::SegmentSize,
    prelude::*,
    widgets::{Block, BorderType, Borders, Widget},
};
use serde::de;
use strum::{EnumCount, FromRepr};

use super::{ImapConfig, MaildirConfig};
use crate::{
    control::Control,
    fields::{SelectField, TextField},
};

#[derive(Debug)]
pub struct AccountConfig {
    focus: Focus,
    account_name: TextField,
    email: TextField,
    display_name: TextField,
    backend: SelectField,
    imap_config: ImapConfig,
    maildir_config: MaildirConfig,
}

#[derive(Debug)]
struct AccountConfigWidget<'a> {
    config: &'a AccountConfig,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FromRepr, EnumCount)]
enum Focus {
    #[default]
    AccountName,
    Email,
    DisplayName,
    Backend,
    BackendConfig,
}

impl Default for AccountConfig {
    fn default() -> Self {
        Self {
            focus: Focus::AccountName,
            account_name: TextField::new("Account Name"),
            email: TextField::new("Email"),
            display_name: TextField::new("Display Name"),
            backend: SelectField::new("Backend", ["IMAP", "Maildir", "None"]),
            imap_config: ImapConfig::default(),
            maildir_config: MaildirConfig::default(),
        }
    }
}

impl Control for AccountConfig {
    fn as_widget<'a>(&'a self) -> impl Widget + 'a {
        AccountConfigWidget { config: self }
    }

    fn focus(&mut self) {
        self.focus = Focus::AccountName;
        self.focus_child();
    }

    fn blur(&mut self) {
        // do nothing as we always have a focus
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> ControlFlow<()> {
        match self.focus {
            Focus::AccountName => self.account_name.handle_key_event(event),
            Focus::Email => self.email.handle_key_event(event),
            Focus::DisplayName => self.display_name.handle_key_event(event),
            Focus::Backend => self.backend.handle_key_event(event),
            Focus::BackendConfig => match self.backend.selected() {
                Some("IMAP") => self.imap_config.handle_key_event(event),
                Some("Maildir") => self.maildir_config.handle_key_event(event),
                _ => ControlFlow::Continue(()),
            },
        }?;
        match event.code {
            KeyCode::Tab | KeyCode::Down => self.focus_next(),
            KeyCode::BackTab | KeyCode::Up => self.focus_prev(),
            _ => ControlFlow::Continue(()),
        }
    }
}

impl AccountConfig {
    fn focus_prev(&mut self) -> ControlFlow<()> {
        self.blur_child();
        let previous_index = (self.focus as usize + Focus::COUNT - 1) % Focus::COUNT;
        self.focus = Focus::from_repr(previous_index).unwrap();
        self.focus_child();
        ControlFlow::Break(())
    }

    fn focus_next(&mut self) -> ControlFlow<()> {
        self.blur_child();
        let next_index = (self.focus as usize + 1) % Focus::COUNT;
        self.focus = Focus::from_repr(next_index).unwrap();
        self.focus_child();
        ControlFlow::Break(())
    }

    fn focus_child(&mut self) {
        match self.focus {
            Focus::AccountName => self.account_name.focus(),
            Focus::Email => self.email.focus(),
            Focus::DisplayName => self.display_name.focus(),
            Focus::Backend => self.backend.focus(),
            Focus::BackendConfig => match self.backend.selected() {
                Some("IMAP") => self.imap_config.focus(),
                Some("Maildir") => self.maildir_config.focus(),
                _ => {}
            },
        }
    }

    fn blur_child(&mut self) {
        match self.focus {
            Focus::AccountName => self.account_name.blur(),
            Focus::Email => self.email.blur(),
            Focus::DisplayName => self.display_name.blur(),
            Focus::Backend => self.backend.blur(),
            Focus::BackendConfig => match self.backend.selected() {
                Some("IMAP") => self.imap_config.blur(),
                Some("Maildir") => self.maildir_config.blur(),
                _ => {}
            },
        }
    }
}

impl Widget for AccountConfigWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Configure Account")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner = block.inner(area);
        block.render(area, buf);

        let backend_size = match self.config.backend.selected() {
            Some("IMAP") => 7,
            Some("Maildir") => 3,
            _ => 0,
        };
        let layout = Layout::vertical([1, 1, 1, 1, backend_size]).segment_size(SegmentSize::None);
        let [account_name, email, display_name, backend, backend_config] = inner.split(&layout);

        let config = self.config;
        config.account_name.as_widget().render(account_name, buf);
        config.email.as_widget().render(email, buf);
        config.display_name.as_widget().render(display_name, buf);
        config.backend.as_widget().render(backend, buf);

        match config.backend.selected() {
            Some("IMAP") => config.imap_config.as_widget().render(backend_config, buf),
            Some("Maildir") => config
                .maildir_config
                .as_widget()
                .render(backend_config, buf),
            _ => {}
        }
    }
}
