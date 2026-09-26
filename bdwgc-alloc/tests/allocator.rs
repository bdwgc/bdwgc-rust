//! A smoke test of the allocator.

use bdwgc_alloc::Allocator;
use core::hint::black_box;
use std::thread::spawn;

const OBJECT_SIZE: usize = 1 << 8;
const TOTAL_COUNT: usize = 1 << 16;
const THREAD_COUNT: usize = 1 << 6;

#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;

fn main() {
    unsafe { Allocator::initialize() }

    free_by_gc();
    free_by_drop();
    static_thread();
    dynamic_threads();
}

fn free_by_gc() {
    for _ in 0..TOTAL_COUNT {
        black_box(Box::leak(Box::new([0u8; OBJECT_SIZE])));
    }
}

fn free_by_drop() {
    for _ in 0..TOTAL_COUNT {
        drop(black_box(Box::new([0u8; OBJECT_SIZE])));
    }
}

fn static_thread() {
    spawn_thread(free_by_gc);
}

fn dynamic_threads() {
    for _ in 0..THREAD_COUNT {
        spawn_thread(free_by_gc);
    }
}

fn spawn_thread(allocate: fn()) {
    spawn(move || {
        unsafe { Allocator::register_current_thread() }.unwrap();
        allocate();
        unsafe { Allocator::unregister_current_thread() }
    })
    .join()
    .unwrap();
}
