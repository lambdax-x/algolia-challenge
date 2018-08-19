pub struct MinHeap<T> {
    nodes: Vec<T>  
}

// Helpers for indexing
macro_rules! index {
    (root) => (0);
    (parent, $i: expr) => (($i - 1) >> 1);
    (left, $i: expr) => (($i << 1) | 1);
    (right, $i: expr) => (($i + 1) << 1);
}

impl<T: Copy + Ord> MinHeap<T> {
    pub fn new() -> Self {
        MinHeap {
            nodes: Vec::new()
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn peek(&self) -> Option<&T> {
        self.nodes.get(index!(root))
    }

    pub fn insert(&mut self, element: T) {
        let element_index = self.nodes.len();
        self.nodes.push(element);
        self.heapify_up(element_index);
    }

    pub fn extract(&mut self) -> Option<T> {
        match self.len() {
            0 => None,
            1 => self.nodes.pop(),
            _ => {
                let element = self.nodes.get(index!(root))
                                        .unwrap()
                                        .clone();
                let new_head = self.nodes.pop().unwrap();
                self.nodes[index!(root)] = new_head;
                self.heapify_down(index!(root));
                Some(element)
            }
        }
    }

    fn heapify_down(&mut self, from: usize) {
        let length = self.nodes.len();
        let left_index = index!(left, from);
        let right_index = index!(right, from);

        let mut min_index = from;

        if left_index < length && self.nodes[left_index] < self.nodes[min_index] {
            min_index = left_index;
        }

        if right_index < length && self.nodes[right_index] < self.nodes[min_index] {
            min_index = right_index;
        }

        if from != min_index {
            self.nodes.swap(from, min_index);
            self.heapify_down(min_index);
        }
    }

    fn heapify_up(&mut self, from: usize) {
        if from == index!(root) {
            return;
        }

        let parent = index!(parent, from);
        if self.nodes[from] < self.nodes[parent] {
            self.nodes.swap(from, parent);
            self.heapify_up(parent);
        }
    }
}

impl<T> IntoIterator for MinHeap<T> {
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}
