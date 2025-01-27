pub trait Evictable<T> {
    fn new(id: usize) -> T;

    fn pinned(&self) -> bool;

    fn id(&self) -> usize;
}

/// [Replacer] implements the page replacement policy.
/// The eviction policy modeled here is similar to
/// Postgres' CLOCK-sweep algorithm.
pub struct Replacer<T>
where
    T: Evictable<T>,
{
    size: usize,
    node_store: Vec<Option<T>>,
    ref_bits: Vec<u8>,
    ref_pos: usize,
}

impl<T> Replacer<T>
where
    T: PartialEq<T> + Clone,
    T: Evictable<T>,
{
    pub fn new(size: usize) -> Self {
        Self {
            size,
            node_store: vec![None; size],
            ref_bits: vec![0; size],
            ref_pos: 0,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn insert_and_evict(&mut self, node_id: usize) -> Option<T> {
        let new_node = Some(T::new(node_id));
        for (i, page) in self.node_store.iter().enumerate() {
            match page {
                Some(v) => {
                    if v.id() == node_id {
                        self.ref_pos = i;
                        self.ref_bits[i] += 1;
                        return None;
                    }
                }
                None => {
                    self.node_store[i] = new_node;
                    return None;
                }
            };
        }

        loop {
            let curr_index = self.ref_pos % self.size;
            // Advances the reference pointer to the next possible index
            self.ref_pos = curr_index + 1;
            if self.ref_bits[curr_index] > 0 {
                self.ref_bits[curr_index] -= 1;
            } else {
                let owned_node = &mut self.node_store[curr_index];
                let owned_node = owned_node.take().unwrap();

                if !owned_node.pinned() {
                    let evicted = std::mem::replace(&mut self.node_store[curr_index], new_node);
                    return evicted;
                }
            }
        }
    }
}

mod tests {
    use super::*;

    impl Evictable<u8> for u8 {
        fn new(id: usize) -> u8 {
            id as u8
        }

        fn pinned(&self) -> bool {
            false
        }

        fn id(&self) -> usize {
            *self as usize
        }
    }

    #[test]
    fn test_replacer() {
        let mut p = Replacer::<u8>::new(5);

        assert_eq!(p.size(), 5);
        // Inserting 5 elements in to Replacer
        assert_eq!(p.insert_and_evict(3), None);
        assert_eq!(p.insert_and_evict(4), None);
        assert_eq!(p.insert_and_evict(5), None);
        assert_eq!(p.insert_and_evict(6), None);
        assert_eq!(p.insert_and_evict(7), None);
        assert_eq!(
            p.node_store,
            vec![Some(3), Some(4), Some(5), Some(6), Some(7)]
        );
    }
}
