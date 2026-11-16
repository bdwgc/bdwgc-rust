#![doc = include_str!("../README.md")]
#![no_std]

mod error;

use bdwgc_alloc_sys::{
    GC_SUCCESS, GC_alloc_lock, GC_alloc_unlock, GC_allow_register_threads, GC_free, GC_gcollect,
    GC_get_stack_base, GC_init, GC_memalign, GC_realloc, GC_register_finalizer,
    GC_register_my_thread, GC_set_stackbottom, GC_stack_base, GC_unregister_my_thread,
};
use core::{
    alloc::{self, AllocError, GlobalAlloc, Layout},
    ffi::c_void,
    marker::PhantomData,
    ptr::{NonNull, copy_nonoverlapping, null_mut},
};

// A collector aligns memory blocks to granules of two pointers.
const GRANULE_SIZE: usize = 2 * size_of::<*const c_void>();

/// An allocator.
#[derive(Clone, Copy)]
pub struct Allocator(());

impl Allocator {
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

    /// Locks a collector.
    pub fn lock() {
        unsafe { GC_alloc_lock() }
    }

    /// Unlocks a collector.
    pub fn unlock() {
        unsafe { GC_alloc_unlock() }
    }

    /// Initializes a collector.
    ///
    /// # Safety
    ///
    /// This function must be called in a main thread.
    pub unsafe fn initialize() {
        unsafe {
            GC_init();
            GC_allow_register_threads();
        }
    }

    /// Registers a current thread to a collector.
    ///
    /// The thread is removed from a collector when the returned registration is
    /// dropped.
    ///
    /// # Safety
    ///
    /// A collector must be initialized already, and a pointer into it held by
    /// the thread must be handed off to another thread before the registration
    /// is dropped.
    pub unsafe fn register_current_thread() -> Result<ThreadRegistration, error::Error> {
        let mut base = GC_stack_base {
            mem_base: null_mut(),
        };

        if unsafe { GC_get_stack_base(&mut base) } != GC_SUCCESS {
            return Err(error::Error::new("failed to get stack base"));
        } else if unsafe { GC_register_my_thread(&base) } != GC_SUCCESS {
            return Err(error::Error::new("failed to register a thread for GC"));
        }

        Ok(ThreadRegistration(PhantomData))
    }

    /// Sets a bottom of a stack.
    ///
    /// You do not have to call this function in most cases.
    /// A collector detects the bottom on initialization automatically.
    ///
    /// # Safety
    ///
    /// The bottom address must be valid.
    pub unsafe fn set_stack_bottom(bottom: *const u8) {
        unsafe {
            GC_set_stackbottom(
                null_mut(),
                &GC_stack_base {
                    mem_base: bottom.cast_mut().cast(),
                },
            )
        }
    }

    /// Runs a garbage collection forcibly.
    pub fn force_collect() {
        unsafe { GC_gcollect() }
    }

    /// Registers a finalizer of an object.
    ///
    /// # Safety
    ///
    /// The given finalizer must not be null and handle pointers properly.
    pub unsafe fn register_finalizer(
        ptr: *const c_void,
        finalizer: extern "C" fn(*mut c_void, *mut c_void),
        client_data: *const c_void,
    ) {
        unsafe {
            GC_register_finalizer(
                ptr.cast_mut(),
                Some(finalizer),
                client_data.cast_mut(),
                null_mut(),
                null_mut(),
            )
        };
    }
}

/// A registration of a current thread to a collector.
///
/// The thread is removed from a collector when this value is dropped.
#[must_use]
pub struct ThreadRegistration(PhantomData<*mut ()>);

impl Drop for ThreadRegistration {
    fn drop(&mut self) {
        unsafe { GC_unregister_my_thread() };
    }
}

unsafe impl alloc::Allocator for Allocator {
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

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (unsafe { GC_memalign(layout.align(), layout.size()) }) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { GC_free(ptr as *mut c_void) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if layout.align() <= GRANULE_SIZE {
            return (unsafe { GC_realloc(ptr as *mut c_void, size) }) as *mut u8;
        }

        let new_ptr =
            unsafe { self.alloc(Layout::from_size_align_unchecked(size, layout.align())) };

        if !new_ptr.is_null() {
            unsafe {
                copy_nonoverlapping(ptr, new_ptr, layout.size().min(size));
                self.dealloc(ptr, layout);
            }
        }

        new_ptr
    }
}
