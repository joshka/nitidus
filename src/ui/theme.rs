//! Colours. Panels are told apart by background tint, not by borders.
//!
//! Contrast (WCAG ratio) is checked by the test below: body text and `dim`
//! stay at 4.5:1 or more on every surface they are drawn on, and `faint`
//! (labels, read dates, signatures) at 3:1 or more, never on `selected`.

use ratatui::style::{Color, Style};

/// A palette, named by role.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    /// The list and main background.
    pub bg: Color,

    /// The sidebar's tint, darker than the list.
    pub sidebar: Color,

    /// The reader's tint, lighter than the list.
    pub reader: Color,

    /// Header cards and floating panels.
    pub header: Color,

    /// The title and status bars.
    pub bar: Color,

    /// The focused selection.
    pub selected: Color,

    /// A selection without focus, and marked rows.
    pub selected_dim: Color,

    /// Body text.
    pub fg: Color,

    /// Secondary text: read senders and subjects, field values.
    pub dim: Color,

    /// Labels, read dates and signatures; never on a selected row.
    pub faint: Color,

    /// Focus edges, links and titles.
    pub accent: Color,

    /// Unread dots, counts and dates.
    pub unread: Color,

    /// The flag star.
    pub flagged: Color,

    /// Errors and missing fields.
    pub error: Color,

    /// Success, and marked rows.
    pub ok: Color,

    /// Key chips, attachment chips and panel titles.
    pub chip: Color,

    /// Date bands and the scrollbar track.
    pub band: Color,
}

/// The default palette.
pub const DARK: Theme = Theme {
    bg: Color::Rgb(0x1a, 0x1b, 0x26),
    sidebar: Color::Rgb(0x13, 0x14, 0x1c),
    reader: Color::Rgb(0x1f, 0x22, 0x33),
    header: Color::Rgb(0x26, 0x2a, 0x40),
    bar: Color::Rgb(0x13, 0x14, 0x1c),
    selected: Color::Rgb(0x2e, 0x3c, 0x64),
    selected_dim: Color::Rgb(0x26, 0x2b, 0x40),
    fg: Color::Rgb(0xc0, 0xca, 0xf5),
    dim: Color::Rgb(0xa0, 0xa8, 0xd0),
    faint: Color::Rgb(0x78, 0x80, 0xa8),
    accent: Color::Rgb(0x7a, 0xa2, 0xf7),
    unread: Color::Rgb(0x7d, 0xcf, 0xff),
    flagged: Color::Rgb(0xff, 0x9e, 0x64),
    error: Color::Rgb(0xf7, 0x76, 0x8e),
    ok: Color::Rgb(0x9e, 0xce, 0x6a),
    chip: Color::Rgb(0x34, 0x3a, 0x58),
    band: Color::Rgb(0x21, 0x23, 0x33),
};

/// The palette for light terminals.
pub const LIGHT: Theme = Theme {
    bg: Color::Rgb(0xfa, 0xfa, 0xfc),
    sidebar: Color::Rgb(0xee, 0xef, 0xf4),
    reader: Color::Rgb(0xff, 0xff, 0xff),
    header: Color::Rgb(0xe8, 0xeb, 0xf5),
    bar: Color::Rgb(0xe4, 0xe6, 0xee),
    selected: Color::Rgb(0xd4, 0xdd, 0xf7),
    selected_dim: Color::Rgb(0xe2, 0xe5, 0xef),
    fg: Color::Rgb(0x1f, 0x23, 0x35),
    dim: Color::Rgb(0x4a, 0x50, 0x68),
    faint: Color::Rgb(0x6b, 0x70, 0x88),
    accent: Color::Rgb(0x2e, 0x5c, 0xd6),
    unread: Color::Rgb(0x0b, 0x5c, 0x9e),
    flagged: Color::Rgb(0xb4, 0x4c, 0x00),
    error: Color::Rgb(0xc0, 0x1c, 0x3c),
    ok: Color::Rgb(0x2b, 0x7a, 0x1e),
    chip: Color::Rgb(0xd8, 0xdc, 0xea),
    band: Color::Rgb(0xf0, 0xf1, 0xf6),
};

/// The palette chosen at startup.
static ACTIVE: std::sync::OnceLock<Theme> = std::sync::OnceLock::new();

/// Sets the theme once, at startup.
pub fn set(theme: Theme) {
    let _already_set = ACTIVE.set(theme);
}

/// The active theme, dark unless set.
pub fn theme() -> Theme {
    *ACTIVE.get_or_init(|| DARK)
}

impl Theme {
    /// Body text on `bg`, the base style of a panel.
    pub fn base(&self, bg: Color) -> Style {
        Style::new().fg(self.fg).bg(bg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The WCAG ratio body text and `dim` need: AA for normal text.
    const TEXT: f64 = 4.5;

    /// The WCAG ratio glyphs, accents and `faint` labels need.
    const GLYPHS: f64 = 3.0;

    /// The WCAG relative luminance of an RGB color.
    fn luminance(color: Color) -> f64 {
        let Color::Rgb(red, green, blue) = color else {
            panic!("theme colors are RGB")
        };
        let channel = |value: u8| {
            let linear = f64::from(value) / 255.0;
            if linear <= 0.039_28 {
                linear / 12.92
            } else {
                ((linear + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(red) + 0.7152 * channel(green) + 0.0722 * channel(blue)
    }

    /// The WCAG contrast ratio between two colors.
    fn ratio(first: Color, second: Color) -> f64 {
        let (first, second) = (luminance(first), luminance(second));
        (first.max(second) + 0.05) / (first.min(second) + 0.05)
    }

    /// Every pairing of `foregrounds` on `backgrounds` that falls under `minimum`.
    fn failures(foregrounds: &[Color], backgrounds: &[Color], minimum: f64) -> Vec<String> {
        let pairs = foregrounds
            .iter()
            .flat_map(|&fg| backgrounds.iter().map(move |&bg| (fg, bg)));
        let low = pairs.filter(|&(fg, bg)| ratio(fg, bg) < minimum);
        low.map(|(fg, bg)| format!("{fg:?} on {bg:?}: {:.2}", ratio(fg, bg)))
            .collect()
    }

    /// The contrast failures of a palette: text and glyphs on every surface, `faint` on every
    /// surface but the selections and chips.
    fn contrast_failures(t: Theme) -> Vec<String> {
        let surfaces = [
            t.bg,
            t.sidebar,
            t.reader,
            t.header,
            t.bar,
            t.selected,
            t.selected_dim,
            t.band,
            t.chip,
        ];
        let unselected = [t.bg, t.sidebar, t.reader, t.header, t.bar, t.band];
        let mut failed = failures(&[t.fg, t.dim, t.unread], &surfaces, TEXT);
        failed.extend(failures(
            &[t.accent, t.flagged, t.error, t.ok],
            &surfaces,
            GLYPHS,
        ));
        failed.extend(failures(&[t.faint], &unselected, GLYPHS));
        failed
    }

    #[test]
    fn both_palettes_keep_text_readable() {
        assert_eq!(contrast_failures(DARK), Vec::<String>::new());
        assert_eq!(contrast_failures(LIGHT), Vec::<String>::new());
    }
}
