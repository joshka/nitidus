Output target/screens/read.mp4
Output target/screens/read.gif
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

Type@400ms "jjjj"
Sleep 900ms
Screenshot target/screens/read-html.png
Type "L"
Sleep 900ms
Screenshot target/screens/read-links.png
Escape
Sleep 400ms
Type "j"
Sleep 800ms
Type "S"
Wait+Screen /Saved 1 attachment/
Sleep 900ms
Screenshot target/screens/read-saved.png
Type "H"
Sleep 1.2s
Screenshot target/screens/read-source.png
Escape
Sleep 500ms
Type "q"
