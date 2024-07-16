#![no_std]
#![no_main]

use user_lib::syscall::{sys_exec, sys_fork};

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main(_argc: usize, _argv: &[&str]) -> i32 {
    if sys_fork() == 0 {
        println!("I am child");
        sys_exec("1\0".as_ptr());
    } else {
        println!("I am parent");
    }
    0
}
