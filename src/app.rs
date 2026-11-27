//! The app: screen state, the event loop, and the step from a key to a worker request.
//!
//! A key becomes an [`Action`](action::Action) through the [`keymap`], or goes to the open overlay. Each action
//! group is performed by the module that owns it: [`navigation`], [`triage`], [`compose`],
//! [`reading`] and [`mailboxes`]. Worker answers come back through [`responses`], and [`view`]
//! draws the components.

mod action;
mod compose;
mod keymap;
mod mailboxes;
mod navigation;
mod notice;
mod overlay;
mod perform;
mod reading;
mod responses;
mod triage;
mod view;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use self::compose::Drafting;
use self::navigation::Focus;
use self::notice::Notice;
use self::overlay::Overlay;
use crate::ui::envelope_list::EnvelopeList;
use crate::ui::reader::Reader;
use crate::ui::sidebar::Sidebar;
use crate::worker::{AccountSpec, Request, Worker};

/// How long the loop waits for a key before drawing again, which also turns the spinner.
const FRAME: Duration = Duration::from_millis(80);

/// How often the open mailbox is checked for new mail while idle.
const CHECK_EVERY: Duration = Duration::from_secs(30);

/// The app.
pub struct App {
    /// The config files, kept to open another account.
    config: Vec<PathBuf>,

    /// The worker running mail commands.
    worker: Worker,

    /// The mailbox list.
    sidebar: Sidebar,

    /// The envelope list.
    list: EnvelopeList,

    /// The message reader.
    reader: Reader,

    /// The draft being written, if any, which takes over the main area.
    drafting: Drafting,

    /// Which pane keys go to.
    focus: Focus,

    /// The floating panel taking keys, if any.
    overlay: Option<Overlay>,

    /// What the worker is doing, newest last, as the top bar shows it.
    pending: Vec<&'static str>,

    /// A message for the status bar, until it expires.
    notice: Option<Notice>,

    /// The active search of the open mailbox.
    search: Option<String>,

    /// The mailbox to open once mailboxes are listed, from `--mailbox`.
    initial_mailbox: Option<String>,

    /// When the open mailbox was last listed or checked.
    last_check: Instant,

    /// Frames drawn, which turn the spinner.
    tick: usize,

    /// Whether the loop should end.
    quit: bool,
}

impl App {
    /// An app on `account`, opening `mailbox` first when given; it asks for the mailboxes at once.
    pub fn new(account: AccountSpec, mailbox: Option<String>) -> Self {
        let name = account.account.clone().unwrap_or_else(|| "default".into());
        let mut app = Self {
            config: account.config.clone(),
            worker: Worker::spawn(account),
            sidebar: Sidebar::new(name),
            list: EnvelopeList::loading("Inbox".into()),
            reader: Reader::default(),
            drafting: Drafting::default(),
            focus: Focus::List,
            overlay: None,
            pending: Vec::new(),
            notice: None,
            search: None,
            initial_mailbox: mailbox,
            last_check: Instant::now(),
            tick: 0,
            quit: false,
        };
        app.set_focus(Focus::List);
        app.request("Loading mailboxes", Request::Mailboxes);
        app
    }

    /// Runs until the user quits: draw, take worker answers, take a key, then housekeeping.
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.quit {
            terminal.draw(|frame| self.render(frame))?;
            self.take_responses();
            self.take_key()?;
            if self.drafting.take_edit_request() {
                self.edit_draft(terminal)?;
            }
            self.housekeeping();
        }
        Ok(())
    }

    /// Sends a request, naming it in the top bar until it is answered.
    fn request(&mut self, doing: &'static str, request: Request) {
        self.pending.push(doing);
        self.worker.send(request);
    }

    /// Folds in every answer that has arrived.
    fn take_responses(&mut self) {
        while let Some(response) = self.worker.try_recv() {
            self.pending.pop();
            self.on_response(response);
        }
    }

    /// Waits up to a frame for a key press and handles it.
    fn take_key(&mut self) -> Result<()> {
        if !event::poll(FRAME)? {
            return Ok(());
        }
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            self.on_key(key);
        }
        Ok(())
    }

    /// Ctrl-C quits anywhere; an open overlay takes the key; otherwise the keymap maps it.
    fn on_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.quit = true;
        } else if let Some(overlay) = self.overlay.take() {
            self.on_overlay_key(overlay, key);
        } else if let Some(action) = self.context().action(key) {
            self.perform(action);
        }
    }

    /// Turns the spinner, expires the notice, and checks for new mail when idle.
    fn housekeeping(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        if self.notice.as_ref().is_some_and(Notice::expired) {
            self.notice = None;
        }
        let idle = self.pending.is_empty() && self.search.is_none();
        if idle && self.last_check.elapsed() > CHECK_EVERY {
            self.check_for_mail();
        }
    }

    /// Says something went well.
    fn info(&mut self, text: impl Into<String>) {
        self.notice = Some(Notice::info(text));
    }

    /// Says something failed.
    fn error(&mut self, text: impl Into<String>) {
        self.notice = Some(Notice::error(text));
    }
}
