Output target/screens/compose.mp4
Output target/screens/compose.gif
Set Shell "bash"
Set Width 1500
Set Height 820
Set FontSize 15
Set Padding 0
Set Theme "TokyoNight"
Set KeyboardOverlay Keys
Set TypingSpeed 60ms

Hide
Type "clear && export EDITOR=\"vim -Nu NONE\" NITIDUS_DOWNLOADS=target/screens/dl && mkdir -p target/screens/dl && ./target/debug/nitidus demo"
Enter
Wait+Screen /Ada Lovelace/
Sleep 600ms
Show
Sleep 800ms

Type "c"
Wait+Screen /Subject:/
Sleep 500ms
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
Type "Sent through himalaya 3, from a TUI, to myself."
Escape
Sleep 600ms
Screenshot target/screens/compose-editor.png
Type ":wq"
Enter
Wait+Screen /New message/
Sleep 900ms
Type "a"
Sleep 300ms
Type "Cargo.toml"
Sleep 400ms
Screenshot target/screens/compose-attach.png
Enter
Sleep 900ms
Screenshot target/screens/compose-review.png
Type "s"
Wait+Screen /Message sent/
Sleep 900ms
Ctrl+R
Wait+Screen /Hello from nitidus/
Sleep 1.2s
Screenshot target/screens/compose-delivered.png
Type@400ms "jjjjj"
Sleep 500ms
Type "r"
Wait+Screen /Subject: Re:/
Sleep 500ms
Type "/^>"
Enter
Type "O"
Type "Yes! Count me in. I'll bring the good snacks."
Enter
Escape
Type ":wq"
Enter
Wait+Screen /Reply/
Sleep 1.2s
Screenshot target/screens/compose-reply.png
Type "s"
Wait+Screen /Message sent/
Sleep 1.5s
Screenshot target/screens/compose-answered.png
Type "q"
