# A captioned tour of what nitidus does, on the demo account.
Output target/screens/tour.webm
Output target/screens/tour.mp4

Set Shell "bash"
Set Width 1500
Set Height 860
Set FontSize 15
Set Padding 0
Set Theme "TokyoNight"
Set KeyboardOverlay Keys
Set TypingSpeed 50ms
Set Framerate 24

Hide
Type "clear && export EDITOR=\"vim -Nu NONE\" NITIDUS_DOWNLOADS=target/screens/dl && mkdir -p target/screens/dl && ./target/debug/nitidus demo"
Enter
Wait+Screen /Ada Lovelace/
Sleep 500ms
Caption "nitidus: a terminal mail client on Himalaya 3, here on the demo account (no login)"
Show
Sleep 3s

Caption "Panels by background tint, not borders; two-line rows grouped by date"
Type@600ms "jjj"
Sleep 2s

Caption "HTML mail rendered as text; attachments shown as chips"
Type@600ms "jj"
Sleep 1500ms
Type "j"
Sleep 2s

Caption "T: thread view groups replies under the message they answer"
Type "T"
Sleep 3s
Type "T"
Sleep 500ms

Caption "The list pages through the whole mailbox as you scroll (50 at a time)"
Type "G"
Sleep 2s
Type "G"
Sleep 1500ms
Type "g"
Sleep 1s

Caption "/: search with the pimdir query language"
Type "/"
Sleep 500ms
Type@90ms "from:ada"
Sleep 1s
Enter
Sleep 2500ms
Escape
Sleep 1s

Caption "x marks several messages; f, s, a, d and m act on all of them"
Type@250ms "jjjjjjjj"
Type@500ms "xxx"
Sleep 1500ms
Type "f"
Sleep 1s
Caption "m: move through a mailbox picker you can type to filter"
Type "m"
Sleep 800ms
Type@120ms "arch"
Sleep 1200ms
Enter
Wait+Screen /Moved 3 messages/
Sleep 2s

Caption "c: compose in $EDITOR, a plain file of headers and body"
Type "c"
Wait+Screen /Subject:/
Sleep 800ms
Type ":2"
Enter
Type "A"
Type "Rae Field <rae@nitidus.test>"
Escape
Type ":5"
Enter
Type "A"
Type "Hello from nitidus"
Escape
Type "Go"
Type "Sent through Himalaya 3, from a TUI, to myself."
Escape
Sleep 1s
Type ":wq"
Enter
Wait+Screen /New message/
Caption "Review before sending: a attaches a file, d saves a draft, e edits again"
Sleep 1500ms
Type "a"
Sleep 400ms
Type "Cargo.toml"
Sleep 600ms
Enter
Sleep 1500ms
Caption "s sends; the demo's local SMTP server delivers mail for this address back to the inbox"
Type "s"
Wait+Screen /Message sent/
Sleep 1500ms
Ctrl+R
Wait+Screen /Hello from nitidus/
Sleep 2500ms

Caption "r: reply with the quote laid out; Himalaya keeps the threading headers"
Type "j"
Sleep 600ms
Type "r"
Wait+Screen /Subject: Re:/
Sleep 800ms
Type "/^>"
Enter
Type "O"
Type "Thanks! Count me in."
Enter
Escape
Type ":wq"
Enter
Wait+Screen /Reply/
Sleep 2s
Type "s"
Wait+Screen /Message sent/
Sleep 2s

Caption "L opens a link picker; S saves attachments; H shows the raw source"
Type "/"
Sleep 300ms
Type@90ms "subject:ratatui weekly"
Enter
Sleep 1500ms
Type "L"
Sleep 2s
Escape
Sleep 500ms
Escape
Sleep 1s
Type "/"
Type@90ms "has:attachment"
Enter
Sleep 1500ms
Type "S"
Wait+Screen /Saved 1 attachment/
Sleep 1500ms
Type "H"
Sleep 2500ms
Escape
Sleep 500ms
Escape
Sleep 1s

Caption "In the sidebar: n creates a mailbox, R renames it, D deletes it"
Type "h"
Sleep 600ms
Type "n"
Sleep 400ms
Type@90ms "Receipts"
Sleep 800ms
Enter
Wait+Screen /Created Receipts/
Sleep 2s

Caption "A switches account: the demo has a second, work account"
Type "A"
Sleep 1200ms
Type "work"
Sleep 600ms
Enter
Wait+Screen /Nightly build/
Sleep 2500ms

Caption "?: every key in one place, drawn from the same keymap as the hints below"
Type "?"
Sleep 4s
Escape
Sleep 800ms

Caption "Also: a light theme (--theme light), pimdir stores (demo --pimdir), a narrow one-pane layout"
Sleep 4s
Type "q"
