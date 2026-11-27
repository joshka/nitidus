//! A minimal SMTP server on localhost that accepts every message.
//!
//! Mail to the demo address is delivered to the inbox's `new/`, so sending to yourself shows up on
//! the next refresh. It speaks just enough of RFC 5321 for Himalaya's client: no TLS, no auth.

use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::{fs, thread};

use anyhow::{Context, Result};

use super::ME;

/// Starts the server on a free localhost port, returning the port.
pub fn start(inbox_new: PathBuf) -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").context("bind the SMTP sink")?;
    let port = listener
        .local_addr()
        .context("read the SMTP sink's port")?
        .port();
    thread::spawn(move || accept(&listener, &inbox_new));
    Ok(port)
}

/// Serves each connection on its own thread.
fn accept(listener: &TcpListener, inbox_new: &Path) {
    for stream in listener.incoming().flatten() {
        let inbox_new = inbox_new.to_owned();
        thread::spawn(move || Session::new(stream, inbox_new).and_then(Session::serve));
    }
}

/// One SMTP conversation.
struct Session {
    /// Where replies go.
    out: TcpStream,

    /// Where commands come from, line by line.
    lines: BufReader<TcpStream>,

    /// Where mail for the demo address is delivered.
    inbox_new: PathBuf,

    /// Whether the message being sent has the demo address among its recipients.
    to_me: bool,
}

impl Session {
    /// A session over `stream`.
    fn new(stream: TcpStream, inbox_new: PathBuf) -> io::Result<Self> {
        let out = stream.try_clone()?;
        Ok(Self {
            out,
            lines: BufReader::new(stream),
            inbox_new,
            to_me: false,
        })
    }

    /// Greets the client and answers commands until it quits or hangs up.
    fn serve(mut self) -> io::Result<()> {
        self.reply("220 nitidus.test ESMTP demo sink")?;
        while let Some(command) = self.read_line()? {
            if !self.answer(&command.trim_end().to_ascii_uppercase())? {
                break;
            }
        }
        Ok(())
    }

    /// Answers one command; `false` once the client quit.
    fn answer(&mut self, command: &str) -> io::Result<bool> {
        match command {
            "QUIT" => self.reply("221 bye")?,
            "DATA" => self.receive()?,
            _ if command.starts_with("EHLO") => self.reply("250-nitidus.test\r\n250 8BITMIME")?,
            _ if command.starts_with("RCPT") => self.recipient(command)?,
            _ => self.reply("250 OK")?,
        }
        Ok(command != "QUIT")
    }

    /// Notes whether a recipient is the demo address.
    fn recipient(&mut self, command: &str) -> io::Result<()> {
        self.to_me |= command.contains(&ME.1.to_ascii_uppercase());
        self.reply("250 OK")
    }

    /// Reads a message up to the lone `.`, delivering it when it is for the demo address.
    fn receive(&mut self) -> io::Result<()> {
        self.reply("354 go ahead")?;
        let mut message = String::new();
        while let Some(line) = self.read_line()? {
            if line == ".\r\n" || line == ".\n" {
                break;
            }
            message.push_str(
                line.strip_prefix('.')
                    .filter(|_| line.starts_with(".."))
                    .unwrap_or(&line),
            );
        }
        if std::mem::take(&mut self.to_me) {
            self.deliver(&message)?;
        }
        self.reply("250 queued")
    }

    /// Writes a message into the inbox's `new/` under a unique name.
    fn deliver(&self, message: &str) -> io::Result<()> {
        let name = format!("{}.sink", chrono::Utc::now().timestamp_micros());
        fs::write(self.inbox_new.join(name), message)
    }

    /// The next line, `None` once the client hung up.
    fn read_line(&mut self) -> io::Result<Option<String>> {
        let mut line = String::new();
        let read = self.lines.read_line(&mut line)?;
        Ok((read > 0).then_some(line))
    }

    /// Sends one reply line.
    fn reply(&mut self, text: &str) -> io::Result<()> {
        write!(self.out, "{text}\r\n")
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read as _;

    use super::*;

    /// Sends one message to `recipient` through the sink, answering the server's whole transcript.
    fn send(port: u16, recipient: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("a connection");
        let script = format!(
            "EHLO test\r\nMAIL FROM:<a@x.test>\r\nRCPT TO:<{recipient}>\r\nDATA\r\nSubject: hi\r\n\r\n..dotted\r\n.\r\nQUIT\r\n"
        );
        stream.write_all(script.as_bytes()).expect("the script");
        let mut transcript = String::new();
        stream.read_to_string(&mut transcript).expect("the replies");
        transcript
    }

    #[test]
    fn mail_for_the_demo_address_is_delivered_with_dots_unstuffed() {
        let inbox = tempfile::tempdir().expect("an inbox");
        let port = start(inbox.path().to_owned()).expect("the sink");

        let transcript = send(port, ME.1);

        assert!(
            transcript.ends_with("250 queued\r\n221 bye\r\n"),
            "{transcript}"
        );
        let delivered: Vec<String> = fs::read_dir(inbox.path())
            .expect("the inbox")
            .map(|entry| fs::read_to_string(entry.expect("an entry").path()).expect("a message"))
            .collect();
        assert_eq!(delivered, ["Subject: hi\r\n\r\n.dotted\r\n"]);
    }

    #[test]
    fn mail_for_anyone_else_is_accepted_and_dropped() {
        let inbox = tempfile::tempdir().expect("an inbox");
        let port = start(inbox.path().to_owned()).expect("the sink");

        let transcript = send(port, "someone@x.test");

        assert!(transcript.contains("250 queued"), "{transcript}");
        assert_eq!(fs::read_dir(inbox.path()).expect("the inbox").count(), 0);
    }
}
