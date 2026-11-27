//! The envelope list: two-line rows under date bands, selection and marks
//! shown by background, an optional thread view.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, FixedOffset, Local};
use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::theme;
use crate::mail::Envelope;

const ROW: u16 = 3;

#[derive(Default)]
pub struct EnvelopeList {
    /// In display order: newest first, or by thread.
    pub envelopes: Vec<Envelope>,
    /// Thread depth of each envelope, all zero outside the thread view.
    pub depth: Vec<u8>,
    pub selected: usize,
    pub offset: usize,
    pub focused: bool,
    pub title: String,
    pub query: Option<String>,
    pub loading: bool,
    pub marked: HashSet<String>,
    pub threaded: bool,
    /// The next page to load, `None` once the mailbox is complete.
    pub next_page: Option<usize>,
    pub loading_more: bool,
}

impl EnvelopeList {
    pub fn current(&self) -> Option<&Envelope> {
        self.envelopes.get(self.selected)
    }
    pub fn current_mut(&mut self) -> Option<&mut Envelope> {
        self.envelopes.get_mut(self.selected)
    }
    pub fn move_by(&mut self, delta: isize) {
        let len = self.envelopes.len() as isize;
        if len > 0 {
            self.selected = (self.selected as isize + delta).clamp(0, len - 1) as usize;
        }
    }
    pub fn unread(&self) -> usize {
        self.envelopes.iter().filter(|e| !e.has("seen")).count()
    }

    /// Replaces the list (page 1) or appends a later page.
    pub fn load(&mut self, envelopes: Vec<Envelope>, append: bool) {
        let keep = self.current().map(|e| e.id.clone());
        if append {
            let known: HashSet<String> = self.envelopes.iter().map(|e| e.id.clone()).collect();
            self.envelopes
                .extend(envelopes.into_iter().filter(|e| !known.contains(&e.id)));
        } else {
            let ids: HashSet<&str> = envelopes.iter().map(|e| e.id.as_str()).collect();
            self.marked.retain(|id| ids.contains(id.as_str()));
            self.envelopes = envelopes;
        }
        self.arrange(keep);
    }

    /// Prepends envelopes not yet listed, returning how many were new.
    pub fn merge_new(&mut self, envelopes: Vec<Envelope>) -> usize {
        let keep = self.current().map(|e| e.id.clone());
        let known: HashSet<String> = self.envelopes.iter().map(|e| e.id.clone()).collect();
        let new: Vec<Envelope> = envelopes
            .into_iter()
            .filter(|e| !known.contains(&e.id))
            .collect();
        let n = new.len();
        self.envelopes.extend(new);
        self.arrange(keep);
        n
    }

    pub fn set_threaded(&mut self, on: bool) {
        self.threaded = on;
        let keep = self.current().map(|e| e.id.clone());
        self.arrange(keep);
    }

    /// The marked envelopes, else the selected one.
    pub fn targets(&self) -> Vec<String> {
        if self.marked.is_empty() {
            self.current()
                .map(|e| vec![e.id.clone()])
                .unwrap_or_default()
        } else {
            self.envelopes
                .iter()
                .filter(|e| self.marked.contains(&e.id))
                .map(|e| e.id.clone())
                .collect()
        }
    }

    pub fn toggle_mark(&mut self) {
        if let Some(id) = self.current().map(|e| e.id.clone())
            && !self.marked.remove(&id)
        {
            self.marked.insert(id);
        }
    }

    pub fn mark_all(&mut self) {
        if self.marked.len() == self.envelopes.len() {
            self.marked.clear();
        } else {
            self.marked = self.envelopes.iter().map(|e| e.id.clone()).collect();
        }
    }

    /// Drops envelopes, as after a move or delete.
    pub fn remove(&mut self, ids: &[String]) {
        let gone: HashSet<&String> = ids.iter().collect();
        let before = self.selected;
        self.envelopes.retain(|e| !gone.contains(&e.id));
        self.marked.retain(|id| !gone.contains(id));
        self.selected = before.min(self.envelopes.len().saturating_sub(1));
        let keep = self.current().map(|e| e.id.clone());
        self.arrange(keep);
    }

    fn arrange(&mut self, keep: Option<String>) {
        self.envelopes.sort_by_key(|e| std::cmp::Reverse(e.date));
        self.depth = vec![0; self.envelopes.len()];
        if self.threaded {
            let (order, depth) = threads(&self.envelopes);
            let mut slots: Vec<Option<Envelope>> = std::mem::take(&mut self.envelopes)
                .into_iter()
                .map(Some)
                .collect();
            self.envelopes = order.iter().map(|&i| slots[i].take().unwrap()).collect();
            self.depth = depth;
        }
        if let Some(id) = keep
            && let Some(i) = self.envelopes.iter().position(|e| e.id == id)
        {
            self.selected = i;
        }
        self.selected = self.selected.min(self.envelopes.len().saturating_sub(1));
    }
}

/// Orders newest-first envelopes into threads: each thread placed by its
/// latest message, replies under their parent oldest first.
fn threads(envelopes: &[Envelope]) -> (Vec<usize>, Vec<u8>) {
    let by_id: HashMap<&str, usize> = envelopes
        .iter()
        .enumerate()
        .filter_map(|(i, e)| Some((e.message_id.as_deref()?, i)))
        .collect();
    let parent: Vec<Option<usize>> = envelopes
        .iter()
        .map(|e| {
            e.in_reply_to
                .iter()
                .find_map(|id| by_id.get(id.as_str()).copied())
        })
        .collect();
    let mut children: Vec<Vec<usize>> = vec![vec![]; envelopes.len()];
    let mut roots = vec![];
    for (i, p) in parent.iter().enumerate() {
        match p {
            Some(p) if *p != i => children[*p].push(i),
            _ => roots.push(i),
        }
    }
    for c in &mut children {
        c.reverse(); // oldest reply first
    }
    // Input is newest first, so a root's latest activity is its smallest index.
    fn latest(i: usize, children: &[Vec<usize>]) -> usize {
        children[i]
            .iter()
            .map(|&c| latest(c, children))
            .min()
            .unwrap_or(i)
            .min(i)
    }
    roots.sort_by_key(|&r| latest(r, &children));
    let (mut order, mut depth) = (vec![], vec![]);
    fn walk(i: usize, d: u8, children: &[Vec<usize>], order: &mut Vec<usize>, depth: &mut Vec<u8>) {
        order.push(i);
        depth.push(d);
        for &c in &children[i] {
            walk(c, d.saturating_add(1), children, order, depth);
        }
    }
    for r in roots {
        walk(r, 0, &children, &mut order, &mut depth);
    }
    (order, depth)
}

pub fn short_date(date: Option<DateTime<FixedOffset>>) -> String {
    let Some(date) = date else {
        return String::new();
    };
    let date = date.with_timezone(&Local);
    let now = Local::now();
    let age = now.signed_duration_since(date);
    if date.date_naive() == now.date_naive() {
        date.format("%H:%M").to_string()
    } else if age.num_days() < 6 {
        date.format("%a %H:%M").to_string()
    } else if date.format("%Y").to_string() == now.format("%Y").to_string() {
        date.format("%b %e").to_string()
    } else {
        date.format("%Y-%m-%d").to_string()
    }
}

fn section(date: Option<DateTime<FixedOffset>>) -> &'static str {
    let Some(date) = date else { return "Undated" };
    let date = date.with_timezone(&Local).date_naive();
    let today = Local::now().date_naive();
    match (today - date).num_days() {
        ..=0 => "Today",
        1 => "Yesterday",
        2..=6 => "This week",
        7..=30 => "This month",
        _ => "Older",
    }
}

impl Widget for &mut EnvelopeList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        buf.set_style(area, t.base(t.bg));

        // Header: mailbox name, counts, marks, active search, thread view.
        let head = Rect {
            y: area.y + 1,
            height: 1,
            ..area
        }
        .inner(Margin::new(2, 0));
        let mut spans = vec![
            Span::styled(
                self.title.clone(),
                Style::new().fg(t.fg).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "  {}{}",
                    self.envelopes.len(),
                    if self.next_page.is_some() { "+" } else { "" }
                ),
                Style::new().fg(t.dim),
            ),
        ];
        let unread = self.unread();
        if unread > 0 {
            spans.push(Span::styled(
                format!(" · {unread} unread"),
                Style::new().fg(t.unread),
            ));
        }
        if !self.marked.is_empty() {
            spans.push(Span::styled(
                format!(" · {} marked", self.marked.len()),
                Style::new().fg(t.ok),
            ));
        }
        if let Some(q) = &self.query {
            spans.push(Span::styled("   ⌕ ", Style::new().fg(t.accent)));
            spans.push(Span::styled(
                format!(" {q} "),
                Style::new().fg(t.fg).bg(t.chip),
            ));
        }
        Line::from(spans).render(head, buf);
        if self.threaded {
            Line::styled("threads ", Style::new().fg(t.accent))
                .right_aligned()
                .render(head, buf);
        }

        let body = Rect {
            y: area.y + 3,
            height: area.height.saturating_sub(3),
            ..area
        };
        if self.envelopes.is_empty() {
            let msg = if self.loading {
                "Loading…"
            } else {
                "No messages"
            };
            Line::styled(msg, Style::new().fg(t.faint))
                .centered()
                .render(
                    Rect {
                        y: body.y + body.height / 3,
                        height: 1,
                        ..body
                    },
                    buf,
                );
            return;
        }

        // Lay out in lines: a band and a blank line before each date
        // section (not in the thread view), ROW lines per envelope, and a
        // loading line at the end while the next page loads.
        let mut tops = Vec::with_capacity(self.envelopes.len());
        let mut bands = vec![];
        let mut y = 0usize;
        let mut last = None;
        for e in &self.envelopes {
            let s = section(e.date);
            if !self.threaded && last != Some(s) {
                bands.push((y, s));
                y += 2;
                last = Some(s);
            }
            tops.push(y);
            y += ROW as usize;
        }
        let more_at = y;
        if self.next_page.is_some() {
            y += 2;
        }
        let total = y;
        let height = body.height as usize;
        let sel_top = tops[self.selected];
        let sel_from = if bands.iter().any(|(by, _)| by + 2 == sel_top) {
            sel_top - 2
        } else {
            sel_top
        };
        let sel_to = if self.selected + 1 == self.envelopes.len() {
            total
        } else {
            sel_top + ROW as usize
        };
        if sel_from < self.offset {
            self.offset = sel_from;
        } else if sel_to > self.offset + height {
            self.offset = sel_to - height;
        }
        let offset = self.offset;
        let visible = |top: usize, h: usize| top + h > offset && top < offset + height;
        let rect_at = |top: usize, h: u16| {
            let y = body.y as isize + top as isize - offset as isize;
            let y0 = y.max(body.y as isize) as u16;
            let y1 = (y + h as isize).min(body.bottom() as isize) as u16;
            Rect {
                y: y0,
                height: y1.saturating_sub(y0),
                ..body
            }
        };

        for &(top, name) in &bands {
            if visible(top, 1) {
                let rect = rect_at(top, 1);
                buf.set_style(rect, Style::new().bg(t.band));
                Line::styled(
                    format!("  {name}"),
                    Style::new().fg(t.faint).add_modifier(Modifier::BOLD),
                )
                .render(rect, buf);
            }
        }
        for (i, e) in self.envelopes.iter().enumerate() {
            if visible(tops[i], ROW as usize - 1) {
                let rect = rect_at(tops[i], ROW - 1);
                if rect.height == ROW - 1 {
                    let state = RowState {
                        selected: i == self.selected,
                        focused: self.focused,
                        marked: self.marked.contains(&e.id),
                        depth: self.depth.get(i).copied().unwrap_or(0),
                    };
                    render_row(e, state, rect, buf);
                }
            }
        }
        // The current section's band sticks to the top once scrolled past.
        if let Some(&(_, name)) = bands.iter().rev().find(|(top, _)| *top < offset) {
            let rect = Rect { height: 1, ..body };
            buf.set_style(rect, Style::new().bg(t.band));
            Line::styled(
                format!("  {name}"),
                Style::new().fg(t.faint).add_modifier(Modifier::BOLD),
            )
            .render(rect, buf);
        }
        if self.next_page.is_some() && visible(more_at, 1) {
            let msg = if self.loading_more {
                "Loading more…"
            } else {
                "More below"
            };
            Line::styled(msg, Style::new().fg(t.faint))
                .centered()
                .render(rect_at(more_at, 1), buf);
        }

        // A thin scrollbar on the right edge.
        if total > height && body.width > 2 {
            let x = body.right() - 1;
            let thumb = (height * height / total).max(1);
            let start = offset * height / total;
            for row in 0..height {
                let on = row >= start && row < start + thumb;
                buf[(x, body.y + row as u16)].set_symbol("▕").set_fg(if on {
                    t.dim
                } else {
                    t.band
                });
            }
        }
    }
}

#[derive(Clone, Copy)]
struct RowState {
    selected: bool,
    focused: bool,
    marked: bool,
    depth: u8,
}

fn render_row(e: &Envelope, s: RowState, area: Rect, buf: &mut Buffer) {
    let t = theme();
    let bg = match (s.selected, s.focused, s.marked) {
        (true, true, _) => t.selected,
        (true, false, _) | (false, _, true) => t.selected_dim,
        _ => t.bg,
    };
    let lit = s.selected || s.marked;
    buf.set_style(area, Style::new().bg(bg));
    let edge = match (s.selected && s.focused, s.marked) {
        (_, true) => Some(("▌", t.ok)),
        (true, false) => Some(("▌", t.accent)),
        _ => None,
    };
    if let Some((sym, color)) = edge {
        for y in area.top()..area.bottom() {
            buf[(area.x, y)].set_symbol(sym).set_fg(color);
        }
    }
    let indent = u16::from(s.depth.min(4)) * 2;
    let inner = area.inner(Margin::new(2, 0));
    let inner = Rect {
        x: inner.x + indent,
        width: inner.width.saturating_sub(indent),
        ..inner
    };
    let unread = !e.has("seen");
    let line1 = Rect { height: 1, ..inner };
    let line2 = Rect {
        y: inner.y + 1,
        height: 1,
        ..inner
    };

    let from = e
        .from
        .first()
        .map_or_else(|| "(unknown)".into(), |a| a.display_name().to_string());
    let bold = if unread {
        Modifier::BOLD
    } else {
        Modifier::empty()
    };
    let from_style = if lit || unread {
        Style::new().fg(t.fg).add_modifier(bold)
    } else {
        Style::new().fg(t.dim)
    };
    let lead = if s.marked {
        Span::styled("✓ ", Style::new().fg(t.ok))
    } else if unread {
        Span::styled("● ", Style::new().fg(t.unread))
    } else if s.depth > 0 {
        Span::styled("↳ ", Style::new().fg(if lit { t.dim } else { t.faint }))
    } else {
        Span::raw("  ")
    };
    Line::from(vec![lead, Span::styled(from, from_style)]).render(line1, buf);

    let mut right = vec![];
    if e.has_attachment == Some(true) {
        right.push(Span::styled("⎘ ", Style::new().fg(t.dim)));
    }
    if e.has("answered") {
        right.push(Span::styled("↩ ", Style::new().fg(t.dim)));
    }
    if e.has("flagged") {
        right.push(Span::styled("★ ", Style::new().fg(t.flagged)));
    }
    let date_fg = if lit {
        t.fg
    } else if unread {
        t.unread
    } else {
        t.faint
    };
    right.push(Span::styled(short_date(e.date), Style::new().fg(date_fg)));
    Line::from(right).right_aligned().render(line1, buf);

    let subject = e.subject.clone().unwrap_or_else(|| "(no subject)".into());
    let subject_style = Style::new().fg(if unread || lit { t.fg } else { t.dim });
    Line::from(vec![Span::raw("  "), Span::styled(subject, subject_style)]).render(line2, buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(id: &str, hours_ago: i64, mid: &str, parent: Option<&str>) -> Envelope {
        Envelope {
            id: id.into(),
            flags: Default::default(),
            subject: Some(id.into()),
            from: vec![],
            date: Some((Local::now() - chrono::Duration::hours(hours_ago)).fixed_offset()),
            has_attachment: None,
            message_id: Some(mid.into()),
            in_reply_to: parent.map(|p| vec![p.into()]).unwrap_or_default(),
        }
    }

    #[test]
    fn threads_group_replies_under_parents_newest_thread_first() {
        let mut list = EnvelopeList::default();
        list.load(
            vec![
                env("root", 30, "a", None),
                env("other", 10, "x", None),
                env("reply", 5, "b", Some("a")),
                env("reply2", 1, "c", Some("b")),
            ],
            false,
        );
        let ids = |l: &EnvelopeList| l.envelopes.iter().map(|e| e.id.clone()).collect::<Vec<_>>();
        assert_eq!(ids(&list), ["reply2", "reply", "other", "root"]);
        list.set_threaded(true);
        assert_eq!(ids(&list), ["root", "reply", "reply2", "other"]);
        assert_eq!(list.depth, [0, 1, 2, 0]);
    }

    #[test]
    fn targets_are_marks_or_the_selection() {
        let mut list = EnvelopeList::default();
        list.load(vec![env("1", 1, "a", None), env("2", 2, "b", None)], false);
        assert_eq!(list.targets(), ["1"]);
        list.move_by(1);
        list.toggle_mark();
        list.move_by(-1);
        assert_eq!(list.targets(), ["2"]);
    }
}
