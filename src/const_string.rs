use core::{
    intrinsics::{const_allocate, const_deallocate, const_eval_select},
    ptr::{copy_nonoverlapping, slice_from_raw_parts_mut, write_bytes, NonNull},
};
use std::{
    alloc::{handle_alloc_error, AllocError, Allocator, Layout, System},
    mem::ManuallyDrop,
};

type ConstBox<T> = Box<T, ConstAlloc>;

pub(crate) struct ConstString {
    buf: ConstBox<[u8]>,
    size: usize,
    cap: usize,
}

impl const Default for ConstString {
    fn default() -> Self {
        Self {
            buf: <ConstBox<[u8; 16]>>::new_in([0; 16], ConstAlloc) as ConstBox<[u8]>,
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
        let slice = ConstBox::leak(self.buf);

        unsafe { std::str::from_utf8_unchecked(slice) }
    }

    pub(crate) const fn as_str(&self) -> &'static str {
        unsafe { std::str::from_utf8_unchecked(&*self.buf.as_ptr()) }
    }

    const fn end(&mut self) -> *mut u8 {
        unsafe { self.buf.as_mut_ptr().add(self.size) }
    }

    const fn reserve(&mut self, additional: usize) {
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
                let mut new_box = Box::from_raw_in(new_ptr.as_ptr(), ConstAlloc);
                std::mem::swap(&mut self.buf, &mut new_box);

                let ptr = Box::into_raw_with_allocator(new_box).0 as *mut u8;
                let ptr = NonNull::new_unchecked(ptr);
                let layout =
                    Layout::from_size_align_unchecked(self.cap, std::mem::align_of::<u8>());
                ConstAlloc.deallocate(ptr, layout);
            }

            self.cap = next_cap;
        }
    }

    const fn shrink_to_size(&mut self) {
        let new_ptr = unsafe {
            let old_ptr = NonNull::new_unchecked(self.buf.as_mut_ptr());

            let align = std::mem::align_of::<u8>();
            let old_layout = Layout::from_size_align_unchecked(self.cap, align);
            let new_layout = Layout::from_size_align_unchecked(self.size, align);

            match ConstAlloc.shrink(old_ptr, old_layout, new_layout) {
                Ok(m) => m,
                Err(_) => handle_alloc_error(new_layout),
            }
        };

        unsafe {
            let mut new_box = Box::from_raw_in(new_ptr.as_ptr(), ConstAlloc);
            std::mem::swap(&mut self.buf, &mut new_box);

            let _ = ManuallyDrop::new(new_box);
        }
    }
}

struct ConstAlloc;

type AllocResult = Result<NonNull<[u8]>, AllocError>;

const fn to_alloc_result(ptr: *mut u8, size: usize) -> AllocResult {
    NonNull::new(slice_from_raw_parts_mut(ptr, size)).ok_or(AllocError)
}

const fn alloc(layout: Layout) -> AllocResult {
    const fn ct(layout: Layout) -> AllocResult {
        unsafe { to_alloc_result(const_allocate(layout.size(), layout.align()), layout.size()) }
    }

    fn rt(layout: Layout) -> AllocResult {
        System.allocate(layout)
    }

    unsafe { const_eval_select((layout,), ct, rt) }
}

const unsafe fn dealloc(ptr: NonNull<u8>, layout: Layout) {
    fn rt(ptr: NonNull<u8>, layout: Layout) {
        unsafe { System.deallocate(ptr, layout) };
    }

    const fn ct(ptr: NonNull<u8>, layout: Layout) {
        unsafe { const_deallocate(ptr.as_ptr(), layout.size(), layout.align()) };
    }
    const_eval_select((ptr, layout), ct, rt)
}

const fn alloc_zeroed(layout: Layout) -> AllocResult {
    const fn ct(layout: Layout) -> AllocResult {
        unsafe {
            let ptr = const_allocate(layout.size(), layout.align());
            if !ptr.is_null() {
                write_bytes(ptr, 0, layout.size());
            }
            to_alloc_result(ptr, layout.size())
        }
    }

    unsafe { const_eval_select((layout,), ct, |l| System.allocate_zeroed(l)) }
}

const unsafe fn grow(ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> AllocResult {
    fn rt(ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> AllocResult {
        unsafe { System.grow(ptr, old_layout, new_layout) }
    }

    const fn ct(ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> AllocResult {
        unsafe {
            let new_ptr = const_allocate(new_layout.size(), new_layout.align());
            if !new_ptr.is_null() {
                copy_nonoverlapping(ptr.as_ptr(), new_ptr, old_layout.size());
            }
            const_deallocate(ptr.as_ptr(), old_layout.size(), old_layout.align());
            to_alloc_result(new_ptr, new_layout.size())
        }
    }

    const_eval_select((ptr, old_layout, new_layout), ct, rt)
}

const unsafe fn grow_zeroed(
    ptr: NonNull<u8>,
    old_layout: Layout,
    new_layout: Layout,
) -> AllocResult {
    fn rt(ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> AllocResult {
        unsafe { System.grow_zeroed(ptr, old_layout, new_layout) }
    }

    const fn ct(ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> AllocResult {
        unsafe {
            let new_ptr = const_allocate(new_layout.size(), new_layout.align());
            if !new_ptr.is_null() {
                copy_nonoverlapping(ptr.as_ptr(), new_ptr, old_layout.size());
                write_bytes(
                    ptr.as_ptr().add(old_layout.size()),
                    0,
                    new_layout.size() - old_layout.size(),
                );
            }
            const_deallocate(ptr.as_ptr(), old_layout.size(), old_layout.align());
            to_alloc_result(new_ptr, new_layout.size())
        }
    }

    const_eval_select((ptr, old_layout, new_layout), ct, rt)
}

const unsafe fn shrink(ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> AllocResult {
    fn rt(ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> AllocResult {
        unsafe { System.shrink(ptr, old_layout, new_layout) }
    }

    const fn ct(ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> AllocResult {
        unsafe {
            let new_ptr = const_allocate(new_layout.size(), new_layout.align());
            if !new_ptr.is_null() {
                copy_nonoverlapping(ptr.as_ptr(), new_ptr, new_layout.size());
            }
            const_deallocate(ptr.as_ptr(), old_layout.size(), old_layout.align());
            to_alloc_result(new_ptr, new_layout.size())
        }
    }

    const_eval_select((ptr, old_layout, new_layout), ct, rt)
}

unsafe impl const Allocator for ConstAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        alloc(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        dealloc(ptr, layout);
    }

    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        alloc_zeroed(layout)
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        grow(ptr, old_layout, new_layout)
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        grow_zeroed(ptr, old_layout, new_layout)
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        shrink(ptr, old_layout, new_layout)
    }
    fn by_ref(&self) -> &Self {
        self
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
        const TEST: &'static str = {
            let mut string = ConstString::default();
            string.push_str("testing a functio");
            string.push_ascii(b'n');
            string.leak()
        };

        assert_eq!(TEST, "testing a function");
    }
}
