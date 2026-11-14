//! A smoke test of the allocator API.

use bdwgc_alloc::Allocator;
use core::{
    alloc::{AllocError, Allocator as _, Layout},
    ffi::c_void,
    ptr::null,
    sync::atomic::{AtomicUsize, Ordering},
};

const OBJECT_SIZE: usize = 1 << 8;
const LARGE_OBJECT_SIZE: usize = 1 << 16;
const MAX_ALIGNMENT_EXPONENT: u32 = 16;
const TOTAL_COUNT: usize = 1 << 10;

static FINALIZED_COUNT: AtomicUsize = AtomicUsize::new(0);

fn main() {
    unsafe { Allocator::initialize() }

    allocate();
    allocate_zero_size();
    allocate_aligned();
    allocate_too_large();
    allocate_box();
    clone_box();
    grow_vector();
    shrink_vector();
    free_by_gc();
}

fn allocate() {
    let layout = Layout::new::<[u8; OBJECT_SIZE]>();
    let block = Allocator.allocate(layout).unwrap();

    assert_eq!(block.len(), OBJECT_SIZE);

    unsafe {
        block.cast::<u8>().write_bytes(42, OBJECT_SIZE);
        Allocator.deallocate(block.cast(), layout);
    }
}

fn allocate_zero_size() {
    let layout = Layout::new::<[u64; 0]>();
    let block = Allocator.allocate(layout).unwrap();

    assert_eq!(block.len(), 0);
    assert!(block.cast::<u64>().is_aligned());

    unsafe { Allocator.deallocate(block.cast(), layout) }
}

fn allocate_aligned() {
    for size in [1, OBJECT_SIZE, LARGE_OBJECT_SIZE] {
        for exponent in 0..=MAX_ALIGNMENT_EXPONENT {
            let layout = Layout::from_size_align(size, 1 << exponent).unwrap();
            let block = Allocator.allocate(layout).unwrap();

            assert_eq!(block.len(), size);
            assert_eq!(block.addr().get() % layout.align(), 0);

            unsafe {
                block.cast::<u8>().write_bytes(42, size);
                Allocator.deallocate(block.cast(), layout);
            }
        }
    }
}

fn allocate_too_large() {
    assert_eq!(
        Allocator.allocate(Layout::from_size_align(isize::MAX.cast_unsigned(), 1).unwrap()),
        Err(AllocError)
    );
}

fn allocate_box() {
    assert_eq!(*Box::new_in(42, Allocator), 42);
}

fn clone_box() {
    let number = Box::new_in(42, Allocator);

    assert_eq!(number.clone(), number);
}

fn grow_vector() {
    let mut numbers = Vec::new_in(Allocator);

    for number in 0..TOTAL_COUNT {
        numbers.push(number);
    }

    assert!(numbers.iter().copied().eq(0..TOTAL_COUNT));
}

fn shrink_vector() {
    let mut numbers = Vec::with_capacity_in(TOTAL_COUNT, Allocator);

    numbers.push(42);
    numbers.shrink_to_fit();

    assert_eq!(numbers, [42]);
}

fn free_by_gc() {
    extern "C" fn count(_object: *mut c_void, _client_data: *mut c_void) {
        FINALIZED_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    for _ in 0..TOTAL_COUNT {
        unsafe {
            Allocator::register_finalizer(
                Allocator
                    .allocate(Layout::new::<[u8; OBJECT_SIZE]>())
                    .unwrap()
                    .as_ptr()
                    .cast(),
                count,
                null(),
            );
        }
    }

    Allocator::force_collect();

    assert!(FINALIZED_COUNT.load(Ordering::Relaxed) > TOTAL_COUNT / 2);
}
