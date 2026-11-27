Output target/screens/mailboxes.mp4
Output target/screens/mailboxes.gif
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

Type "h"
Sleep 400ms
Type "n"
Sleep 300ms
Type "Receipts"
Sleep 500ms
Screenshot target/screens/mailboxes-new.png
Enter
Wait+Screen /Created Receipts/
Sleep 900ms
Type "G"
Sleep 400ms
Type "k"
Sleep 600ms
Type "R"
Sleep 300ms
Backspace 8
Type "Paperwork"
Sleep 400ms
Enter
Wait+Screen /Renamed/
Sleep 900ms
Screenshot target/screens/mailboxes-renamed.png
Type "A"
Sleep 800ms
Screenshot target/screens/mailboxes-accounts.png
Type "work"
Sleep 300ms
Enter
Wait+Screen /Nightly build/
Sleep 1.2s
Screenshot target/screens/mailboxes-work.png
Type "q"
