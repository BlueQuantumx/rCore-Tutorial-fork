//! The panic handler

use crate::sbi::shutdown;
use core::panic::PanicInfo;
use log::*;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "Panicked at {}:{}",
            location.file(),
            location.line(),
            // info.message().unwrap()
        );
    } else {
        error!("Panicked at unknown location");
    }
    shutdown(true)
}
