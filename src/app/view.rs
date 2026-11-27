//! Drawing the app: the title bar, the main area for the focus or the draft, the status bar, and
//! any overlay on top.

use ratatui::Frame;
use ratatui::prelude::*;

use super::navigation::Focus;
use super::overlay::Overlay;
use super::{App, keymap};
use crate::ui::help::Help;
use crate::ui::status::Status;
use crate::ui::topbar::TopBar;

/// The narrowest terminal that shows all three panes; narrower ones show one at a time.
const THREE_PANES: u16 = 110;

/// The sidebar's width.
const SIDEBAR_WIDTH: u16 = 22;

/// The list's share of the width beside the reader.
const LIST_PERCENT: u16 = 38;

impl App {
    /// Draws the whole screen.
    pub(super) fn render(&mut self, frame: &mut Frame<'_>) {
        let [top, main, bottom] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());
        frame.render_widget(self.top_bar(), top);
        self.render_main(frame, main);
        self.render_status(frame, bottom);
        self.render_overlay(frame, main);
    }

    /// The title bar.
    fn top_bar(&self) -> TopBar<'_> {
        let place = if self.drafting.is_open() {
            "Compose"
        } else {
            self.list.title.as_str()
        };
        TopBar {
            account: &self.sidebar.account,
            place,
            busy: self.pending.last().copied(),
            tick: self.tick,
        }
    }

    /// The draft under review, three panes, or the focused pane alone on a narrow terminal.
    fn render_main(&mut self, frame: &mut Frame<'_>, area: Rect) {
        if let Some(composer) = self.drafting.composer() {
            frame.render_widget(composer, area);
        } else if area.width >= THREE_PANES {
            let [sidebar, list, reader] = Layout::horizontal([
                Constraint::Length(SIDEBAR_WIDTH),
                Constraint::Percentage(LIST_PERCENT),
                Constraint::Fill(1),
            ])
            .areas(area);
            frame.render_widget(&self.sidebar, sidebar);
            frame.render_widget(&mut self.list, list);
            frame.render_widget(&mut self.reader, reader);
        } else {
            self.render_narrow(frame, area);
        }
    }

    /// One pane at a time; the sidebar sits beside the list.
    fn render_narrow(&mut self, frame: &mut Frame<'_>, area: Rect) {
        match self.focus {
            Focus::Sidebar => {
                let [sidebar, list] =
                    Layout::horizontal([Constraint::Length(SIDEBAR_WIDTH), Constraint::Fill(1)])
                        .areas(area);
                frame.render_widget(&self.sidebar, sidebar);
                frame.render_widget(&mut self.list, list);
            }
            Focus::List => frame.render_widget(&mut self.list, area),
            Focus::Reader => frame.render_widget(&mut self.reader, area),
        }
    }

    /// The notice, else the key hints for where keys go now.
    fn render_status(&self, frame: &mut Frame<'_>, area: Rect) {
        let hints = self.context().hints();
        let status = self
            .notice
            .as_ref()
            .map_or(Status::Hints(&hints), super::notice::Notice::status);
        frame.render_widget(status, area);
    }

    /// The open overlay, over the main area.
    fn render_overlay(&self, frame: &mut Frame<'_>, area: Rect) {
        match &self.overlay {
            None => {}
            Some(Overlay::Help) => frame.render_widget(Help(&keymap::help()), area),
            Some(Overlay::Picker(picker, _)) => frame.render_widget(picker, area),
            Some(Overlay::Prompt(prompt, _)) => frame.render_widget(prompt, area),
            Some(Overlay::Confirm(confirm, _)) => frame.render_widget(confirm, area),
        }
    }
}
