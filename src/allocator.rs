use std::alloc::{GlobalAlloc, System, Layout};

extern "C" {
    fn pool_alloc(size: usize) -> *mut u8;
    fn pool_free(*mut u8);
}

struct MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        //System.alloc(layout)
	pool_alloc(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        //System.dealloc(ptr, layout)
	pool_free(ptr);
    }
}

#[global_allocator]
static GLOBAL: MyAllocator = MyAllocator;
