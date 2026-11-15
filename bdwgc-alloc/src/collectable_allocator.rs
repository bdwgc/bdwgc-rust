use bdwgc_alloc_sys::{GC_free, GC_memalign};
use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

/// A collectable allocator.
#[derive(Clone, Copy)]
pub struct CollectableAllocator(());

impl CollectableAllocator {
    /// Creates an allocator.
    ///
    /// # Safety
    ///
    /// The allocator must be used only in a main thread or threads registered
    /// to a collector, and its memory blocks must not be referenced only from
    /// memory the collector does not scan, such as memory of other allocators.
    pub const unsafe fn new() -> Self {
        Self(())
    }
}

unsafe impl Allocator for CollectableAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Ok(NonNull::slice_from_raw_parts(
            if layout.size() == 0 {
                layout.dangling_ptr()
            } else {
                NonNull::new(unsafe { GC_memalign(layout.align(), layout.size()) }.cast())
                    .ok_or(AllocError)?
            },
            layout.size(),
        ))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() != 0 {
            unsafe { GC_free(ptr.as_ptr().cast()) }
        }
    }
}
