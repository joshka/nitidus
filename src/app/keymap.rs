//! Which key does what, as tables. Key dispatch, the status bar's hints and the help sheet all
//! read these tables, so a binding changes in one place.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::action::{
    Action, ComposeAction, DraftAction, FocusMove, General, ListAction, MailboxAction, ReadAction,
    Triage,
};
use crate::ui::help::HelpSection;
use crate::ui::movement::Movement;

/// A key, as bindings name it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    /// The key code; Shift is part of a letter's case.
    code: KeyCode,

    /// Whether Control is held.
    ctrl: bool,
}

/// A character key.
const fn ch(c: char) -> Key {
    Key {
        code: KeyCode::Char(c),
        ctrl: false,
    }
}

/// A character key with Control.
const fn ctrl(c: char) -> Key {
    Key {
        code: KeyCode::Char(c),
        ctrl: true,
    }
}

/// A named key.
const fn named(code: KeyCode) -> Key {
    Key { code, ctrl: false }
}

/// Keys bound to an action.
pub struct Binding {
    /// The keys, the first being the one hints and help show.
    keys: &'static [Key],

    /// The action.
    action: Action,
}

/// Binds `keys` to `action`.
const fn bind(keys: &'static [Key], action: Action) -> Binding {
    Binding { keys, action }
}

/// Where keys are read, which decides the tables consulted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context {
    /// The sidebar has focus.
    Sidebar,

    /// The envelope list has focus.
    List,

    /// The envelope list has focus with messages marked; keys are the list's, hints differ.
    Marked,

    /// The reader has focus.
    Reader,

    /// A draft is under review.
    Draft,
}

/// Keys that work in every pane.
const GLOBAL: &[Binding] = &[
    bind(&[ch('q')], Action::General(General::Quit)),
    bind(&[ch('?')], Action::General(General::Help)),
    bind(&[ctrl('r')], Action::General(General::Refresh)),
    bind(&[ch('/')], Action::General(General::Search)),
    bind(&[ch('A')], Action::General(General::Accounts)),
    bind(&[ch('T')], Action::General(General::ToggleThreads)),
    bind(&[named(KeyCode::Esc)], Action::General(General::Back)),
    bind(&[ch('c')], Action::Compose(ComposeAction::New)),
];

/// Moving selections and focus, in every pane.
const NAVIGATION: &[Binding] = &[
    bind(
        &[ch('j'), named(KeyCode::Down)],
        Action::Move(Movement::Down),
    ),
    bind(&[ch('k'), named(KeyCode::Up)], Action::Move(Movement::Up)),
    bind(
        &[ch('g'), named(KeyCode::Home)],
        Action::Move(Movement::Top),
    ),
    bind(
        &[ch('G'), named(KeyCode::End)],
        Action::Move(Movement::Bottom),
    ),
    bind(
        &[named(KeyCode::PageDown)],
        Action::Move(Movement::PageDown),
    ),
    bind(&[named(KeyCode::PageUp)], Action::Move(Movement::PageUp)),
    bind(&[named(KeyCode::Tab)], Action::Focus(FocusMove::Next)),
    bind(
        &[named(KeyCode::BackTab)],
        Action::Focus(FocusMove::Previous),
    ),
    bind(
        &[ch('h'), named(KeyCode::Left)],
        Action::Focus(FocusMove::Left),
    ),
    bind(
        &[ch('l'), named(KeyCode::Right)],
        Action::Focus(FocusMove::Right),
    ),
];

/// Keys of the sidebar.
const SIDEBAR: &[Binding] = &[
    bind(&[named(KeyCode::Enter)], Action::Focus(FocusMove::Right)),
    bind(&[ch('n')], Action::Mailbox(MailboxAction::Create)),
    bind(&[ch('R')], Action::Mailbox(MailboxAction::Rename)),
    bind(&[ch('D')], Action::Mailbox(MailboxAction::Delete)),
];

/// Keys of the envelope list.
const LIST: &[Binding] = &[
    bind(&[named(KeyCode::Enter)], Action::List(ListAction::Open)),
    bind(&[ch('x'), ch(' ')], Action::List(ListAction::ToggleMark)),
    bind(&[ch('V')], Action::List(ListAction::MarkAll)),
];

/// Keys of the reader.
const READER: &[Binding] = &[
    bind(&[ch(' ')], Action::Move(Movement::PageDown)),
    bind(&[ch('n')], Action::List(ListAction::NextMessage)),
    bind(&[ch('p')], Action::List(ListAction::PreviousMessage)),
];

/// Keys acting on messages, from the list or the reader.
const MESSAGE: &[Binding] = &[
    bind(&[ch('s')], Action::Triage(Triage::ToggleSeen)),
    bind(&[ch('f')], Action::Triage(Triage::ToggleFlag)),
    bind(&[ch('a')], Action::Triage(Triage::Archive)),
    bind(&[ch('d')], Action::Triage(Triage::Delete)),
    bind(&[ch('m')], Action::Triage(Triage::MoveTo)),
    bind(&[ch('y')], Action::Triage(Triage::CopyTo)),
    bind(&[ch('r')], Action::Compose(ComposeAction::Reply)),
    bind(&[ch('R')], Action::Compose(ComposeAction::ReplyAll)),
    bind(&[ch('F')], Action::Compose(ComposeAction::Forward)),
    bind(&[ch('e')], Action::Compose(ComposeAction::EditDraft)),
    bind(&[ch('o')], Action::Read(ReadAction::OpenAttachments)),
    bind(&[ch('S')], Action::Read(ReadAction::SaveAttachments)),
    bind(&[ch('L')], Action::Read(ReadAction::Links)),
    bind(&[ch('B')], Action::Read(ReadAction::Browser)),
    bind(&[ch('H')], Action::Read(ReadAction::Source)),
];

/// Keys of the draft review screen; nothing else applies there.
const DRAFT: &[Binding] = &[
    bind(&[ch('s'), ch('y')], Action::Draft(DraftAction::Send)),
    bind(&[ch('d')], Action::Draft(DraftAction::Save)),
    bind(
        &[ch('e'), named(KeyCode::Enter)],
        Action::Draft(DraftAction::Edit),
    ),
    bind(&[ch('a')], Action::Draft(DraftAction::Attach)),
    bind(
        &[ch('x'), ch('q'), named(KeyCode::Esc)],
        Action::Draft(DraftAction::Discard),
    ),
    bind(
        &[ch('j'), named(KeyCode::Down)],
        Action::Move(Movement::Down),
    ),
    bind(&[ch('k'), named(KeyCode::Up)], Action::Move(Movement::Up)),
];

/// A status-bar hint: the actions whose keys it shows, and what it says.
type Hint = (&'static [Action], &'static str);

/// Hints in the sidebar.
const SIDEBAR_HINTS: &[Hint] = &[
    (
        &[Action::Move(Movement::Down), Action::Move(Movement::Up)],
        "move",
    ),
    (&[Action::Focus(FocusMove::Right)], "open"),
    (&[Action::Mailbox(MailboxAction::Create)], "new"),
    (&[Action::Mailbox(MailboxAction::Rename)], "rename"),
    (&[Action::Mailbox(MailboxAction::Delete)], "delete"),
    (&[Action::General(General::Accounts)], "accounts"),
    (&[Action::General(General::Help)], "help"),
];

/// Hints in the list.
const LIST_HINTS: &[Hint] = &[
    (
        &[Action::Move(Movement::Down), Action::Move(Movement::Up)],
        "move",
    ),
    (&[Action::Compose(ComposeAction::New)], "compose"),
    (&[Action::Compose(ComposeAction::Reply)], "reply"),
    (&[Action::List(ListAction::ToggleMark)], "mark"),
    (&[Action::Triage(Triage::Archive)], "archive"),
    (&[Action::Triage(Triage::Delete)], "delete"),
    (&[Action::Triage(Triage::MoveTo)], "move"),
    (&[Action::General(General::Search)], "search"),
    (&[Action::General(General::Help)], "help"),
];

/// Hints in the list while messages are marked.
const MARKED_HINTS: &[Hint] = &[
    (&[Action::List(ListAction::ToggleMark)], "mark"),
    (&[Action::List(ListAction::MarkAll)], "all"),
    (&[Action::Triage(Triage::ToggleSeen)], "read"),
    (&[Action::Triage(Triage::ToggleFlag)], "flag"),
    (&[Action::Triage(Triage::Archive)], "archive"),
    (&[Action::Triage(Triage::MoveTo)], "move"),
    (&[Action::Triage(Triage::Delete)], "delete"),
    (&[Action::General(General::Back)], "clear"),
];

/// Hints in the reader.
const READER_HINTS: &[Hint] = &[
    (
        &[Action::Move(Movement::Down), Action::Move(Movement::Up)],
        "scroll",
    ),
    (
        &[
            Action::List(ListAction::NextMessage),
            Action::List(ListAction::PreviousMessage),
        ],
        "next/prev",
    ),
    (&[Action::Compose(ComposeAction::Reply)], "reply"),
    (&[Action::Compose(ComposeAction::Forward)], "forward"),
    (&[Action::Read(ReadAction::OpenAttachments)], "open file"),
    (&[Action::Read(ReadAction::Links)], "links"),
    (&[Action::Read(ReadAction::Source)], "source"),
    (&[Action::General(General::Help)], "help"),
];

/// Hints on the draft review screen.
const DRAFT_HINTS: &[Hint] = &[
    (&[Action::Draft(DraftAction::Send)], "send"),
    (&[Action::Draft(DraftAction::Edit)], "edit"),
    (&[Action::Draft(DraftAction::Attach)], "attach"),
    (&[Action::Draft(DraftAction::Save)], "save draft"),
    (&[Action::Draft(DraftAction::Discard)], "discard"),
];

/// One help line: the context its keys are read in, the actions, and what they do.
type HelpLine = (Context, &'static [Action], &'static str);

/// The help sheet: each section's title and lines.
const HELP: &[(&str, &[HelpLine])] = &[
    (
        "Move around",
        &[
            (
                Context::List,
                &[Action::Move(Movement::Down), Action::Move(Movement::Up)],
                "next / previous",
            ),
            (
                Context::List,
                &[Action::Move(Movement::Top), Action::Move(Movement::Bottom)],
                "first / last",
            ),
            (
                Context::List,
                &[
                    Action::Focus(FocusMove::Next),
                    Action::Focus(FocusMove::Left),
                    Action::Focus(FocusMove::Right),
                ],
                "switch pane",
            ),
            (Context::List, &[Action::List(ListAction::Open)], "open"),
            (
                Context::List,
                &[Action::General(General::Back)],
                "back / clear",
            ),
            (
                Context::Reader,
                &[
                    Action::List(ListAction::NextMessage),
                    Action::List(ListAction::PreviousMessage),
                ],
                "next / prev (reader)",
            ),
        ],
    ),
    (
        "Messages",
        &[
            (
                Context::List,
                &[
                    Action::Triage(Triage::ToggleSeen),
                    Action::Triage(Triage::ToggleFlag),
                ],
                "toggle read / flag",
            ),
            (
                Context::List,
                &[
                    Action::Triage(Triage::Archive),
                    Action::Triage(Triage::Delete),
                ],
                "archive / delete",
            ),
            (
                Context::List,
                &[
                    Action::Triage(Triage::MoveTo),
                    Action::Triage(Triage::CopyTo),
                ],
                "move / copy to…",
            ),
            (
                Context::List,
                &[
                    Action::List(ListAction::ToggleMark),
                    Action::List(ListAction::MarkAll),
                ],
                "mark / mark all",
            ),
            (
                Context::List,
                &[Action::General(General::ToggleThreads)],
                "thread view",
            ),
        ],
    ),
    (
        "Write",
        &[
            (
                Context::List,
                &[Action::Compose(ComposeAction::New)],
                "compose",
            ),
            (
                Context::List,
                &[
                    Action::Compose(ComposeAction::Reply),
                    Action::Compose(ComposeAction::ReplyAll),
                ],
                "reply / reply all",
            ),
            (
                Context::List,
                &[Action::Compose(ComposeAction::Forward)],
                "forward",
            ),
            (
                Context::List,
                &[Action::Compose(ComposeAction::EditDraft)],
                "edit draft",
            ),
        ],
    ),
    (
        "Read",
        &[
            (
                Context::List,
                &[
                    Action::Read(ReadAction::OpenAttachments),
                    Action::Read(ReadAction::SaveAttachments),
                ],
                "open / save attachments",
            ),
            (
                Context::List,
                &[
                    Action::Read(ReadAction::Links),
                    Action::Read(ReadAction::Browser),
                ],
                "links / open in browser",
            ),
            (
                Context::List,
                &[Action::Read(ReadAction::Source)],
                "message source",
            ),
        ],
    ),
    (
        "Mailboxes",
        &[
            (Context::List, &[Action::General(General::Search)], "search"),
            (
                Context::List,
                &[Action::General(General::Refresh)],
                "refresh",
            ),
            (
                Context::Sidebar,
                &[
                    Action::Mailbox(MailboxAction::Create),
                    Action::Mailbox(MailboxAction::Rename),
                    Action::Mailbox(MailboxAction::Delete),
                ],
                "new / rename / delete",
            ),
            (
                Context::List,
                &[Action::General(General::Accounts)],
                "switch account",
            ),
            (Context::List, &[Action::General(General::Quit)], "quit"),
        ],
    ),
];

impl Context {
    /// The tables consulted in this context, first match winning.
    fn tables(self) -> &'static [&'static [Binding]] {
        match self {
            Self::Sidebar => &[SIDEBAR, NAVIGATION, GLOBAL],
            Self::List | Self::Marked => &[LIST, MESSAGE, NAVIGATION, GLOBAL],
            Self::Reader => &[READER, MESSAGE, NAVIGATION, GLOBAL],
            Self::Draft => &[DRAFT],
        }
    }

    /// The action `event` stands for here.
    pub fn action(self, event: KeyEvent) -> Option<Action> {
        let pressed = Key::of(event);
        let mut bindings = self.tables().iter().flat_map(|table| table.iter());
        bindings
            .find(|binding| binding.keys.contains(&pressed))
            .map(|binding| binding.action)
    }

    /// The status-bar hints here.
    pub fn hints(self) -> Vec<(String, &'static str)> {
        let hints = match self {
            Self::Sidebar => SIDEBAR_HINTS,
            Self::List => LIST_HINTS,
            Self::Marked => MARKED_HINTS,
            Self::Reader => READER_HINTS,
            Self::Draft => DRAFT_HINTS,
        };
        hints
            .iter()
            .map(|&(actions, text)| (self.keys_for(actions, "/"), text))
            .collect()
    }

    /// The first key of each of `actions` here, joined by `separator`.
    fn keys_for(self, actions: &[Action], separator: &str) -> String {
        let label = |action: &Action| self.first_key(*action).map_or_default(Key::label);
        actions
            .iter()
            .map(label)
            .collect::<Vec<_>>()
            .join(separator)
    }

    /// The first key bound to `action` here.
    fn first_key(self, action: Action) -> Option<Key> {
        let mut bindings = self.tables().iter().flat_map(|table| table.iter());
        bindings
            .find(|binding| binding.action == action)
            .and_then(|binding| binding.keys.first().copied())
    }
}

/// The help sheet, keys read from the bindings.
pub fn help() -> Vec<HelpSection> {
    let line = |&(context, actions, text): &(Context, &[Action], &'static str)| {
        (context.keys_for(actions, " "), text)
    };
    HELP.iter()
        .map(|&(title, lines)| HelpSection {
            title,
            lines: lines.iter().map(line).collect(),
        })
        .collect()
}

impl Key {
    /// The key `event` names; Shift is ignored, as it is already in a letter's case.
    fn of(event: KeyEvent) -> Self {
        Self {
            code: event.code,
            ctrl: event.modifiers.contains(KeyModifiers::CONTROL),
        }
    }

    /// The key as hints and help write it.
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "other keys are named by crossterm's own display"
    )]
    fn label(self) -> String {
        let name = match self.code {
            KeyCode::Char(' ') => "space".to_owned(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "enter".to_owned(),
            KeyCode::Esc => "esc".to_owned(),
            KeyCode::Tab => "tab".to_owned(),
            KeyCode::BackTab => "shift-tab".to_owned(),
            other => format!("{other}").to_lowercase(),
        };
        if self.ctrl {
            format!("ctrl-{name}")
        } else {
            name
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pane_key_wins_over_a_shared_one() {
        let n = KeyEvent::from(KeyCode::Char('n'));

        assert_eq!(
            Context::Sidebar.action(n),
            Some(Action::Mailbox(MailboxAction::Create))
        );
        assert_eq!(
            Context::Reader.action(n),
            Some(Action::List(ListAction::NextMessage))
        );
        assert_eq!(
            Context::Draft.action(KeyEvent::from(KeyCode::Char('q'))),
            Some(Action::Draft(DraftAction::Discard))
        );
    }

    #[test]
    fn hints_show_the_first_key_of_each_action() {
        let hints = Context::List.hints();

        assert_eq!(hints[0], ("j/k".to_owned(), "move"));
    }
}
