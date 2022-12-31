use std::collections::{HashMap, VecDeque};

pub struct LRU<K, V> {
    map: HashMap<K, V>,
    list: VecDeque<V>,
    capacity: usize,
}

impl<K, V> LRU<K, V>
where
    K: std::cmp::Eq + std::hash::Hash + Clone,
    V: Clone + PartialEq,
{
    /// Creates a new [`LRU<K, V>`].
    /// `K` is a key that must be unique.
    /// `V` is a value that can be duplicated.
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            list: VecDeque::new(),
            capacity,
        }
    }

    /// Gets the value corresponding to the key.
    /// If the key is not in the map, returns `None`.
    /// If the key is in the map, returns a reference to the value.
    /// If the key is in the map, moves the value to the front of the list.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let val = self.map.get(key)?;
        // check if the value is already at the front of the list
        if self.list.front() != Some(val) {
            self.list.retain(|v| v != val);
            self.list.push_front(val.clone());
        }
        Some(val)
    }

    /// Inserts a key-value pair into the map.
    /// If the key is already in the map, moves the value to the front of the list.
    /// # Panics
    ///
    /// Panics if the list is empty.
    pub fn insert(&mut self, key: K, val: V) {
        // check if the key is already in the map
        if self.map.contains_key(&key) {
            // if it is, remove it from the list and add it to the front
            self.list.retain(|v| v != &val);
            self.list.push_front(val.clone());
            self.map.insert(key, val);
        }

        // check if the list is at capacity
        if self.list.len() == self.capacity {
            // if it is, remove the last element from the list and the map
            let last = self.list.pop_back().expect("list is empty");
            self.map.retain(|_, v| v != &last);
        }
    }
}
