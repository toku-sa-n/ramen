use {
    super::{get_slot_id, manager, Pid, ReceiveFrom},
    crate::{
        mem::{self, accessor::Single},
        process::Process,
    },
    alloc::collections::{BTreeMap, VecDeque},
    conquer_once::spin::Lazy,
    mem::paging::pml4::PML4,
    message::Message,
    spinning_top::{Spinlock, SpinlockGuard},
    x86_64::{
        structures::paging::{PhysFrame, Translate},
        PhysAddr, VirtAddr,
    },
};

static MANAGER: Spinlock<Manager> = Spinlock::new(Manager::new());

static WOKEN_PIDS: Lazy<Spinlock<VecDeque<Pid>>> = Lazy::new(|| Spinlock::new(VecDeque::new()));

pub(crate) fn send(msg: VirtAddr, to: Pid) {
    lock_manager().send(msg, to);
}

pub(crate) fn receive_from_any(msg_buf: VirtAddr) {
    lock_manager().receive_from_any(msg_buf);
}

pub(crate) fn receive_from(msg_buf: VirtAddr, from: Pid) {
    lock_manager().receive_from(msg_buf, from);
}

pub(super) fn add(p: Process) {
    lock_manager().add(p);
}

pub(super) fn handle_running_mut<T, U>(f: T) -> U
where
    T: FnOnce(&mut Process) -> U,
{
    lock_manager().handle_running_mut(f)
}

pub(super) fn handle_running<T, U>(f: T) -> U
where
    T: FnOnce(&Process) -> U,
{
    lock_manager().handle_running(f)
}

pub(super) fn pml4_of_running_process() -> PhysFrame {
    lock_manager().pml4_of_running_process()
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

    fn send(&mut self, msg: VirtAddr, to: Pid) {
        Sender::new(self, msg, to).send();
    }

    fn receive_from_any(&mut self, msg_buf: VirtAddr) {
        Receiver::new_from_any(self, msg_buf).receive();
    }

    fn receive_from(&mut self, msg_buf: VirtAddr, from: Pid) {
        Receiver::new_from(self, msg_buf, from).receive();
    }

    fn pml4_of_running_process(&self) -> PhysFrame {
        self.handle_running(|p| p.pml4)
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

struct Sender<'a> {
    manager: &'a mut Manager,
    msg: PhysAddr,
    to: Pid,
}
impl<'a> Sender<'a> {
    fn new(manager: &'a mut Manager, msg: VirtAddr, to: Pid) -> Self {
        assert_ne!(get_slot_id(), to, "Tried to send a message to self.");

        let msg = virt_to_phys(msg);

        Self { manager, msg, to }
    }

    fn send(mut self) {
        if self.is_receiver_waiting() {
            self.copy_msg_and_wake();
        } else {
            self.set_msg_buf_and_sleep();
        }
    }

    fn is_receiver_waiting(&self) -> bool {
        self.manager.handle(self.to, |p| {
            [Some(ReceiveFrom::Id(get_slot_id())), Some(ReceiveFrom::Any)].contains(&p.receive_from)
        })
    }

    fn copy_msg_and_wake(&mut self) {
        self.copy_msg();
        self.remove_msg_buf();
        self.wake_dst();
    }

    fn copy_msg(&self) {
        let dst = self.manager.handle(self.to, |p| p.msg_ptr);
        let dst = dst.expect("Message destination address is not specified.");

        unsafe { copy_msg(self.msg, dst, get_slot_id()) }
    }

    fn remove_msg_buf(&mut self) {
        self.manager.handle_running_mut(|p| {
            p.msg_ptr = None;
            p.send_to = None;
        });
    }

    fn wake_dst(&mut self) {
        self.manager.handle_mut(self.to, |p| {
            p.msg_ptr = None;
            p.receive_from = None;
        });
        manager::push(self.to);
    }

    fn set_msg_buf_and_sleep(&mut self) {
        self.set_msg_buf();
        self.add_self_as_trying_to_send();
        self.mark_as_sending();
        sleep();
    }

    fn set_msg_buf(&mut self) {
        self.manager.handle_running_mut(|p| {
            if p.msg_ptr.is_none() {
                p.msg_ptr = Some(self.msg);
            } else {
                panic!("Message is already stored.");
            }
        });
    }

    fn add_self_as_trying_to_send(&mut self) {
        let pid = get_slot_id();
        self.manager.handle_mut(self.to, |p| {
            p.pids_try_to_send_this_process.push_back(pid);
        });
    }

    fn mark_as_sending(&mut self) {
        self.manager
            .handle_running_mut(|p| p.send_to = Some(self.to));
    }
}

struct Receiver<'a> {
    manager: &'a mut Manager,
    msg_buf: PhysAddr,
    from: ReceiveFrom,
}
impl<'a> Receiver<'a> {
    fn new_from_any(manager: &'a mut Manager, msg_buf: VirtAddr) -> Self {
        let msg_buf = virt_to_phys(msg_buf);

        Self {
            manager,
            msg_buf,
            from: ReceiveFrom::Any,
        }
    }

    fn new_from(manager: &'a mut Manager, msg_buf: VirtAddr, from: Pid) -> Self {
        assert_ne!(get_slot_id(), from, "Tried to receive a message from self.");

        let msg_buf = virt_to_phys(msg_buf);

        Self {
            manager,
            msg_buf,
            from: ReceiveFrom::Id(from),
        }
    }

    fn receive(mut self) {
        if self.is_sender_waiting() {
            self.copy_msg_and_wake();
        } else {
            self.set_msg_buf_and_sleep();
        }
    }

    fn is_sender_waiting(&self) -> bool {
        if let ReceiveFrom::Id(id) = self.from {
            self.manager.handle(id, |p| p.send_to == Some(id))
        } else {
            self.manager
                .handle_running(|p| !p.pids_try_to_send_this_process.is_empty())
        }
    }

    fn copy_msg_and_wake(&mut self) {
        let src_pid = self.src_pid();

        self.copy_msg(src_pid);
        self.wake_sender(src_pid);
    }

    fn src_pid(&mut self) -> Pid {
        if let ReceiveFrom::Id(id) = self.from {
            id
        } else {
            self.manager.handle_running_mut(|p| {
                p.pids_try_to_send_this_process
                    .pop_front()
                    .expect("No process is waiting to send.")
            })
        }
    }

    fn copy_msg(&self, src_slot_id: Pid) {
        let src = self.manager.handle(src_slot_id, |p| p.msg_ptr);
        let src = src.expect("The message pointer of the sender is not set.");

        unsafe { copy_msg(src, self.msg_buf, src_slot_id) }
    }

    fn wake_sender(&mut self, src_pid: Pid) {
        self.manager.handle_mut(src_pid, |p| {
            p.msg_ptr = None;
            p.send_to = None;
        });
        manager::push(src_pid);
    }

    fn set_msg_buf_and_sleep(&mut self) {
        self.set_msg_buf();
        self.mark_as_receiving();
        sleep();
    }

    fn set_msg_buf(&mut self) {
        self.manager.handle_running_mut(|p| {
            if p.msg_ptr.is_none() {
                p.msg_ptr = Some(self.msg_buf);
            } else {
                panic!("Message is already stored.");
            }
        });
    }

    fn mark_as_receiving(&mut self) {
        self.manager
            .handle_running_mut(|p| p.receive_from = Some(self.from));
    }
}

/// # Safety
///
/// `src` and `dst` must be the correct addresses where a message is located and copied.
unsafe fn copy_msg(src: PhysAddr, dst: PhysAddr, sender_slot_id: Pid) {
    // SAFETY: The caller must ensure that `src` is the correct address of the message.
    let mut src: Single<Message> = unsafe { mem::accessor::new(src) };

    // SAFETY: The caller must ensure that `dst` is the correct address to save a message.
    let mut dst = unsafe { mem::accessor::new(dst) };

    src.update_volatile(|m| m.header.sender = sender_slot_id);
    dst.write_volatile(src.read_volatile());
}

fn virt_to_phys(v: VirtAddr) -> PhysAddr {
    PML4.lock()
        .translate_addr(v)
        .expect("Failed to convert a virtual address to physical one.")
}

fn sleep() {
    super::block_running();
}
