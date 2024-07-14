#![no_std]
#![no_main]

use user_lib::syscall::sys_fork;

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main(_argc: usize, _argv: &[&str]) -> i32 {
    let ret = sys_fork();
    if ret == 0 {
        println!("I am child");
    } else {
        println!("I am parent");
    }
    0
}
