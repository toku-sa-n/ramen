use {
    super::{Pid, ReceiveFrom},
    crate::{
        mem::{self, accessor::Single, paging},
        process::Process,
        tests, tss,
    },
    alloc::{collections::BTreeMap, vec::Vec},
    message::Message,
    predefined_mmap::INTERRUPT_STACK,
    spinning_top::{Spinlock, SpinlockGuard},
    x86_64::{
        registers::control::Cr3, software_interrupt, structures::paging::PhysFrame, PhysAddr,
        VirtAddr,
    },
};

static SCHEDULER: Spinlock<Scheduler> = Spinlock::new(Scheduler::new());

pub(crate) fn send(msg: VirtAddr, to: Pid) {
    lock_manager().send(msg, to);
}

pub(crate) fn receive_from_any(msg_buf: VirtAddr) {
    lock_manager().receive_from_any(msg_buf);
}

pub(crate) fn receive_from(msg_buf: VirtAddr, from: Pid) {
    lock_manager().receive_from(msg_buf, from);
}

pub(crate) fn exit_process() -> ! {
    set_temporary_stack_frame();

    // TODO: Call this. Currently this calling will cause a panic because the `KBox` is not mapped
    // to this process.
    // super::collections::process::remove(super::manager::getpid().into());

    pop();
    cause_timer_interrupt();
}

pub(crate) fn switch() -> VirtAddr {
    lock_manager().switch()
}

pub(crate) fn current_process_name() -> &'static str {
    lock_manager().current_process_name()
}

pub(super) fn add(p: Process) {
    lock_manager().add(p);
}

pub(super) fn pop() -> Pid {
    lock_manager().pop()
}

pub(super) fn push(pid: Pid) {
    lock_manager().push(pid);
}

fn set_temporary_stack_frame() {
    tss::set_interrupt_stack(*INTERRUPT_STACK);
}

fn cause_timer_interrupt() -> ! {
    unsafe {
        software_interrupt!(0x20);
    }

    unreachable!();
}

struct Scheduler {
    processes: BTreeMap<Pid, Process>,

    woken_pids: Vec<Pid>,
}
impl Scheduler {
    const fn new() -> Self {
        Self {
            processes: BTreeMap::new(),

            woken_pids: Vec::new(),
        }
    }

    fn add(&mut self, p: Process) {
        let pid = p.id();

        let r = self.processes.insert(pid, p);

        assert!(r.is_none(), "Duplicated process with PID {}.", pid);
    }

    fn push(&mut self, pid: Pid) {
        self.woken_pids.push(pid);
    }

    fn pop(&mut self) -> Pid {
        self.woken_pids.remove(0)
    }

    fn active_pid(&self) -> Pid {
        self.woken_pids[0]
    }

    fn change_active_pid(&mut self) {
        self.woken_pids.rotate_left(1);
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

    fn switch(&mut self) -> VirtAddr {
        Switcher(self).switch()
    }

    fn current_process_name(&self) -> &'static str {
        self.handle_running(|p| p.name)
    }

    fn current_pml4(&self) -> PhysFrame {
        self.handle_running(|p| p.pml4)
    }

    fn current_stack_frame_top_addr(&self) -> VirtAddr {
        self.handle_running(Process::stack_frame_top_addr)
    }

    fn current_stack_frame_bottom_addr(&self) -> VirtAddr {
        self.handle_running(Process::stack_frame_bottom_addr)
    }

    fn handle_running<T, U>(&self, f: T) -> U
    where
        T: FnOnce(&Process) -> U,
    {
        self.handle(self.active_pid(), f)
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
        self.handle_mut(self.active_pid(), f)
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

struct Sender<'a> {
    manager: &'a mut Scheduler,
    msg: PhysAddr,
    to: Pid,
}
impl<'a> Sender<'a> {
    fn new(manager: &'a mut Scheduler, msg: VirtAddr, to: Pid) -> Self {
        assert_ne!(manager.active_pid(), to, "Tried to send a message to self.");

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
            [
                Some(ReceiveFrom::Id(self.manager.active_pid())),
                Some(ReceiveFrom::Any),
            ]
            .contains(&p.receive_from)
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

        unsafe { copy_msg(self.msg, dst, self.manager.active_pid()) }
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
        self.manager.push(self.to);
    }

    fn set_msg_buf_and_sleep(&mut self) {
        self.set_msg_buf();
        self.add_self_as_trying_to_send();
        self.mark_as_sending();
        self.manager.pop();
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
        let pid = self.manager.active_pid();
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
    manager: &'a mut Scheduler,
    msg_buf: PhysAddr,
    from: ReceiveFrom,
}
impl<'a> Receiver<'a> {
    fn new_from_any(manager: &'a mut Scheduler, msg_buf: VirtAddr) -> Self {
        let msg_buf = virt_to_phys(msg_buf);

        Self {
            manager,
            msg_buf,
            from: ReceiveFrom::Any,
        }
    }

    fn new_from(manager: &'a mut Scheduler, msg_buf: VirtAddr, from: Pid) -> Self {
        assert_ne!(
            manager.active_pid(),
            from,
            "Tried to receive a message from self."
        );

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
        self.manager.push(src_pid);
    }

    fn set_msg_buf_and_sleep(&mut self) {
        self.set_msg_buf();
        self.mark_as_receiving();
        self.manager.pop();
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

struct Switcher<'a>(&'a mut Scheduler);
impl Switcher<'_> {
    fn switch(self) -> VirtAddr {
        if cfg!(feature = "qemu_test") {
            tests::process::count_switch();
        }

        self.0.change_active_pid();
        self.switch_pml4();
        self.register_current_stack_frame_with_tss();
        self.current_stack_frame_top_addr()
    }

    fn switch_pml4(&self) {
        let (_, f) = Cr3::read();
        let pml4 = self.0.current_pml4();

        // SAFETY: The PML4 frame is correct one and flags are unchanged.
        unsafe { Cr3::write(pml4, f) }
    }

    fn register_current_stack_frame_with_tss(&self) {
        tss::set_interrupt_stack(self.0.current_stack_frame_bottom_addr());
    }

    pub(super) fn current_stack_frame_top_addr(&self) -> VirtAddr {
        self.0.current_stack_frame_top_addr()
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
    paging::translate_addr(v).expect("Failed to convert a virtual address to physical one.")
}

fn lock_manager() -> SpinlockGuard<'static, Scheduler> {
    SCHEDULER
        .try_lock()
        .expect("Failed to acquire the lock of `PROCESSES`.")
}
