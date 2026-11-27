//! The terminal around the app: its background color while nitidus runs.
//!
//! OSC 11 paints the whole terminal (window padding and any cell nitidus doesn't draw) in the
//! theme's base color, and OSC 111 restores the user's own on exit or panic.

use std::io::Write as _;

use anyhow::Result;
use ratatui::DefaultTerminal;
use ratatui::style::Color;

/// Restores the terminal's own background color.
const RESET_BACKGROUND: &str = "\x1b]111\x1b\\";

/// Runs `app` in the alternate screen with the terminal's background set to `background`.
pub fn run(background: Color, app: impl FnOnce(&mut DefaultTerminal) -> Result<()>) -> Result<()> {
    write_escape(&set_background(background));
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        write_escape(RESET_BACKGROUND);
        hook(info);
    }));
    let result = ratatui::run(app);
    write_escape(RESET_BACKGROUND);
    result
}

/// The OSC 11 sequence setting the background to `color`; nothing for a non-RGB color.
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "only an RGB color has a hex form"
)]
fn set_background(color: Color) -> String {
    match color {
        Color::Rgb(r, g, b) => format!("\x1b]11;#{r:02x}{g:02x}{b:02x}\x1b\\"),
        _ => String::new(),
    }
}

/// Writes an escape sequence to the terminal; a terminal that can't take it is left as it is.
fn write_escape(sequence: &str) {
    let mut out = std::io::stdout();
    let _unsupported = out
        .write_all(sequence.as_bytes())
        .and_then(|()| out.flush());
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::rgb(Color::Rgb(0x1a, 0x1b, 0x26), "\x1b]11;#1a1b26\x1b\\")]
    #[case::named(Color::Blue, "")]
    fn only_rgb_backgrounds_are_sent(#[case] color: Color, #[case] sequence: &str) {
        assert_eq!(set_background(color), sequence);
    }
}
