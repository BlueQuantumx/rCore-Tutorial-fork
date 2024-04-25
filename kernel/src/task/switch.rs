use super::TaskContext;
use core::arch::asm;

fn switch(current_task_cx_addr: *mut TaskContext, next_task_cx_addr: *const TaskContext) {
    unsafe {
        asm!("sd sp, {}", out(reg) current_task_cx_addr.as_mut().unwrap().sp);
        asm!("sd ra, {}", out(reg) current_task_cx_addr.as_mut().unwrap().ra);
        asm!("sd s0, {}", out(reg) current_task_cx_addr.as_mut().unwrap().s[0]);
        // TODO: save s1-s11
    }

    unsafe {
        asm!("ld sp, {}", in(reg) next_task_cx_addr.as_ref().unwrap().sp);
        asm!("ld ra, {}", in(reg) next_task_cx_addr.as_ref().unwrap().ra);
        asm!("ld s0, {}", in(reg) next_task_cx_addr.as_ref().unwrap().s[0]);
        // TODO: restore s1-s11
    }
}
