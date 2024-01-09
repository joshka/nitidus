use crate::account_config::AccountConfigView;
use color_eyre::eyre::WrapErr;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;

pub fn run(terminal: Terminal<impl Backend>) -> color_eyre::Result<()> {
    let mut app = App::default();
    app.run(terminal)
}

#[derive(Debug, Default)]
struct App {
    running_state: RunningState,
    configure_account: AccountConfigView,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Finished,
}

impl App {
    fn run(&mut self, mut terminal: Terminal<impl Backend>) -> color_eyre::Result<()> {
        terminal.show_cursor()?;
        while self.is_running() {
            self.draw(&mut terminal)?;
            self.update()?;
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running_state == RunningState::Running
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> color_eyre::Result<()> {
        terminal.draw(|frame| {
            self.configure_account.render(frame);
        })?;
        Ok(())
    }

    fn update(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            Event::Key(key_event) => self
                .handle_key_event(key_event)
                .wrap_err("handling key event failed"),
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if key_event.kind != KeyEventKind::Press {
            return Ok(());
        }
        match key_event.code {
            KeyCode::Esc => self.quit(),
            _ => self.configure_account.handle_key_event(key_event),
        }
        Ok(())
    }

    fn quit(&mut self) {
        self.running_state = RunningState::Finished;
    }
}
