//! A smoke test of the allocator.

use bdwgc_alloc::Allocator;
use core::{fmt::Debug, hint::black_box};
use std::thread::spawn;

const OBJECT_SIZE: usize = 1 << 8;
const TOTAL_COUNT: usize = 1 << 16;
const THREAD_COUNT: usize = 1 << 6;

#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = unsafe { Allocator::new() };

#[repr(align(16))]
#[derive(Clone, Copy, Debug, PartialEq)]
struct Aligned16(usize);

#[repr(align(128))]
#[derive(Clone, Copy, Debug, PartialEq)]
struct Aligned128(usize);

fn main() {
    unsafe { Allocator::initialize() }

    free_by_gc();
    free_by_drop();
    allocate_aligned(Aligned16);
    allocate_aligned(Aligned128);
    reallocate_aligned(Aligned16);
    reallocate_aligned(Aligned128);
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

// Pointers go through black boxes because the compiler assumes allocations to
// be aligned.
fn allocate_aligned<T: Debug + PartialEq>(create: fn(usize) -> T) {
    for index in 0..TOTAL_COUNT {
        let value = Box::new(create(index));

        assert!(black_box(&raw const *value).is_aligned());
        assert_eq!(*value, create(index));
    }
}

fn reallocate_aligned<T: Copy + Debug + PartialEq>(create: fn(usize) -> T) {
    let mut values = Vec::new();

    for index in 0..TOTAL_COUNT {
        values.push(create(index));

        assert!(black_box(values.as_ptr()).is_aligned());
    }

    assert!(values.iter().copied().eq((0..TOTAL_COUNT).map(create)));

    values.truncate(1);
    values.shrink_to_fit();

    assert!(black_box(values.as_ptr()).is_aligned());
    assert_eq!(values, [create(0)]);
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
