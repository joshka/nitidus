//! The thread view's order: each thread placed by its latest message, replies under their parent
//! oldest first.

use std::collections::HashMap;

use crate::mail::Envelope;

/// Reorders newest-first `envelopes` into threads, returning each one's depth.
pub fn arrange(envelopes: &mut Vec<Envelope>) -> Vec<u8> {
    let children = children(envelopes);
    let mut order = Vec::with_capacity(envelopes.len());
    for root in roots(envelopes, &children) {
        walk(root, 0, &children, &mut order);
    }
    let mut slots: Vec<Option<Envelope>> = envelopes.drain(..).map(Some).collect();
    envelopes.extend(order.iter().filter_map(|&(index, _)| slots[index].take()));
    order.into_iter().map(|(_, depth)| depth).collect()
}

/// Each envelope's replies, oldest first.
fn children(envelopes: &[Envelope]) -> Vec<Vec<usize>> {
    let mut children = vec![Vec::new(); envelopes.len()];
    for (index, parent) in parents(envelopes).into_iter().enumerate() {
        if let Some(parent) = parent.filter(|&parent| parent != index) {
            children[parent].insert(0, index);
        }
    }
    children
}

/// Each envelope's parent among the listed ones, by `In-Reply-To`.
fn parents(envelopes: &[Envelope]) -> Vec<Option<usize>> {
    let by_id: HashMap<&str, usize> = envelopes
        .iter()
        .enumerate()
        .filter_map(|(index, envelope)| Some((envelope.message_id.as_deref()?, index)))
        .collect();
    let parent = |envelope: &Envelope| {
        envelope
            .in_reply_to
            .iter()
            .find_map(|id| by_id.get(id.as_str()).copied())
    };
    envelopes.iter().map(parent).collect()
}

/// The envelopes starting a thread, the thread with the latest message first.
fn roots(envelopes: &[Envelope], children: &[Vec<usize>]) -> Vec<usize> {
    let is_child: Vec<bool> = (0..envelopes.len())
        .map(|index| children.iter().any(|c| c.contains(&index)))
        .collect();
    let mut roots: Vec<usize> = (0..envelopes.len())
        .filter(|&index| !is_child[index])
        .collect();
    roots.sort_by_key(|&root| latest(root, children));
    roots
}

/// The newest message in a thread: the smallest index, as the input is newest first.
fn latest(index: usize, children: &[Vec<usize>]) -> usize {
    children[index]
        .iter()
        .map(|&child| latest(child, children))
        .fold(index, usize::min)
}

/// Lists a thread depth first, each message with its depth.
fn walk(index: usize, depth: u8, children: &[Vec<usize>], order: &mut Vec<(usize, u8)>) {
    order.push((index, depth));
    let next = depth.saturating_add(1);
    for &child in &children[index] {
        walk(child, next, children, order);
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Local};

    use super::*;

    fn envelope(id: &str, hours_ago: i64, parent: Option<&str>) -> Envelope {
        let date = (Local::now() - Duration::hours(hours_ago)).fixed_offset();
        let json = serde_json::json!({ "id": id, "date": date, "messageId": id, "inReplyTo": parent.into_iter().collect::<Vec<_>>() });
        serde_json::from_value(json).expect("an envelope")
    }

    #[test]
    fn replies_sit_under_their_parent_and_threads_order_by_latest() {
        let mut envelopes = vec![
            envelope("reply2", 1, Some("reply")),
            envelope("reply", 5, Some("root")),
            envelope("other", 10, None),
            envelope("root", 30, None),
        ];

        let depth = arrange(&mut envelopes);

        let ids: Vec<_> = envelopes.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(
            (ids, depth),
            (vec!["root", "reply", "reply2", "other"], vec![0, 1, 2, 0])
        );
    }
}
