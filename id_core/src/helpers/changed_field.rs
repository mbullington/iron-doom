use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct ChangedField<T> {
    value: T,
    _changed: bool,
}

impl<T> ChangedField<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            _changed: false,
        }
    }

    pub fn changed(&self) -> bool {
        self._changed
    }

    pub fn clear_changed(&mut self) {
        self._changed = false;
    }
}

impl<T> Deref for ChangedField<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for ChangedField<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self._changed = true;
        &mut self.value
    }
}

impl<T> AsMut<T> for ChangedField<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}
