use super::Pid;

use {
    crate::process::Process,
    alloc::collections::{BTreeMap, VecDeque},
    conquer_once::spin::Lazy,
    spinning_top::{Spinlock, SpinlockGuard},
};

static PROCESSES: Spinlock<BTreeMap<Pid, Process>> = Spinlock::new(BTreeMap::new());

static WOKEN_PIDS: Lazy<Spinlock<VecDeque<Pid>>> = Lazy::new(|| Spinlock::new(VecDeque::new()));

pub(super) fn add(p: Process) {
    let id = p.id();
    let r = PROCESSES.lock().insert(id, p);
    assert!(r.is_none(), "Duplicated process.");
}

pub(super) fn handle_running_mut<T, U>(f: T) -> U
where
    T: FnOnce(&mut Process) -> U,
{
    let id = active_pid();
    handle_mut(id, f)
}

pub(super) fn handle_mut<T, U>(id: Pid, f: T) -> U
where
    T: FnOnce(&mut Process) -> U,
{
    let mut l = lock_processes();
    let p = l
        .get_mut(&id)
        .unwrap_or_else(|| panic!("Process of PID {} does not exist.", id));
    f(p)
}

pub(super) fn handle_running<T, U>(f: T) -> U
where
    T: FnOnce(&Process) -> U,
{
    let id = active_pid();
    handle(id, f)
}

pub(super) fn handle<T, U>(id: Pid, f: T) -> U
where
    T: FnOnce(&Process) -> U,
{
    let l = lock_processes();
    let p = l
        .get(&id)
        .unwrap_or_else(|| panic!("Process of PID {} does not exist.", id));
    f(p)
}

fn lock_processes() -> SpinlockGuard<'static, BTreeMap<Pid, Process>> {
    PROCESSES
        .try_lock()
        .expect("Failed to acquire the lock of `PROCESSES`.")
}

pub(super) fn change_active_pid() {
    lock_queue().rotate_left(1);
}

pub(super) fn active_pid() -> Pid {
    lock_queue()[0]
}

pub(super) fn pop() -> Pid {
    lock_queue()
        .pop_front()
        .expect("All processes are terminated.")
}

pub(super) fn push(id: Pid) {
    lock_queue().push_back(id);
}

fn lock_queue() -> SpinlockGuard<'static, VecDeque<Pid>> {
    WOKEN_PIDS
        .try_lock()
        .expect("Failed to acquire the lock of `WOKEN_PIDS`.")
}
