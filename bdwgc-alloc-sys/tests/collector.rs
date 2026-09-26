//! A smoke test of the bindings.
//!
//! The test runs without the test harness because a collector must be
//! initialized in a main thread.

use bdwgc_alloc_sys::{
    GC_SUCCESS, GC_allow_register_threads, GC_free, GC_gcollect, GC_get_stack_base, GC_init,
    GC_malloc, GC_realloc, GC_register_finalizer, GC_register_my_thread, GC_stack_base,
    GC_unregister_my_thread,
};
use core::{
    ffi::c_void,
    ptr::null_mut,
    sync::atomic::{AtomicUsize, Ordering},
};
use std::thread;

static FINALIZED_COUNT: AtomicUsize = AtomicUsize::new(0);

fn main() {
    unsafe {
        GC_init();
        GC_allow_register_threads();
    }

    allocate();
    finalize();
    register_thread();
}

fn allocate() {
    let ptr = unsafe { GC_malloc(42) };
    assert!(!ptr.is_null());

    let ptr = unsafe { GC_realloc(ptr, 84) };
    assert!(!ptr.is_null());

    unsafe { GC_free(ptr) };
}

fn finalize() {
    extern "C" fn count(_object: *mut c_void, _client_data: *mut c_void) {
        FINALIZED_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    // A conservative collector might keep some of the objects alive but not all of them.
    for _ in 0..1000 {
        unsafe {
            GC_register_finalizer(
                GC_malloc(42),
                Some(count),
                null_mut(),
                null_mut(),
                null_mut(),
            );
        }
    }

    unsafe { GC_gcollect() };

    // TODO Is there any way to collect all?
    assert!(FINALIZED_COUNT.load(Ordering::Relaxed) > 0);
}

fn register_thread() {
    thread::spawn(|| {
        let mut base = GC_stack_base {
            mem_base: null_mut(),
        };

        assert_eq!(unsafe { GC_get_stack_base(&mut base) }, GC_SUCCESS);
        assert_eq!(unsafe { GC_register_my_thread(&base) }, GC_SUCCESS);
        assert!(!unsafe { GC_malloc(42) }.is_null());
        assert_eq!(unsafe { GC_unregister_my_thread() }, GC_SUCCESS);
    })
    .join()
    .unwrap();
}
