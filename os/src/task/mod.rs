//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;
pub use crate::syscall;
use crate::config::MAX_APP_NUM;
use crate::config::MAX_SYSCALL_NUM;
use crate::loader::{get_num_app, init_app_cx};
use crate::sync::UPSafeCell;
use lazy_static::*;
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};
pub use syscall::TaskInfo;
pub use syscall::TimeVal;
use crate::timer::*;
pub use context::TaskContext;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    pub num_app: usize,
    /// use inner value to get mutable access
    pub inner: UPSafeCell<TaskManagerInner>,
}

/// Inner of Task Manager
pub struct TaskManagerInner {
    /// task list
    pub tasks: [TaskControlBlock; MAX_APP_NUM],
    /// TaskInfo list
    pub task_infos: [TaskInfo; MAX_APP_NUM],
    /// id of current `Running` task
    pub current_task: usize,
}

lazy_static! {
    /// Global variable: TASK_MANAGER
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
        }; MAX_APP_NUM];
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_cx = TaskContext::goto_restore(init_app_cx(i));
            task.task_status = TaskStatus::Ready;
        }
        // 对 task_infos 进行初始化
        let task_infos = [TaskInfo{
            status: TaskStatus::UnInit,
            syscall_times:[0; MAX_SYSCALL_NUM],
            time: 0,
        }; MAX_APP_NUM];

        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    task_infos,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch3, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        // 取出第一个 taskinfos 
        inner.tasks[0].task_status = TaskStatus::Running;
        // 将 taskinfos 的 status 修改为 Running
        inner.task_infos[0].status = TaskStatus::Running;
        // 记录下 taskinfo0 被调用的时间
        inner.task_infos[0].time = get_time_us();
        let next_task_cx_ptr = &inner.tasks[0].task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner: core::cell::RefMut<TaskManagerInner> = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            //let mut inners = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.task_infos[next].status = TaskStatus::Running;
            // 如果 task 的 time 字段为 0 ，则是第一次被调度，记录下调用时间
            if inner.task_infos[next].time == 0{
                inner.task_infos[next].time = get_time_us();
            }
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }

    /// 记录当前系统调用的类型及次数
    pub fn record_current_syscall(&self, sys_id: usize){
    // 获取当前 task
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.task_infos[current].syscall_times[sys_id] += 1;
        drop(inner);
    }

    /// 记录当前系统调用距离 task 第一次被调用的 TimeVal
    pub fn get_time_val(&self){
        // 获取现在时间
        let current_time = get_time_us();
        // 获取当前 task 
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        // 获取时间差
        let time = current_time - inner.task_infos[current].time;
        let time_val = TimeVal{
            sec: time / 1_000_000,
            usec: time % 1_000_000,
        };
        inner.task_infos[current].time = ((time_val.sec & 0xffff) * 1000 + time_val.usec / 1000) as usize;
        drop(inner);
    }
    

}

/// Run the first task in task list.
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// Switch current `Running` task to the task we have found,
/// or there is no `Ready` task and we can exit with all applications completed
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// Change the status of current `Running` task into `Ready`.
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// Change the status of current `Running` task into `Exited`.
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}
