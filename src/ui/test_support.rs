//! Rendering components into a buffer as text, so a test can show what the screen looks like.

use itertools::Itertools as _;
use ratatui::prelude::*;

/// Draws into a `width` by `height` buffer and answers its text, one line per row, trailing
/// spaces trimmed. Colors aren't shown; the contrast test covers those.
pub fn render(width: u16, height: u16, draw: impl FnOnce(Rect, &mut Buffer)) -> String {
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    draw(area, &mut buf);
    text(&buf)
}

/// A buffer's text, one line per row, trailing spaces trimmed.
pub fn text(buf: &Buffer) -> String {
    let area = buf.area;
    let row = |y: u16| {
        (area.left()..area.right())
            .map(|x| buf[(x, y)].symbol())
            .collect::<String>()
            .trim_end()
            .to_owned()
    };
    (area.top()..area.bottom()).map(row).join("\n")
}
