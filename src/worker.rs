//! A thread owning the mail session, so the UI never blocks on the backend.

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::mail::{ComposeKind, Draft, EnvelopePage, MailClient, Mailbox, Message, Receipt};

pub enum Request {
    Accounts,
    Mailboxes,
    Envelopes {
        mailbox: String,
        page: usize,
        query: Option<String>,
        quiet: bool,
    },
    Read {
        mailbox: String,
        id: String,
    },
    Raw {
        mailbox: String,
        id: String,
    },
    Flag {
        mailbox: String,
        ids: Vec<String>,
        flag: &'static str,
        on: bool,
    },
    Move {
        from: String,
        to: String,
        ids: Vec<String>,
    },
    Copy {
        from: String,
        to: String,
        ids: Vec<String>,
    },
    Delete {
        mailbox: String,
        ids: Vec<String>,
    },
    CreateMailbox(String),
    RenameMailbox {
        mailbox: String,
        name: String,
    },
    DeleteMailbox(String),
    Template(ComposeKind),
    Submit {
        draft: Box<Draft>,
        send: bool,
    },
    Download {
        mailbox: String,
        id: String,
        dir: PathBuf,
        open: bool,
    },
}

pub enum Response {
    Accounts(Vec<String>),
    Mailboxes(Vec<Mailbox>),
    Envelopes {
        mailbox: String,
        page: usize,
        query: Option<String>,
        quiet: bool,
        result: EnvelopePage,
    },
    Read(Box<Message>),
    Raw {
        id: String,
        text: String,
    },
    /// A write on messages finished; `what` names it for the status bar.
    Wrote {
        what: String,
        receipt: Receipt,
    },
    MailboxesChanged(String),
    Template(Box<Draft>),
    Submitted {
        sent: bool,
        receipt: Receipt,
    },
    Downloaded {
        paths: Vec<String>,
        open: bool,
    },
    Failed {
        action: &'static str,
        error: String,
    },
}

pub const PAGE_SIZE: usize = 50;

pub struct Worker {
    tx: mpsc::Sender<Request>,
    pub rx: mpsc::Receiver<Response>,
}

impl Worker {
    pub fn spawn(config: Vec<PathBuf>, account: Option<String>) -> Self {
        let (tx, requests) = mpsc::channel::<Request>();
        let (responses, rx) = mpsc::channel();
        thread::spawn(move || {
            // Built here: the session need not be Send.
            let mut client = MailClient::new(config, account);
            loop {
                let request = match requests.recv_timeout(Duration::from_secs(60)) {
                    Ok(request) => request,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        let _ = client.keep_alive();
                        continue;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                };
                if responses.send(handle(&mut client, request)).is_err() {
                    return;
                }
            }
        });
        Self { tx, rx }
    }

    pub fn send(&self, request: Request) {
        let _ = self.tx.send(request);
    }
}

fn plural(n: usize, what: &str) -> String {
    if n == 1 {
        format!("1 {what}")
    } else {
        format!("{n} {what}s")
    }
}

fn handle(c: &mut MailClient, request: Request) -> Response {
    let fail = |action: &'static str| {
        move |err: anyhow::Error| Response::Failed {
            action,
            error: format!("{err:#}"),
        }
    };
    let wrote = |what: String| move |receipt| Response::Wrote { what, receipt };
    match request {
        Request::Accounts => c
            .accounts()
            .map_or_else(fail("list accounts"), Response::Accounts),
        Request::Mailboxes => c
            .list_mailboxes()
            .map_or_else(fail("list mailboxes"), Response::Mailboxes),
        Request::Envelopes {
            mailbox,
            page,
            query,
            quiet,
        } => c
            .list_envelopes(&mailbox, page, PAGE_SIZE, query.as_deref())
            .map_or_else(fail("list envelopes"), |result| Response::Envelopes {
                mailbox,
                page,
                query,
                quiet,
                result,
            }),
        Request::Read { mailbox, id } => c
            .read_message(&mailbox, &id)
            .map_or_else(fail("read message"), |m| Response::Read(Box::new(m))),
        Request::Raw { mailbox, id } => {
            c.read_raw(&mailbox, &id)
                .map_or_else(fail("read source"), |raw| Response::Raw {
                    id,
                    text: String::from_utf8_lossy(&raw).replace("\r\n", "\n"),
                })
        }
        Request::Flag {
            mailbox,
            ids,
            flag,
            on,
        } => {
            let what = format!("{} {flag}", if on { "Marked" } else { "Unmarked" });
            c.set_flag(&mailbox, &ids, flag, on)
                .map_or_else(fail("update flag"), wrote(what))
        }
        Request::Move { from, to, ids } => {
            let what = format!("Moved {} to {to}", plural(ids.len(), "message"));
            c.move_messages(&from, &to, &ids)
                .map_or_else(fail("move"), wrote(what))
        }
        Request::Copy { from, to, ids } => {
            let what = format!("Copied {} to {to}", plural(ids.len(), "message"));
            c.copy_messages(&from, &to, &ids)
                .map_or_else(fail("copy"), wrote(what))
        }
        Request::Delete { mailbox, ids } => {
            let n = ids.len();
            c.delete_messages(&mailbox, &ids)
                .map_or_else(fail("delete"), |receipt| {
                    let verb = match receipt.action.as_deref() {
                        Some("moved-to-trash") => "Moved to trash:",
                        _ => "Deleted",
                    };
                    Response::Wrote {
                        what: format!("{verb} {}", plural(n, "message")),
                        receipt,
                    }
                })
        }
        Request::CreateMailbox(name) => c
            .create_mailbox(&name)
            .map_or_else(fail("create mailbox"), |()| {
                Response::MailboxesChanged(format!("Created {name}"))
            }),
        Request::RenameMailbox { mailbox, name } => c
            .rename_mailbox(&mailbox, &name)
            .map_or_else(fail("rename mailbox"), |()| {
                Response::MailboxesChanged(format!("Renamed to {name}"))
            }),
        Request::DeleteMailbox(mailbox) => c
            .delete_mailbox(&mailbox)
            .map_or_else(fail("delete mailbox"), |()| {
                Response::MailboxesChanged(format!("Deleted mailbox {mailbox}"))
            }),
        Request::Template(kind) => c.template(&kind).map_or_else(fail("prepare message"), |t| {
            Response::Template(Box::new(Draft::from_template(kind, t)))
        }),
        Request::Submit { draft, send } => c.submit(&draft, send).map_or_else(
            fail(if send { "send" } else { "save draft" }),
            |receipt| Response::Submitted {
                sent: send,
                receipt,
            },
        ),
        Request::Download {
            mailbox,
            id,
            dir,
            open,
        } => c.download_attachments(&mailbox, &id, &dir).map_or_else(
            fail("download attachments"),
            |paths| Response::Downloaded { paths, open },
        ),
    }
}
