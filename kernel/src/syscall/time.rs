use crate::timer;

pub fn sys_get_time() -> isize {
    timer::read() as isize
}
