#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main(_argc: usize, _argv: &[&str]) -> i32 {
    println!("Hello, world!");
    1
}
