//! File and filesystem-related syscalls

use log::trace;

use crate::{memory::translated_byte_buffer, sbi::console_read, task::current_user_token};

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

/// write buf of length `len` to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            trace!("sys_write: fd={}, buf={:p}, len={}", fd, buf, len);
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd:{fd} in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            trace!("sys_read: fd={}, buf={:p}, len={}", fd, buf, len);
            let mut buf = translated_byte_buffer(current_user_token(), buf, len).into_iter();
            let mut read_len = 0;
            loop {
                if let Some(buffer) = buf.next() {
                    let ret = console_read(buffer);
                    read_len += ret;
                    if ret != buffer.len() {
                        break;
                    }
                } else {
                    break;
                }
            }
            read_len as isize
        }
        _ => {
            panic!("Unsupported fd:{fd} in sys_read!");
        }
    }
}
