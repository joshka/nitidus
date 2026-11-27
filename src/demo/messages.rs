//! The demo's mail, as data: who sent what, when, and with which flags.

/// One fake message.
pub struct Fake {
    /// Which account holds it.
    pub account: Account,

    /// The Maildir folder holding it, `""` for the inbox.
    pub folder: &'static str,

    /// A `Message-ID` local part, so replies can point at it.
    pub id: Option<&'static str>,

    /// The local part of the `Message-ID` this replies to.
    pub reply_to: Option<&'static str>,

    /// The sender's name and address.
    pub from: (&'static str, &'static str),

    /// The subject.
    pub subject: &'static str,

    /// How long ago it was sent.
    pub hours_ago: i64,

    /// Maildir flag letters, sorted (`S` seen, `F` flagged, `D` draft).
    pub flags: &'static str,

    /// Whether the body is HTML rather than plain text.
    pub html: bool,

    /// The name of a PDF attached to it.
    pub attachment: Option<&'static str>,

    /// The body.
    pub body: &'static str,
}

/// The demo's accounts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Account {
    /// The default account, `demo`.
    Personal,

    /// A second account to switch to, `work`.
    Work,
}

impl Fake {
    /// An unread plain-text inbox message in the personal account.
    pub const DEFAULT: Self = Self {
        account: Account::Personal,
        folder: "",
        id: None,
        reply_to: None,
        from: ("", ""),
        subject: "",
        hours_ago: 0,
        flags: "",
        html: false,
        attachment: None,
        body: "",
    };
}

/// The sender of mail the demo account wrote itself.
const ME: (&str, &str) = super::ME;

/// The demo's messages, in the order they are written.
pub const MESSAGES: &[Fake] = &[
    Fake {
        from: ("Ada Lovelace", "ada@analytical.test"),
        subject: "Notes on the engine's loop construct",
        hours_ago: 1,
        body: "Hi Rae,\n\nI've been sketching how the engine might repeat a sequence of operations\nwithout re-punching the cards. Attached are my notes - would love your eyes on\nsection G before Thursday.\n\nThe short version: a loop is just a jump backwards with a counter, but the\ncounter has to live somewhere the mill can see it.\n\n> Do we need a new card type for this?\n\nI don't think so. The variable cards can carry it.\n\nAda",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("GitHub", "noreply@github.test"),
        subject: "[ratatui/ratatui] Release v0.30.2",
        hours_ago: 3,
        flags: "S",
        body: "ratatui v0.30.2 has been released.\n\n## Fixes\n\n- fix(table): column spacing when highlight symbol is wider than one cell\n- fix(buffer): diff no longer emits redundant cursor moves\n\n## Docs\n\n- Many small doc improvements\n\nView it on GitHub: https://github.com/ratatui/ratatui/releases",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Grace Hopper", "grace@cobol.test"),
        subject: "Re: Compiler meeting moved to 3pm",
        hours_ago: 5,
        body: "Works for me. I'll bring the nanoseconds.\n\n-- \nGrace",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Linus Field", "linus@field.test"),
        subject: "Weekend hike?",
        hours_ago: 9,
        flags: "F",
        body: "Thinking of doing the ridge trail on Saturday if the weather holds.\nForecast says clear until about 4pm. You in?\n\nBring the good snacks this time.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Ferris", "ferris@crab.test"),
        subject: "Your crate was mentioned in This Week in Rust",
        hours_ago: 26,
        flags: "S",
        body: "Congrats! Your crate `nitidus` was featured in This Week in Rust #742.\n\nKeep on crabbing 🦀",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Margaret Hamilton", "margaret@apollo.test"),
        subject: "Priority display review",
        hours_ago: 30,
        flags: "FS",
        body: "Rae,\n\nThe priority displays worked exactly as designed during the sim. When the\nexecutive overflowed, it shed the low-priority jobs and kept the landing\nguidance running. That's the behaviour we want to document.\n\nCan you write up the restart path? Keep it short; the astronauts need to\nread it in under a minute.\n\nMargaret",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Calendar", "calendar@nitidus.test"),
        subject: "Invitation: Design review @ Fri 10:00",
        hours_ago: 33,
        flags: "S",
        body: "You have been invited to: Design review\nWhen: Friday 10:00 - 11:00\nWhere: Room 4 / video\n\nAgenda:\n  1. Sidebar density\n  2. Reader typography\n  3. Keybinding audit",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Brian Kernighan", "bwk@bell.test"),
        subject: "Why is the pointer syntax like that?",
        hours_ago: 70,
        flags: "S",
        id: Some("ptr-1"),
        body: "Every new C programmer asks me why `int *p` and not `int* p`.\nDennis, care to settle it once and for all?",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Ken Thompson", "ken@bell.test"),
        subject: "Re: Why is the pointer syntax like that?",
        hours_ago: 60,
        flags: "S",
        id: Some("ptr-2"),
        reply_to: Some("ptr-1"),
        body: "> Dennis, care to settle it once and for all?\n\nHe will. He always does.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Dennis Ritchie", "dmr@bell.test"),
        subject: "Re: Re: Why is the pointer syntax like that?",
        hours_ago: 50,
        flags: "S",
        id: Some("ptr-3"),
        reply_to: Some("ptr-2"),
        body: "Declaration mirrors use. If you read the declaration the way you'd use the\nvariable in an expression, it falls out.\n\n> > Why not put the * with the type?\n> Because then int* a, b; lies to you.\n\nExactly.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Barbara Liskov", "barbara@clu.test"),
        subject: "Substitutability, one more time",
        hours_ago: 75,
        body: "A subtype should be usable anywhere its supertype is, without surprising the\ncaller. If your Square breaks Rectangle's setWidth contract, it isn't a\nsubtype, whatever the class hierarchy says.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Ken Thompson", "ken@bell.test"),
        subject: "Trusting trust",
        hours_ago: 100,
        flags: "S",
        body: "You can't trust code that you did not totally create yourself.\nEspecially code from companies that employ people like me.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Hedy Lamarr", "hedy@spread.test"),
        subject: "Frequency hopping patent draft",
        hours_ago: 130,
        flags: "S",
        body: "Draft attached. The piano-roll synchronisation section needs a diagram.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Alan Kay", "alan@parc.test"),
        subject: "The best way to predict the future",
        hours_ago: 160,
        flags: "S",
        body: "...is to invent it. Slides from Tuesday attached-ish (I forgot to attach them).",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Joan Clarke", "joan@hut8.test"),
        subject: "Crossword results",
        hours_ago: 200,
        flags: "S",
        body: "You finished in under 12 minutes. We'd like to talk to you about a job.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Radia Perlman", "radia@stp.test"),
        subject: "An Algorhyme",
        hours_ago: 260,
        flags: "S",
        body: "I think that I shall never see\nA graph more lovely than a tree.\nA tree whose crucial property\nIs loop-free connectivity.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Bank of Examples", "statements@bank.test"),
        subject: "Your October statement is ready",
        hours_ago: 700,
        flags: "S",
        folder: "Archive",
        body: "Your statement for October is available. Balance: $1,024.00",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Ada Lovelace", "ada@analytical.test"),
        subject: "Bernoulli numbers, note G",
        hours_ago: 900,
        flags: "S",
        folder: "Archive",
        body: "Here's the full table for note G. Row 4 had an error; fixed now.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ME,
        subject: "Re: Compiler meeting moved to 3pm",
        hours_ago: 6,
        flags: "S",
        folder: "Sent",
        body: "Sounds good, see you at 3.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ME,
        subject: "Restart path write-up (draft)",
        hours_ago: 20,
        flags: "DS",
        folder: "Drafts",
        body: "1. Executive overflow detected\n2. Shed low-priority jobs\n3. ...",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Prize Committee", "winner@totally-legit.test"),
        subject: "You have WON!!!",
        hours_ago: 48,
        flags: "S",
        folder: "Trash",
        body: "Click here to claim.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Build Bot", "ci@work.test"),
        subject: "Nightly build passed",
        hours_ago: 2,
        account: Account::Work,
        body: "All 1,204 tests passed on main.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Priya Shah", "priya@work.test"),
        subject: "1:1 notes",
        hours_ago: 20,
        flags: "S",
        account: Account::Work,
        body: "Notes from today are in the doc. Next week: roadmap.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("IT Desk", "it@work.test"),
        subject: "Laptop refresh",
        hours_ago: 90,
        flags: "S",
        account: Account::Work,
        body: "Your new laptop is ready for pickup.",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Ratatui Weekly", "news@ratatui.test"),
        subject: "Ratatui Weekly #88: less boxes, more backgrounds",
        hours_ago: 14,
        html: true,
        body: "<h1>Ratatui Weekly #88</h1><p>This week: why <b>background colours</b> beat box-drawing borders for dense UIs.</p><ul><li>Panels by tint, not by lines</li><li>Components that render themselves</li><li>Testing TUIs with betamax</li></ul><p><a href=\"https://ratatui.rs\">Read more on ratatui.rs</a></p>",
        ..Fake::DEFAULT
    },
    Fake {
        from: ("Charles Babbage", "charles@difference.test"),
        subject: "Difference engine no. 2 - drawings",
        hours_ago: 20,
        attachment: Some("engine-notes.pdf"),
        body: "Rae,\n\nAttached are the revised drawings. The carry mechanism changed again.\n\nCB",
        ..Fake::DEFAULT
    },
];

/// Senders of the older mail that makes the inbox span several pages, and their subjects; `{n}`
/// is the message's number.
pub const FILLER_SENDERS: [(&str, &str, &str); 6] = [
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

/// How many filler messages there are.
pub const FILLER_COUNT: usize = 140;

/// The body of every filler message.
pub const FILLER_BODY: &str = "An older message, here so the mailbox spans more than one page.";
