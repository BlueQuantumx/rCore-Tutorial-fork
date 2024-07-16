use alloc::sync::Arc;
use lazy_static::lazy_static;
use log::info;

use crate::{sbi::shutdown, task::switch::__switch, trap::TrapContext};

use super::{
    process::{Process, ProcessStatus},
    TaskContext, PROCESS_MANAGER,
};

pub struct Processor {
    current_process: Option<Arc<Process>>,
    idle_task_cx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current_process: None,
            idle_task_cx: TaskContext::zero(),
        }
    }
    pub fn current(&mut self) -> &mut Option<Arc<Process>> {
        &mut self.current_process
    }
    fn idle_task_cx(&mut self) -> &mut TaskContext {
        &mut self.idle_task_cx
    }
}

lazy_static! {
    pub static ref PROCESSOR: spin::Mutex<Processor> = spin::Mutex::new(Processor::new());
}

pub fn run_processes() -> ! {
    loop {
        let mut process_manager = PROCESS_MANAGER.lock();
        if let Some(process) = process_manager.fetch() {
            drop(process_manager);

            let mut process_inner = process.lock_inner();
            process_inner.status = ProcessStatus::Running;
            let next_task_cx_ptr = &process_inner.task_cx as *const TaskContext;
            drop(process_inner);

            let mut processor = PROCESSOR.lock();
            processor.current_process = Some(process);
            let idle_task_cx_ptr = processor.idle_task_cx() as *mut TaskContext;
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            info!("No process to run, shutdown");
            shutdown(false);
        }
    }
}

pub fn current_process() -> Arc<Process> {
    PROCESSOR.lock().current().as_ref().unwrap().clone()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    PROCESSOR
        .lock()
        .current()
        .as_ref()
        .unwrap()
        .lock_inner()
        .trap_cx()
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

pub fn schedule(switched_process_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.lock();
    let idle_process_cx_ptr = processor.idle_task_cx() as *const TaskContext;
    drop(processor);
    unsafe {
        __switch(switched_process_cx_ptr, idle_process_cx_ptr);
    }
}
