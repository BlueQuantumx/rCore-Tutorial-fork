use log::{info, warn};

use crate::task::{
    add_process, current_process, exit_current_and_run_next_task, suspend_current_and_run_next_task,
};

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    match exit_code {
        0 => info!("Application exited with code {}", exit_code),
        _ => warn!("Application exited with code {}", exit_code),
    }
    exit_current_and_run_next_task();
    unreachable!("Unreachable in sys_exit");
}

pub fn sys_yield() -> isize {
    info!("Yield current task");
    suspend_current_and_run_next_task();
    0
}

pub fn sys_fork() -> isize {
    let new_process = current_process().fork();
    new_process.trap_cx().x[10] = 0;
    add_process(new_process.clone());
    info!("Fork: {} -> {}", current_process().pid.0, new_process.pid.0);
    new_process.pid.0 as isize
}

pub fn sys_exec() -> isize {
    unimplemented!()
}
