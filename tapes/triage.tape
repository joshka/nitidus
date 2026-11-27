Output target/screens/triage.mp4
Output target/screens/triage.gif
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

Type@350ms "jjj"
Type@500ms "xxx"
Sleep 800ms
Screenshot target/screens/triage-marked.png
Type "f"
Sleep 800ms
Type "m"
Sleep 600ms
Type "arch"
Sleep 600ms
Screenshot target/screens/triage-move.png
Enter
Wait+Screen /Moved 3 messages/
Sleep 900ms
Screenshot target/screens/triage-moved.png
Type "d"
Sleep 800ms
Type "h"
Sleep 300ms
Type "G"
Wait+Screen /You have WON/
Sleep 600ms
Type "l"
Sleep 300ms
Type "d"
Sleep 800ms
Screenshot target/screens/triage-confirm.png
Type "y"
Sleep 1.2s
Type "h"
Type@500ms "kk"
Sleep 1.2s
Screenshot target/screens/triage-archive.png
Type "q"
