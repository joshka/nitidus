Output target/screens/pimdir.mp4
Output target/screens/pimdir.gif
Set Shell "bash"
Set Width 1500
Set Height 820
Set FontSize 15
Set Padding 0
Set Theme "TokyoNight"
Set KeyboardOverlay Keys
Set TypingSpeed 60ms

Hide
Type "clear && export EDITOR=\"vim -Nu NONE\" NITIDUS_DOWNLOADS=target/screens/dl && mkdir -p target/screens/dl && ./target/debug/nitidus demo --pimdir"
Enter
Wait+Screen /Ada Lovelace/
Sleep 600ms
Show
Sleep 800ms

Screenshot target/screens/pimdir-inbox.png
Type@450ms "jjjj"
Sleep 900ms
Type "T"
Sleep 1.2s
Screenshot target/screens/pimdir-threads.png
Type "T"
Sleep 600ms
Type "G"
Sleep 1.5s
Screenshot target/screens/pimdir-paging.png
Type "G"
Sleep 1s
Type "g"
Sleep 600ms
Type "/"
Sleep 300ms
Type "from:ada"
Sleep 600ms
Screenshot target/screens/pimdir-search-prompt.png
Enter
Sleep 1.2s
Screenshot target/screens/pimdir-search.png
Escape
Sleep 600ms
Type "f"
Sleep 400ms
Type "a"
Wait+Screen /queued for sync/
Sleep 1.2s
Screenshot target/screens/pimdir-queued.png
Type "q"
