use crate::memory::address::PAGE_SIZE;

pub const KERNEL_HEAP_SIZE: usize = 0x10_0000; // 1 MiB
pub const MEMORY_END: usize = 0x80800000; // 8 MiB
pub const USER_STACK_SIZE: usize = 0x8000; // 32 KiB
pub const KERNEL_STACK_SIZE: usize = 0x1000; // 4 KiB
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
pub const MAX_APP_NUM: usize = 16;
pub const TASK_SWITCH_TICK: u64 = 100000;
pub const CLOCK_FREQ: usize = 12500000; // 12.5 MHz
