//! Trap handling functionality
//!
//! For rCore, we have a single trap entry point, namely `__alltraps`. At
//! initialization in [`init()`], we set the `stvec` CSR to point to it.
//!
//! All traps go through `__alltraps`, which is defined in `trap.S`. The
//! assembly language code does just enough work restore the kernel space
//! context, ensuring that Rust code safely runs, and transfers control to
//! [`trap_handler()`].
//!
//! It then calls different functionality based on what exactly the exception
//! was. For example, timer interrupts trigger task preemption, and syscalls go
//! to [`syscall()`].

mod context;

use crate::{
    config::{TRAMPOLINE, TRAP_CONTEXT},
    syscall::syscall,
    task::{
        current_trap_cx, current_user_token, exit_current_and_run_next_task,
        suspend_current_and_run_next_task,
    },
    timer::set_next_trigger,
};
pub use context::TrapContext;
use core::arch::{asm, global_asm};
use log::{trace, warn};
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie, stval,
    stvec::{self, TrapMode},
};

global_asm!(include_str!("trap.S"));

extern "C" {
    pub fn __alltraps() -> !;
    pub fn __restore(trap_cx_addr: usize) -> !;
}

/// initialize CSR `stvec` as the entry of `__alltraps`
pub fn init() {
    set_kernel_trap_entry();
    #[cfg(feature = "time-sharing")]
    {
        enable_timer_interrupt();
        set_next_trigger();
    }
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

fn trap_from_kernel() {
    panic!("a trap from kernel");
}

#[no_mangle]
/// handle an interrupt, exception, or system call from user space
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let cx = current_trap_cx();
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            let ret = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
            // reacquire the TrapContext because syscall may change it
            let cx = current_trap_cx();
            cx.x[10] = ret;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            warn!("PageFault in application, kernel killed it.");
            exit_current_and_run_next_task();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            warn!("IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next_task();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            trace!("Supervisor timer triggered");
            set_next_trigger();
            suspend_current_and_run_next_task();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",         // jump to new addr of __restore asm function
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,      // a0 = virt addr of Trap Context
            in("a1") user_satp,        // a1 = phy addr of usr page table
            options(noreturn)
        );
    }
}
