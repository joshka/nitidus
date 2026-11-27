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
