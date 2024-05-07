use super::TaskContext;
use core::arch::asm;

pub fn switch_rust(
    current_task_cx_addr: *mut TaskContext,
    next_task_cx_addr: *const TaskContext,
) -> ! {
    let current_task = unsafe { current_task_cx_addr.as_mut().unwrap() };
    unsafe {
        asm!("sd sp, {}", out(reg) current_task.sp);
        asm!("sd ra, {}", out(reg) current_task.ra);
        asm!("sd s0, {}", out(reg) current_task.s[0]);
        asm!("sd s1, {}", out(reg) current_task.s[1]);
        asm!("sd s2, {}", out(reg) current_task.s[2]);
        asm!("sd s3, {}", out(reg) current_task.s[3]);
        asm!("sd s4, {}", out(reg) current_task.s[4]);
        asm!("sd s5, {}", out(reg) current_task.s[5]);
        asm!("sd s6, {}", out(reg) current_task.s[6]);
        asm!("sd s7, {}", out(reg) current_task.s[7]);
        asm!("sd s8, {}", out(reg) current_task.s[8]);
        asm!("sd s9, {}", out(reg) current_task.s[9]);
        asm!("sd s10, {}", out(reg) current_task.s[10]);
        asm!("sd s11, {}", out(reg) current_task.s[11]);
    }

    let next_task = unsafe { next_task_cx_addr.as_ref().unwrap() };
    unsafe {
        asm!("ld sp, {}", in(reg) next_task.sp);
        asm!("ld ra, {}", in(reg) next_task.ra);
        asm!("ld s0, {}", in(reg) next_task.s[0]);
        asm!("ld s1, {}", in(reg) next_task.s[1]);
        asm!("ld s2, {}", in(reg) next_task.s[2]);
        asm!("ld s3, {}", in(reg) next_task.s[3]);
        asm!("ld s4, {}", in(reg) next_task.s[4]);
        asm!("ld s5, {}", in(reg) next_task.s[5]);
        asm!("ld s6, {}", in(reg) next_task.s[6]);
        asm!("ld s7, {}", in(reg) next_task.s[7]);
        asm!("ld s8, {}", in(reg) next_task.s[8]);
        asm!("ld s9, {}", in(reg) next_task.s[9]);
        asm!("ld s10, {}", in(reg) next_task.s[10]);
        asm!("ld s11, {}", in(reg) next_task.s[11], options(noreturn));
    }
}
