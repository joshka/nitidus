use std::ops::ControlFlow;

use crossterm::event::KeyEvent;
use ratatui::widgets::Widget;

pub trait Control {
    fn as_widget<'a>(&'a self) -> impl Widget + 'a;
    fn focus(&mut self);
    fn blur(&mut self);
    fn handle_key_event(&mut self, key: KeyEvent) -> ControlFlow<()>;
}
