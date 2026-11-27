Output target/screens/browse.mp4
Output target/screens/browse.gif
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

Screenshot target/screens/browse-inbox.png
Type@450ms "jjjj"
Sleep 900ms
Type "T"
Sleep 1.2s
Screenshot target/screens/browse-threads.png
Type "T"
Sleep 600ms
Type "G"
Sleep 1.5s
Screenshot target/screens/browse-paging.png
Type "G"
Sleep 1s
Type "g"
Sleep 600ms
Type "/"
Sleep 300ms
Type "from:ada"
Sleep 600ms
Screenshot target/screens/browse-search-prompt.png
Enter
Sleep 1.2s
Screenshot target/screens/browse-search.png
Escape
Sleep 600ms
Type "?"
Sleep 1.2s
Screenshot target/screens/browse-help.png
Escape
Sleep 300ms
Type "q"
