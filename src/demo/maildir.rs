//! Writing the demo's messages as Maildir files.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local};

use super::ME;
use super::messages::{Account, FILLER_BODY, FILLER_COUNT, FILLER_SENDERS, Fake, MESSAGES};

/// The folders every account has; `""` is the inbox.
const FOLDERS: [&str; 5] = ["", "Archive", "Drafts", "Sent", "Trash"];

/// The work account's address.
const WORK: (&str, &str) = ("Rae Field", "rae@work.test");

/// Who the demo's own sent mail and drafts are addressed to.
const CORRESPONDENT: (&str, &str) = ("Grace Hopper", "grace@cobol.test");

/// Minutes between consecutive messages written, so none share a date.
const MINUTES_APART: i64 = 7;

/// A tiny PDF, base64, attached where a message names an attachment.
const PDF: &str = "JVBERi0xLjQKJcOkw7zDtsOfCg==";

/// The Maildir roots of the demo's accounts.
pub struct Accounts {
    /// The personal account's root.
    personal: PathBuf,

    /// The work account's root.
    work: PathBuf,
}

impl Accounts {
    /// The personal account's root, which the SMTP sink delivers to.
    pub fn personal(&self) -> &Path {
        &self.personal
    }

    /// Each account's config name and root.
    pub fn named(&self) -> [(&'static str, &Path); 2] {
        [("demo", &self.personal), ("work", &self.work)]
    }

    /// The root of `account`.
    fn root(&self, account: Account) -> &Path {
        match account {
            Account::Personal => &self.personal,
            Account::Work => &self.work,
        }
    }
}

/// A message ready to write: a [`Fake`], its subject (which filler messages number), and its
/// position, which spaces dates and names files.
struct Letter<'a> {
    /// The message.
    fake: &'a Fake,

    /// The subject to write.
    subject: String,

    /// Its position among all the messages written.
    index: usize,
}

/// Writes every demo message into Maildir folders under `dir`.
pub fn write_accounts(dir: &Path) -> Result<Accounts> {
    let accounts = Accounts {
        personal: dir.join("mail"),
        work: dir.join("work"),
    };
    for (_, root) in accounts.named() {
        create_folders(root)?;
    }
    let filler = filler();
    let fakes = MESSAGES.iter().chain(&filler);
    let now = Local::now();
    for (index, fake) in fakes.enumerate() {
        let letter = Letter {
            fake,
            subject: subject(fake, index),
            index,
        };
        write_letter(accounts.root(fake.account), &letter, now)?;
    }
    Ok(accounts)
}

/// The inbox and its sibling folders, each with `cur`, `new` and `tmp`.
fn create_folders(root: &Path) -> Result<()> {
    for folder in FOLDERS {
        for sub in ["cur", "new", "tmp"] {
            let path = root.join(folder).join(sub);
            fs::create_dir_all(&path).with_context(|| format!("create {}", path.display()))?;
        }
    }
    Ok(())
}

/// Older read mail spread over the last few months, about one a day.
fn filler() -> Vec<Fake> {
    let older = |n: usize| {
        let (name, email, _) = FILLER_SENDERS[n % FILLER_SENDERS.len()];
        let flags = if n % 17 == 3 { "FS" } else { "S" };
        let hours_ago = 24 * 9 + 31 * i64::try_from(n).unwrap_or_default();
        Fake {
            from: (name, email),
            hours_ago,
            flags,
            body: FILLER_BODY,
            ..Fake::DEFAULT
        }
    };
    (0..FILLER_COUNT).map(older).collect()
}

/// The subject to write: the message's own, or a filler subject numbered by `index`.
fn subject(fake: &Fake, index: usize) -> String {
    if !fake.subject.is_empty() {
        return fake.subject.to_owned();
    }
    let filler = index - MESSAGES.len();
    let (_, _, pattern) = FILLER_SENDERS[filler % FILLER_SENDERS.len()];
    pattern.replace("{n}", &(filler + 40).to_string())
}

/// Writes one message into its folder's `cur`, named with its flags.
fn write_letter(root: &Path, letter: &Letter<'_>, now: DateTime<Local>) -> Result<()> {
    let name = format!(
        "{}.{}.nitidus:2,{}",
        1_700_000_000 + letter.index,
        letter.index,
        letter.fake.flags
    );
    let path = root.join(letter.fake.folder).join("cur").join(name);
    fs::write(&path, render(letter, now)).with_context(|| format!("write {}", path.display()))
}

/// The RFC 5322 source of a letter.
fn render(letter: &Letter<'_>, now: DateTime<Local>) -> String {
    let mut raw = headers(letter, now);
    raw.push_str(&body(letter.fake));
    raw
}

/// The header block, without the content headers.
fn headers(letter: &Letter<'_>, now: DateTime<Local>) -> String {
    let fake = letter.fake;
    let to = recipient(fake);
    let mut lines = vec![
        format!("From: {} <{}>", fake.from.0, fake.from.1),
        format!("To: {} <{}>", to.0, to.1),
        format!("Subject: {}", letter.subject),
        format!("Date: {}", sent_at(letter, now).to_rfc2822()),
        format!("Message-ID: <{}@nitidus.test>", message_id(letter)),
        "MIME-Version: 1.0".to_owned(),
    ];
    if let Some(parent) = fake.reply_to {
        lines.push(format!("In-Reply-To: <{parent}@nitidus.test>"));
        lines.push(format!("References: <{parent}@nitidus.test>"));
    }
    format!("{}\r\n", lines.join("\r\n"))
}

/// When a letter was sent: its age, spaced a few minutes from its neighbours.
fn sent_at(letter: &Letter<'_>, now: DateTime<Local>) -> DateTime<Local> {
    let index = i64::try_from(letter.index).unwrap_or_default();
    now - Duration::hours(letter.fake.hours_ago) - Duration::minutes(index * MINUTES_APART)
}

/// The local part of a letter's `Message-ID`: its own, else one from its position.
fn message_id(letter: &Letter<'_>) -> String {
    letter
        .fake
        .id
        .map_or_else(|| format!("demo-{}", letter.index), str::to_owned)
}

/// Who a message is addressed to: the account holder, or a correspondent for what they wrote.
fn recipient(fake: &Fake) -> (&'static str, &'static str) {
    match (fake.folder, fake.account) {
        ("Sent" | "Drafts", _) => CORRESPONDENT,
        (_, Account::Work) => WORK,
        (_, Account::Personal) => ME,
    }
}

/// The content headers and body, multipart when the message has an attachment.
fn body(fake: &Fake) -> String {
    let text = fake.body.replace('\n', "\r\n");
    let content_type = if fake.html { "text/html" } else { "text/plain" };
    let part = format!("Content-Type: {content_type}; charset=utf-8\r\n\r\n{text}\r\n");
    match fake.attachment {
        None => part,
        Some(name) => format!(
            "Content-Type: multipart/mixed; boundary=\"b\"\r\n\r\n--b\r\n{part}--b\r\n\
             Content-Type: application/pdf; name=\"{name}\"\r\n\
             Content-Disposition: attachment; filename=\"{name}\"\r\n\
             Content-Transfer-Encoding: base64\r\n\r\n{PDF}\r\n--b--\r\n"
        ),
    }
}
