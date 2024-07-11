#![no_std]
#![no_main]

pub mod console;
pub mod syscall;

#[panic_handler]
fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    println!("[user_lib] {}", panic_info);
    syscall::sys_exit(-1);
    unreachable!("reach after sys_exit in panic_handler!")
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
        syscall::sys_exit(main(0, &[]));
    }
    unreachable!("user_lib: _start returned!")
}
