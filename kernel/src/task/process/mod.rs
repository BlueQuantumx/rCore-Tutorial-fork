mod kernel_stack;
mod pid;

use pid::Pid;

use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use riscv::register::satp;

use crate::{
    config::TRAP_CONTEXT,
    memory::{
        address::{PhysPageNum, VirtAddr},
        MemorySet, KERNEL_SPACE,
    },
    trap::{trap_handler, TrapContext},
};

use self::kernel_stack::KernelStack;

use super::TaskContext;

pub struct Process {
    pub pid: Pid,
    pub kernel_stack: KernelStack,
    inner: spin::Mutex<ProcessInner>,
}

pub struct ProcessInner {
    pub parent: Option<Weak<Process>>,
    pub children: Vec<Arc<Process>>,
    pub status: ProcessStatus,
    pub exit_code: i32,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
}

impl ProcessInner {
    pub fn trap_cx(&self) -> &'static mut TrapContext {
        unsafe { self.trap_cx_ppn.get_mut() }
    }
}

impl Process {
    pub fn lock_inner(&self) -> spin::MutexGuard<ProcessInner> {
        self.inner.lock()
    }
}

impl Process {
    pub fn new(elf_data: &[u8]) -> Self {
        // establish memory set from elf data
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).page_number_floor())
            .unwrap()
            .ppn();
        let pid = Pid::new();
        // establish kernel stack
        let kernel_stack = KernelStack::new(&pid);
        let kernel_stack_top = kernel_stack.top();
        let process = Self {
            pid,
            kernel_stack,
            inner: spin::Mutex::new(ProcessInner {
                status: ProcessStatus::Ready,
                exit_code: 0,
                // set task_cx to trap_return
                task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                memory_set,
                trap_cx_ppn,
                base_size: user_sp,
                parent: None,
                children: Vec::new(),
            }),
        };
        // initialize trap context
        let trap_cx = process.lock_inner().trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            satp::read().bits(),
            kernel_stack_top,
            trap_handler as usize,
        );
        process
    }

    pub fn fork(self: &Arc<Process>) -> Arc<Process> {
        let mut parent_inner = self.inner.lock();
        let memory_set = MemorySet::from_existed(&parent_inner.memory_set);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).page_number_floor())
            .unwrap()
            .ppn();
        let pid = Pid::new();
        let kernel_stack = KernelStack::new(&pid);
        let kernel_stack_top = kernel_stack.top();
        let process = Arc::new(Process {
            pid,
            kernel_stack,
            inner: spin::Mutex::new(ProcessInner {
                parent: Some(Arc::downgrade(self)),
                children: Vec::new(),
                status: ProcessStatus::Ready,
                exit_code: 0,
                task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                memory_set,
                trap_cx_ppn,
                base_size: parent_inner.base_size,
            }),
        });
        parent_inner.children.push(process.clone());
        process.lock_inner().trap_cx().kernel_sp = kernel_stack_top;
        process
    }

    pub fn exec(&self, elf_data: &[u8]) {
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).page_number_floor())
            .unwrap()
            .ppn();
        let mut inner = self.lock_inner();
        inner.memory_set = memory_set;
        inner.trap_cx_ppn = trap_cx_ppn;
        inner.base_size = user_sp;

        *inner.trap_cx() = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().satp_token(),
            self.kernel_stack.top(),
            trap_handler as usize,
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum ProcessStatus {
    Ready,
    Running,
    Exited,
}
