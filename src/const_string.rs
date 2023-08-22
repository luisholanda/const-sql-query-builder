use std::{
    alloc::{handle_alloc_error, Allocator, Layout},
    ptr::NonNull,
};

use crate::const_alloc::ConstAlloc;

pub(crate) struct ConstString {
    buf: NonNull<[u8]>,
    size: usize,
    cap: usize,
}

impl const Default for ConstString {
    fn default() -> Self {
        let Ok(layout) = Layout::array::<u8>(16) else {
            panic!();
        };
        let Ok(ptr) = ConstAlloc.allocate(layout) else {
            panic!();
        };
        Self {
            buf: ptr,
            size: 0,
            cap: 16,
        }
    }
}

impl ConstString {
    pub(crate) const fn push_str(&mut self, str: &str) {
        self.reserve(str.len());

        unsafe {
            self.end().copy_from_nonoverlapping(str.as_ptr(), str.len());
            self.size += str.len();
        }
    }

    pub(crate) const fn push_ascii(&mut self, ch: u8) {
        self.reserve(1);

        unsafe {
            self.end().write(ch);
            self.size += 1;
        }
    }

    pub(crate) const fn leak(mut self) -> &'static str {
        self.shrink_to_size();
        unsafe { std::str::from_utf8_unchecked(self.buf.as_ref()) }
    }

    pub(crate) const fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.buf.as_ref()) }
    }

    pub(crate) const fn reserve(&mut self, additional: usize) {
        let next_cap = (self.size + additional).next_power_of_two();
        if self.cap >= next_cap {
            return;
        }

        let new_ptr = unsafe {
            let layout = Layout::from_size_align_unchecked(next_cap, std::mem::align_of::<u8>());

            match ConstAlloc.allocate(layout) {
                Ok(m) => m,
                Err(_) => handle_alloc_error(layout),
            }
        };

        unsafe {
            self.buf
                .as_mut_ptr()
                .copy_to_nonoverlapping(new_ptr.as_ptr() as *mut u8, self.size);

            {
                let layout =
                    Layout::from_size_align_unchecked(self.cap, std::mem::align_of::<u8>());
                ConstAlloc.deallocate(self.buf.cast(), layout);
                self.buf = new_ptr;
            }

            self.cap = next_cap;
        }
    }

    const fn end(&mut self) -> *mut u8 {
        unsafe { self.buf.as_mut_ptr().add(self.size) }
    }

    const fn shrink_to_size(&mut self) {
        self.buf = unsafe {
            let align = std::mem::align_of::<u8>();
            let old_layout = Layout::from_size_align_unchecked(self.cap, align);
            let new_layout = Layout::from_size_align_unchecked(self.size, align);

            match ConstAlloc.shrink(self.buf.cast(), old_layout, new_layout) {
                Ok(m) => m,
                Err(_) => handle_alloc_error(new_layout),
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_alloc() {
        const fn alloc_and_dealloc<A: ~const Allocator>(a: &A) {
            let ptr = a.allocate(Layout::new::<[u8; 128]>());
            if let Ok(ptr) = ptr {
                unsafe { a.deallocate(ptr.cast(), Layout::new::<[u8; 128]>()) };
            }
        }

        static A: ConstAlloc = ConstAlloc;
        #[used]
        static CT: () = alloc_and_dealloc(&A);
        alloc_and_dealloc(&A);
    }

    #[test]
    fn test_const_string() {
        const TEST: &str = {
            let mut string = ConstString::default();
            string.push_str("testing a functio");
            string.push_ascii(b'n');
            string.leak()
        };

        assert_eq!(TEST, "testing a function");
    }
}
