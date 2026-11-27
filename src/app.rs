//! App state and the event loop.
//!
//! The app routes keys to the focused component or the open overlay, turns
//! actions into worker requests, and folds worker responses back into the
//! components. Components own their state and draw themselves.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;
use ratatui::{DefaultTerminal, Frame};

use crate::compose;
use crate::mail::{ComposeKind, Draft, Receipt};
use crate::ui::composer::Composer;
use crate::ui::envelopes::EnvelopeList;
use crate::ui::help::Help;
use crate::ui::overlay::{Confirm, Picker, Prompt};
use crate::ui::reader::Reader;
use crate::ui::sidebar::Sidebar;
use crate::ui::status::Status;
use crate::ui::topbar::TopBar;
use crate::worker::{Request, Response, Worker};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Sidebar,
    List,
    Reader,
}

enum Notice {
    Info(String),
    Error(String),
}

/// What a picker's choice does.
#[derive(Clone, Copy)]
enum PickFor {
    MoveTo,
    CopyTo,
    Account,
    Link,
    OpenFile,
}

enum PromptFor {
    Search,
    CreateMailbox,
    RenameMailbox(String),
    Attach,
}

enum ConfirmFor {
    DeleteForever(Vec<String>),
    DeleteMailbox(String),
    Discard,
}

enum Overlay {
    Help,
    Picker(Picker<String>, PickFor),
    Prompt(Prompt, PromptFor),
    Confirm(Confirm, ConfirmFor),
}

pub struct App {
    config: Vec<PathBuf>,
    worker: Worker,
    sidebar: Sidebar,
    list: EnvelopeList,
    reader: Reader,
    focus: Focus,
    overlay: Option<Overlay>,
    /// The draft under review, which takes over the main area.
    draft: Option<Draft>,
    /// Ask the run loop to open the draft in the editor.
    edit_draft: bool,
    /// Editor text that failed to parse, kept so editing again loses nothing.
    unparsed: Option<String>,
    draft_scroll: u16,
    pending: Vec<&'static str>,
    notice: Option<(Notice, Instant)>,
    search: Option<String>,
    tick: usize,
    initial_mailbox: Option<String>,
    last_refresh: Instant,
    quit: bool,
}

const REFRESH_EVERY: Duration = Duration::from_secs(30);

impl App {
    pub fn new(config: Vec<PathBuf>, account: Option<String>, mailbox: Option<String>) -> Self {
        let worker = Worker::spawn(config.clone(), account.clone());
        let mut app = Self {
            config,
            worker,
            sidebar: Sidebar {
                account: account.unwrap_or_else(|| "default".into()),
                ..Default::default()
            },
            list: EnvelopeList {
                title: "Inbox".into(),
                loading: true,
                ..Default::default()
            },
            reader: Reader::default(),
            focus: Focus::List,
            overlay: None,
            draft: None,
            edit_draft: false,
            unparsed: None,
            draft_scroll: 0,
            pending: vec![],
            notice: None,
            search: None,
            tick: 0,
            initial_mailbox: mailbox,
            last_refresh: Instant::now(),
            quit: false,
        };
        app.request("Loading mailboxes", Request::Mailboxes);
        app
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.quit {
            self.sync_focus();
            terminal.draw(|f| self.render(f))?;
            while let Ok(response) = self.worker.rx.try_recv() {
                self.pending.pop();
                self.on_response(response);
            }
            if event::poll(Duration::from_millis(80))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.on_key(key);
            }
            if self.edit_draft {
                self.edit_draft = false;
                ratatui::restore();
                let result = self.run_editor();
                *terminal = ratatui::init();
                terminal.clear()?;
                if let Err(err) = result {
                    self.error(format!("{err:#}"));
                }
            }
            self.tick += 1;
            if matches!(&self.notice, Some((_, at)) if at.elapsed() > Duration::from_secs(5)) {
                self.notice = None;
            }
            if self.pending.is_empty()
                && self.last_refresh.elapsed() > REFRESH_EVERY
                && self.search.is_none()
            {
                self.last_refresh = Instant::now();
                let mailbox = self.mailbox();
                self.worker.send(Request::Envelopes {
                    mailbox,
                    page: 1,
                    query: None,
                    quiet: true,
                });
                self.pending.push("Checking for mail");
            }
        }
        Ok(())
    }

    // ---- requests -------------------------------------------------------

    fn request(&mut self, what: &'static str, request: Request) {
        self.pending.push(what);
        self.worker.send(request);
    }

    fn info(&mut self, msg: impl Into<String>) {
        self.notice = Some((Notice::Info(msg.into()), Instant::now()));
    }

    fn error(&mut self, msg: impl Into<String>) {
        self.notice = Some((Notice::Error(msg.into()), Instant::now()));
    }

    fn mailbox(&self) -> String {
        self.sidebar
            .current()
            .map_or_else(|| "INBOX".into(), |m| m.id.clone())
    }

    fn in_role(&self, role: &str) -> bool {
        self.sidebar.current().is_some_and(|m| {
            m.role.as_deref() == Some(role) || m.label().eq_ignore_ascii_case(role)
        })
    }

    fn load_envelopes(&mut self) {
        let mailbox = self.mailbox();
        self.list.title = self
            .sidebar
            .current()
            .map(|m| m.label().to_string())
            .unwrap_or(mailbox.clone());
        self.list.query = self.search.clone();
        self.list.loading = true;
        self.last_refresh = Instant::now();
        self.request(
            "Loading messages",
            Request::Envelopes {
                mailbox,
                page: 1,
                query: self.search.clone(),
                quiet: false,
            },
        );
    }

    fn load_more_if_near_end(&mut self) {
        if let Some(page) = self.list.next_page
            && !self.list.loading_more
            && self.list.selected + 6 >= self.list.envelopes.len()
        {
            self.list.loading_more = true;
            let mailbox = self.mailbox();
            self.request(
                "Loading more",
                Request::Envelopes {
                    mailbox,
                    page,
                    query: self.search.clone(),
                    quiet: false,
                },
            );
        }
    }

    fn open_current(&mut self) {
        let Some(e) = self.list.current() else {
            self.reader.clear();
            return;
        };
        let id = e.id.clone();
        if self.reader.message.as_ref().is_some_and(|m| m.id == id) {
            return;
        }
        self.reader.clear();
        self.reader.loading = true;
        let mailbox = self.mailbox();
        self.request("Reading", Request::Read { mailbox, id });
        if let Some(e) = self.list.current_mut() {
            e.set("seen", true);
        }
        self.load_more_if_near_end();
    }

    fn switch_account(&mut self, name: String) {
        self.worker = Worker::spawn(self.config.clone(), Some(name.clone()));
        self.pending.clear();
        self.sidebar = Sidebar {
            account: name.clone(),
            ..Default::default()
        };
        self.list = EnvelopeList {
            title: "Inbox".into(),
            loading: true,
            threaded: self.list.threaded,
            ..Default::default()
        };
        self.reader = Reader::default();
        self.search = None;
        self.focus = Focus::List;
        self.request("Loading mailboxes", Request::Mailboxes);
        self.info(format!("Switched to {name}"));
    }

    // ---- responses ------------------------------------------------------

    fn on_response(&mut self, response: Response) {
        match response {
            Response::Accounts(accounts) => {
                let items = accounts.into_iter().map(|a| (a.clone(), a)).collect();
                self.overlay = Some(Overlay::Picker(
                    Picker::new("Switch account", items),
                    PickFor::Account,
                ));
            }
            Response::Mailboxes(mailboxes) => {
                let current = self.sidebar.current().map(|m| m.id.clone());
                self.sidebar.set_mailboxes(mailboxes);
                match self.initial_mailbox.take().or(current) {
                    Some(name) => self.sidebar.select_name(&name),
                    None => self.sidebar.select_role("inbox"),
                }
                self.load_envelopes();
            }
            Response::Envelopes {
                mailbox,
                page,
                query,
                quiet,
                result,
            } => {
                if mailbox != self.mailbox() || query != self.search {
                    return;
                }
                let more = result.next_page.is_some();
                if quiet {
                    let n = self.list.merge_new(result.envelopes);
                    if n > 0 {
                        self.info(format!("{n} new message{}", if n == 1 { "" } else { "s" }));
                    }
                } else if page == 1 {
                    self.list.load(result.envelopes, false);
                    self.list.next_page = more.then_some(2);
                    self.list.loading = false;
                    self.list.offset = 0;
                    self.reader.clear();
                    self.open_current();
                } else {
                    self.list.load(result.envelopes, true);
                    self.list.next_page = more.then_some(page + 1);
                    self.list.loading_more = false;
                }
            }
            Response::Read(message) => {
                if self.list.current().is_some_and(|e| e.id == message.id) {
                    self.reader.message = Some(*message);
                    self.reader.loading = false;
                }
            }
            Response::Raw { id, text } => {
                if self.list.current().is_some_and(|e| e.id == id) {
                    self.reader.raw = Some(text);
                }
            }
            Response::Wrote { what, receipt } => {
                let quiet_flag = what.starts_with("Marked") || what.starts_with("Unmarked");
                self.report(what, &receipt, quiet_flag);
                if !receipt.errors.is_empty() {
                    self.load_envelopes();
                }
            }
            Response::MailboxesChanged(msg) => {
                self.info(msg);
                self.request("Loading mailboxes", Request::Mailboxes);
            }
            Response::Template(draft) => {
                self.draft = Some(*draft);
                self.draft_scroll = 0;
                self.edit_draft = true;
            }
            Response::Submitted { sent, receipt } => {
                let what = match (sent, receipt.queued) {
                    (true, false) => "Message sent",
                    (true, true) => "Queued to send on the next sync",
                    (false, false) => "Saved to Drafts",
                    (false, true) => "Draft queued for the next sync",
                };
                if let Some(Draft {
                    kind: ComposeKind::Reply { id, .. },
                    ..
                }) = &self.draft
                    && sent
                {
                    let id = id.clone();
                    if let Some(e) = self.list.envelopes.iter_mut().find(|e| e.id == id) {
                        e.set("answered", true);
                    }
                }
                self.draft = None;
                self.report(what.into(), &receipt, false);
                if self.in_role("drafts") || self.in_role("sent") {
                    self.load_envelopes();
                }
            }
            Response::Downloaded { paths, open } => {
                if !open {
                    let dir = paths
                        .first()
                        .and_then(|p| std::path::Path::new(p).parent())
                        .map(|d| d.display().to_string());
                    self.info(format!(
                        "Saved {} attachment(s) to {}",
                        paths.len(),
                        dir.unwrap_or_default()
                    ));
                } else if paths.len() == 1 {
                    self.open_target(&paths[0]);
                } else if !paths.is_empty() {
                    let items = paths
                        .into_iter()
                        .map(|p| {
                            (
                                std::path::Path::new(&p)
                                    .file_name()
                                    .map(|f| f.to_string_lossy().into_owned())
                                    .unwrap_or(p.clone()),
                                p,
                            )
                        })
                        .collect();
                    self.overlay = Some(Overlay::Picker(
                        Picker::new("Open attachment", items),
                        PickFor::OpenFile,
                    ));
                } else {
                    self.error("No attachments");
                }
            }
            Response::Failed { action, error } => {
                self.list.loading = false;
                self.list.loading_more = false;
                self.reader.loading = false;
                self.error(format!("{action}: {error}"));
            }
        }
        self.sidebar.unread = Some((self.list.unread(), self.list.envelopes.len()));
    }

    fn report(&mut self, what: String, receipt: &Receipt, quiet: bool) {
        let what = if receipt.queued && !what.contains("sync") {
            format!("{what} (queued for sync)")
        } else {
            what
        };
        match receipt.errors.as_slice() {
            [] if quiet => {}
            [] => self.info(what),
            errors => {
                let list: Vec<String> = errors.iter().map(ToString::to_string).collect();
                self.error(format!(
                    "{what}, but {} failed: {}",
                    errors.len(),
                    list.join("; ")
                ));
            }
        }
    }

    fn open_target(&mut self, target: &str) {
        match compose::open(target) {
            Ok(()) => self.info(format!("Opened {target}")),
            Err(err) => self.error(format!("{err:#}")),
        }
    }

    // ---- composing ------------------------------------------------------

    fn compose(&mut self, kind: ComposeKind) {
        self.request("Preparing message", Request::Template(kind));
    }

    fn run_editor(&mut self) -> Result<()> {
        let Some(draft) = &mut self.draft else {
            return Ok(());
        };
        let file = tempfile::Builder::new()
            .prefix("nitidus-")
            .suffix(".eml")
            .tempfile()?;
        let text = self.unparsed.take().unwrap_or_else(|| draft.to_file());
        std::fs::write(file.path(), &text)?;
        compose::edit(file.path())?;
        let edited = std::fs::read_to_string(file.path())?;
        if let Err(err) = draft.update_from_file(&edited) {
            self.unparsed = Some(edited);
            return Err(err.context("the draft has a problem (e to edit again)"));
        }
        Ok(())
    }

    fn on_draft_key(&mut self, key: KeyEvent) {
        let Some(draft) = &self.draft else { return };
        match key.code {
            KeyCode::Char('s' | 'y') => match draft.problem() {
                _ if self.unparsed.is_some() => self.error("Fix the draft first (e to edit)"),
                Some(problem) => self.error(format!("Can't send: {problem}")),
                None => {
                    let draft = Box::new(draft.clone());
                    self.request("Sending", Request::Submit { draft, send: true });
                }
            },
            KeyCode::Char('d') => {
                let draft = Box::new(draft.clone());
                self.request("Saving draft", Request::Submit { draft, send: false });
            }
            KeyCode::Char('e') | KeyCode::Enter => self.edit_draft = true,
            KeyCode::Char('a') => {
                let prompt = Prompt {
                    title: "Attach file".into(),
                    input: String::new(),
                    hint: "A path; ~ is your home".into(),
                };
                self.overlay = Some(Overlay::Prompt(prompt, PromptFor::Attach));
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.draft_scroll = self.draft_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.draft_scroll = self.draft_scroll.saturating_sub(1);
            }
            KeyCode::Char('x' | 'q') | KeyCode::Esc => {
                let confirm = Confirm {
                    question: "Discard this message?".into(),
                };
                self.overlay = Some(Overlay::Confirm(confirm, ConfirmFor::Discard));
            }
            _ => {}
        }
    }

    // ---- keys -----------------------------------------------------------

    fn on_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.quit = true;
            return;
        }
        if let Some(overlay) = self.overlay.take() {
            self.on_overlay_key(overlay, key);
            return;
        }
        if self.draft.is_some() {
            self.on_draft_key(key);
            return;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('?') => self.overlay = Some(Overlay::Help),
            KeyCode::Char('r') if ctrl => self.load_envelopes(),
            KeyCode::Char('c') => self.compose(ComposeKind::New),
            KeyCode::Char('A') => self.request("Listing accounts", Request::Accounts),
            KeyCode::Char('T') => {
                self.list.set_threaded(!self.list.threaded);
                self.info(if self.list.threaded {
                    "Thread view"
                } else {
                    "Date view"
                });
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Sidebar => Focus::List,
                    Focus::List => Focus::Reader,
                    Focus::Reader => Focus::Sidebar,
                }
            }
            KeyCode::BackTab => {
                self.focus = match self.focus {
                    Focus::Sidebar => Focus::Reader,
                    Focus::List => Focus::Sidebar,
                    Focus::Reader => Focus::List,
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.focus = match self.focus {
                    Focus::Reader => Focus::List,
                    _ => Focus::Sidebar,
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.focus = match self.focus {
                    Focus::Sidebar => Focus::List,
                    _ => Focus::Reader,
                }
            }
            KeyCode::Esc => {
                if self.reader.show_raw {
                    self.reader.show_raw = false;
                } else if self.focus == Focus::Reader {
                    self.focus = Focus::List;
                } else if !self.list.marked.is_empty() {
                    self.list.marked.clear();
                } else if self.search.is_some() {
                    self.search = None;
                    self.load_envelopes();
                }
            }
            KeyCode::Char('/') => {
                let prompt = Prompt {
                    title: "Search".into(),
                    input: self.search.clone().unwrap_or_default(),
                    hint: "from:ada  subject:engine  is:unread  has:attachment  date:2d".into(),
                };
                self.overlay = Some(Overlay::Prompt(prompt, PromptFor::Search));
            }
            _ => match self.focus {
                Focus::Sidebar => self.on_sidebar_key(key),
                Focus::List => self.on_list_key(key),
                Focus::Reader => self.on_reader_key(key),
            },
        }
    }

    fn on_overlay_key(&mut self, overlay: Overlay, key: KeyEvent) {
        match overlay {
            Overlay::Help => {}
            Overlay::Picker(mut picker, for_) => match key.code {
                KeyCode::Esc => {}
                KeyCode::Enter => {
                    if let Some(choice) = picker.chosen() {
                        self.picked(for_, choice);
                    }
                }
                KeyCode::Down | KeyCode::Tab => {
                    picker.move_by(1);
                    self.overlay = Some(Overlay::Picker(picker, for_));
                }
                KeyCode::Up | KeyCode::BackTab => {
                    picker.move_by(-1);
                    self.overlay = Some(Overlay::Picker(picker, for_));
                }
                KeyCode::Char('n' | 'j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    picker.move_by(1);
                    self.overlay = Some(Overlay::Picker(picker, for_));
                }
                KeyCode::Char('p' | 'k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    picker.move_by(-1);
                    self.overlay = Some(Overlay::Picker(picker, for_));
                }
                KeyCode::Backspace => {
                    picker.type_char(None);
                    self.overlay = Some(Overlay::Picker(picker, for_));
                }
                KeyCode::Char(c) => {
                    picker.type_char(Some(c));
                    self.overlay = Some(Overlay::Picker(picker, for_));
                }
                _ => self.overlay = Some(Overlay::Picker(picker, for_)),
            },
            Overlay::Prompt(mut prompt, for_) => match key.code {
                KeyCode::Esc => {}
                KeyCode::Enter => self.prompted(for_, prompt.input.trim().to_string()),
                KeyCode::Backspace => {
                    prompt.input.pop();
                    self.overlay = Some(Overlay::Prompt(prompt, for_));
                }
                KeyCode::Char(c) => {
                    prompt.input.push(c);
                    self.overlay = Some(Overlay::Prompt(prompt, for_));
                }
                _ => self.overlay = Some(Overlay::Prompt(prompt, for_)),
            },
            Overlay::Confirm(confirm, for_) => match key.code {
                KeyCode::Char('y' | 'Y') => self.confirmed(for_),
                KeyCode::Char('n' | 'q') | KeyCode::Esc => {}
                _ => self.overlay = Some(Overlay::Confirm(confirm, for_)),
            },
        }
    }

    fn picked(&mut self, for_: PickFor, choice: String) {
        match for_ {
            PickFor::MoveTo | PickFor::CopyTo => {
                let from = self.mailbox();
                if choice == from {
                    return;
                }
                let ids = self.list.targets();
                self.list.marked.clear();
                if matches!(for_, PickFor::MoveTo) {
                    self.list.remove(&ids);
                    self.reader.clear();
                    self.open_current();
                    self.request(
                        "Moving",
                        Request::Move {
                            from,
                            to: choice,
                            ids,
                        },
                    );
                } else {
                    self.request(
                        "Copying",
                        Request::Copy {
                            from,
                            to: choice,
                            ids,
                        },
                    );
                }
            }
            PickFor::Account => {
                if choice != self.sidebar.account {
                    self.switch_account(choice);
                }
            }
            PickFor::Link | PickFor::OpenFile => self.open_target(&choice),
        }
    }

    fn prompted(&mut self, for_: PromptFor, input: String) {
        match for_ {
            PromptFor::Search => {
                self.search = (!input.is_empty()).then_some(input);
                self.list.selected = 0;
                self.load_envelopes();
            }
            PromptFor::CreateMailbox if !input.is_empty() => {
                self.request("Creating mailbox", Request::CreateMailbox(input));
            }
            PromptFor::RenameMailbox(mailbox) if !input.is_empty() => self.request(
                "Renaming mailbox",
                Request::RenameMailbox {
                    mailbox,
                    name: input,
                },
            ),
            PromptFor::Attach if !input.is_empty() => {
                let path = match (input.strip_prefix("~/"), std::env::var_os("HOME")) {
                    (Some(rest), Some(home)) => std::path::Path::new(&home).join(rest),
                    _ => PathBuf::from(&input),
                };
                if !path.is_file() {
                    self.error(format!("Not a file: {input}"));
                } else if let Some(draft) = &mut self.draft {
                    draft.attach.push(path);
                }
            }
            _ => {}
        }
    }

    fn confirmed(&mut self, for_: ConfirmFor) {
        match for_ {
            ConfirmFor::DeleteForever(ids) => self.delete(ids),
            ConfirmFor::DeleteMailbox(mailbox) => {
                self.request("Deleting mailbox", Request::DeleteMailbox(mailbox));
            }
            ConfirmFor::Discard => {
                self.draft = None;
                self.unparsed = None;
                self.info("Discarded");
            }
        }
    }

    fn delete(&mut self, ids: Vec<String>) {
        let mailbox = self.mailbox();
        self.list.marked.clear();
        self.list.remove(&ids);
        self.reader.clear();
        self.open_current();
        self.request("Deleting", Request::Delete { mailbox, ids });
    }

    fn on_sidebar_key(&mut self, key: KeyEvent) {
        let before = self.sidebar.selected;
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.sidebar.move_by(1),
            KeyCode::Char('k') | KeyCode::Up => self.sidebar.move_by(-1),
            KeyCode::Char('g') | KeyCode::Home => self.sidebar.selected = 0,
            KeyCode::Char('G') | KeyCode::End => self.sidebar.move_by(isize::MAX / 2),
            KeyCode::Enter => self.focus = Focus::List,
            KeyCode::Char('n') => {
                let prompt = Prompt {
                    title: "New mailbox".into(),
                    input: String::new(),
                    hint: "Name of the mailbox to create".into(),
                };
                self.overlay = Some(Overlay::Prompt(prompt, PromptFor::CreateMailbox));
            }
            KeyCode::Char('R') => {
                let Some(m) = self.sidebar.current() else {
                    return;
                };
                let prompt = Prompt {
                    title: format!("Rename {}", m.name),
                    input: m.name.clone(),
                    hint: "New name".into(),
                };
                self.overlay = Some(Overlay::Prompt(
                    prompt,
                    PromptFor::RenameMailbox(m.id.clone()),
                ));
            }
            KeyCode::Char('D') => {
                let Some(m) = self.sidebar.current() else {
                    return;
                };
                let name = m.id.clone();
                let confirm = Confirm {
                    question: format!("Delete mailbox {} and everything in it?", m.name),
                };
                self.overlay = Some(Overlay::Confirm(confirm, ConfirmFor::DeleteMailbox(name)));
            }
            _ => {}
        }
        if self.sidebar.selected != before {
            self.list.envelopes.clear();
            self.list.marked.clear();
            self.list.selected = 0;
            self.list.offset = 0;
            self.list.next_page = None;
            self.search = None;
            self.reader.clear();
            self.load_envelopes();
        }
    }

    fn on_list_key(&mut self, key: KeyEvent) {
        let before = self.list.selected;
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.list.move_by(1),
            KeyCode::Char('k') | KeyCode::Up => self.list.move_by(-1),
            KeyCode::Char('g') | KeyCode::Home => self.list.selected = 0,
            KeyCode::Char('G') | KeyCode::End => self.list.move_by(isize::MAX / 2),
            KeyCode::PageDown => self.list.move_by(10),
            KeyCode::PageUp => self.list.move_by(-10),
            KeyCode::Char(' ' | 'x') => {
                self.list.toggle_mark();
                self.list.move_by(1);
            }
            KeyCode::Char('V') => self.list.mark_all(),
            KeyCode::Enter => {
                self.open_current();
                self.focus = Focus::Reader;
            }
            _ => self.on_message_key(key),
        }
        if self.list.selected != before {
            self.open_current();
        }
    }

    fn on_reader_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.reader.scroll_by(1),
            KeyCode::Char('k') | KeyCode::Up => self.reader.scroll_by(-1),
            KeyCode::Char(' ') | KeyCode::PageDown => self.reader.scroll_by(15),
            KeyCode::PageUp => self.reader.scroll_by(-15),
            KeyCode::Char('g') => self.reader.scroll = 0,
            KeyCode::Char('n') => {
                self.list.move_by(1);
                self.open_current();
            }
            KeyCode::Char('p') => {
                self.list.move_by(-1);
                self.open_current();
            }
            _ => self.on_message_key(key),
        }
    }

    /// Keys acting on the selected or marked messages, from the list or the reader.
    fn on_message_key(&mut self, key: KeyEvent) {
        let mailbox = self.mailbox();
        let Some(current) = self.list.current().map(|e| e.id.clone()) else {
            return;
        };
        let ids = self.list.targets();
        match key.code {
            KeyCode::Char('s' | 'f') => {
                let flag = if key.code == KeyCode::Char('s') {
                    "seen"
                } else {
                    "flagged"
                };
                // Several: set if any lacks it. One: toggle.
                let on = self
                    .list
                    .envelopes
                    .iter()
                    .filter(|e| ids.contains(&e.id))
                    .any(|e| !e.has(flag));
                for e in self
                    .list
                    .envelopes
                    .iter_mut()
                    .filter(|e| ids.contains(&e.id))
                {
                    e.set(flag, on);
                }
                self.request(
                    "Updating",
                    Request::Flag {
                        mailbox,
                        ids,
                        flag,
                        on,
                    },
                );
            }
            KeyCode::Char('d') => {
                if self.in_role("trash") {
                    let n = ids.len();
                    let confirm = Confirm {
                        question: format!(
                            "Delete {n} message{} for good?",
                            if n == 1 { "" } else { "s" }
                        ),
                    };
                    self.overlay = Some(Overlay::Confirm(confirm, ConfirmFor::DeleteForever(ids)));
                } else {
                    self.delete(ids);
                }
            }
            KeyCode::Char('a') => match self.sidebar.find("archive").map(|m| m.id.clone()) {
                None => self.error("No archive mailbox"),
                Some(archive) if archive == mailbox => {}
                Some(archive) => {
                    self.list.marked.clear();
                    self.list.remove(&ids);
                    self.reader.clear();
                    self.open_current();
                    self.request(
                        "Archiving",
                        Request::Move {
                            from: mailbox,
                            to: archive,
                            ids,
                        },
                    );
                }
            },
            KeyCode::Char('m' | 'y') => {
                let for_ = if key.code == KeyCode::Char('m') {
                    PickFor::MoveTo
                } else {
                    PickFor::CopyTo
                };
                let n = ids.len();
                let title = format!(
                    "{} {n} message{} to…",
                    if key.code == KeyCode::Char('m') {
                        "Move"
                    } else {
                        "Copy"
                    },
                    if n == 1 { "" } else { "s" }
                );
                let items = self
                    .sidebar
                    .mailboxes
                    .iter()
                    .filter(|m| m.id != mailbox)
                    .map(|m| (m.label().to_string(), m.id.clone()))
                    .collect();
                self.overlay = Some(Overlay::Picker(Picker::new(title, items), for_));
            }
            KeyCode::Char('r') => self.compose(ComposeKind::Reply {
                mailbox,
                id: current,
                all: false,
            }),
            KeyCode::Char('R') => self.compose(ComposeKind::Reply {
                mailbox,
                id: current,
                all: true,
            }),
            KeyCode::Char('F') => self.compose(ComposeKind::Forward {
                mailbox,
                id: current,
            }),
            KeyCode::Char('e') if self.in_role("drafts") => self.compose(ComposeKind::Draft {
                mailbox,
                id: current,
            }),
            KeyCode::Char('o') => {
                let dir = std::env::temp_dir().join("nitidus").join(&current);
                let _ = std::fs::create_dir_all(&dir);
                self.request(
                    "Fetching attachments",
                    Request::Download {
                        mailbox,
                        id: current,
                        dir,
                        open: true,
                    },
                );
            }
            KeyCode::Char('S') => {
                let dir = std::env::var_os("NITIDUS_DOWNLOADS")
                    .map(PathBuf::from)
                    .or_else(|| {
                        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Downloads"))
                    })
                    .unwrap_or_else(std::env::temp_dir);
                self.request(
                    "Saving attachments",
                    Request::Download {
                        mailbox,
                        id: current,
                        dir,
                        open: false,
                    },
                );
            }
            KeyCode::Char('L') => {
                let links = self.reader.links();
                if links.is_empty() {
                    self.error("No links in this message");
                } else {
                    let items = links.into_iter().map(|l| (l.clone(), l)).collect();
                    self.overlay = Some(Overlay::Picker(
                        Picker::new("Open link", items),
                        PickFor::Link,
                    ));
                }
            }
            KeyCode::Char('B') => match self.reader.message.as_ref().and_then(|m| m.html.clone()) {
                None => self.error("No HTML part"),
                Some(html) => {
                    let path = std::env::temp_dir().join(format!("nitidus-{current}.html"));
                    match std::fs::write(&path, html) {
                        Ok(()) => self.open_target(&path.display().to_string()),
                        Err(err) => self.error(format!("write {}: {err}", path.display())),
                    }
                }
            },
            KeyCode::Char('H') => {
                self.reader.show_raw = !self.reader.show_raw;
                self.reader.scroll = 0;
                if self.reader.show_raw && self.reader.raw.is_none() {
                    self.request(
                        "Reading source",
                        Request::Raw {
                            mailbox,
                            id: current,
                        },
                    );
                }
            }
            _ => {}
        }
        self.sidebar.unread = Some((self.list.unread(), self.list.envelopes.len()));
    }

    fn sync_focus(&mut self) {
        self.sidebar.focused = self.focus == Focus::Sidebar;
        self.list.focused = self.focus == Focus::List;
        self.reader.focused = self.focus == Focus::Reader;
    }

    // ---- drawing --------------------------------------------------------

    fn hints(&self) -> &'static [(&'static str, &'static str)] {
        if self.draft.is_some() {
            return &[
                ("s", "send"),
                ("e", "edit"),
                ("a", "attach"),
                ("d", "save draft"),
                ("x", "discard"),
            ];
        }
        match self.focus {
            Focus::Sidebar => &[
                ("j/k", "move"),
                ("enter", "open"),
                ("n", "new"),
                ("R", "rename"),
                ("D", "delete"),
                ("A", "accounts"),
                ("?", "help"),
            ],
            Focus::List if !self.list.marked.is_empty() => &[
                ("x", "mark"),
                ("V", "all"),
                ("s", "read"),
                ("f", "flag"),
                ("a", "archive"),
                ("m", "move"),
                ("d", "delete"),
                ("esc", "clear"),
            ],
            Focus::List => &[
                ("j/k", "move"),
                ("c", "compose"),
                ("r", "reply"),
                ("x", "mark"),
                ("a", "archive"),
                ("d", "delete"),
                ("m", "move"),
                ("/", "search"),
                ("?", "help"),
            ],
            Focus::Reader => &[
                ("j/k", "scroll"),
                ("n/p", "next/prev"),
                ("r", "reply"),
                ("F", "forward"),
                ("o", "open file"),
                ("L", "links"),
                ("H", "source"),
                ("?", "help"),
            ],
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let [top, main, bottom] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let title = match &self.draft {
            Some(_) => "Compose".to_string(),
            None => self.list.title.clone(),
        };
        frame.render_widget(
            TopBar {
                account: &self.sidebar.account,
                mailbox: &title,
                busy: self.pending.last().copied(),
                tick: self.tick,
            },
            top,
        );

        if let Some(draft) = &self.draft {
            frame.render_widget(
                Composer {
                    draft,
                    scroll: self.draft_scroll,
                },
                main,
            );
        } else if main.width >= 110 {
            let [side, list, reader] = Layout::horizontal([
                Constraint::Length(22),
                Constraint::Percentage(38),
                Constraint::Fill(1),
            ])
            .areas(main);
            frame.render_widget(&self.sidebar, side);
            frame.render_widget(&mut self.list, list);
            frame.render_widget(&mut self.reader, reader);
        } else {
            match self.focus {
                Focus::Sidebar => {
                    let [side, list] =
                        Layout::horizontal([Constraint::Length(22), Constraint::Fill(1)])
                            .areas(main);
                    frame.render_widget(&self.sidebar, side);
                    frame.render_widget(&mut self.list, list);
                }
                Focus::List => frame.render_widget(&mut self.list, main),
                Focus::Reader => frame.render_widget(&mut self.reader, main),
            }
        }

        let status = match &self.notice {
            Some((Notice::Info(m), _)) => Status::Info(m),
            Some((Notice::Error(m), _)) => Status::Error(m),
            None => Status::Hints(self.hints()),
        };
        frame.render_widget(status, bottom);

        match &self.overlay {
            None => {}
            Some(Overlay::Help) => frame.render_widget(Help, main),
            Some(Overlay::Picker(p, _)) => frame.render_widget(p, main),
            Some(Overlay::Prompt(p, _)) => frame.render_widget(p, main),
            Some(Overlay::Confirm(c, _)) => frame.render_widget(c, main),
        }
    }
}
