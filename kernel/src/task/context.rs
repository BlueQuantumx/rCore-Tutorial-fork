#[repr(C)]
pub struct TaskContext {
    /// return address ( e.g. __restore ) of __switch ASM function
    pub ra: usize,
    /// kernel stack pointer of app
    pub sp: usize,
    /// callee saved registers:  s 0..11
    pub s: [usize; 12],
}

use crate::trap::trap_return;

impl TaskContext {
    pub fn goto_trap_return(kernel_stack_top: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kernel_stack_top,
            s: [0; 12],
        }
    }
}
