//! File and filesystem-related syscalls

use log::trace;

use crate::{
    memory::translated_byte_buffer,
    sbi::{console_getchar, console_read},
    task::{current_user_token, suspend_current_and_run_next_task},
};

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

/// write buf of length `len`  to a file with `fd`
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
            panic!("Unsupported fd in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            assert!(len == 1, "only support read one byte each time");
            trace!("sys_read: fd={}, buf={:p}, len={}", fd, buf, len);
            let mut buffer = translated_byte_buffer(current_user_token(), buf, len);
            let mut c = 0;
            loop {
                c = console_getchar();
                match c {
                    0 => {
                        suspend_current_and_run_next_task();
                        continue;
                    }
                    _ => break,
                }
            }
            buffer[0][0] = c as u8;
            1
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}
