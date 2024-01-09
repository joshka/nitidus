use std::ops::ControlFlow;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Widget},
};
use strum::{EnumCount, FromRepr};

use crate::{
    control::Control,
    fields::{SelectField, TextField},
};

// TODO hide password
// TODO auth mechanism (password / oauth2)
// TODO store password in keyring
#[derive(Debug)]
pub struct ImapConfig {
    focus: Focus,
    host: TextField,
    protocol: SelectField,
    port: TextField,
    username: TextField,
    password: TextField,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FromRepr, EnumCount)]
enum Focus {
    #[default]
    Host,
    Protocol,
    Port,
    Username,
    Password,
}

#[derive(Debug)]
struct ImapConfigWidget<'a> {
    config: &'a ImapConfig,
}

impl Default for ImapConfig {
    fn default() -> Self {
        Self {
            focus: Focus::Host,
            host: TextField::new("Host"),
            protocol: SelectField::new("Protocol", ["SSL/TLS", "STARTTLS", "None"]),
            port: TextField::new("Port"),
            username: TextField::new("Username"),
            password: TextField::new("Password"),
        }
    }
}

impl Control for ImapConfig {
    fn as_widget<'a>(&'a self) -> impl Widget + 'a {
        ImapConfigWidget { config: self }
    }

    fn focus(&mut self) {
        self.focus = Focus::Host;
        self.focus_child();
    }

    fn blur(&mut self) {
        self.blur_child();
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> ControlFlow<()> {
        match self.focus {
            Focus::Host => self.host.handle_key_event(event),
            Focus::Protocol => self.protocol.handle_key_event(event),
            Focus::Port => self.port.handle_key_event(event),
            Focus::Username => self.username.handle_key_event(event),
            Focus::Password => self.password.handle_key_event(event),
        }?;
        match event.code {
            KeyCode::Tab | KeyCode::Down => self.focus_next(),
            KeyCode::BackTab | KeyCode::Up => self.focus_previous(),
            _ => ControlFlow::Continue(()),
        }
    }
}

impl ImapConfig {
    fn focus_next(&mut self) -> ControlFlow<()> {
        self.blur_child();
        let next_index = self.focus as usize + 1;
        let focus = Focus::from_repr(next_index);
        if let Some(focus) = focus {
            self.focus = focus;
            self.focus_child();
            return ControlFlow::Break(());
        } else {
            return ControlFlow::Continue(());
        }
    }

    fn focus_previous(&mut self) -> ControlFlow<()> {
        self.blur_child();
        if self.focus == Focus::Host {
            return ControlFlow::Continue(());
        }
        let previous_index = (self.focus as usize + Focus::COUNT - 1) % Focus::COUNT;
        self.focus = Focus::from_repr(previous_index).unwrap();
        self.focus_child();
        return ControlFlow::Break(());
    }

    fn focus_child(&mut self) {
        match self.focus {
            Focus::Host => self.host.focus(),
            Focus::Protocol => self.protocol.focus(),
            Focus::Port => self.port.focus(),
            Focus::Username => self.username.focus(),
            Focus::Password => self.password.focus(),
        }
    }

    fn blur_child(&mut self) {
        match self.focus {
            Focus::Host => self.host.blur(),
            Focus::Protocol => self.protocol.blur(),
            Focus::Port => self.port.blur(),
            Focus::Username => self.username.blur(),
            Focus::Password => self.password.blur(),
        }
    }
}

impl<'a> Widget for ImapConfigWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let config = self.config;

        let block = Block::default()
            .title("IMAP Config")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner = block.inner(area);
        block.render(area, buf);

        let layout = Layout::vertical([1, 1, 1, 1, 1]);
        let [host, protocol, port, username, password] = inner.split(&layout);
        config.host.as_widget().render(host, buf);
        config.protocol.as_widget().render(protocol, buf);
        config.port.as_widget().render(port, buf);
        config.username.as_widget().render(username, buf);
        config.password.as_widget().render(password, buf);
    }
}
