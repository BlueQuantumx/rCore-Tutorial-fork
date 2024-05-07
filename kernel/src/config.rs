use crate::memory::address::PAGE_SIZE;

pub const KERNEL_HEAP_SIZE: usize = 0x20_0000; // 1 MiB
pub const MEMORY_END: usize = 0x80800000; // 8 MiB
pub const USER_STACK_SIZE: usize = 0x4000; // 32 KiB
pub const KERNEL_STACK_SIZE: usize = 0x4000; // 32 KiB
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
pub const MAX_APP_NUM: usize = 16;
pub const CLOCK_FREQ: u64 = 12500000; // 12.5 MHz
pub const TICKS_PER_SEC: u64 = 100;
