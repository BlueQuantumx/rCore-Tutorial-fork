#![allow(unused)]

use log::{error, info};

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: u8) {
    sbi_rt::console_write_byte(c);
}

/// use sbi call to getchar from console (qemu uart handler)
pub fn console_getchar() -> usize {
    #[allow(deprecated)]
    sbi_rt::legacy::console_getchar()
}

/// use sbi call to shutdown the kernel
pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        info!("Shutdown the kernel");
        system_reset(Shutdown, NoReason);
    } else {
        error!("System failure, shutdown the kernel");
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}
