use std::borrow::Cow;

use crossterm::event::{KeyCode, KeyEvent};
use directories::ProjectDirs;
use email::account::AccountConfig;
use ratatui::{layout::SegmentSize, prelude::*, widgets::Widget};
use serde::de;

use super::{ImapConfig, MaildirConfig};
use crate::fields::{SelectField, TextField};

#[derive(Debug)]
pub struct AccountConfigView {
    focus: Focus,
    account_name: TextField,
    email: TextField,
    display_name: TextField,
    backend: SelectField,
    imap_config: ImapConfig,
    maildir_config: MaildirConfig,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum Focus {
    #[default]
    AccountName,
    Email,
    DisplayName,
    Backend,
}

impl Default for AccountConfigView {
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

impl AccountConfigView {
    pub fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            // TODO handle focus generically
            KeyCode::Tab | KeyCode::Down => match self.focus {
                Focus::AccountName => {
                    self.focus = Focus::Email;
                    self.account_name.blur();
                    self.email.focus();
                }
                Focus::Email => {
                    self.focus = Focus::DisplayName;
                    self.email.blur();
                    self.display_name.focus();
                }
                Focus::DisplayName => {
                    self.focus = Focus::Backend;
                    self.display_name.blur();
                    self.backend.focus();
                }
                Focus::Backend => {
                    self.focus = Focus::AccountName;
                    self.backend.blur();
                    self.account_name.focus();
                }
            },
            KeyCode::BackTab | KeyCode::Up => match self.focus {
                Focus::AccountName => {
                    self.focus = Focus::DisplayName;
                    self.account_name.blur();
                    self.backend.focus();
                }
                Focus::Email => {
                    self.focus = Focus::AccountName;
                    self.email.blur();
                    self.account_name.focus();
                }
                Focus::DisplayName => {
                    self.focus = Focus::Email;
                    self.display_name.blur();
                    self.email.focus();
                }
                Focus::Backend => {
                    self.focus = Focus::DisplayName;
                    self.backend.blur();
                    self.display_name.focus();
                }
            },
            _ => match self.focus {
                Focus::AccountName => self.account_name.handle_key_event(event),
                Focus::Email => self.email.handle_key_event(event),
                Focus::DisplayName => self.display_name.handle_key_event(event),
                Focus::Backend => self.backend.handle_key_event(event),
            },
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let view_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
        let main_layout = Layout::vertical([1, 1, 1, 1, 0]);

        let [top, main] = frame.size().split(&view_layout);
        let [account_name, email, display_name, backend, backend_config] = main.split(&main_layout);

        frame.render_widget(Line::from("Configure Account"), top);
        frame.render_widget(self.account_name.as_widget(), account_name);
        frame.render_widget(self.email.as_widget(), email);
        frame.render_widget(self.display_name.as_widget(), display_name);
        frame.render_widget(self.backend.as_widget(), backend);
        match self.backend.selected() {
            Some("IMAP") => frame.render_widget(self.imap_config.as_widget(), backend_config),
            Some("Maildir") => frame.render_widget(self.maildir_config.as_widget(), backend_config),
            _ => {}
        }
    }
}
