use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

// LRU Cache implementation
pub struct LRUCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    capacity: usize,
    cache: HashMap<K, V>,
    order: Vec<K>,
}

impl<K, V> LRUCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::with_capacity(capacity),
            order: Vec::with_capacity(capacity),
        }
    }

    pub fn get(&mut self, key: K) -> Option<V> {
        if let Some(value) = self.cache.get(&key) {
            if let Some(pos) = self.order.iter().position(|k| k == &key) {
                self.order.remove(pos);
            }
            self.order.push(key.clone());
            Some(value.clone())
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.cache.contains_key(&key) {
            self.cache.insert(key.clone(), value);
            if let Some(pos) = self.order.iter().position(|k| k == &key) {
                self.order.remove(pos);
            }
        } else {
            if self.cache.len() >= self.capacity {
                if let Some(lru_key) = self.order.first() {
                    self.cache.remove(lru_key);
                    self.order.remove(0);
                }
            }
            self.cache.insert(key.clone(), value);
        }
        self.order.push(key);
    }

    pub fn size(&self) -> usize {
        self.cache.len()
    }
}
