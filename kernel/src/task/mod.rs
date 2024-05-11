mod context;
mod process;
mod processor;
mod switch;

use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
use log::info;
use spin::Mutex;

use crate::config::MAX_APP_NUM;
use crate::trap::TrapContext;
pub use context::TaskContext;
pub use processor::run_processes;

use self::process::Process;
use self::processor::{schedule, PROCESSOR};

struct AppManager {
    num_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    pub fn print_app_info(&self) {
        info!("num_app = {}", self.num_app);
        for i in 0..self.num_app {
            info!(
                "app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }

    pub unsafe fn load_app(&self, app_id: usize) -> &'static [u8] {
        info!("Loading app_{}", app_id);

        let app_elf = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id],
        );
        app_elf
    }
}

lazy_static! {
    static ref APP_MANAGER: Mutex<AppManager> = unsafe {
        Mutex::new({
            extern "C" {
                fn _num_app();
            }
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = num_app_ptr.read_volatile();
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManager { num_app, app_start }
        })
    };
}

lazy_static! {
    pub static ref PROCESS_MANAGER: Mutex<ProcessManager> = {
        let mut task_manager = ProcessManager::new();
        let app_manager = APP_MANAGER.lock();
        for id in 0..app_manager.num_app {
            let elf_data = unsafe { app_manager.load_app(id) };
            task_manager.add(Arc::new(Process::new(elf_data)));
        }
        Mutex::new(task_manager)
    };
}

pub struct ProcessManager {
    tasks: VecDeque<Arc<Process>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }
    pub fn num(&self) -> usize {
        self.tasks.len()
    }
    pub fn add(&mut self, task: Arc<Process>) {
        self.tasks.push_back(task);
    }
    pub fn fetch(&mut self) -> Option<Arc<Process>> {
        self.tasks.pop_front()
    }
}

pub fn current_user_token() -> usize {
    PROCESSOR
        .lock()
        .current()
        .as_ref()
        .unwrap()
        .lock_inner()
        .memory_set
        .satp_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    PROCESSOR.lock().current().as_ref().unwrap().trap_cx()
}

pub fn suspend_current_and_run_next_task() {
    let current = PROCESSOR
        .lock()
        .current()
        .take()
        .expect("no current process");
    let mut current_inner = current.lock_inner();
    let current_cx_ptr = &mut current_inner.task_cx as *mut TaskContext;
    current_inner.status = process::ProcessStatus::Ready;
    drop(current_inner);
    PROCESS_MANAGER.lock().add(current);
    schedule(current_cx_ptr);
}

pub fn exit_current_and_run_next_task() {
    let current = PROCESSOR
        .lock()
        .current()
        .take()
        .expect("no current process");
    let mut current_inner = current.lock_inner();
    let current_cx_ptr = &mut current_inner.task_cx as *mut TaskContext;
    current_inner.status = process::ProcessStatus::Exited;
    drop(current_inner);
    drop(current);
    schedule(current_cx_ptr);
}
