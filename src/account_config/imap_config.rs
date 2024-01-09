use ratatui::{prelude::*, widgets::Widget};

use crate::fields::{SelectField, TextField};

// TODO hide password
// TODO auth mechanism (password / oauth2)
// TODO store password in keyring
#[derive(Debug)]
pub struct ImapConfig {
    is_focused: bool,
    focus: Focus,
    host: TextField,
    protocol: SelectField,
    port: TextField,
    username: TextField,
    password: TextField,
}

#[derive(Debug, Default, PartialEq, Eq)]
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
            is_focused: false,
            focus: Focus::default(),
            host: TextField::new("Host"),
            protocol: SelectField::new("Protocol", ["SSL/TLS", "STARTTLS", "None"]),
            port: TextField::new("Port"),
            username: TextField::new("Username"),
            password: TextField::new("Password"),
        }
    }
}

impl ImapConfig {
    pub fn as_widget<'a>(&'a self) -> impl Widget + 'a {
        ImapConfigWidget { config: self }
    }

    pub fn focus(&mut self) {
        self.is_focused = true;
    }

    pub fn blur(&mut self) {
        self.is_focused = false;
    }
}

impl<'a> Widget for ImapConfigWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let config = self.config;
        let layout = Layout::vertical([1, 1, 1, 1, 1]);
        let [host, protocol, port, username, password] = area.split(&layout);
        config.host.as_widget().render(host, buf);
        config.protocol.as_widget().render(protocol, buf);
        config.port.as_widget().render(port, buf);
        config.username.as_widget().render(username, buf);
        config.password.as_widget().render(password, buf);
    }
}
