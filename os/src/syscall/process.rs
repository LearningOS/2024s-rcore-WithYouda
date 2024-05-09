//! Process management syscalls
pub use crate::{
    config::MAX_SYSCALL_NUM,
    task::{exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, TASK_MANAGER},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
/// 好好好，一定得要有注释说明是吧
pub struct TimeVal {
    /// 秒？
    pub sec: usize,
    /// ？？？
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    // 成功返回0
    unsafe{
        // 如何把每次系统调用类型和次数记录下来呢？
        // 每一个 task 维护一个自己的 TaskInfo ，当调用这个系统调用时候，将自己维护的 TaskInfo 赋值给传进来的 TaskInfo
        // 先获取一个 TASK_MANAGER 的 inner
        let inner = TASK_MANAGER.inner.exclusive_access();
        let current = inner.current_task;
        // 将获取到当前的 task_infos 赋值给 传入的 _ti
        * _ti = inner.task_infos[current];
        drop(inner);
        0
    }
    // 失败返回-1
    
}
