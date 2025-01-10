/// [Replacer] implements the page replacement policy. This is a custom
/// built replacer algorithm that takes inspiration from second chance
/// algorithms. It uses a bitmask to check for eviction.  
struct Replacer<T> {
    size: usize,
    curr_size: usize,
    node_store: Vec<Option<ReplacerNode<T>>>,
    set_bitmask: usize,
}

struct ReplacerNode<T> {
    data: T,
}

impl<T> Replacer<T> {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            curr_size: 0,
            node_store: vec![None; size],
            set_bitmask: 0,
        }
    }

    pub fn total_size(&self) -> usize { self.size }

    pub fn current_size(&self) -> usize { self.size }

    pub fn evict(&mut self) {}

    pub fn set_evictable(&mut self) {}
}


mod tests {
}
