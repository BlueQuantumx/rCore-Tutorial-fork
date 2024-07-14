use alloc::vec::Vec;
use lazy_static::lazy_static;

lazy_static! {
    static ref PID_ALLOCATOR: spin::Mutex<PidAllocator> = spin::Mutex::new(PidAllocator::new());
}

pub struct Pid(pub usize);

impl Pid {
    pub fn new() -> Self {
        Self(PID_ALLOCATOR.lock().alloc())
    }
}

impl Drop for Pid {
    fn drop(&mut self) {
        PID_ALLOCATOR.lock().dealloc(self.0);
    }
}

struct PidAllocator {
    max_used: usize,
    recycle: Vec<usize>,
}

impl PidAllocator {
    fn new() -> Self {
        Self {
            max_used: 0,
            recycle: Vec::new(),
        }
    }

    fn alloc(&mut self) -> usize {
        // TODO: temporarily disable pid recycle
        // because the previous kernel stack is not released

        // if let Some(pid) = self.recycle.pop() {
        //     pid
        // } else {
        self.max_used += 1;
        self.max_used
        // }
    }

    fn dealloc(&mut self, pid: usize) {
        assert!(pid > 0 && pid <= self.max_used, "invalid pid {}", pid);
        assert!(
            self.recycle.iter().all(|&p| p != pid),
            "pid {} is already deallocated",
            pid
        );
        self.recycle.push(pid);
    }
}
