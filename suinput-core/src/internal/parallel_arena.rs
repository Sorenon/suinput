use itertools::Iterate;
use thunderdome::Index;

#[derive(Debug, Clone)]
pub struct ParallelArena<T> {
    storage: Vec<Option<T>>,
}

impl<T> ParallelArena<T> {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
        }
    }

    pub fn insert_at(&mut self, index: Index, entry: T) -> Option<T> {
        let index = index.slot() as usize;

        if self.storage.len() <= index {
            self.storage.resize_with(index + 1, || None);
            self.storage[index] = Some(entry);
            None
        } else {
            let mut entry = Some(entry);
            std::mem::swap(&mut entry, self.storage.get_mut(index).unwrap());
            entry
        }
    }

    pub fn get(&self, index: Index) -> Option<&T> {
        match self.storage.get(index.slot() as usize) {
            Some(entry) => entry.as_ref(),
            None => None,
        }
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        match self.storage.get_mut(index.slot() as usize) {
            Some(entry) => entry.as_mut(),
            None => None,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            inner: self.storage.iter_mut(),
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.storage.iter(),
        }
    }
}

pub struct Iter<'a, T> {
    inner: core::slice::Iter<'a, Option<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(entry) = self.inner.next()? {
                return Some(entry)
            }
        }
    }
}

pub struct IterMut<'a, T> {
    inner: core::slice::IterMut<'a, Option<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(entry) = self.inner.next()? {
                return Some(entry)
            }
        }
    }
}
