use std::ops::ControlFlow;

use crossterm::event::KeyEvent;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Widget},
};

use crate::{control::Control, fields::TextField};

#[derive(Debug)]
pub struct MaildirConfig {
    is_focused: bool,
    directory: TextField,
}

#[derive(Debug)]
struct MaildirConfigWidget<'a> {
    config: &'a MaildirConfig,
}

impl Default for MaildirConfig {
    fn default() -> Self {
        let default_mail_dir = directories::UserDirs::new()
            .map(|dirs| dirs.home_dir().join("Mail").display().to_string())
            .unwrap_or_default();
        Self {
            is_focused: false,
            directory: TextField::with_default_value("Maildir directory", default_mail_dir),
        }
    }
}

impl Control for MaildirConfig {
    fn as_widget<'a>(&'a self) -> impl Widget + 'a {
        MaildirConfigWidget { config: self }
    }

    fn focus(&mut self) {
        self.is_focused = true;
    }

    fn blur(&mut self) {
        self.is_focused = false;
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> ControlFlow<()> {
        self.directory.handle_key_event(key)
    }
}

impl<'a> Widget for MaildirConfigWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let config = self.config;
        let block = Block::default()
            .title("Maildir Config")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner = block.inner(area);
        block.render(area, buf);
        let layout = Layout::vertical([1]);
        let [directory] = inner.split(&layout);
        config.directory.as_widget().render(directory, buf);
    }
}
