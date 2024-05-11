use log::trace;

use crate::{
    config::{KERNEL_STACK_SIZE, TRAMPOLINE},
    memory::{address::PAGE_SIZE, MapPermission, KERNEL_SPACE},
};

use super::pid::Pid;

pub struct KernelStack {
    pid: usize,
}

impl KernelStack {
    pub fn new(pid: &Pid) -> Self {
        let (stack_left, stack_right) = kernel_stack_position(pid.0);
        trace!(
            "mapping kernel stack for process {} [{:x}, {:x})",
            pid.0,
            stack_left,
            stack_right
        );
        KERNEL_SPACE.lock().insert_framed_area(
            stack_left.into(),
            stack_right.into(),
            MapPermission::R | MapPermission::W,
        );
        Self { pid: pid.0 }
    }

    pub fn top(&self) -> usize {
        kernel_stack_position(self.pid).1
    }

    pub fn bottom(&self) -> usize {
        kernel_stack_position(self.pid).0
    }
}

/// return app's kernel stack with [{0}, {1})
fn kernel_stack_position(pid: usize) -> (usize, usize) {
    let stack_bottom = TRAMPOLINE - pid * (KERNEL_STACK_SIZE + PAGE_SIZE); // stack bottom
    (stack_bottom - KERNEL_STACK_SIZE, stack_bottom)
}
