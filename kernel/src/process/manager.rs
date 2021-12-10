use super::Pid;

use {
    crate::process::Process,
    alloc::collections::{BTreeMap, VecDeque},
    conquer_once::spin::Lazy,
    spinning_top::{Spinlock, SpinlockGuard},
};

static MANAGER: Spinlock<Manager> = Spinlock::new(Manager::new());

static WOKEN_PIDS: Lazy<Spinlock<VecDeque<Pid>>> = Lazy::new(|| Spinlock::new(VecDeque::new()));

pub(super) fn add(p: Process) {
    lock_manager().add(p);
}

pub(super) fn handle_running_mut<T, U>(f: T) -> U
where
    T: FnOnce(&mut Process) -> U,
{
    lock_manager().handle_running_mut(f)
}

pub(super) fn handle_mut<T, U>(pid: Pid, f: T) -> U
where
    T: FnOnce(&mut Process) -> U,
{
    lock_manager().handle_mut(pid, f)
}

pub(super) fn handle_running<T, U>(f: T) -> U
where
    T: FnOnce(&Process) -> U,
{
    lock_manager().handle_running(f)
}

pub(super) fn handle<T, U>(pid: Pid, f: T) -> U
where
    T: FnOnce(&Process) -> U,
{
    lock_manager().handle(pid, f)
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

struct Manager {
    processes: BTreeMap<Pid, Process>,
}
impl Manager {
    const fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
        }
    }

    fn add(&mut self, p: Process) {
        let pid = p.id();

        let r = self.processes.insert(pid, p);

        assert!(r.is_none(), "Duplicated process with PID {}.", pid);
    }

    fn handle_running<T, U>(&self, f: T) -> U
    where
        T: FnOnce(&Process) -> U,
    {
        self.handle(active_pid(), f)
    }

    fn handle<T, U>(&self, pid: Pid, f: T) -> U
    where
        T: FnOnce(&Process) -> U,
    {
        let p = self
            .processes
            .get(&pid)
            .unwrap_or_else(|| panic!("Process of PID {} does not exist.", pid));

        f(p)
    }

    fn handle_running_mut<T, U>(&mut self, f: T) -> U
    where
        T: FnOnce(&mut Process) -> U,
    {
        let pid = active_pid();

        self.handle_mut(pid, f)
    }

    fn handle_mut<T, U>(&mut self, pid: Pid, f: T) -> U
    where
        T: FnOnce(&mut Process) -> U,
    {
        let p = self
            .processes
            .get_mut(&pid)
            .unwrap_or_else(|| panic!("Process of PID {} does not exist.", pid));

        f(p)
    }
}

fn lock_manager() -> SpinlockGuard<'static, Manager> {
    MANAGER
        .try_lock()
        .expect("Failed to acquire the lock of `PROCESSES`.")
}

fn lock_queue() -> SpinlockGuard<'static, VecDeque<Pid>> {
    WOKEN_PIDS
        .try_lock()
        .expect("Failed to acquire the lock of `WOKEN_PIDS`.")
}
