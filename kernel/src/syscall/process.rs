use log::{info, warn};

use crate::{
    memory::translated_str,
    task::{
        add_process, current_process, current_user_token, exit_current_and_run_next_task,
        suspend_current_and_run_next_task, APP_MANAGER,
    },
};

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    match exit_code {
        0 => info!(
            "Process {:?} exited with code {}",
            current_process().pid,
            exit_code
        ),
        _ => warn!(
            "Process {:?} exited with code {}",
            current_process().pid,
            exit_code
        ),
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
    new_process.lock_inner().trap_cx().x[10] = 0;
    add_process(new_process.clone());
    info!("Fork: {} -> {}", current_process().pid.0, new_process.pid.0);
    new_process.pid.0 as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let path = translated_str(current_user_token(), path);
    let process = current_process();
    process.exec(APP_MANAGER.lock().load_app(path.parse().unwrap()));
    info!("Exec: {:?}", path);
    0
}
