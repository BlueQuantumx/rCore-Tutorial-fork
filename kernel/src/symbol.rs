use crate::task::TaskContext;

extern "C" {
    pub fn __alltraps() -> !;
    pub fn __restore(trap_cx_addr: usize) -> !;
    pub fn __switch(current_task_cx_addr: *mut TaskContext, next_task_cx_addr: *const TaskContext);
}
