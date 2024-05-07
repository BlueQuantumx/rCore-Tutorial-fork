use log::info;

pub mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
pub use page_table::translated_byte_buffer;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.lock().activate();
    info!("kernel address space set up");
    memory_set::remap_test();
}
