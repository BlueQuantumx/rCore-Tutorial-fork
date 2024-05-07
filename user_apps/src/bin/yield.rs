#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::syscall::sys_yield;

#[no_mangle]
fn main(_argc: usize, _argv: &[&str]) -> i32 {
    println!("Print before yield");
    sys_yield();
    println!("Print after yield");
    0
}
