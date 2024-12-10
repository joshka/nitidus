use color_eyre::eyre::{eyre, WrapErr};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use email::{
    email::envelope::Envelopes,
    flag::{Flag, Flags},
};
use ratatui::{prelude::*, widgets::*};

use crate::mail_client::MailClient;

pub struct App {
    mail_client: MailClient,
    envelopes: Envelopes,
    body: String,
    running_state: RunningState,
    table_state: TableState,
    folder_name: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Finished,
}

/// A helper struct to display an email envelope in a table.
struct Envelope {
    subject: String,
    from: String,
    date: String,
    flags: Flags,
}

pub async fn run(
    terminal: Terminal<impl Backend>,
    mail_client: MailClient,
) -> color_eyre::Result<()> {
    // note that this should be done in the actual app, but this is just for PoC purposes
    App::new(mail_client).run(terminal).await
}

impl App {
    pub fn new(mail_client: MailClient) -> Self {
        let folder_name = mail_client.folder_or_default().to_string();
        Self {
            mail_client,
            folder_name,
            envelopes: Envelopes::default(),
            body: String::new(),
            running_state: RunningState::Running,
            table_state: TableState::default(),
        }
    }

    pub async fn run(&mut self, mut terminal: Terminal<impl Backend>) -> color_eyre::Result<()> {
        let envelopes = self.mail_client.load_folder().await?;
        self.envelopes = envelopes;
        while self.is_running() {
            self.draw(&mut terminal)?;
            self.update().await.wrap_err("update failed")?;
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running_state == RunningState::Running
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> color_eyre::Result<()> {
        terminal.draw(|frame| self.render_app(frame))?;
        Ok(())
    }

    fn render_app(&mut self, frame: &mut Frame) {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(15),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(frame.area());

        self.render_title(frame, areas[0]);
        self.render_message_list(frame, areas[1]);
        self.render_message(frame, areas[2]);
        self.render_status_bar(frame, areas[3]);
    }

    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let title = Line::from(vec![
            "▁▂▃▄▅▆▇█".light_blue(),
            "   Nitidus   ".black().on_light_blue(),
            "█▇▆▅▄▃▂▁".light_blue(),
        ]);
        frame.render_widget(Paragraph::new(title).alignment(Alignment::Center), area);
    }

    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let text = "j/▼ down | k/▲ up | space/view | q/esc quit".bold();
        frame.render_widget(
            Paragraph::new(text)
                .black()
                .on_white()
                .alignment(Alignment::Center),
            area,
        );
    }

    fn render_message_list(&mut self, frame: &mut Frame, area: Rect) {
        let rows = self.envelopes.iter().map(Envelope::from).map(Row::from);
        let widths = [50, 30, 20].map(Constraint::Percentage);
        let table = Table::new(rows, &widths)
            .header(
                Row::new(vec!["SUBJECT", "FROM", "DATE"])
                    .bold()
                    .underlined()
                    .white(),
            )
            .highlight_symbol(">> ")
            .block(
                Block::default()
                    .title(self.folder_name.clone().bold().white())
                    .borders(Borders::ALL)
                    .border_set(symbols::border::QUADRANT_OUTSIDE)
                    .border_style(Style::new().light_blue()),
            );
        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn render_message(&self, frame: &mut Frame, area: Rect) {
        let text = Paragraph::new(self.body.clone());
        frame.render_widget(text, area);
    }

    async fn update(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            Event::Key(key_event) => self
                .handle_key_event(key_event)
                .await
                .wrap_err("handling key event failed"),
            _ => Ok(()),
        }
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if key_event.kind != KeyEventKind::Press {
            return Ok(());
        }
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit(),
            KeyCode::Char('j') | KeyCode::Down => self.next(),
            KeyCode::Char('k') | KeyCode::Up => self.previous(),
            KeyCode::Char(' ') => self.load_message().await?,
            _ => {}
        }
        Ok(())
    }

    fn quit(&mut self) {
        self.running_state = RunningState::Finished;
    }

    fn next(&mut self) {
        let next = (self.table_state.selected().unwrap_or(0) + 1) % self.envelope_count();
        self.table_state.select(Some(next));
    }

    fn previous(&mut self) {
        let previous = (self.table_state.selected().unwrap_or(0) + self.envelope_count() - 1)
            % self.envelope_count();
        self.table_state.select(Some(previous));
    }

    fn envelope_count(&self) -> usize {
        self.envelopes.len()
    }

    async fn load_message(&mut self) -> Result<(), color_eyre::eyre::Error> {
        let index = self.table_state.selected().unwrap_or(0);
        let id = self.envelopes[index].id.clone();
        let messages = self.mail_client.load_messages(&id).await?;
        if let Some(message) = messages.first() {
            let body = message
                .raw()
                .map_err(|err| eyre!("cannot get raw message: {}", err))?;
            self.body = String::from_utf8_lossy(body).into_owned();
        }
        Ok(())
    }
}

impl From<&email::email::envelope::Envelope> for Envelope {
    fn from(envelope: &email::email::envelope::Envelope) -> Self {
        let subject = envelope.subject.clone();
        let from = envelope.from.to_string();
        let date = envelope.date.format("%F %R%:z").to_string();
        let flags = envelope.flags.clone();
        Self {
            subject,
            from,
            date,
            flags,
        }
    }
}

impl From<Envelope> for Row<'_> {
    fn from(envelope: Envelope) -> Self {
        if envelope.flags.contains(&Flag::Seen) {
            Row::new(vec![
                envelope.subject.green(),
                envelope.from.blue(),
                envelope.date.yellow(),
            ])
        } else {
            Row::new(vec![
                envelope.subject.light_green(),
                envelope.from.light_blue(),
                envelope.date.light_yellow(),
            ])
        }
    }
}
