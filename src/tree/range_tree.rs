use std::boxed::Box;
use super::GenericTree;

#[derive(Clone)]
pub struct RangeTree<T> {
    root: Option<Box<Node<T>>>
}

#[derive(Clone)]
struct Node<T> {
    value: T,
    left: RangeTree<T>,
    right: RangeTree<T>
}

impl<T: Copy> GenericTree<T> for RangeTree<T> {
    fn nil() -> Self {
        RangeTree { root: None }
    }

    fn with_root(root: T) -> Self {
        RangeTree {
            root: Some(Box::new(Node {
                value: root,
                left: RangeTree::nil(),
                right: RangeTree:: nil()
            }))
        }
    }

    /// Create the range tree given a sorted array of leaves.
    fn with_leaves(leaves: &[T]) -> Self {
        match leaves.len() {
            0 => RangeTree::nil(),
            1 => RangeTree::with_root(leaves[0]),
            length => {
                let (left_leaves, right_leaves) = leaves.split_at(length >> 1);
                let left_tree = RangeTree::with_leaves(left_leaves);
                let right_tree = RangeTree::with_leaves(right_leaves);
                RangeTree {
                    root: Some(Box::new(Node {
                        value: *left_leaves.last().unwrap(),
                        left: left_tree,
                        right: right_tree
                    }))
                }
            }
        }
    }

    fn root(&self) -> Option<T> {
        match self.root {
            None => None,
            Some(ref boxed_node) => Some(boxed_node.value)
        }
    }
}

impl<T: Copy + Ord> RangeTree<T> {
    /// Find the first sub-tree in which the given range fits.
    /// Requirement: `from <= to`
    fn find_split_node(&self, from: &T, to: &T) -> Option<&Self> {
        match self.root {
            None => None,

            // [ ... from <= to <= root ... ]: explore left sub-tree
            Some(ref boxed_node) if to <= &boxed_node.value => {
                let maybe_left = boxed_node.left.find_split_node(from, to);
                if maybe_left.is_none() {
                    return Some(self);
                }
                maybe_left
            },

            // [ ... root < from <= to ... ]: explore right sub-tree
            Some(ref boxed_node) if from > &boxed_node.value => {
                let maybe_right = boxed_node.right.find_split_node(from, to);
                if maybe_right.is_none() {
                    return Some(self);
                }
                maybe_right
            },

            // [ ... from <= root < to ...]: split node found
            Some(_) => {
                Some(self)
            }
        }
    }

    /// Find the smallest `bound >= from`
    fn left_bound(&self, from: &T) -> Option<T> {
        match self.root {
            None => None,

            Some(ref boxed_node) if &boxed_node.value >= from => {
                let maybe_left = boxed_node.left.left_bound(from);
                if maybe_left.is_none() {
                    return Some(boxed_node.value);
                }
                maybe_left
            },

            Some(ref boxed_node) => {
                let maybe_left = boxed_node.right.left_bound(from);
                if maybe_left.is_none() {
                    if &boxed_node.value < from {
                        return None;
                    }
                    return Some(boxed_node.value);
                }
                maybe_left
            }
        }
    }

    /// Find the largest bound `bound <= to`
    fn right_bound(&self, to: &T) -> Option<T> {
        match self.root {
            None => None,

            Some(ref boxed_node) if &boxed_node.value < to => {
                let maybe_right = boxed_node.right.right_bound(to);
                if maybe_right.is_none() {
                    return Some(boxed_node.value);
                }
                maybe_right
            },

            Some(ref boxed_node) => {
                let maybe_right = boxed_node.left.right_bound(to);
                if maybe_right.is_none() {
                    if &boxed_node.value > to {
                        return None;
                    }
                    return Some(boxed_node.value);
                }
                maybe_right
            }
        }
    }

    /// Find the largest range included in the given range [from ; to[
    pub fn largest_range(&self, from: &T, to: &T) -> Option<(T, T)> {
        if from > to {
            return None;
        }
        self.find_split_node(from, to)
            .map(|tree| {
                let maybe_left = tree.left_bound(from);
                let maybe_right = tree.right_bound(to);
                match (maybe_left, maybe_right) {
                    (Some(left), Some(right)) => Some((left, right)),
                    _ => None
                }
            })
            .unwrap_or(None)
    }
}
