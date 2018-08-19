use super::GenericTree;
use monoid::Monoid;

#[derive(Clone)]
pub struct SegmentTree<T> {
    nodes: Vec<T> // For efficiency and convenience we throw up the first emplacement of the vector
}

// Helpers for indexing
macro_rules! index {
    (left, $i: expr) => ($i << 1);
    (right, $i: expr) => (($i << 1) | 1);
    (parent, $i: expr) => ($i >> 1);
}

impl<T: Copy + Monoid> GenericTree<T> for SegmentTree<T> {
    fn nil() -> Self {
        unimplemented!();
    }

    fn with_root(_: T) -> Self {
        unimplemented!();
    }

    fn with_leaves(leaves: &[T]) -> Self {
        let length = leaves.len();
        let mut v = Vec::with_capacity(length << 1);

        // Pre-fill the vector with neutral element and append the leaves
        for _ in 0 .. length {
            v.push(T::m_empty());
        }
        v.extend_from_slice(leaves);

        // Compute internal nodes all the way up
        for i in (1 .. length).rev() {
            let left = v[index!(left, i)];
            let right = v[index!(right, i)];
            v[i] = left.m_append(&right);
        }

        SegmentTree { nodes: v }
    }

    fn root(&self) -> Option<T> {
        match self.nodes.len() {
            0 | 1 => None,
            _ => Some(self.nodes[1])
        }
    }
}

impl<T: Copy + Monoid> SegmentTree<T> {
    /// Query the segment tree in the range `left`-`right`
    pub fn query(&self, left: usize, right: usize) -> T {
        let half = self.nodes.len() >> 1;
        let mut left = half + left;
        let mut right = half + right;
        let mut acc = T::m_empty();

        while left <= right {
            if left & 1 == 1 {
                acc = acc.m_append(&self.nodes[left]);
                left += 1;
            }
            if right & 1 == 0 {
                acc = acc.m_append(&self.nodes[right]);
                right -= 1;
            }
            left = index!(parent, left);
            right = index!(parent, right);
        }

        acc
    }
}
