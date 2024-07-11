#![allow(unused)]

use log::{error, info, trace};
use sbi_rt::Physical;

use crate::memory::{translated_byte_buffer, KERNEL_SPACE};

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: u8) {
    sbi_rt::console_write_byte(c);
}

pub fn console_getchar() -> usize {
    let mut buf = [0u8; 1];
    let ret = console_read(&mut buf);
    match ret {
        0 => 0,
        _ => buf[0] as usize,
    }
}

/// use sbi call to read from console (qemu uart handler)
pub fn console_read(buf: &mut [u8]) -> usize {
    let buffers = translated_byte_buffer(KERNEL_SPACE.lock().satp_token(), buf.as_ptr(), buf.len());
    let mut len = 0;
    for buffer in buffers {
        let ret = sbi_rt::console_read(Physical::new(
            buffer.len(),
            buffer.as_ptr() as usize & 0xffffffff,
            buffer.as_ptr() as usize >> 32,
        ));
        match ret.error {
            0 => {
                len += ret.value;
                if ret.value == 0 {
                    break;
                }
            }
            _ => panic!("console_read failed: {}", ret.error),
        }
    }
    len
}

pub fn console_write(buf: &mut [u8]) -> usize {
    sbi_rt::console_write(Physical::new(
        buf.len(),
        buf.as_ptr() as u32 as usize,
        buf.as_ptr() as usize >> 32,
    ))
    .value
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
