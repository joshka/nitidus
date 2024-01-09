use std::{borrow::Cow, ops::ControlFlow};

use crossterm::event::{KeyCode, KeyEvent};
use directories::ProjectDirs;
use email::account::AccountConfig;
use ratatui::{layout::SegmentSize, prelude::*, widgets::Widget};
use serde::de;

use crate::control::Control;

#[derive(Debug)]
pub struct SelectField {
    name: &'static str,
    options: Vec<&'static str>,
    index: usize,
    is_focused: bool,
}

#[derive(Debug)]
struct SelectFieldWidget<'a> {
    field: &'a SelectField,
}

impl SelectField {
    pub fn new<T: IntoIterator<Item = &'static str>>(name: &'static str, options: T) -> Self {
        Self {
            name,
            index: 0,
            options: options.into_iter().collect(),
            is_focused: false,
        }
    }

    pub fn selected(&self) -> Option<&'static str> {
        self.options.get(self.index).copied()
    }
}
impl Control for SelectField {
    fn as_widget<'a>(&'a self) -> impl Widget + 'a {
        SelectFieldWidget { field: self }
    }

    fn focus(&mut self) {
        self.is_focused = true;
    }

    fn blur(&mut self) {
        self.is_focused = false;
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> ControlFlow<()> {
        match key.code {
            KeyCode::Left => {
                self.index = (self.index + self.options.len() - 1) % self.options.len();
            }
            KeyCode::Right => {
                self.index = (self.index + 1) % self.options.len();
            }
            _ => return ControlFlow::Continue(()),
        }
        ControlFlow::Break(())
    }
}

impl<'a> Widget for SelectFieldWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let field = self.field;
        let name_style = if field.is_focused {
            Style::default().bold()
        } else {
            Style::default()
        };

        let mut spans = vec![Span::styled(format!("{}:", field.name), name_style)];
        for (index, &option) in field.options.iter().enumerate() {
            spans.push(Span::raw(" "));
            if index == field.index {
                spans.push(Span::styled(option, Style::default().underlined()));
            } else {
                spans.push(Span::raw(option));
            }
        }
        let line = Line::from(spans);
        line.render(area, buf);
    }
}
