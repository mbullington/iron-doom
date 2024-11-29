use std::{collections::HashSet, hash::Hash};

pub struct ChangedSet<T: Eq + Hash> {
    _spawned: HashSet<T>,
    _changed: HashSet<T>,
    _removed: HashSet<T>,
}

impl<T: Eq + Hash> Default for ChangedSet<T> {
    fn default() -> Self {
        Self {
            _spawned: HashSet::new(),
            _changed: HashSet::new(),
            _removed: HashSet::new(),
        }
    }
}

impl<T: Eq + Hash> ChangedSet<T> {
    pub fn clear(&mut self) {
        self._spawned.clear();
        self._changed.clear();
        self._removed.clear();
    }

    pub fn spawn(&mut self, entity: T) {
        self._spawned.insert(entity);
    }

    pub fn spawned(&self) -> &HashSet<T> {
        &self._spawned
    }

    pub fn change(&mut self, entity: T) {
        self._changed.insert(entity);
    }

    pub fn changed(&self) -> &HashSet<T> {
        &self._changed
    }

    pub fn remove(&mut self, entity: T) {
        self._removed.insert(entity);
    }

    pub fn removed(&self) -> &HashSet<T> {
        &self._removed
    }
}
