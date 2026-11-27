//! A fake mail setup, so nitidus can be tried and recorded without logging
//! in anywhere: two Maildir accounts and a local SMTP server that accepts
//! everything (mail to the demo address lands back in its inbox).

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{Duration, Local};

struct Fake {
    account: &'static str,
    folder: &'static str,
    /// A message id local to the demo, and the one this replies to.
    mid: Option<&'static str>,
    reply_to: Option<&'static str>,
    from: (&'static str, &'static str),
    subject: &'static str,
    hours_ago: i64,
    flags: &'static str,
    body: &'static str,
    html: bool,
    attachment: Option<&'static str>,
}

const ME: (&str, &str) = ("Rae Field", "rae@nitidus.test");

macro_rules! fake {
    (
        $folder:expr, $name:expr, $email:expr, $subject:expr, $hours:expr, $flags:expr, $body:expr
    ) => {
        Fake {
            account: "demo",
            folder: $folder,
            mid: None,
            reply_to: None,
            from: ($name, $email),
            subject: $subject,
            hours_ago: $hours,
            flags: $flags,
            body: $body,
            html: false,
            attachment: None,
        }
    };
}

fn messages() -> Vec<Fake> {
    let mut v = vec![
        fake!(
            "",
            "Ada Lovelace",
            "ada@analytical.test",
            "Notes on the engine's loop construct",
            1,
            "",
            "Hi Rae,\n\nI've been sketching how the engine might repeat a sequence of operations\nwithout re-punching the cards. Attached are my notes - would love your eyes on\nsection G before Thursday.\n\nThe short version: a loop is just a jump backwards with a counter, but the\ncounter has to live somewhere the mill can see it.\n\n> Do we need a new card type for this?\n\nI don't think so. The variable cards can carry it.\n\nAda"
        ),
        fake!(
            "",
            "GitHub",
            "noreply@github.test",
            "[ratatui/ratatui] Release v0.30.2",
            3,
            "S",
            "ratatui v0.30.2 has been released.\n\n## Fixes\n\n- fix(table): column spacing when highlight symbol is wider than one cell\n- fix(buffer): diff no longer emits redundant cursor moves\n\n## Docs\n\n- Many small doc improvements\n\nView it on GitHub: https://github.com/ratatui/ratatui/releases"
        ),
        fake!(
            "",
            "Grace Hopper",
            "grace@cobol.test",
            "Re: Compiler meeting moved to 3pm",
            5,
            "",
            "Works for me. I'll bring the nanoseconds.\n\n-- \nGrace"
        ),
        fake!(
            "",
            "Linus Field",
            "linus@field.test",
            "Weekend hike?",
            9,
            "F",
            "Thinking of doing the ridge trail on Saturday if the weather holds.\nForecast says clear until about 4pm. You in?\n\nBring the good snacks this time."
        ),
        fake!(
            "",
            "Ferris",
            "ferris@crab.test",
            "Your crate was mentioned in This Week in Rust",
            26,
            "S",
            "Congrats! Your crate `nitidus` was featured in This Week in Rust #742.\n\nKeep on crabbing 🦀"
        ),
        fake!(
            "",
            "Margaret Hamilton",
            "margaret@apollo.test",
            "Priority display review",
            30,
            "FS",
            "Rae,\n\nThe priority displays worked exactly as designed during the sim. When the\nexecutive overflowed, it shed the low-priority jobs and kept the landing\nguidance running. That's the behaviour we want to document.\n\nCan you write up the restart path? Keep it short; the astronauts need to\nread it in under a minute.\n\nMargaret"
        ),
        fake!(
            "",
            "Calendar",
            "calendar@nitidus.test",
            "Invitation: Design review @ Fri 10:00",
            33,
            "S",
            "You have been invited to: Design review\nWhen: Friday 10:00 - 11:00\nWhere: Room 4 / video\n\nAgenda:\n  1. Sidebar density\n  2. Reader typography\n  3. Keybinding audit"
        ),
        Fake {
            mid: Some("ptr-1"),
            ..fake!(
                "",
                "Brian Kernighan",
                "bwk@bell.test",
                "Why is the pointer syntax like that?",
                70,
                "S",
                "Every new C programmer asks me why `int *p` and not `int* p`.\nDennis, care to settle it once and for all?"
            )
        },
        Fake {
            mid: Some("ptr-2"),
            reply_to: Some("ptr-1"),
            ..fake!(
                "",
                "Ken Thompson",
                "ken@bell.test",
                "Re: Why is the pointer syntax like that?",
                60,
                "S",
                "> Dennis, care to settle it once and for all?\n\nHe will. He always does."
            )
        },
        Fake {
            mid: Some("ptr-3"),
            reply_to: Some("ptr-2"),
            ..fake!(
                "",
                "Dennis Ritchie",
                "dmr@bell.test",
                "Re: Re: Why is the pointer syntax like that?",
                50,
                "S",
                "Declaration mirrors use. If you read the declaration the way you'd use the\nvariable in an expression, it falls out.\n\n> > Why not put the * with the type?\n> Because then int* a, b; lies to you.\n\nExactly."
            )
        },
        fake!(
            "",
            "Barbara Liskov",
            "barbara@clu.test",
            "Substitutability, one more time",
            75,
            "",
            "A subtype should be usable anywhere its supertype is, without surprising the\ncaller. If your Square breaks Rectangle's setWidth contract, it isn't a\nsubtype, whatever the class hierarchy says."
        ),
        fake!(
            "",
            "Ken Thompson",
            "ken@bell.test",
            "Trusting trust",
            100,
            "S",
            "You can't trust code that you did not totally create yourself.\nEspecially code from companies that employ people like me."
        ),
        fake!(
            "",
            "Hedy Lamarr",
            "hedy@spread.test",
            "Frequency hopping patent draft",
            130,
            "S",
            "Draft attached. The piano-roll synchronisation section needs a diagram."
        ),
        fake!(
            "",
            "Alan Kay",
            "alan@parc.test",
            "The best way to predict the future",
            160,
            "S",
            "...is to invent it. Slides from Tuesday attached-ish (I forgot to attach them)."
        ),
        fake!(
            "",
            "Joan Clarke",
            "joan@hut8.test",
            "Crossword results",
            200,
            "S",
            "You finished in under 12 minutes. We'd like to talk to you about a job."
        ),
        fake!(
            "",
            "Radia Perlman",
            "radia@stp.test",
            "An Algorhyme",
            260,
            "S",
            "I think that I shall never see\nA graph more lovely than a tree.\nA tree whose crucial property\nIs loop-free connectivity."
        ),
        fake!(
            "Archive",
            "Bank of Examples",
            "statements@bank.test",
            "Your October statement is ready",
            700,
            "S",
            "Your statement for October is available. Balance: $1,024.00"
        ),
        fake!(
            "Archive",
            "Ada Lovelace",
            "ada@analytical.test",
            "Bernoulli numbers, note G",
            900,
            "S",
            "Here's the full table for note G. Row 4 had an error; fixed now."
        ),
        fake!(
            "Sent",
            ME.0,
            ME.1,
            "Re: Compiler meeting moved to 3pm",
            6,
            "S",
            "Sounds good, see you at 3."
        ),
        fake!(
            "Drafts",
            ME.0,
            ME.1,
            "Restart path write-up (draft)",
            20,
            "DS",
            "1. Executive overflow detected\n2. Shed low-priority jobs\n3. ..."
        ),
        fake!(
            "Trash",
            "Prize Committee",
            "winner@totally-legit.test",
            "You have WON!!!",
            48,
            "S",
            "Click here to claim."
        ),
    ];
    // A second account to switch to.
    for (name, email, subject, hours, flags, body) in [
        (
            "Build Bot",
            "ci@work.test",
            "Nightly build passed",
            2,
            "",
            "All 1,204 tests passed on main.",
        ),
        (
            "Priya Shah",
            "priya@work.test",
            "1:1 notes",
            20,
            "S",
            "Notes from today are in the doc. Next week: roadmap.",
        ),
        (
            "IT Desk",
            "it@work.test",
            "Laptop refresh",
            90,
            "S",
            "Your new laptop is ready for pickup.",
        ),
    ] {
        v.push(Fake {
            account: "work",
            ..fake!("", name, email, subject, hours, flags, body)
        });
    }
    v.push(Fake { html: true, ..fake!("", "Ratatui Weekly", "news@ratatui.test", "Ratatui Weekly #88: less boxes, more backgrounds", 14, "",
        "<h1>Ratatui Weekly #88</h1><p>This week: why <b>background colours</b> beat box-drawing borders for dense UIs.</p><ul><li>Panels by tint, not by lines</li><li>Components that render themselves</li><li>Testing TUIs with betamax</li></ul><p><a href=\"https://ratatui.rs\">Read more on ratatui.rs</a></p>") });
    v.push(Fake {
        attachment: Some("engine-notes.pdf"),
        ..fake!(
            "",
            "Charles Babbage",
            "charles@difference.test",
            "Difference engine no. 2 - drawings",
            20,
            "",
            "Rae,\n\nAttached are the revised drawings. The carry mechanism changed again.\n\nCB"
        )
    });
    v
}

/// Older mail, so the inbox spans several pages.
fn filler() -> Vec<Fake> {
    const SENDERS: [(&str, &str, &str); 6] = [
        ("Rust Blog", "blog@rust.test", "Announcing Rust 1.{n}"),
        (
            "crates.io",
            "noreply@crates.test",
            "Crate published: tui-widget-{n}",
        ),
        ("Hacker Letter", "digest@hn.test", "Top stories, issue {n}"),
        (
            "Calendar",
            "calendar@nitidus.test",
            "Reminder: standup #{n}",
        ),
        (
            "GitHub",
            "noreply@github.test",
            "[nitidus] Issue #{n} opened",
        ),
        (
            "Bookshop",
            "orders@books.test",
            "Your order #{n} has shipped",
        ),
    ];
    (0..140)
        .map(|n| {
            let (name, email, subject) = SENDERS[n % SENDERS.len()];
            let subject: &'static str = Box::leak(
                subject
                    .replace("{n}", &(n + 40).to_string())
                    .into_boxed_str(),
            );
            let flags = if n % 17 == 3 { "FS" } else { "S" };
            let hours = 24 * 9 + (n as i64) * 31;
            fake!(
                "",
                name,
                email,
                subject,
                hours,
                flags,
                "An older message, here so the mailbox spans more than one page."
            )
        })
        .collect()
}

/// Writes the fake accounts under `dir` and starts the SMTP sink,
/// returning the config path.
pub fn create(dir: &Path, pimdir: bool) -> Result<PathBuf> {
    let roots = [("demo", dir.join("mail")), ("work", dir.join("work"))];
    for (_, root) in &roots {
        for folder in ["", "Archive", "Drafts", "Sent", "Trash"] {
            for sub in ["cur", "new", "tmp"] {
                fs::create_dir_all(root.join(folder).join(sub))?;
            }
        }
    }
    let now = Local::now();
    for (i, m) in messages().into_iter().chain(filler()).enumerate() {
        let root = if m.account == "work" {
            &roots[1].1
        } else {
            &roots[0].1
        };
        let date =
            (now - Duration::hours(m.hours_ago) - Duration::minutes(i as i64 * 7)).to_rfc2822();
        let me = if m.account == "work" {
            ("Rae Field", "rae@work.test")
        } else {
            ME
        };
        let (to, from) = if m.folder == "Sent" || m.folder == "Drafts" {
            (("Grace Hopper", "grace@cobol.test"), m.from)
        } else {
            (me, m.from)
        };
        let mid = m.mid.map_or_else(
            || format!("<demo-{i}@nitidus.test>"),
            |m| format!("<{m}@nitidus.test>"),
        );
        let mut raw = String::new();
        write!(
            raw,
            "From: {} <{}>\r\nTo: {} <{}>\r\nSubject: {}\r\nDate: {date}\r\nMessage-ID: {mid}\r\nMIME-Version: 1.0\r\n",
            from.0, from.1, to.0, to.1, m.subject
        )?;
        if let Some(parent) = m.reply_to {
            write!(
                raw,
                "In-Reply-To: <{parent}@nitidus.test>\r\nReferences: <{parent}@nitidus.test>\r\n"
            )?;
        }
        let body = m.body.replace('\n', "\r\n");
        let ctype = if m.html { "text/html" } else { "text/plain" };
        match m.attachment {
            None => write!(
                raw,
                "Content-Type: {ctype}; charset=utf-8\r\n\r\n{body}\r\n"
            )?,
            Some(name) => write!(
                raw,
                "Content-Type: multipart/mixed; boundary=\"b\"\r\n\r\n--b\r\nContent-Type: {ctype}; charset=utf-8\r\n\r\n{body}\r\n--b\r\nContent-Type: application/pdf; name=\"{name}\"\r\nContent-Disposition: attachment; filename=\"{name}\"\r\nContent-Transfer-Encoding: base64\r\n\r\nJVBERi0xLjQKJcOkw7zDtsOfCg==\r\n--b--\r\n"
            )?,
        }
        let file = root.join(m.folder).join("cur").join(format!(
            "{}.{i}.nitidus:2,{}",
            1_700_000_000 + i,
            m.flags
        ));
        fs::write(file, raw)?;
    }
    if pimdir {
        return create_pimdir(dir, &roots[0].1);
    }
    let port = smtp_sink(roots[0].1.join("new"))?;
    let config = dir.join("config.toml");
    let mut toml = String::new();
    for (name, root) in &roots {
        let (email, display) = if *name == "demo" {
            (ME.1, ME.0)
        } else {
            ("rae@work.test", "Rae Field")
        };
        let default = if *name == "demo" {
            "default = true\n"
        } else {
            ""
        };
        write!(
            toml,
            "[accounts.{name}]\n{default}email = \"{email}\"\ndisplay-name = \"{display}\"\nsignature = \"Rae\"\n\
             maildir.root = {:?}\nsmtp.server = \"smtp://127.0.0.1:{port}\"\n\
             mailbox.alias.sent = \"Sent\"\nmailbox.alias.drafts = \"Drafts\"\nmailbox.alias.trash = \"Trash\"\n\
             message.send.save-copy = true\n\n",
            root.display().to_string(),
        )?;
    }
    fs::write(&config, toml)?;
    Ok(config)
}

/// A minimal SMTP server on localhost accepting every message. Mail to the
/// demo address is delivered to `inbox_new`, so sending to yourself shows
/// up on the next refresh.
fn smtp_sink(inbox_new: PathBuf) -> Result<u16> {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    std::thread::spawn(move || {
        for (n, stream) in listener.incoming().flatten().enumerate() {
            let inbox_new = inbox_new.clone();
            std::thread::spawn(move || -> std::io::Result<()> {
                let mut out = stream.try_clone()?;
                let mut lines = BufReader::new(stream);
                out.write_all(b"220 nitidus.test ESMTP demo sink\r\n")?;
                let (mut to_me, mut line) = (false, String::new());
                loop {
                    line.clear();
                    if lines.read_line(&mut line)? == 0 {
                        return Ok(());
                    }
                    let cmd = line.trim_end().to_ascii_uppercase();
                    if cmd.starts_with("EHLO") {
                        out.write_all(b"250-nitidus.test\r\n250 8BITMIME\r\n")?;
                    } else if cmd.starts_with("RCPT") {
                        to_me |= cmd.contains(&ME.1.to_ascii_uppercase());
                        out.write_all(b"250 OK\r\n")?;
                    } else if cmd == "DATA" {
                        out.write_all(b"354 go ahead\r\n")?;
                        let mut data = String::new();
                        loop {
                            line.clear();
                            if lines.read_line(&mut line)? == 0 || line == ".\r\n" || line == ".\n"
                            {
                                break;
                            }
                            // Undo dot-stuffing.
                            data.push_str(if line.starts_with("..") {
                                &line[1..]
                            } else {
                                &line
                            });
                        }
                        if to_me {
                            let name =
                                format!("{}.{n}.sink", chrono::Utc::now().timestamp_micros());
                            let _ = fs::write(inbox_new.join(name), data);
                        }
                        to_me = false;
                        out.write_all(b"250 queued\r\n")?;
                    } else if cmd == "QUIT" {
                        out.write_all(b"221 bye\r\n")?;
                        return Ok(());
                    } else {
                        out.write_all(b"250 OK\r\n")?;
                    }
                }
            });
        }
    });
    Ok(port)
}

/// A pimdir store seeded from the demo Maildir, as a sync engine's `imap`
/// source would fill it, with whole bodies so messages can be read.
///
/// Reads come from the store; writes queue for a sync engine, which the
/// demo does not run, so they stay queued.
fn create_pimdir(dir: &Path, maildir: &Path) -> Result<PathBuf> {
    use io_pimdir::change::PimdirWriteOp;
    use io_pimdir::client::PimdirStore;
    use io_pimdir::collection::PimdirCollectionId;
    use io_pimdir::placement::{
        PimdirBase, PimdirFlags, PimdirHandle, PimdirLevel, PimdirPlacement, PimdirStatus,
    };
    use io_pimdir::summary::mail;
    let root = dir.join("pimdir");
    fs::create_dir_all(&root)?;
    let mut store = PimdirStore::open(&root)?.for_source("imap")?;
    let stage = store.blob_stage();
    for (folder, name) in [
        ("", "INBOX"),
        ("Archive", "Archive"),
        ("Drafts", "Drafts"),
        ("Sent", "Sent"),
        ("Trash", "Trash"),
    ] {
        let collection = format!("imap/{name}");
        store.ensure_collection(&collection, "message/rfc822")?;
        let mut ops = vec![];
        for entry in fs::read_dir(maildir.join(folder).join("cur"))? {
            let path = entry?.path();
            let body = fs::read(&path)?;
            let file = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let letters = file
                .rsplit_once(":2,")
                .map(|(_, f)| f.to_string())
                .unwrap_or_default();
            let flags = PimdirFlags::from_iter(letters.chars().filter_map(|c| match c {
                'S' => Some("\\Seen"),
                'F' => Some("\\Flagged"),
                'R' => Some("\\Answered"),
                'D' => Some("\\Draft"),
                _ => None,
            }));
            let object = stage.stage_body(&body)?;
            let hash = object.hash.clone();
            ops.push(PimdirWriteOp::StoreObject { object, body: None });
            let derived = mail::derive(&body);
            ops.push(PimdirWriteOp::UpsertPlacement(PimdirPlacement {
                collection: PimdirCollectionId(collection.clone()),
                handle: PimdirHandle(file),
                link_id: Some(derived.link_id),
                object: Some(hash),
                level: PimdirLevel::Full,
                summary: derived.summary,
                sort_key: derived.sort_key,
                server_object_id: None,
                flags: flags.clone(),
                status: PimdirStatus::Clean,
                conflict_revision: None,
                conflict_object: None,
                base: Some(PimdirBase {
                    flags,
                    revision: None,
                    object: None,
                }),
                origin: None,
            }));
        }
        store.write(ops)?;
    }
    let config = dir.join("config.toml");
    fs::write(
        &config,
        format!(
            "[accounts.demo]\ndefault = true\nemail = \"{}\"\ndisplay-name = \"{}\"\npimdir.root = {:?}\n\
             mailbox.alias.inbox = \"imap/INBOX\"\nmailbox.alias.sent = \"imap/Sent\"\n\
             mailbox.alias.drafts = \"imap/Drafts\"\nmailbox.alias.trash = \"imap/Trash\"\n",
            ME.1,
            ME.0,
            root.display().to_string()
        ),
    )?;
    Ok(config)
}
