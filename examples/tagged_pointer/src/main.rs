use bdwgc_alloc::Allocator;
use std::alloc::{alloc, Layout};

const BITS: usize = usize::MAX << 48 | 0x7;

#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;

fn main() {
    unsafe { Allocator::initialize() }

    loop {
        let x = allocate();
        unsafe { *x = 42 };
        let x = x as usize | BITS;

        assert_eq!(x & BITS, BITS);

        Allocator::force_collect();

        assert_eq!(unsafe { *((x & !BITS) as *mut usize) }, 42);
    }
}

fn allocate() -> *mut usize {
    (unsafe { alloc(Layout::from_size_align(1 << 8, 8).unwrap()) }) as *mut usize
}
