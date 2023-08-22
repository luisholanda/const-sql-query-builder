use std::{
    alloc::{handle_alloc_error, Allocator, Layout},
    mem::MaybeUninit, ptr::NonNull,
};

use crate::const_alloc::ConstAlloc;

pub(crate) struct ConstVec<T> {
    buf: *mut MaybeUninit<T>,
    size: usize,
    cap: usize,
}

impl<T> const Default for ConstVec<T> {
    fn default() -> Self {
        let Ok(layout) = Layout::array::<T>(16) else {
            panic!();
        };

        Self {
            buf: if layout.size() > 0 {
                let Ok(ptr) = ConstAlloc.allocate(layout) else {
                    panic!();
                };

                ptr.as_mut_ptr() as *mut _
            } else {
                core::ptr::null_mut()
            },
            size: 0,
            cap: 0,
        }
    }
}

impl<T> Drop for ConstVec<T> {
    fn drop(&mut self) {
        let mut idx = 0;

        // FIXME: deal with panics in Drop of T.
        unsafe {
            while idx < self.size {
                core::ptr::drop_in_place(self.buf.add(idx));
                idx += 1;
            }
            
            let Some(ptr) = NonNull::new(self.buf as *mut u8) else { return; };

            ConstAlloc.deallocate(
                ptr,
                Layout::from_size_align_unchecked(self.cap, core::mem::align_of::<T>()),
            )
        }
    }
}

impl<T> ConstVec<T> {
    //pub(crate) const fn push_str(&mut self, str: &str) {
    //    self.reserve(str.len());

    //    unsafe {
    //        self.end().copy_from_nonoverlapping(str.as_ptr(), str.len());
    //        self.size += str.len();
    //    }
    //}

    pub(crate) const fn push(&mut self, item: T) {
        self.reserve(1);

        unsafe {
            (*self.end()).write(item);
            self.size += 1;
        }
    }

    pub(crate) const fn leak(mut self) -> &'static [T] {
        self.shrink_to_size();

        unsafe { 
            if self.buf.is_null() {
                panic!()
            }

            core::slice::from_raw_parts(self.buf as *const _, self.size)
        }
    }

    pub(crate) const fn reserve(&mut self, additional: usize) {
        let next_cap = (self.size + additional).next_power_of_two();
        if self.cap >= next_cap {
            return;
        }

        self.buf = unsafe {
            let old_layout = Layout::from_size_align_unchecked(self.cap, std::mem::align_of::<T>());
            let new_layout = Layout::from_size_align_unchecked(next_cap, std::mem::align_of::<T>());


            let alloc = if let Some(ptr) = NonNull::new(self.buf as *mut _) {
                ConstAlloc.grow(ptr, old_layout, new_layout)
            } else {
                ConstAlloc.allocate(new_layout)
            };

            match alloc {
                Ok(m) => m.as_mut_ptr() as *mut _,
                Err(_) => handle_alloc_error(new_layout),
            }
        };
    }

    const fn end(&mut self) -> *mut MaybeUninit<T> {
        unsafe { self.buf.add(self.size) }
    }

    const fn shrink_to_size(&mut self) {
        self.buf = unsafe {
            let align = std::mem::align_of::<T>();
            let old_layout = Layout::from_size_align_unchecked(self.cap, align);
            let new_layout = Layout::from_size_align_unchecked(self.size, align);

            let Some(ptr) = NonNull::new(self.buf as *mut _) else { return; };

            match ConstAlloc.shrink(ptr, old_layout, new_layout) {
                Ok(m) => m.as_mut_ptr() as *mut _,
                Err(_) => handle_alloc_error(new_layout),
            }
        };
    }
}
