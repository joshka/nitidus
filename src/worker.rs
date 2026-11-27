//! A thread owning the mail session, so the screen never waits on the backend.
//!
//! The app sends [`Request`]s and folds the [`Response`]s back in as they arrive; the session lives
//! on the worker thread alone, so it needn't be `Send`.

mod request;
mod response;

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

pub use self::request::{
    AfterDownload, Download, FlagUpdate, Listing, ListingPurpose, MailboxRename, Request,
};
pub use self::response::{Failure, Response, Write};
use crate::mail::MailClient;

/// How long the worker waits for a request before keeping the connection alive.
const KEEP_ALIVE_AFTER: Duration = Duration::from_secs(60);

/// The app's end of the worker thread.
pub struct Worker {
    /// Requests to the worker.
    requests: Sender<Request>,

    /// Responses from the worker, in the order the requests were sent.
    responses: Receiver<Response>,
}

/// Which account the worker's session opens.
pub struct AccountSpec {
    /// Himalaya config files; Himalaya's default when empty.
    pub config: Vec<PathBuf>,

    /// The account name; the default account when `None`.
    pub account: Option<String>,
}

impl Worker {
    /// Starts a worker thread with a session over `spec`.
    pub fn spawn(spec: AccountSpec) -> Self {
        let (requests, inbox) = mpsc::channel();
        let (outbox, responses) = mpsc::channel();
        thread::spawn(move || serve(MailClient::new(spec.config, spec.account), &inbox, &outbox));
        Self {
            requests,
            responses,
        }
    }

    /// Queues a request; one sent after the worker stopped is dropped.
    pub fn send(&self, request: Request) {
        let _stopped = self.requests.send(request);
    }

    /// The next response, if one has arrived.
    pub fn try_recv(&self) -> Option<Response> {
        self.responses.try_recv().ok()
    }
}

/// Answers requests until the app hangs up, keeping the connection alive when idle.
fn serve(mut client: MailClient, inbox: &Receiver<Request>, outbox: &Sender<Response>) {
    loop {
        let carry_on = match inbox.recv_timeout(KEEP_ALIVE_AFTER) {
            Ok(request) => outbox.send(request.run(&mut client)).is_ok(),
            Err(RecvTimeoutError::Timeout) => keep_alive(&mut client),
            Err(RecvTimeoutError::Disconnected) => false,
        };
        if !carry_on {
            return;
        }
    }
}

/// Keeps an idle connection alive; a failure only means the next command reconnects, so the
/// worker carries on either way.
fn keep_alive(client: &mut MailClient) -> bool {
    let _reconnects_next_time = client.keep_alive();
    true
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::demo::{Demo, Store};

    /// The first response within a few seconds.
    fn next(worker: &Worker) -> Option<Response> {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if let Some(response) = worker.try_recv() {
                return Some(response);
            }
            thread::sleep(Duration::from_millis(10));
        }
        None
    }

    #[test]
    fn a_request_is_answered_on_the_worker_thread() {
        let demo = Demo::create(None, Store::Maildir).expect("the demo");
        let worker = Worker::spawn(demo.account());

        worker.send(Request::Accounts);

        assert!(
            matches!(next(&worker), Some(Response::Accounts(names)) if names == ["demo", "work"])
        );
    }

    #[test]
    fn a_failure_keeps_every_cause() {
        let worker = Worker::spawn(AccountSpec {
            config: vec!["/nonexistent/config.toml".into()],
            account: None,
        });

        worker.send(Request::Mailboxes);

        let Some(Response::Failed(failure)) = next(&worker) else {
            panic!("a failure")
        };
        assert_eq!(failure.action, "list mailboxes");
        assert!(
            failure.error.contains("/nonexistent/config.toml"),
            "{}",
            failure.error
        );
    }
}
