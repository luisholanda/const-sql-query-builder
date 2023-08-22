use core::{
    intrinsics::{const_allocate, const_deallocate, const_eval_select},
    ptr::{copy_nonoverlapping, slice_from_raw_parts_mut, write_bytes, NonNull},
};
use std::alloc::{AllocError, Allocator, Layout, System};

pub(crate) struct ConstAlloc;

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
    fn rt(layout: Layout) -> AllocResult {
        System.allocate_zeroed(layout)
    }

    const fn ct(layout: Layout) -> AllocResult {
        unsafe {
            let ptr = const_allocate(layout.size(), layout.align());
            if !ptr.is_null() {
                write_bytes(ptr, 0, layout.size());
            }
            to_alloc_result(ptr, layout.size())
        }
    }

    unsafe { const_eval_select((layout,), ct, rt) }
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
