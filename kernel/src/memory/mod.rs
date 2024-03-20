pub mod address;
mod frame_allocator;
mod heap_allocator;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
}
