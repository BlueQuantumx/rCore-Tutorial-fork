//! The panic handler

use crate::sbi::shutdown;
use core::panic::PanicInfo;
use log::*;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!("Panicked at {}:{} {info}", location.file(), location.line(),);
    } else {
        error!("Panicked at unknown location");
    }
    shutdown(true)
}
