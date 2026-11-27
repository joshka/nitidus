# The README screenshot: the demo inbox on a transparent background, without captions or key
# labels. Upload target/screens/readme.png to the readme-assets release as nitidus.png.
Output target/screens/readme.png

Set Shell "bash"
Set Width 1400
Set Height 720
Set FontSize 15
Set Padding 0
Set Margin 16
Set MarginFill "transparent"
Set Theme "TokyoNight"

Hide
Type "clear && ./target/debug/nitidus demo"
Enter
Wait+Screen /Ada Lovelace/
Sleep 800ms
Show
Sleep 500ms
