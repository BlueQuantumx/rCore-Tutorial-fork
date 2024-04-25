//! App management syscalls
use log::info;

use crate::task::run_next_ready_app;

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!("Application exited with code {}", exit_code);
    run_next_ready_app();
}
