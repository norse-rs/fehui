use super::*;
use std::cell::UnsafeCell;

const CHUNK_SIZE: usize = 1; // TODO

#[derive(Derivative)]
#[derivative(Debug)]
struct Chunk<T> {
    data: Box<[T; CHUNK_SIZE]>,
    len: usize,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Storage<T> {
    chunks: UnsafeCell<Vec<Chunk<T>>>,
    num_elements: UnsafeCell<usize>,
}

impl<T> Storage<T> {
    pub fn new() -> Self {
        Storage {
            chunks: UnsafeCell::new(Vec::new()),
            num_elements: UnsafeCell::new(0),
        }
    }

    pub fn clear(&mut self) {
        let chunks = unsafe { &mut *self.chunks.get() };
        let num_elems = unsafe { &mut *self.num_elements.get() };

        chunks.clear();
        *num_elems = 0;
    }

    pub fn push(&self, value: T) -> usize {
        let chunks = unsafe { &mut *self.chunks.get() };
        let num_elems = unsafe { &mut *self.num_elements.get() };
        let id = *num_elems;
        let chunk_id = id / CHUNK_SIZE;
        let data_id = id % CHUNK_SIZE;

        if chunk_id >= chunks.len() {
            chunks.push(Chunk {
                data: Box::new(unsafe { std::mem::uninitialized() }),
                len: 0,
            });
        }

        std::mem::forget(std::mem::replace(
            &mut chunks[chunk_id].data[data_id],
            value,
        ));
        chunks[chunk_id].len += 1;
        *num_elems += 1;

        id
    }

    pub fn len(&self) -> usize {
        unsafe { *self.num_elements.get() }
    }

    pub fn get(&self, idx: usize) -> &T {
        let chunks = unsafe { &*self.chunks.get() };
        let chunk_id = idx / CHUNK_SIZE;
        let data_id = idx % CHUNK_SIZE;

        &chunks[chunk_id].data[data_id]
    }

    pub fn get_mut(&self, idx: usize) -> &mut T {
        let chunks = unsafe { &mut *self.chunks.get() };
        let chunk_id = idx / CHUNK_SIZE;
        let data_id = idx % CHUNK_SIZE;

        &mut chunks[chunk_id].data[data_id]
    }
}

pub type Widgets = Storage<Box<Widget>>;

impl std::ops::Index<WidgetId> for Widgets {
    type Output = Box<Widget>;
    fn index(&self, idx: WidgetId) -> &Self::Output {
        self.get(idx.0)
    }
}

pub type Keys = storage::Storage<Option<Key>>;

impl std::ops::Index<WidgetId> for Keys {
    type Output = Option<Key>;
    fn index(&self, idx: WidgetId) -> &Self::Output {
        self.get(idx.0)
    }
}

impl std::ops::IndexMut<WidgetId> for Keys {
    fn index_mut(&mut self, idx: WidgetId) -> &mut Self::Output {
        self.get_mut(idx.0)
    }
}
