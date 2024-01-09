use std::{borrow::Cow, ops::ControlFlow};

use crossterm::event::{KeyCode, KeyEvent};
use directories::ProjectDirs;
use email::account::AccountConfig;
use ratatui::{layout::SegmentSize, prelude::*, widgets::Widget};
use serde::de;

use crate::control::Control;

#[derive(Debug)]
pub struct TextField {
    name: &'static str,
    value: String,
    is_focused: bool,
}

#[derive(Debug)]
struct TextFieldWidget<'a> {
    field: &'a TextField,
}

impl TextField {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            value: String::new(),
            is_focused: false,
        }
    }

    pub fn with_default_value(name: &'static str, value: String) -> Self {
        Self {
            name,
            value,
            is_focused: false,
        }
    }
}
impl Control for TextField {
    fn as_widget<'a>(&'a self) -> impl Widget + 'a {
        TextFieldWidget { field: self }
    }

    fn focus(&mut self) {
        self.is_focused = true;
    }

    fn blur(&mut self) {
        self.is_focused = false;
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> ControlFlow<()> {
        match key.code {
            KeyCode::Char(c) => {
                self.value.push(c);
            }
            KeyCode::Backspace => {
                self.value.pop();
            }
            _ => {
                return ControlFlow::Continue(());
            }
        }
        ControlFlow::Break(())
    }
}

impl<'a> Widget for TextFieldWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let field = self.field;
        let name_style = if field.is_focused {
            Style::default().bold()
        } else {
            Style::default()
        };
        let line = Line::from(vec![
            Span::styled(format!("{}: ", field.name), name_style),
            Span::raw(&field.value),
        ]);
        line.render(area, buf);
    }
}
