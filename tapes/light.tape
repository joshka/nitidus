Output target/screens/light.mp4
Output target/screens/light.gif
Set Shell "bash"
Set Width 1500
Set Height 820
Set FontSize 15
Set Padding 0
Set Theme "TokyoNight Day"
Set KeyboardOverlay Keys
Set TypingSpeed 60ms

Hide
Type "clear && export EDITOR=\"vim -Nu NONE\" NITIDUS_DOWNLOADS=target/screens/dl && mkdir -p target/screens/dl && ./target/debug/nitidus --theme light demo"
Enter
Wait+Screen /Ada Lovelace/
Sleep 600ms
Show
Sleep 800ms

Screenshot target/screens/light-inbox.png
Type@450ms "jjjj"
Sleep 900ms
Type "T"
Sleep 1.2s
Screenshot target/screens/light-threads.png
Type "T"
Sleep 600ms
Type "G"
Sleep 1.5s
Screenshot target/screens/light-paging.png
Type "G"
Sleep 1s
Type "g"
Sleep 600ms
Type "/"
Sleep 300ms
Type "from:ada"
Sleep 600ms
Screenshot target/screens/light-search-prompt.png
Enter
Sleep 1.2s
Screenshot target/screens/light-search.png
Escape
Sleep 600ms
Type "?"
Sleep 1.2s
Screenshot target/screens/light-help.png
Escape
Sleep 300ms
Type "q"
