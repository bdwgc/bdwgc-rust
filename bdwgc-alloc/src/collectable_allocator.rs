use bdwgc_alloc_sys::{GC_free, GC_memalign};
use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

/// A collectable allocator.
///
/// It allocates memory blocks that a collector reclaims once they become
/// unreachable. Pass it to collections to allocate only some objects by the
/// collector without making [`Allocator`](crate::Allocator) a global allocator.
#[derive(Clone, Copy)]
pub struct CollectableAllocator(());

impl CollectableAllocator {
    /// Creates an allocator.
    ///
    /// # Safety
    ///
    /// A collector must be initialized, and the allocator and its copies must
    /// be used only in a main thread or threads registered to the collector.
    ///
    /// The collector reclaims memory blocks unreachable from memory it scans,
    /// which consists of static data, stacks of a main thread and registered
    /// threads, and memory blocks allocated by the collector. So memory blocks
    /// allocated by the allocator must be kept reachable from such memory while
    /// they are in use. For example, collections using the allocator must not
    /// be kept only in memory of other allocators.
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
