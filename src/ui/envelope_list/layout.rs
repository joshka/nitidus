//! Where each part of the list sits, in lines, before any is drawn: a date band and a blank line
//! before each section, a row per envelope, and a line saying more is loading.

use crate::ui::dates::Section;

/// Lines an envelope row takes: sender, subject, and a gap.
pub const ROW_LINES: usize = 3;

/// Lines a date band takes: the band and a blank line.
const BAND_LINES: usize = 2;

/// Lines the "more below" note takes, with the gap above it.
const MORE_LINES: usize = 2;

/// The list's layout, in lines from its top.
pub struct ListLayout {
    /// The first line of each envelope's row.
    pub tops: Vec<usize>,

    /// Each date band's line and section.
    pub bands: Vec<(usize, Section)>,

    /// The line of the "more below" note, when another page exists.
    pub more: Option<usize>,

    /// The lines everything takes.
    pub total: usize,
}

impl ListLayout {
    /// Lays out rows for envelopes in `sections`, banded by section unless `banded` is off.
    pub fn new(
        sections: impl IntoIterator<Item = Section>,
        banded: Banding,
        paging: Paging,
    ) -> Self {
        let mut layout = Self {
            tops: Vec::new(),
            bands: Vec::new(),
            more: None,
            total: 0,
        };
        let mut previous = None;
        for section in sections {
            if banded == Banding::BySection && previous != Some(section) {
                layout.bands.push((layout.total, section));
                layout.total += BAND_LINES;
                previous = Some(section);
            }
            layout.tops.push(layout.total);
            layout.total += ROW_LINES;
        }
        if paging == Paging::More {
            layout.more = Some(layout.total);
            layout.total += MORE_LINES;
        }
        layout
    }

    /// The offset showing row `selected` (its band too, when it opens a section) in `height`
    /// lines, moving as little as possible from `offset`. The last row brings the "more" note.
    pub fn scroll_to(&self, selected: usize, offset: usize, height: usize) -> usize {
        let top = self.tops[selected];
        let opens_section = self.bands.iter().any(|&(band, _)| band + BAND_LINES == top);
        let from = if opens_section { top - BAND_LINES } else { top };
        let is_last = selected + 1 == self.tops.len();
        let to = if is_last { self.total } else { top + ROW_LINES };
        if from < offset {
            from
        } else if to > offset + height {
            to - height
        } else {
            offset
        }
    }

    /// The section whose band has scrolled above `offset`, which sticks to the top.
    pub fn sticky(&self, offset: usize) -> Option<Section> {
        self.bands
            .iter()
            .rev()
            .find(|&&(band, _)| band < offset)
            .map(|&(_, section)| section)
    }
}

/// Whether another page follows the listed envelopes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Paging {
    /// The mailbox is fully listed.
    Complete,

    /// Another page can be loaded.
    More,
}

/// Whether rows are grouped under date bands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Banding {
    /// A band opens each date section.
    BySection,

    /// No bands, as in the thread view.
    None,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    /// Two rows today and one older, with another page to come: tops 2, 5 and 10, total 15.
    fn layout() -> ListLayout {
        ListLayout::new(
            [Section::Today, Section::Today, Section::Older],
            Banding::BySection,
            Paging::More,
        )
    }

    #[test]
    fn bands_open_each_section_and_the_more_note_follows_the_rows() {
        let layout = layout();

        assert_eq!(
            (layout.tops.as_slice(), layout.more, layout.total),
            ([2, 5, 10].as_slice(), Some(13), 15)
        );
    }

    #[rstest]
    #[case::already_in_view(1, 0, 8, 0)]
    #[case::below_scrolls_down(1, 0, 4, 4)]
    #[case::opening_a_section_brings_its_band(2, 12, 4, 8)]
    #[case::the_last_row_brings_the_more_note(2, 0, 4, 11)]
    #[case::above_scrolls_up(0, 6, 4, 0)]
    fn the_selection_scrolls_into_view(
        #[case] selected: usize,
        #[case] offset: usize,
        #[case] height: usize,
        #[case] expected: usize,
    ) {
        assert_eq!(layout().scroll_to(selected, offset, height), expected);
    }

    #[rstest]
    #[case::at_the_top(0, None)]
    #[case::past_the_first_band(3, Some(Section::Today))]
    #[case::past_the_second(9, Some(Section::Older))]
    fn the_band_scrolled_past_sticks(#[case] offset: usize, #[case] sticky: Option<Section>) {
        assert_eq!(layout().sticky(offset), sticky);
    }

    #[test]
    fn the_thread_view_has_no_bands() {
        let layout = ListLayout::new(
            [Section::Today, Section::Older],
            Banding::None,
            Paging::Complete,
        );

        assert_eq!((layout.tops, layout.bands.len()), (vec![0, 3], 0));
    }
}
