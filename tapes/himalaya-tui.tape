# Side-by-side reference: himalaya-tui master on the same demo mail.
# Needs HIMALAYA_TUI to point at a himalaya-tui build.
Output target/screens/himalaya-tui.mp4
Output target/screens/himalaya-tui.gif
Set Shell "bash"
Set Width 1500
Set Height 820
Set FontSize 15
Set Padding 0
Set Theme "TokyoNight"
Set KeyboardOverlay Keys
Hide
Type "clear && D=$(mktemp -d) && ./target/debug/nitidus demo --dir $D --setup-only && clear && $HIMALAYA_TUI -c $D/config.toml"
Enter
Sleep 4s
Show
Sleep 1s
Screenshot target/screens/himalaya-tui-inbox.png
Type@500ms "jjj"
Sleep 800ms
Enter
Sleep 1.5s
Screenshot target/screens/himalaya-tui-read.png
Escape
Sleep 800ms
Type "?"
Sleep 1s
Screenshot target/screens/himalaya-tui-help.png
Sleep 500ms
