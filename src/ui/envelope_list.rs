//! The envelope list's state: the envelopes in display order, the selection, marks, paging and
//! the thread view. [`view`] draws it.

mod layout;
mod row;
mod threads;
mod view;

use std::collections::HashSet;

use super::layout::PaneFocus;
use super::movement::{Movement, Navigate};
use crate::mail::{Envelope, Keyword};

/// The envelope list.
#[derive(Default)]
pub struct EnvelopeList {
    /// The envelopes in display order: newest first, or by thread.
    envelopes: Vec<Envelope>,

    /// Each envelope's thread depth, all zero outside the thread view.
    depth: Vec<u8>,

    /// The index of the selected envelope.
    selected: usize,

    /// The first line drawn, in the layout's lines.
    offset: usize,

    /// Whether keys move the list's selection.
    pub focus: PaneFocus,

    /// The mailbox's name, shown in the header.
    pub title: String,

    /// The active search, shown in the header.
    pub search: Option<String>,

    /// What is loading, if anything.
    pub loading: Loading,

    /// The ids of marked envelopes, which actions act on instead of the selection.
    marked: HashSet<String>,

    /// Whether replies are grouped under their parents.
    threaded: bool,

    /// The next page to load, `None` once the mailbox is complete.
    pub next_page: Option<usize>,
}

/// What the list is waiting for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Loading {
    /// Nothing.
    #[default]
    Idle,

    /// The first page, the list empty until it arrives.
    FirstPage,

    /// The next page, appended when it arrives.
    NextPage,
}

impl EnvelopeList {
    /// An empty list titled `title`, loading its first page.
    pub fn loading(title: String) -> Self {
        Self {
            title,
            loading: Loading::FirstPage,
            ..Self::default()
        }
    }

    /// The selected envelope.
    pub fn current(&self) -> Option<&Envelope> {
        self.envelopes.get(self.selected)
    }

    /// How many envelopes are listed.
    pub fn len(&self) -> usize {
        self.envelopes.len()
    }

    /// How many listed envelopes are unread.
    pub fn unread(&self) -> usize {
        self.envelopes
            .iter()
            .filter(|envelope| !envelope.has(Keyword::Seen))
            .count()
    }

    /// Whether the selection is within a few rows of the end, where the next page should load.
    pub fn near_end(&self) -> bool {
        self.selected + 6 >= self.envelopes.len()
    }

    /// Replaces every envelope with a first page, keeping marks still listed.
    pub fn replace(&mut self, envelopes: Vec<Envelope>) {
        let listed: HashSet<&str> = envelopes
            .iter()
            .map(|envelope| envelope.id.as_str())
            .collect();
        self.marked.retain(|id| listed.contains(id.as_str()));
        self.envelopes = envelopes;
        self.selected = 0;
        self.offset = 0;
        self.arrange(None);
    }

    /// Adds envelopes not yet listed (a later page, or new mail), answering how many were new.
    pub fn add(&mut self, envelopes: Vec<Envelope>) -> usize {
        let keep = self.current().map(|envelope| envelope.id.clone());
        let known: HashSet<String> = self
            .envelopes
            .iter()
            .map(|envelope| envelope.id.clone())
            .collect();
        let before = self.envelopes.len();
        self.envelopes.extend(
            envelopes
                .into_iter()
                .filter(|envelope| !known.contains(&envelope.id)),
        );
        let added = self.envelopes.len() - before;
        self.arrange(keep);
        added
    }

    /// Drops envelopes, as after a move or delete, keeping the selection's position.
    pub fn remove(&mut self, ids: &[String]) {
        self.envelopes
            .retain(|envelope| !ids.contains(&envelope.id));
        self.marked.retain(|id| !ids.contains(id));
        self.selected = self.selected.min(self.envelopes.len().saturating_sub(1));
        let keep = self.current().map(|envelope| envelope.id.clone());
        self.arrange(keep);
    }

    /// Applies `edit` to every listed envelope among `ids`.
    pub fn edit(&mut self, ids: &[String], edit: impl Fn(&mut Envelope)) {
        self.envelopes
            .iter_mut()
            .filter(|envelope| ids.contains(&envelope.id))
            .for_each(edit);
    }

    /// Whether every listed envelope among `ids` carries `keyword`.
    pub fn all_have(&self, ids: &[String], keyword: Keyword) -> bool {
        let mut targets = self
            .envelopes
            .iter()
            .filter(|envelope| ids.contains(&envelope.id));
        targets.all(|envelope| envelope.has(keyword))
    }

    /// The ids actions act on: the marked envelopes, else the selected one.
    pub fn targets(&self) -> Vec<String> {
        if self.marked.is_empty() {
            return self
                .current()
                .map_or_default(|envelope| vec![envelope.id.clone()]);
        }
        let marked = self
            .envelopes
            .iter()
            .filter(|envelope| self.marked.contains(&envelope.id));
        marked.map(|envelope| envelope.id.clone()).collect()
    }

    /// Whether any envelope is marked.
    pub fn has_marks(&self) -> bool {
        !self.marked.is_empty()
    }

    /// Marks or unmarks the selected envelope.
    pub fn toggle_mark(&mut self) {
        let Some(id) = self.current().map(|envelope| envelope.id.clone()) else {
            return;
        };
        if !self.marked.remove(&id) {
            self.marked.insert(id);
        }
    }

    /// Marks every envelope, or clears the marks when all are marked.
    pub fn toggle_mark_all(&mut self) {
        if self.marked.len() == self.envelopes.len() {
            self.marked.clear();
        } else {
            self.marked = self
                .envelopes
                .iter()
                .map(|envelope| envelope.id.clone())
                .collect();
        }
    }

    /// Clears every mark.
    pub fn clear_marks(&mut self) {
        self.marked.clear();
    }

    /// Switches between the date and thread views, answering whether threads are now shown.
    pub fn toggle_threads(&mut self) -> bool {
        self.threaded = !self.threaded;
        let keep = self.current().map(|envelope| envelope.id.clone());
        self.arrange(keep);
        self.threaded
    }

    /// Puts the envelopes in display order and reselects `keep` when given.
    fn arrange(&mut self, keep: Option<String>) {
        self.envelopes
            .sort_by_key(|envelope| std::cmp::Reverse(envelope.date));
        self.depth = if self.threaded {
            threads::arrange(&mut self.envelopes)
        } else {
            vec![0; self.envelopes.len()]
        };
        if let Some(index) =
            keep.and_then(|id| self.envelopes.iter().position(|envelope| envelope.id == id))
        {
            self.selected = index;
        }
    }
}

impl Navigate for EnvelopeList {
    fn navigate(&mut self, movement: Movement) -> bool {
        let before = self.selected;
        self.selected = movement.index(self.selected, self.envelopes.len());
        self.selected != before
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Local};

    use super::*;

    fn envelope(id: &str, hours_ago: i64) -> Envelope {
        let date = (Local::now() - Duration::hours(hours_ago)).fixed_offset();
        let json = serde_json::json!({ "id": id, "date": date, "messageId": id });
        serde_json::from_value(json).expect("an envelope")
    }

    #[test]
    fn actions_target_the_marks_or_else_the_selection() {
        let mut list = EnvelopeList::default();
        list.replace(vec![envelope("1", 1), envelope("2", 2)]);
        let unmarked = list.targets();

        list.navigate(Movement::Down);
        list.toggle_mark();
        list.navigate(Movement::Up);

        assert_eq!(
            (unmarked, list.targets()),
            (vec!["1".to_owned()], vec!["2".to_owned()])
        );
    }

    #[test]
    fn a_later_page_adds_only_new_envelopes() {
        let mut list = EnvelopeList::default();
        list.replace(vec![envelope("1", 1)]);

        let added = list.add(vec![envelope("1", 1), envelope("2", 2)]);

        assert_eq!((added, list.len()), (1, 2));
    }
}
