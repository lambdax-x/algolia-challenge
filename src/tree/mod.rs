/// A generic binary tree is either the nil value, or an allocated node which has a value, a left
/// tree and a right tree.
pub trait GenericTree<T: Copy> {
    /// Create an empty tree.
    fn nil() -> Self;

    /// Create a tree containing one element
    fn with_root(root: T) -> Self;

    /// Create a tree from its leaves.
    fn with_leaves(leaves: &[T]) -> Self;

    /// Clone the value contained in the root
    fn root(&self) -> Option<T>;
}

pub mod range_tree;
pub mod segment_tree;
pub mod heap;
