//! The main module and entrypoint
//!
//! The operating system and app also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
mod console;
mod config;
mod lang_items;
mod logging;
mod memory;
mod sbi;
mod syscall;
mod task;
mod timer;
mod trap;

extern crate alloc;
use core::arch::global_asm;
use log::*;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!(concat!(env!("OUT_DIR"), "/link_apps.S")));

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    use crate::sbi::shutdown;

    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    shutdown(false);
}

/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

/// the rust entry-point of kernel
#[no_mangle]
pub fn rust_main() -> ! {
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
    }
    clear_bss();

    logging::init();
    trace!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    trace!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    trace!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    trace!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    trace!(
        "boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize,
        boot_stack_lower_bound as usize
    );

    memory::init();
    trap::init();

    #[cfg(test)]
    test_main();

    task::run_processes();
}
