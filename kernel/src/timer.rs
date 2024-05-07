//! RISC-V timer-related functionality

use crate::config::{CLOCK_FREQ, TICKS_PER_SEC};
use crate::sbi::set_timer;
use riscv::register::time;

pub fn read() -> u64 {
    time::read() as u64
}

/// set the next timer interrupt
pub fn set_next_trigger() {
    set_timer((read() + CLOCK_FREQ / TICKS_PER_SEC) as u64);
}
