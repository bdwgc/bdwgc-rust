use bdwgc_alloc::Allocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;

fn main() {
    unsafe { Allocator::initialize() }

    let mut _n: Box<[u8; 2 ^ 8]> = Box::new([0; 2 ^ 8]);

    loop {
        *_n = [0; 2 ^ 8];
    }
}
