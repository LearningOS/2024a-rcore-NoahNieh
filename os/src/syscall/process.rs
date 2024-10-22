//! Process management syscalls

use crate::{
    config::MAX_SYSCALL_NUM, mm::{write_to_user_buffer, VirtAddr}, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_task_running_time, get_current_task_status, get_current_task_sys_call_times, mmap, munmap, suspend_current_and_run_next, TaskStatus
    }, timer::get_time_us
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let result = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
    };
    let token = current_user_token();
    let len = core::mem::size_of::<TimeVal>();
    write_to_user_buffer(token, _ts as *const u8, len, unsafe {
        core::slice::from_raw_parts(&result as *const TimeVal as *const u8, len)
    });
    0
}

pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let status = get_current_task_status();
    let syscall_times = get_current_task_sys_call_times();
    let running_time = get_current_task_running_time();
    if let (Some(status), Some(syscall_times), Some(running_time)) =
        (status, syscall_times, running_time)
    {
        let result = TaskInfo {
            status,
            syscall_times,
            time: running_time,
        };
        let token = current_user_token();
        let len = core::mem::size_of::<TaskInfo>();
        write_to_user_buffer(token, ti as *const u8, len, unsafe {
            core::slice::from_raw_parts(&result as *const TaskInfo as *const u8, len)
        });
        0
    } else {
        -1
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    let start_va:VirtAddr= _start.into();
    if !start_va.aligned() {
        return -1;
    }
    if (_port & (!0x7)) != 0 || (_port & 0x7) == 0 {
        return -1;
    }
    let end_va: VirtAddr = (_start + _len).into();
    let start_vpn = start_va.floor();
    let end_vpn = end_va.ceil();
    match mmap(start_vpn, end_vpn, _port) {
        Ok(_) => 0,
        Err(_) => -1
    }

}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let start_va:VirtAddr= _start.into();
    if !start_va.aligned() {
        return -1;
    }
    let start_vpn = start_va.floor();
    let end_va: VirtAddr = (_start + _len).into();
    let end_vpn = end_va.ceil();
    match munmap(start_vpn, end_vpn) {
        Ok(_) => 0,
        Err(_) => -1
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
