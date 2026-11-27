//! Pick one item from a list, typing to filter.

use ratatui::crossterm::event::KeyEvent;
use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::{Edit, Outcome, draw_panel, panel_area};
use crate::ui::layout::{Emphasis, PaneFocus, Placeholder, below, cells, line};
use crate::ui::movement::Movement;
use crate::ui::theme;

/// The panel's width.
const WIDTH: u16 = 48;

/// The most items shown at once.
const MAX_ROWS: usize = 14;

/// Lines besides the items: title, padding, filter and a gap.
const CHROME: u16 = 6;

/// A filterable list of labeled choices.
pub struct Picker<T> {
    /// The panel's title.
    title: String,

    /// Each choice's label and value.
    items: Vec<(String, T)>,

    /// The filter typed so far.
    filter: String,

    /// The index of the highlighted choice among those the filter keeps.
    selected: usize,
}

impl<T: Clone> Picker<T> {
    /// A picker titled `title` over `items`.
    pub fn new(title: impl Into<String>, items: Vec<(String, T)>) -> Self {
        Self {
            title: title.into(),
            items,
            filter: String::new(),
            selected: 0,
        }
    }

    /// Applies a key: typing filters, arrows move, Enter chooses, Esc cancels.
    pub fn handle(&mut self, key: KeyEvent) -> Outcome<T> {
        match Edit::of(key) {
            Some(Edit::Cancel) => Outcome::Cancelled,
            Some(Edit::Submit) => self.chosen().map_or(Outcome::Cancelled, Outcome::Done),
            Some(Edit::Next) => self.step(Movement::Down),
            Some(Edit::Previous) => self.step(Movement::Up),
            Some(Edit::Erase) => self.refilter(|filter| {
                filter.pop();
            }),
            Some(Edit::Type(c)) => self.refilter(|filter| filter.push(c)),
            None => Outcome::Open,
        }
    }

    /// The choices the filter keeps, in order.
    fn visible(&self) -> Vec<&(String, T)> {
        let filter = self.filter.to_lowercase();
        self.items
            .iter()
            .filter(|(label, _)| label.to_lowercase().contains(&filter))
            .collect()
    }

    /// The highlighted choice's value.
    fn chosen(&self) -> Option<T> {
        self.visible()
            .get(self.selected)
            .map(|(_, value)| value.clone())
    }

    /// Moves the highlight.
    fn step(&mut self, movement: Movement) -> Outcome<T> {
        self.selected = movement.index(self.selected, self.visible().len());
        Outcome::Open
    }

    /// Edits the filter and returns the highlight to the first choice.
    fn refilter(&mut self, edit: impl FnOnce(&mut String)) -> Outcome<T> {
        edit(&mut self.filter);
        self.selected = 0;
        Outcome::Open
    }
}

impl<T: Clone> Widget for &Picker<T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let visible = self.visible();
        let rows = cells(visible.len().clamp(1, MAX_ROWS));
        let inner = draw_panel(panel_area(area, WIDTH, rows + CHROME), &self.title, buf);
        self.render_filter(line(inner, 0), buf);
        let list = below(inner, 2);
        if visible.is_empty() {
            Placeholder("no match").render(line(list, 0), buf);
        }
        let first = self
            .selected
            .saturating_sub(usize::from(list.height).saturating_sub(1));
        let shown = visible.iter().enumerate().skip(first);
        for (row, (index, (label, _))) in (0..list.height).zip(shown) {
            let emphasis = Emphasis::of_row(index, self.selected, PaneFocus::Focused);
            render_choice(label, emphasis, line(list, row), buf);
        }
    }
}

impl<T> Picker<T> {
    /// The filter typed so far, with a cursor.
    fn render_filter(&self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        let filter = vec![
            Span::styled("⌕ ", Style::new().fg(t.accent)),
            Span::styled(self.filter.clone(), Style::new().fg(t.fg)),
            Span::styled("▏", Style::new().fg(t.accent)),
        ];
        Line::from(filter).render(area, buf);
    }
}

/// One choice, filled when highlighted.
fn render_choice(label: &str, emphasis: Emphasis, area: Rect, buf: &mut Buffer) {
    let t = theme();
    buf.set_style(area, Style::new().bg(emphasis.background(t.header)));
    let (edge, color) = match emphasis {
        Emphasis::Focused | Emphasis::Unfocused => ("▌ ", t.fg),
        Emphasis::Plain => ("  ", t.dim),
    };
    let choice = vec![
        Span::styled(edge, Style::new().fg(t.accent)),
        Span::styled(label.to_owned(), Style::new().fg(color)),
    ];
    Line::from(choice).render(area, buf);
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use ratatui::crossterm::event::KeyCode;
    use rstest::rstest;

    use super::*;
    use crate::ui::test_support::render;

    fn picker() -> Picker<&'static str> {
        Picker::new(
            "Move to…",
            vec![
                ("Archive".into(), "a"),
                ("Projects".into(), "p"),
                ("Trash".into(), "t"),
            ],
        )
    }

    /// The outcome after pressing `keys` in turn.
    fn press(picker: &mut Picker<&'static str>, keys: &[KeyCode]) -> Option<&'static str> {
        keys.iter()
            .fold(None, |_, &key| match picker.handle(KeyEvent::from(key)) {
                Outcome::Done(choice) => Some(choice),
                Outcome::Open | Outcome::Cancelled => None,
            })
    }

    #[rstest]
    #[case::first(&[KeyCode::Enter], Some("a"))]
    #[case::down(&[KeyCode::Down, KeyCode::Enter], Some("p"))]
    #[case::filtered(&[KeyCode::Char('t'), KeyCode::Char('r'), KeyCode::Enter], Some("t"))]
    #[case::filter_erased(&[KeyCode::Char('t'), KeyCode::Backspace, KeyCode::Enter], Some("a"))]
    #[case::no_match(&[KeyCode::Char('z'), KeyCode::Enter], None)]
    fn keys_filter_move_and_choose(#[case] keys: &[KeyCode], #[case] chosen: Option<&'static str>) {
        assert_eq!(press(&mut picker(), keys), chosen);
    }

    #[test]
    fn the_filter_narrows_the_list_and_the_highlight_is_filled() {
        let mut picker = picker();
        press(&mut picker, &[KeyCode::Char('r')]);

        let screen = render(40, 12, |area, buf| (&picker).render(area, buf));

        assert_snapshot!(screen, @"

        Move to…

        ⌕ r▏

        ▌ Archive
          Projects
          Trash
        ");
    }
}
