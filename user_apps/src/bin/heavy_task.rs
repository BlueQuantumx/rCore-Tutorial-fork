#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

static mut COUNT: usize = 0;

#[no_mangle]
fn main(_argc: usize, _argv: &[&str]) -> i32 {
    println!("Heavy task starts");
    for _ in 0..1000000 {
        unsafe {
            COUNT += 1;
        }
    }
    println!("Heavy task ends, COUNT = {}", unsafe { COUNT });
    0
}
