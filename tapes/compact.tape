Output target/screens/compact.mp4
Output target/screens/compact.gif
Set Shell "bash"
Set Width 1500
Set Height 820
Set FontSize 15
Set Padding 0
Set Theme "TokyoNight"
Set KeyboardOverlay Keys
Set KeyboardOverlayLocation CaptionRow
Set TypingSpeed 60ms

Hide
Type "clear && ./target/debug/nitidus demo"
Enter
Wait+Screen /Ada Lovelace/
Sleep 600ms
Show

Caption "Two lines per message, grouped by date"
Sleep 1.5s
Caption "C: one line per message"
Type "C"
Sleep 1.5s
Screenshot target/screens/compact-dates.png
Type@300ms "jjjj"
Sleep 800ms
Caption "T: compact threads"
Type "T"
Sleep 1.5s
Screenshot target/screens/compact-threads.png
Type@300ms "jjj"
Sleep 800ms
Caption "C again: back to two lines"
Type "C"
Sleep 1.5s
Caption ""
Type "T"
Sleep 600ms
Hide
Type "q"
