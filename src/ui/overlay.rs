//! Floating panels: a filterable picker, a one-line prompt, a yes/no
//! question. Each owns its state and draws itself centred over the screen.

use ratatui::prelude::*;
use ratatui::widgets::{Clear, Widget};

use super::theme;

fn panel(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width.saturating_sub(4));
    let h = height.min(area.height.saturating_sub(2));
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 3,
        width: w,
        height: h,
    }
}

fn draw_panel(rect: Rect, title: &str, buf: &mut Buffer) -> Rect {
    let t = theme();
    Clear.render(rect, buf);
    buf.set_style(rect, t.base(t.header));
    let title_row = Rect { height: 1, ..rect };
    buf.set_style(title_row, Style::new().bg(t.chip));
    Line::styled(
        format!("  {title}"),
        Style::new().fg(t.fg).add_modifier(Modifier::BOLD),
    )
    .render(title_row, buf);
    Rect {
        y: rect.y + 1,
        height: rect.height - 1,
        ..rect
    }
    .inner(Margin::new(2, 1))
}

/// Pick one item from a list, typing to filter.
pub struct Picker<T> {
    pub title: String,
    pub items: Vec<(String, T)>,
    pub filter: String,
    pub selected: usize,
}

impl<T: Clone> Picker<T> {
    pub fn new(title: impl Into<String>, items: Vec<(String, T)>) -> Self {
        Self {
            title: title.into(),
            items,
            filter: String::new(),
            selected: 0,
        }
    }
    pub fn visible(&self) -> Vec<&(String, T)> {
        let f = self.filter.to_lowercase();
        self.items
            .iter()
            .filter(|(label, _)| label.to_lowercase().contains(&f))
            .collect()
    }
    pub fn chosen(&self) -> Option<T> {
        self.visible().get(self.selected).map(|(_, v)| v.clone())
    }
    pub fn move_by(&mut self, delta: isize) {
        let n = self.visible().len() as isize;
        if n > 0 {
            self.selected = (self.selected as isize + delta).rem_euclid(n) as usize;
        }
    }
    pub fn type_char(&mut self, c: Option<char>) {
        match c {
            Some(c) => self.filter.push(c),
            None => {
                self.filter.pop();
            }
        }
        self.selected = 0;
    }
}

impl<T: Clone> Widget for &Picker<T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        let visible = self.visible();
        let rect = panel(area, 48, visible.len().clamp(1, 14) as u16 + 6);
        let inner = draw_panel(rect, &self.title, buf);
        Line::from(vec![
            Span::styled("⌕ ", Style::new().fg(t.accent)),
            Span::styled(self.filter.clone(), Style::new().fg(t.fg)),
            Span::styled("▏", Style::new().fg(t.accent)),
        ])
        .render(Rect { height: 1, ..inner }, buf);
        let list = Rect {
            y: inner.y + 2,
            height: inner.height.saturating_sub(2),
            ..inner
        };
        let skip = self.selected.saturating_sub(list.height as usize - 1);
        for (row, (i, (label, _))) in visible
            .iter()
            .enumerate()
            .skip(skip)
            .take(list.height as usize)
            .enumerate()
        {
            let r = Rect {
                y: list.y + row as u16,
                height: 1,
                ..list
            };
            let selected = i == self.selected;
            if selected {
                buf.set_style(r, Style::new().bg(t.selected));
            }
            Line::from(vec![
                Span::styled(
                    if selected { "▌ " } else { "  " },
                    Style::new().fg(t.accent),
                ),
                Span::styled(
                    label.clone(),
                    Style::new().fg(if selected { t.fg } else { t.dim }),
                ),
            ])
            .render(r, buf);
        }
        if visible.is_empty() {
            Line::styled("  no match", Style::new().fg(t.faint))
                .render(Rect { height: 1, ..list }, buf);
        }
    }
}

/// A line of text input, for names and paths.
pub struct Prompt {
    pub title: String,
    pub input: String,
    pub hint: String,
}

impl Widget for &Prompt {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        let rect = panel(area, 56, 7);
        let inner = draw_panel(rect, &self.title, buf);
        let field = Rect { height: 1, ..inner };
        buf.set_style(field, Style::new().bg(t.bg));
        Line::from(vec![
            Span::styled(format!(" {}", self.input), Style::new().fg(t.fg)),
            Span::styled("▏", Style::new().fg(t.accent)),
        ])
        .render(field, buf);
        Line::styled(self.hint.clone(), Style::new().fg(t.faint)).render(
            Rect {
                y: inner.y + 2,
                height: 1,
                ..inner
            },
            buf,
        );
    }
}

/// A yes/no question.
pub struct Confirm {
    pub question: String,
}

impl Widget for &Confirm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        let rect = panel(area, (self.question.len() as u16 + 8).max(36), 6);
        let inner = draw_panel(rect, "Are you sure?", buf);
        Line::styled(self.question.clone(), Style::new().fg(t.fg))
            .render(Rect { height: 1, ..inner }, buf);
        Line::from(vec![
            Span::styled(" y ", Style::new().fg(t.fg).bg(t.chip)),
            Span::styled(" yes   ", Style::new().fg(t.dim)),
            Span::styled(" n ", Style::new().fg(t.fg).bg(t.chip)),
            Span::styled(" no", Style::new().fg(t.dim)),
        ])
        .render(
            Rect {
                y: inner.y + 2,
                height: 1,
                ..inner
            },
            buf,
        );
    }
}
