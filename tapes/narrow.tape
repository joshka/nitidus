Output target/screens/narrow.gif

Set Shell "bash"
Set Width 760
Set Height 620
Set FontSize 15
Set Padding 0
Set Theme "TokyoNight"
Set KeyboardOverlay Keys

Hide
Type "clear && ./target/debug/nitidus demo"
Enter
Wait+Screen /Ada Lovelace/
Sleep 800ms
Show
Sleep 1s
Screenshot target/screens/narrow-list.png
Enter
Sleep 1s
Screenshot target/screens/narrow-reader.png
Escape
Sleep 500ms
Type "h"
Sleep 1s
Screenshot target/screens/narrow-mailboxes.png
Type "q"
