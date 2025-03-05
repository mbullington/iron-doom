pub mod egui;
pub mod gpu;
pub mod system;
pub mod window;

mod movement_controller;

use std::collections::HashMap;

pub use movement_controller::*;

/// [SparseVec] is just a wrapper for a HashMap, but is used to buffer
/// writes to GPU buffers.
pub struct SparseVec<T> {
    items: HashMap<usize, T>,
}

impl<T> Default for SparseVec<T> {
    fn default() -> Self {
        Self {
            items: Default::default(),
        }
    }
}

impl<T> SparseVec<T> {
    pub fn insert(&mut self, index: usize, item: T) {
        self.items.insert(index, item);
    }
}

/// Returns an iterator of (starting index, items).
///
/// For example:
/// > 2: a, 3: b, 5: c
///
/// Would return:
/// > (2, [a, b]), (5, c)
///
/// This is useful for writing to GPU buffers, where we want to minimize
/// the number of writes.
impl<T> Iterator for SparseVec<T> {
    type Item = (usize, Vec<T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.items.is_empty() {
            return None;
        }

        let mut keys = self.items.keys().cloned().collect::<Vec<_>>();
        keys.sort_unstable();

        let mut start = *keys.first().unwrap();
        let mut items = Vec::new();

        for key in keys {
            if key == start {
                items.push(self.items.remove(&key).unwrap());
            } else {
                break;
            }

            start += 1;
        }

        Some((start - items.len(), items))
    }
}
