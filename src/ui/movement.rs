//! Moving through whatever has focus: a list's selection, or a page's scroll.

/// How far a page key moves a selection.
const PAGE: usize = 10;

/// A move of the focused component's selection or scroll.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Movement {
    /// One item or line down.
    Down,

    /// One item or line up.
    Up,

    /// A page down.
    PageDown,

    /// A page up.
    PageUp,

    /// To the first item or the top.
    Top,

    /// To the last item or the bottom.
    Bottom,
}

/// A component that moves its selection or scroll.
pub trait Navigate {
    /// Applies `movement`, answering whether the selection changed.
    fn navigate(&mut self, movement: Movement) -> bool;
}

impl Movement {
    /// The index `current` moves to in a list of `len` items, clamped to the list.
    pub fn index(self, current: usize, len: usize) -> usize {
        let last = len.saturating_sub(1);
        let moved = match self {
            Self::Down => current.saturating_add(1),
            Self::Up => current.saturating_sub(1),
            Self::PageDown => current.saturating_add(PAGE),
            Self::PageUp => current.saturating_sub(PAGE),
            Self::Top => 0,
            Self::Bottom => last,
        };
        moved.min(last)
    }

    /// The scroll offset `current` moves to, `page` lines a page, unclamped below.
    pub fn scroll(self, current: u16, page: u16) -> u16 {
        match self {
            Self::Down => current.saturating_add(1),
            Self::Up => current.saturating_sub(1),
            Self::PageDown => current.saturating_add(page),
            Self::PageUp => current.saturating_sub(page),
            Self::Top => 0,
            Self::Bottom => u16::MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::down(Movement::Down, 3, 4)]
    #[case::down_at_the_end(Movement::Down, 19, 19)]
    #[case::up_at_the_start(Movement::Up, 0, 0)]
    #[case::page_down_clamps(Movement::PageDown, 15, 19)]
    #[case::page_up_clamps(Movement::PageUp, 4, 0)]
    #[case::top(Movement::Top, 12, 0)]
    #[case::bottom(Movement::Bottom, 2, 19)]
    fn an_index_moves_within_a_list_of_twenty(
        #[case] movement: Movement,
        #[case] from: usize,
        #[case] to: usize,
    ) {
        assert_eq!(movement.index(from, 20), to);
    }

    #[test]
    fn an_empty_list_stays_at_zero() {
        assert_eq!(Movement::Down.index(0, 0), 0);
    }

    #[rstest]
    #[case::line(Movement::Down, 5, 6)]
    #[case::page(Movement::PageDown, 5, 20)]
    #[case::back_past_the_top(Movement::PageUp, 5, 0)]
    #[case::bottom_is_clamped_by_the_view(Movement::Bottom, 5, u16::MAX)]
    fn a_scroll_moves_by_lines_or_pages(
        #[case] movement: Movement,
        #[case] from: u16,
        #[case] to: u16,
    ) {
        assert_eq!(movement.scroll(from, 15), to);
    }
}
