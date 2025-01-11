/// [Replacer] implements the page replacement policy.
/// CLOCK is used for this eviction policy.
struct Replacer<T> {
    size: usize,
    node_store: Vec<Option<T>>,
    ref_bits: Vec<bool>,
    ref_pos: usize,
}

impl<T> Replacer<T>
where
    T: PartialEq<T> + Clone,
{
    pub fn new(size: usize) -> Self {
        Self {
            size,
            node_store: vec![None; size],
            ref_bits: vec![false; size],
            ref_pos: 0,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn get(&mut self, node: T) -> T {
        let value = &node;
        let mut new_node = Some(node.clone());
        for (i, page) in self.node_store.iter().enumerate() {
            match page {
                Some(v) => {
                    if v == value {
                        self.ref_pos = i;
                        self.ref_bits[i] = true;
                        return value.to_owned();
                    }
                }
                None => {
                    self.node_store[i] = new_node;
                    self.ref_bits[i] = false;
                    return value.to_owned();
                }
            };
        }

        loop {
            let curr_index = self.ref_pos % self.size;
            // Advances the reference pointer to the next possible index
            self.ref_pos = curr_index + 1;
            if self.ref_bits[curr_index] {
                self.ref_bits[curr_index] = false;
            } else {
                std::mem::swap(&mut new_node, &mut self.node_store[curr_index]);
                self.ref_bits[curr_index] = false;
                return value.to_owned();
            }
        }
    }
}

mod tests {
    use super::*;
    #[test]
    fn test_replacer() {
        let mut p = Replacer::<u8>::new(5);

        assert_eq!(p.size(), 5);
        // Inserting 5 elements in to Replacer
        assert_eq!(p.get(3), 3);
        assert_eq!(p.ref_bits, vec![false, false, false, false, false]);
        assert_eq!(p.get(4), 4);
        assert_eq!(p.ref_bits, vec![false, false, false, false, false]);
        assert_eq!(p.get(5), 5);
        assert_eq!(p.ref_bits, vec![false, false, false, false, false]);
        assert_eq!(p.get(6), 6);
        assert_eq!(p.ref_bits, vec![false, false, false, false, false]);
        assert_eq!(p.get(7), 7);
        assert_eq!(p.ref_bits, vec![false, false, false, false, false]);
        assert_eq!(
            p.node_store,
            vec![Some(3), Some(4), Some(5), Some(6), Some(7)]
        );

        // Insert some new data
        assert_eq!(p.get(8), 8);
        assert_eq!(p.ref_bits, vec![false, false, false, false, false]);
        assert_eq!(
            p.node_store,
            vec![Some(8), Some(4), Some(5), Some(6), Some(7)]
        );
        assert_eq!(p.get(9), 9);
        assert_eq!(
            p.node_store,
            vec![Some(8), Some(9), Some(5), Some(6), Some(7)]
        );
        assert_eq!(p.get(10), 10);
        assert_eq!(
            p.node_store,
            vec![Some(8), Some(9), Some(10), Some(6), Some(7)]
        );
        assert_eq!(p.get(11), 11);
        assert_eq!(
            p.node_store,
            vec![Some(8), Some(9), Some(10), Some(11), Some(7)]
        );
        assert_eq!(p.get(12), 12);
        assert_eq!(
            p.node_store,
            vec![Some(8), Some(9), Some(10), Some(11), Some(12)]
        );

        // Check one last wrap around with the reference bit pointer
        assert_eq!(p.get(13), 13);
        assert_eq!(
            p.node_store,
            vec![Some(13), Some(9), Some(10), Some(11), Some(12)]
        );

        // Check to ensure that we fill ref_bits
        assert_eq!(p.get(11), 11);
        assert_eq!(p.ref_bits, vec![false, false, false, true, false]);
        assert_eq!(p.get(13), 13);
        assert_eq!(p.ref_bits, vec![true, false, false, true, false]);
        assert_eq!(p.get(9), 9);
        assert_eq!(p.ref_bits, vec![true, true, false, true, false]);
        assert_eq!(p.get(10), 10);
        assert_eq!(p.ref_bits, vec![true, true, true, true, false]);
        assert_eq!(p.get(12), 12);
        assert_eq!(p.ref_bits, vec![true, true, true, true, true]);

        assert_eq!(p.get(13), 13);
        assert_eq!(p.ref_bits, vec![true, true, true, true, true]);

        assert_eq!(p.get(20), 20);
        assert_eq!(p.ref_bits, vec![false, false, false, false, false]);
        assert_eq!(p.get(21), 21);
        assert_eq!(p.ref_bits, vec![false, false, false, false, false]);
        assert_eq!(p.get(22), 22);
        assert_eq!(p.ref_bits, vec![false, false, false, false, false]);
        assert_eq!(p.get(21), 21);
        assert_eq!(p.ref_bits, vec![false, true, false, false, false]);
        assert_eq!(
            p.node_store,
            vec![Some(20), Some(21), Some(22), Some(11), Some(12)]
        );
        assert_eq!(p.get(23), 23);
        assert_eq!(p.ref_bits, vec![false, false, false, false, false]);
        // ref_bit is set to 2 for the pointer
        assert_eq!(
            p.node_store,
            vec![Some(20), Some(21), Some(23), Some(11), Some(12)]
        );
    }
}
