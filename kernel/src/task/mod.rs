mod context;
mod switch;

use alloc::vec::Vec;
use lazy_static::*;
use log::{info, trace};
use riscv::register::satp;
use spin::Mutex;

use crate::config::{KERNEL_STACK_SIZE, MAX_APP_NUM, TRAMPOLINE, TRAP_CONTEXT};
use crate::memory::{
    address::{PhysPageNum, VirtAddr, PAGE_SIZE},
    MapPermission, MemorySet, KERNEL_SPACE,
};
use crate::sbi::shutdown;
use crate::trap::{trap_handler, TrapContext};
pub use context::TaskContext;
use switch::__switch;

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

/// print apps info
pub fn print_app_info() {
    APP_MANAGER.lock().print_app_info();
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    let task_manager = TASK_MANAGER.lock();
    task_manager.current_task().get_trap_cx()
}

pub fn current_user_token() -> usize {
    TASK_MANAGER.lock().current_task().memory_set.satp_token()
}

pub fn run_first_app() -> ! {
    let mut task_manager = TASK_MANAGER.lock();
    let current_task_id = task_manager.current_task_id;
    let tasks = &mut task_manager.tasks;
    tasks[current_task_id].task_status = TaskStatus::Running;
    let mut _unused_cx = TaskContext {
        ra: 0,
        sp: 0,
        s: [0; 12],
    };
    let next_task_cx_ptr = &tasks[0].task_cx as *const TaskContext;
    drop(task_manager);
    info!("running app 0");
    unsafe {
        __switch(&mut _unused_cx, next_task_cx_ptr);
    }
    panic!("unreachable in run_first_task!");
}

pub fn suspend_current_and_run_next_task() {
    let mut task_manager = TASK_MANAGER.lock();
    task_manager.suspend_current_task();
    drop(task_manager);
    run_next_ready_app();
}

pub fn exit_current_and_run_next_task() {
    let mut task_manager = TASK_MANAGER.lock();
    task_manager.exit_current_task();
    drop(task_manager);
    run_next_ready_app();
}

/// run next app
fn run_next_ready_app() {
    let mut task_manager = TASK_MANAGER.lock();
    if let Some(next) = task_manager.next_ready_task() {
        let current_task_cx_ptr = &mut task_manager.current_task_mut().task_cx as *mut TaskContext;
        task_manager.current_task_id = next;
        task_manager.current_task_mut().task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task_manager.current_task().task_cx as *const TaskContext;
        trace!("running app {}", next);
        drop(task_manager);
        unsafe {
            __switch(current_task_cx_ptr, next_task_cx_ptr);
        }
    } else {
        for task in task_manager.tasks.iter() {
            info!("{:?}", task.task_status);
        }
        shutdown(false);
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: Mutex<TaskManager> = {
        let mut task_manager = TaskManager::new();
        let app_manager = APP_MANAGER.lock();
        for id in 0..app_manager.num_app {
            let elf_data = unsafe { app_manager.load_app(id) };
            task_manager.add_task(id, elf_data);
        }
        Mutex::new(task_manager)
    };
}

pub struct TaskManager {
    current_task_id: usize,
    pub tasks: Vec<TaskControlBlock>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            current_task_id: 0,
            tasks: Vec::new(),
        }
    }
    pub fn add_task(&mut self, app_id: usize, elf_data: &[u8]) {
        self.tasks.push(TaskControlBlock::new(elf_data, app_id));
    }
    pub fn current_task(&self) -> &TaskControlBlock {
        &self.tasks[self.current_task_id]
    }
    pub fn current_task_mut(&mut self) -> &mut TaskControlBlock {
        &mut self.tasks[self.current_task_id]
    }
}

impl TaskManager {
    fn suspend_current_task(&mut self) {
        let current_task = &mut self.tasks[self.current_task_id];
        if current_task.task_status == TaskStatus::Running {
            current_task.task_status = TaskStatus::Ready;
        }
    }

    fn exit_current_task(&mut self) {
        let current_task = &mut self.tasks[self.current_task_id];
        current_task.task_status = TaskStatus::Exited;
    }

    fn next_ready_task(&mut self) -> Option<usize> {
        (self.current_task_id + 1..self.current_task_id + self.tasks.len() + 1)
            .map(|i| i % self.tasks.len())
            .find(|i| self.tasks[*i].task_status == TaskStatus::Ready)
    }
}

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    // pub heap_bottom: usize,
    // pub program_brk: usize,
}

impl TaskControlBlock {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        unsafe { self.trap_cx_ppn.get_mut() }
    }
}

impl TaskControlBlock {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).page_number_floor())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;

        // map a kernel-stack in kernel space
        let (kernel_stack_left, kernel_stack_right) = kernel_stack_position(app_id);
        trace!(
            "mapping kernel stack for app_{} [{:x}, {:x})",
            app_id,
            kernel_stack_left,
            kernel_stack_right
        );
        KERNEL_SPACE.lock().insert_framed_area(
            kernel_stack_left.into(),
            kernel_stack_right.into(),
            MapPermission::R | MapPermission::W,
        );

        let task_control_block = Self {
            task_status,
            task_cx: TaskContext::goto_trap_return(kernel_stack_right),
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            satp::read().bits(),
            kernel_stack_right - 8,
            trap_handler as usize,
        );
        task_control_block
    }
}

#[derive(Debug, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}

/// return app's kernel stack with [{0}, {1})
fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let stack_bottom = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE); // stack bottom
    (stack_bottom - KERNEL_STACK_SIZE, stack_bottom)
}
