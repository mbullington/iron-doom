use std::collections::HashSet;

#[derive(Default)]
pub struct ChangedSet {
    _spawned: HashSet<hecs::Entity>,
    _changed: HashSet<hecs::Entity>,
    _removed: HashSet<hecs::Entity>,
}

impl ChangedSet {
    pub fn clear(&mut self) {
        self._spawned.clear();
        self._changed.clear();
        self._removed.clear();
    }

    pub fn spawn(&mut self, entity: hecs::Entity) {
        self._spawned.insert(entity);
    }

    pub fn spawed(&self) -> &HashSet<hecs::Entity> {
        &self._spawned
    }

    pub fn change(&mut self, entity: hecs::Entity) {
        self._changed.insert(entity);
    }

    pub fn changed(&self) -> &HashSet<hecs::Entity> {
        &self._changed
    }

    pub fn remove(&mut self, entity: hecs::Entity) {
        self._removed.insert(entity);
    }

    pub fn removed(&self) -> &HashSet<hecs::Entity> {
        &self._removed
    }
}
