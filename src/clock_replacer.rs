/// [Replacer] implements the page replacement policy.
/// CLOCK is used for this eviction policy.
struct Replacer<T> {
    size: usize,
    node_store: Vec<Option<ReplacerNode<T>>>,
    ref_bits: Vec<bool>,
    ref_pos: usize,
}

struct ReplacerNode<T> {
    data: T,
}

impl<T> Replacer<T> {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            node_store: vec![None; size],
            ref_bits: vec![false; size],
            ref_pos: 0,
        }
    }

    pub fn total_size(&self) -> usize { self.size }

    pub fn current_size(&self) -> usize { self.size }

    pub fn record_access(&mut self, node: T) {
        for (page, i) in self.node_store.iter().enumerate() {
            if page.is_some_and(|node| node.data == node) {
                self.ref_bits[]
            }
        }
    }

    pub fn evict(&mut self) {}

    pub fn set_evictable(&mut self) {}

    fn first_unset_bit() {}
}


mod tests {
    use crate::clock_replacer::Replacer;

    #[test]
    fn test_replacer_simple() {
        let mut p = Replacer::<u8>::new(5);
    }
}
