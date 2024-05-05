use core::arch::asm;

use riscv::register::sstatus::{self, set_spp, Sstatus, SPP};

use super::trap_handler;

/// Trap Context
#[repr(C)]
pub struct TrapContext {
    /// general regs[0..31]
    pub x: [usize; 32],
    /// CSR sstatus
    pub sstatus: Sstatus,
    /// CSR sepc
    pub sepc: usize,
    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub trap_handler: usize,
}

#[no_mangle]
fn __alltraps_rust() {
    let mut context = TrapContext {
        x: [0; 32],
        sstatus: sstatus::read(),
        sepc: 0,
        kernel_satp: 0,
        kernel_sp: 0,
        trap_handler: trap_handler as usize,
    };
    unsafe {
        asm!("csrrw sp, sscratch, sp");
        asm!("", out("x11") context.x[0]);
    }
}

impl TrapContext {
    /// set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    /// init app context
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let sstatus = sstatus::read(); // CSR sstatus
        unsafe {
            set_spp(SPP::User); //previous privilege mode: user mode
        }
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry, // entry point of app
            kernel_satp,
            kernel_sp, // kernel stack pointer
            trap_handler,
        };
        cx.set_sp(sp); // app's user stack pointer
        cx // return initial Trap Context of app
    }
}
