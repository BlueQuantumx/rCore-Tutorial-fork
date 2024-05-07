use log::info;

use crate::task::{exit_current_and_run_next_task, suspend_current_and_run_next_task};

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!("Application exited with code {}", exit_code);
    exit_current_and_run_next_task();
    panic!("Unreachable in sys_exit");
}

pub fn sys_yield() -> isize {
    info!("Yield current task");
    suspend_current_and_run_next_task();
    0
}
