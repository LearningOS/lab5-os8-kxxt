use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task, TaskControlBlock};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use alloc::vec::{self, Vec};

use super::{SYSERR_DEADLOCK, SYSERR_UNKNOWN};

pub fn sys_sleep(ms: usize) -> isize {
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}

// LAB5 HINT: you might need to maintain data structures used for deadlock detection
// during sys_mutex_* and sys_semaphore_* syscalls
pub fn sys_mutex_create(blocking: bool) -> isize {
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        id
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_list.len() - 1
    };
    let len = process_inner.mutex_list.len();
    let mut detector_inner = process_inner
        .mutex_deadlock_detector
        .inner
        .exclusive_access();
    detector_inner.resize_update_res_cnt(len);
    detector_inner.available[id] = 1;
    id as isize
}

fn find_task_pos_by_tid(tasks: &[Option<Arc<TaskControlBlock>>], tid: usize) -> Option<usize> {
    tasks.iter().position(|t| {
        t.is_some_and(|tcb| {
            tcb.inner_exclusive_access()
                .res
                .is_some_and(|other| other.tid == tid)
        })
    })
}

fn get_tid_unchecked(task: &Option<Arc<TaskControlBlock>>) -> usize {
    task.as_ref()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid
}

// LAB5 HINT: Return -0xDEAD if deadlock is detected
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let process = current_process();
    let tid = get_tid_unchecked(&current_task());
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    let tasks = &process_inner.tasks;
    let mut detector_inner = process_inner
        .mutex_deadlock_detector
        .inner
        .exclusive_access();
    debug!("Task {tid} trying to lock {mutex_id}");
    let tidx = find_task_pos_by_tid(tasks, tid).unwrap();
    // let need_size1 = detector_inner.need.len();
    // let need_size2 = detector_inner.need[0].len();
    // debug!("tidx = {tidx}, mutex_id = {mutex_id}, shape: {need_size1}x{need_size2}");
    if process_inner.deadlock_detection && !detector_inner.request(tasks, tidx, mutex_id) {
        // Dead Lock!
        error!("Deadlock detected!");
        return SYSERR_DEADLOCK;
    }
    drop(detector_inner);
    drop(process_inner);
    drop(process);
    mutex.lock();
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mut detector_inner = process_inner
        .mutex_deadlock_detector
        .inner
        .exclusive_access();
    detector_inner.allocate(tidx, mutex_id);
    0
}

pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let tid = get_tid_unchecked(&current_task());
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mut detector_inner = process_inner
        .mutex_deadlock_detector
        .inner
        .exclusive_access();
    let tidx = find_task_pos_by_tid(&process_inner.tasks, tid).unwrap();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    detector_inner.free(tidx, mutex_id);
    drop(detector_inner);
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}

pub fn sys_semaphore_create(res_count: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}

pub fn sys_semaphore_up(sem_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.up();
    0
}

// LAB5 HINT: Return -0xDEAD if deadlock is detected
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.down();
    0
}

pub fn sys_condvar_create(_arg: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}

pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}

pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}

// LAB5 YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    let task = current_task();
    let tid = get_tid_unchecked(&task);
    let pid = current_process().pid.0;
    let enabled = if enabled == 0 {
        debug!("DEADLOCK detection disabled for task {tid} of process {pid}!");
        false
    } else if enabled == 1 {
        debug!("DEADLOCK detection enabled for task {tid} of process {pid}!");
        true
    } else {
        return SYSERR_UNKNOWN;
    };
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    inner.deadlock_detection = enabled;
    0
}
