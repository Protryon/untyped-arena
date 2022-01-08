#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::{
    cell::{Cell, RefCell},
    ffi::c_void,
    marker::PhantomData,
};

const INITIAL_SIZE: usize = 1024;

/// An untyped arena. `'a` is a lifetime used to constrain allocated references to allow the drop checker to prevent undefined behavior with some drop implementations
pub struct Arena<'a> {
    inner: RefCell<ChunkList>,
    _p: PhantomData<Cell<&'a ()>>,
}

type Dropper = fn(*mut c_void);

struct ChunkList {
    current_chunk: Vec<u8>,
    chunks: Vec<Vec<u8>>,
    drops: Vec<(*mut c_void, Dropper)>,
}

const PADDING: [u8; 16] = [0u8; 16];

#[inline]
fn pad_len(len: usize, align: usize) -> usize {
    let out = len % align;
    if out == 0 {
        0
    } else {
        align - out
    }
}

impl<'a> Arena<'a> {
    /// Creates a new, empty [`Arena`]
    pub fn new() -> Self {
        Self::with_capacity(INITIAL_SIZE)
    }

    /// Creates a new, empty [`Arena`] with a capacity of [`capacity`] bytes
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: RefCell::new(ChunkList {
                current_chunk: Vec::with_capacity(
                    capacity
                        .checked_next_power_of_two()
                        .expect("arena capacity overflow"),
                ),
                chunks: Vec::new(),
                drops: Vec::new(),
            }),
            _p: PhantomData,
        }
    }

    /// Moves [`item`] into the Arena, allocating space for it as needed.
    pub fn alloc<T: 'a>(&self, item: T) -> &mut T {
        let mut inner = self.inner.borrow_mut();
        let size = core::mem::size_of::<T>();
        let alignment = core::mem::align_of::<*const ()>();
        let raw_item = unsafe { core::slice::from_raw_parts(&item as *const T as *const u8, size) };

        let padding = pad_len(inner.current_chunk.len(), alignment);

        let remaining = inner.current_chunk.capacity() - inner.current_chunk.len();
        if remaining < padding + size {
            inner.reserve(padding + size)
        }
        let out_ptr = {
            inner.current_chunk.extend_from_slice(&PADDING[..padding]);
            let index = inner.current_chunk.len();
            inner.current_chunk.extend_from_slice(raw_item);
            let out = unsafe { inner.current_chunk.as_mut_ptr().offset(index as isize) as *mut T };
            unsafe { out.as_mut().unwrap() }
        };
        core::mem::forget(item);

        inner
            .drops
            .push((out_ptr as *mut T as *mut c_void, |x: *mut c_void| unsafe {
                core::ptr::drop_in_place(x as *mut T)
            }));
        out_ptr
    }

    /// Reserves [`capacity`] bytes in the arena for future allocations.
    pub fn reserve(&self, capacity: usize) {
        let mut inner = self.inner.borrow_mut();
        inner.reserve(capacity)
    }

    /// Reserves enough space for [`count`] aligned spaces of [`T`]
    pub fn reserve_type<T>(&self, count: usize) {
        let mut inner = self.inner.borrow_mut();
        inner.reserve_type::<T>(count)
    }
}

impl ChunkList {
    fn reserve(&mut self, capacity: usize) {
        let next_chunk_size = (self
            .current_chunk
            .capacity()
            .checked_mul(2)
            .expect("arena capacity overflow"))
        .max(
            capacity
                .checked_next_power_of_two()
                .expect("arena capacity overflow"),
        );
        let new_chunk = Vec::with_capacity(next_chunk_size);
        let old_chunk = core::mem::replace(&mut self.current_chunk, new_chunk);

        // this is not guaranteed to not move the vec buffer, unfortunately
        // old_chunk.shrink_to_fit();

        self.chunks.push(old_chunk);
    }

    fn reserve_type<T>(&mut self, count: usize) {
        let size = core::mem::size_of::<T>();
        let alignment = core::mem::align_of::<*const ()>();

        let initial_padding = pad_len(self.current_chunk.len(), alignment);

        let size_padded = pad_len(size, alignment);

        self.reserve(initial_padding + size_padded * count);
    }
}

impl<'a> Drop for Arena<'a> {
    fn drop(&mut self) {
        let inner = self.inner.borrow();
        for (ptr, dropper) in &inner.drops {
            dropper(*ptr);
        }
    }
}
