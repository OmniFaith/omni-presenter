use std::{collections::HashSet, hash::Hash};

#[derive(Clone, Debug)]
pub struct Selection<T> {
    pub selected: HashSet<T>,
    pub anchor: Option<T>,
}

impl<T> Default for Selection<T> {
    fn default() -> Self {
        Self {
            selected: HashSet::new(),
            anchor: None,
        }
    }
}

impl<T: Clone + Eq + Hash> Selection<T> {
    pub fn click(&mut self, id: T, order: &[T], range: bool, additive: bool) {
        select(
            &mut self.selected,
            &mut self.anchor,
            id,
            order,
            range,
            additive,
        );
    }

    pub fn clear(&mut self) {
        self.selected.clear();
        self.anchor = None;
    }

    pub fn retain(&mut self, order: &[T]) {
        self.selected.retain(|id| order.contains(id));
        if self.anchor.as_ref().is_some_and(|id| !order.contains(id)) {
            self.anchor = None;
        }
    }

    pub fn drag_ids(&self, id: &T, order: &[T]) -> Vec<T> {
        if self.selected.contains(id) {
            order
                .iter()
                .filter(|id| self.selected.contains(*id))
                .cloned()
                .collect()
        } else {
            vec![id.clone()]
        }
    }
}

pub fn select<T: Clone + Eq + Hash>(
    selected: &mut HashSet<T>,
    anchor: &mut Option<T>,
    id: T,
    order: &[T],
    range: bool,
    additive: bool,
) {
    if range
        && let Some((start, end)) = anchor.as_ref().and_then(|anchor| {
            Some((
                order.iter().position(|item| item == anchor)?,
                order.iter().position(|item| item == &id)?,
            ))
        }) {
            if !additive {
                selected.clear();
            }
            selected.extend(order[start.min(end)..=start.max(end)].iter().cloned());
            return;
        }
    if additive {
        if !selected.remove(&id) {
            selected.insert(id.clone());
        }
    } else {
        selected.clear();
        selected.insert(id.clone());
    }
    *anchor = Some(id);
}

/// Moves a group in its existing order. A target inside that group is a no-op.
pub fn move_items<T: Clone + PartialEq, K: Eq>(
    items: &mut Vec<T>,
    ids: &[K],
    target: &K,
    after: bool,
    key: impl Fn(&T) -> &K,
) -> bool {
    if ids.contains(target) || !items.iter().any(|item| key(item) == target) {
        return false;
    }
    let moved: Vec<_> = items
        .iter()
        .filter(|item| ids.contains(key(item)))
        .cloned()
        .collect();
    if moved.is_empty() {
        return false;
    }
    let mut remaining: Vec<_> = items
        .iter()
        .filter(|item| !ids.contains(key(item)))
        .cloned()
        .collect();
    let index = remaining
        .iter()
        .position(|item| key(item) == target)
        .unwrap()
        + usize::from(after);
    remaining.splice(index..index, moved);
    if *items == remaining {
        return false;
    }
    *items = remaining;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_moves_preserve_relative_order_in_both_directions() {
        let mut values = vec!["a", "b", "c", "d", "e"];
        assert!(move_items(&mut values, &["b", "d"], &"e", true, |id| id));
        assert_eq!(values, ["a", "c", "e", "b", "d"]);
        assert!(move_items(&mut values, &["b", "d"], &"a", false, |id| id));
        assert_eq!(values, ["b", "d", "a", "c", "e"]);
        assert!(!move_items(&mut values, &["b", "d"], &"d", true, |id| id));
    }

    #[test]
    fn selection_ranges_follow_visible_order_and_drag_preserves_selection() {
        let mut selection = Selection::default();
        let order = [5, 2, 9, 1];
        selection.click(2, &order, false, false);
        selection.click(1, &order, true, false);
        assert_eq!(selection.drag_ids(&9, &order), [2, 9, 1]);
        assert_eq!(selection.drag_ids(&5, &order), [5]);
        selection.retain(&[5, 9]);
        assert_eq!(selection.selected, [9].into_iter().collect());
    }
}
