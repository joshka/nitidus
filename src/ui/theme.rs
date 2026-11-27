//! Colours. Panels are told apart by background tint, not by borders.
//!
//! Contrast (WCAG ratio) is checked by the test below: body text and `dim`
//! stay at 4.5:1 or more on every surface they are drawn on, and `faint`
//! (labels, read dates, signatures) at 3:1 or more, never on `selected`.

use ratatui::style::{Color, Style};

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub bg: Color,
    pub sidebar: Color,
    pub reader: Color,
    pub header: Color,
    pub bar: Color,
    pub selected: Color,
    pub selected_dim: Color,
    pub fg: Color,
    pub dim: Color,
    pub faint: Color,
    pub accent: Color,
    pub unread: Color,
    pub flagged: Color,
    pub error: Color,
    pub ok: Color,
    pub chip: Color,
    pub band: Color,
}

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

static ACTIVE: std::sync::OnceLock<Theme> = std::sync::OnceLock::new();

/// Sets the theme once, at startup.
pub fn set(theme: Theme) {
    let _ = ACTIVE.set(theme);
}

/// The active theme, dark unless set.
pub fn theme() -> Theme {
    *ACTIVE.get_or_init(|| DARK)
}

impl Theme {
    pub fn base(&self, bg: Color) -> Style {
        Style::new().fg(self.fg).bg(bg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lum(c: Color) -> f64 {
        let Color::Rgb(r, g, b) = c else {
            panic!("theme colours are RGB")
        };
        let ch = |v: u8| {
            let x = f64::from(v) / 255.0;
            if x <= 0.03928 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b)
    }

    fn ratio(a: Color, b: Color) -> f64 {
        let (a, b) = (lum(a), lum(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn text_has_enough_contrast() {
        check(DARK);
        check(LIGHT);
    }

    fn check(t: Theme) {
        let all = [
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
        for bg in all {
            for fg in [t.fg, t.dim, t.unread] {
                assert!(
                    ratio(fg, bg) >= 4.5,
                    "{fg:?} on {bg:?}: {:.2}",
                    ratio(fg, bg)
                );
            }
            for fg in [t.accent, t.flagged, t.error, t.ok] {
                assert!(
                    ratio(fg, bg) >= 3.0,
                    "{fg:?} on {bg:?}: {:.2}",
                    ratio(fg, bg)
                );
            }
        }
        for bg in [t.bg, t.sidebar, t.reader, t.header, t.bar, t.band] {
            assert!(
                ratio(t.faint, bg) >= 3.0,
                "faint on {bg:?}: {:.2}",
                ratio(t.faint, bg)
            );
        }
    }
}
