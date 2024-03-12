#![no_std]
#![no_main]

use crate::syscall::sys_exit;

pub mod console;
pub mod syscall;

#[panic_handler]
fn panic_handler(_panic_info: &core::panic::PanicInfo) -> ! {
    println!("[user_lib] PANIC!");
    loop {}
}

fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }
    (start_bss as usize..end_bss as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    });
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    extern "Rust" {
        fn main(_argc: usize, _argv: &[&str]) -> i32;
    }
    clear_bss();
    unsafe {
        sys_exit(main(0, &[]));
    }
    unreachable!("user_lib: _start returned!")
}
