#![allow(unused)]

use log::{error, info, trace};
use sbi_rt::Physical;

use crate::{
    memory::{translated_byte_buffer, KERNEL_SPACE},
    task::current_user_token,
};

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: u8) {
    sbi_rt::console_write_byte(c);
}

/// use sbi call to read from console (qemu uart handler)
pub fn console_read(buf: &mut [u8]) -> usize {
    match sbi_rt::console_read(Physical::new(
        buf.len(),
        buf.as_ptr() as u32 as usize,
        buf.as_ptr() as usize >> 32,
    )) {
        sbi_rt::SbiRet { error: 0, value } => value,
        sbi_rt::SbiRet { error, value } => panic!("console_read failed: {error}"),
    }
}

pub fn console_write(buf: &mut [u8]) -> usize {
    match sbi_rt::console_write(Physical::new(
        buf.len(),
        buf.as_ptr() as u32 as usize,
        buf.as_ptr() as usize >> 32,
    )) {
        sbi_rt::SbiRet { error: 0, value } => value,
        sbi_rt::SbiRet { error, value } => panic!("console_write failed: {error}"),
    }
}

pub fn set_timer(duration: u64) {
    sbi_rt::set_timer(duration);
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
