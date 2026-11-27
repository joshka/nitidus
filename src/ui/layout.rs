//! Geometry helpers shared by components, and the placeholder they show when empty.

use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::theme;

/// A count as terminal cells, saturating at the largest a terminal can have.
pub fn cells(count: usize) -> u16 {
    u16::try_from(count).unwrap_or(u16::MAX)
}

/// The one-line row `offset` lines into `area`, empty once past its bottom.
pub fn line(area: Rect, offset: u16) -> Rect {
    let y = area.y.saturating_add(offset).min(area.bottom());
    Rect {
        y,
        height: u16::from(y < area.bottom()),
        ..area
    }
}

/// `area` below its first `rows` lines.
pub fn below(area: Rect, rows: u16) -> Rect {
    let rows = rows.min(area.height);
    Rect {
        y: area.y + rows,
        height: area.height - rows,
        ..area
    }
}

/// `area` no wider than `width`, keeping its left edge.
pub fn at_most_wide(area: Rect, width: u16) -> Rect {
    Rect {
        width: area.width.min(width),
        ..area
    }
}

/// A faint centred message a third of the way down an empty panel.
pub struct Placeholder<'a>(pub &'a str);

impl Widget for Placeholder<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::new().fg(theme().faint);
        Line::styled(self.0, style)
            .centered()
            .render(line(area, area.height / 3), buf);
    }
}

/// How strongly a row is set off: the focused selection, a selection without focus, or not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emphasis {
    /// The selection of the focused pane.
    Focused,

    /// The selection of a pane without focus, or a marked row.
    Unfocused,

    /// An ordinary row.
    Plain,
}

impl Emphasis {
    /// The emphasis of row `index` when `selected` is selected, given whether the pane has focus.
    pub fn of_row(index: usize, selected: usize, focus: PaneFocus) -> Self {
        match (index == selected, focus) {
            (true, PaneFocus::Focused) => Self::Focused,
            (true, PaneFocus::Unfocused) => Self::Unfocused,
            (false, _) => Self::Plain,
        }
    }

    /// The row's background, `plain` when not set off.
    pub fn background(self, plain: Color) -> Color {
        let t = theme();
        match self {
            Self::Focused => t.selected,
            Self::Unfocused => t.selected_dim,
            Self::Plain => plain,
        }
    }
}

/// Whether a pane has focus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneFocus {
    /// Keys go to the pane.
    Focused,

    /// Keys go elsewhere.
    #[default]
    Unfocused,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::first(0, Rect::new(2, 3, 10, 1))]
    #[case::last(3, Rect::new(2, 6, 10, 1))]
    #[case::past_the_bottom(4, Rect::new(2, 7, 10, 0))]
    fn a_line_is_one_row_of_the_area_or_empty_below_it(
        #[case] offset: u16,
        #[case] expected: Rect,
    ) {
        assert_eq!(line(Rect::new(2, 3, 10, 4), offset), expected);
    }

    #[test]
    fn below_never_runs_past_the_area() {
        let area = Rect::new(0, 0, 10, 4);

        assert_eq!(
            (below(area, 1), below(area, 9)),
            (Rect::new(0, 1, 10, 3), Rect::new(0, 4, 10, 0))
        );
    }

    #[rstest]
    #[case::focused_selection(1, PaneFocus::Focused, Emphasis::Focused)]
    #[case::selection_elsewhere(1, PaneFocus::Unfocused, Emphasis::Unfocused)]
    #[case::other_row(2, PaneFocus::Focused, Emphasis::Plain)]
    fn a_row_is_set_off_by_selection_and_focus(
        #[case] index: usize,
        #[case] focus: PaneFocus,
        #[case] expected: Emphasis,
    ) {
        assert_eq!(Emphasis::of_row(index, 1, focus), expected);
    }
}
