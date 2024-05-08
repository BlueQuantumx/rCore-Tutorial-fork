use core::arch::global_asm;

use super::TaskContext;

global_asm!(include_str!("switch.s"));

extern "C" {
    pub fn __switch(current_task_cx_addr: *mut TaskContext, next_task_cx_addr: *const TaskContext);
}
