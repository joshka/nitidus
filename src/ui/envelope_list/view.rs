//! Drawing the envelope list: a header line, then bands and rows scrolled to keep the selection in
//! view, a sticky band, and a thin scrollbar.

use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::layout::{Banding, ListLayout, Paging, ROW_LINES};
use super::row::{Row, RowState};
use super::{EnvelopeList, Loading};
use crate::ui::dates::Section;
use crate::ui::layout::{Emphasis, Placeholder, below, cells, line};
use crate::ui::theme;

/// Lines above the rows: a gap, the header, and another gap.
const HEADER_LINES: u16 = 3;

impl Widget for &mut EnvelopeList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        buf.set_style(area, t.base(t.bg));
        self.render_header(line(area, 1).inner(Margin::new(2, 0)), buf);
        let body = below(area, HEADER_LINES);
        if self.envelopes.is_empty() {
            Placeholder(if self.loading == Loading::FirstPage {
                "Loading…"
            } else {
                "No messages"
            })
            .render(body, buf);
            return;
        }
        let layout = self.layout();
        self.offset = layout.scroll_to(self.selected, self.offset, usize::from(body.height));
        let view = Viewport {
            area: body,
            offset: self.offset,
        };
        self.render_lines(&layout, view, buf);
        render_scrollbar(&layout, view, buf);
    }
}

/// The part of the layout on screen: the area drawn into and the first line shown.
#[derive(Clone, Copy)]
struct Viewport {
    /// Where the rows are drawn.
    area: Rect,

    /// The first layout line shown.
    offset: usize,
}

impl Viewport {
    /// The screen rows of layout lines `top..top + height`, clipped to the viewport; `None` when
    /// none of them shows.
    fn rows(self, top: usize, height: usize) -> Option<Rect> {
        let start = top.saturating_sub(self.offset);
        let end = (top + height)
            .checked_sub(self.offset)?
            .min(usize::from(self.area.height));
        (start < end).then(|| Rect {
            y: self.area.y + cells(start),
            height: cells(end - start),
            ..self.area
        })
    }
}

impl EnvelopeList {
    /// The mailbox name, counts, marks and search on the left; the thread view on the right.
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        Line::from(self.header_spans()).render(area, buf);
        if self.threaded {
            Line::styled("threads ", Style::new().fg(theme().accent))
                .right_aligned()
                .render(area, buf);
        }
    }

    /// The header's left side: the mailbox, its count, then what is unread, marked or searched.
    fn header_spans(&self) -> Vec<Span<'static>> {
        let t = theme();
        let more = if self.next_page.is_some() { "+" } else { "" };
        let title = Style::new().fg(t.fg).add_modifier(Modifier::BOLD);
        let mut spans = vec![
            Span::styled(self.title.clone(), title),
            Span::styled(format!("  {}{more}", self.len()), Style::new().fg(t.dim)),
        ];
        spans.extend(self.header_notes());
        spans
    }

    /// The unread and marked counts, and the active search, each when there is one.
    fn header_notes(&self) -> Vec<Span<'static>> {
        let t = theme();
        let unread = self.unread();
        let marked = self.marked.len();
        let mut notes = Vec::new();
        if unread > 0 {
            notes.push(Span::styled(
                format!(" · {unread} unread"),
                Style::new().fg(t.unread),
            ));
        }
        if marked > 0 {
            notes.push(Span::styled(
                format!(" · {marked} marked"),
                Style::new().fg(t.ok),
            ));
        }
        if let Some(search) = &self.search {
            notes.push(Span::styled("   ⌕ ", Style::new().fg(t.accent)));
            notes.push(Span::styled(
                format!(" {search} "),
                Style::new().fg(t.fg).bg(t.chip),
            ));
        }
        notes
    }

    /// The layout of the current envelopes.
    fn layout(&self) -> ListLayout {
        let sections = self
            .envelopes
            .iter()
            .map(|envelope| Section::of(envelope.date));
        let banding = if self.threaded {
            Banding::None
        } else {
            Banding::BySection
        };
        let paging = if self.next_page.is_some() {
            Paging::More
        } else {
            Paging::Complete
        };
        ListLayout::new(sections, banding, paging)
    }

    /// The bands, rows, sticky band and "more" note on screen.
    fn render_lines(&self, layout: &ListLayout, view: Viewport, buf: &mut Buffer) {
        for &(top, section) in &layout.bands {
            if let Some(area) = view.rows(top, 1) {
                render_band(section, area, buf);
            }
        }
        self.render_rows(layout, view, buf);
        if let Some(section) = layout.sticky(view.offset) {
            render_band(section, line(view.area, 0), buf);
        }
        if let Some(area) = layout.more.and_then(|more| view.rows(more, 1)) {
            let note = if self.loading == Loading::NextPage {
                "Loading more…"
            } else {
                "More below"
            };
            Placeholder(note).render(area, buf);
        }
    }

    /// Every row wholly on screen.
    fn render_rows(&self, layout: &ListLayout, view: Viewport, buf: &mut Buffer) {
        let row_height = ROW_LINES - 1;
        for (index, envelope) in self.envelopes.iter().enumerate() {
            let area = view.rows(layout.tops[index], row_height);
            if let Some(area) = area.filter(|area| usize::from(area.height) == row_height) {
                Row {
                    envelope,
                    state: self.row_state(index),
                }
                .render(area, buf);
            }
        }
    }

    /// How the row at `index` is shown.
    fn row_state(&self, index: usize) -> RowState {
        RowState {
            emphasis: Emphasis::of_row(index, self.selected, self.focus),
            marked: self.marked.contains(&self.envelopes[index].id),
            depth: self.depth.get(index).copied().unwrap_or_default(),
        }
    }
}

/// A date band.
fn render_band(section: Section, area: Rect, buf: &mut Buffer) {
    let t = theme();
    buf.set_style(area, Style::new().bg(t.band));
    let label = Style::new().fg(t.faint).add_modifier(Modifier::BOLD);
    Line::styled(format!("  {}", section.label()), label).render(area, buf);
}

/// A one-cell scrollbar on the right edge, when the list is taller than the view.
fn render_scrollbar(layout: &ListLayout, view: Viewport, buf: &mut Buffer) {
    let height = usize::from(view.area.height);
    if layout.total <= height || view.area.width <= 2 {
        return;
    }
    let t = theme();
    let thumb = (height * height / layout.total).max(1);
    let start = view.offset * height / layout.total;
    let x = view.area.right() - 1;
    for row in 0..height {
        let color = if (start..start + thumb).contains(&row) {
            t.dim
        } else {
            t.band
        };
        buf[(x, view.area.y + cells(row))]
            .set_symbol("▕")
            .set_fg(color);
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;
    use crate::mail::fixtures::envelope;
    use crate::ui::layout::PaneFocus;
    use crate::ui::movement::{Movement, Navigate};
    use crate::ui::test_support::render;

    fn list() -> EnvelopeList {
        let mut list = EnvelopeList::loading("INBOX".into());
        list.replace(vec![
            envelope("1", "Ada", "Notes on the loop", 9, &[]),
            envelope("2", "Grace", "Compiler meeting", 8, &["seen", "flagged"]),
            envelope("3", "Ken", "Trusting trust", 7, &["seen"]),
        ]);
        list.loading = Loading::Idle;
        list.focus = PaneFocus::Focused;
        list
    }

    #[test]
    fn rows_sit_under_their_date_band_with_unread_and_flags_marked() {
        let mut list = list();

        let screen = render(44, 13, |area, buf| list.render(area, buf));

        assert_snapshot!(screen, @"

          INBOX  3 · 1 unread

          Older                                    ▕
                                                   ▕
        ▌ ● Ada                         2020-01-09 ▕
        ▌   Notes on the loop                      ▕
                                                   ▕
            Grace                     ★ 2020-01-08 ▕
            Compiler meeting                       ▕
                                                   ▕
            Ken                         2020-01-07 ▕
            Trusting trust                         ▕
        ");
    }

    #[test]
    fn marks_and_an_unfinished_mailbox_show_in_the_header() {
        let mut list = list();
        list.toggle_mark();
        list.navigate(Movement::Down);
        list.next_page = Some(2);

        let screen = render(44, 14, |area, buf| list.render(area, buf));

        assert_snapshot!(screen, @"

          INBOX  3+ · 1 unread · 1 marked

          Older                                    ▕
                                                   ▕
        ▌ ✓ Ada                         2020-01-09 ▕
        ▌   Notes on the loop                      ▕
                                                   ▕
        ▌   Grace                     ★ 2020-01-08 ▕
        ▌   Compiler meeting                       ▕
                                                   ▕
            Ken                         2020-01-07 ▕
            Trusting trust                         ▕
                                                   ▕
        ");
    }

    #[test]
    fn an_empty_list_says_whether_it_is_still_loading() {
        let mut loading = EnvelopeList::loading("INBOX".into());
        let mut empty = EnvelopeList::default();

        let screens = [
            render(30, 6, |area, buf| loading.render(area, buf)),
            render(30, 6, |area, buf| empty.render(area, buf)),
        ];

        assert_snapshot!(screens.join("\n---\n"), @"

          INBOX  0


                   Loading…

        ---

            0


                 No messages
        ");
    }
}
