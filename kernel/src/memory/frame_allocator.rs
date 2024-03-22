use core::ops::Range;

use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::{config::MEMORY_END, memory::address::PhysAddr};

use super::address::{PhysPageNum, PAGE_SIZE};

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

struct StackFrameAllocator {
    range: Range<usize>,
    recycled: Vec<PhysPageNum>,
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        StackFrameAllocator {
            range: 0..0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(addr) = self.recycled.pop() {
            Some(addr)
        } else if !self.range.is_empty() {
            let addr = self.range.start;
            self.range.start += 1;
            Some(addr.into())
        } else {
            None
        }
    }

    fn dealloc(&mut self, addr: PhysPageNum) {
        if self.range.contains(&addr.into()) || self.recycled.contains(&addr.into()) {
            panic!("Frame ppn={:#x} has not been allocated!", usize::from(addr));
        }
        self.recycled.push(addr);
    }
}

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        let address: usize = PhysAddr::from(ppn).into();
        for i in address..address + PAGE_SIZE {
            unsafe {
                *(i as *mut u8) = 0;
            }
        }
        Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    static ref FRAME_ALLOCATOR: spin::Mutex<FrameAllocatorImpl> =
        spin::Mutex::new(FrameAllocatorImpl::new());
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.lock().range = PhysAddr::from(ekernel as usize).page_number().into()
        ..PhysAddr::from(MEMORY_END).page_number().into();
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .lock()
        .alloc()
        .map(|ppn| FrameTracker::new(ppn))
}

fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.lock().dealloc(ppn);
}
