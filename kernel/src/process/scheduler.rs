use {
    super::{
        priority::{Priority, LEAST_PRIORITY},
        receive_from::ReceiveFrom,
        Pid,
    },
    crate::{
        mem::{self, accessor::Single, paging},
        process::{status::Status, Process},
        tests,
    },
    alloc::collections::{BTreeMap, VecDeque},
    array_init::array_init,
    conquer_once::spin::Lazy,
    message::Message,
    spinning_top::{Spinlock, SpinlockGuard},
    x86_64::{
        instructions::interrupts::without_interrupts, registers::control::Cr3,
        structures::paging::PhysFrame, PhysAddr, VirtAddr,
    },
};

static SCHEDULER: Lazy<Spinlock<Scheduler>> = Lazy::new(|| Spinlock::new(Scheduler::new()));

pub(crate) fn send(msg: VirtAddr, to: Pid) {
    // The kernel process calls this function, and the interrupts may be enabled at that time. If
    // we forget to disable interrupts, a timer interrupt may happen when the kernel process holds
    // the lock of the process scheduler, and the subsequent process fails to lock the scheduler
    // because the previous process already locks it. Thus, we disable the interrupts.
    without_interrupts(|| lock().send(msg, to));
}

pub(crate) fn receive_from_any(msg_buf: VirtAddr) {
    // Ditto as `send` for `without_interrupts`.
    without_interrupts(|| lock().receive_from_any(msg_buf));
}

pub(crate) fn receive_from(msg_buf: VirtAddr, from: Pid) {
    // Ditto as `send` for `without_interrupts`.
    without_interrupts(|| lock().receive_from(msg_buf, from));
}

pub(crate) fn switch() {
    lock().switch();
}

pub(crate) fn current_process_name() -> &'static str {
    lock().current_process_name()
}

pub(super) fn add_process_as_runnable(p: Process) {
    lock().add_process_as_runnable(p);
}

pub(super) fn init() {
    lock().init();
}

struct Scheduler {
    processes: BTreeMap<Pid, Process>,

    runnable_pids: RunnablePids,

    running: Pid,
}
impl Scheduler {
    fn new() -> Self {
        Self {
            processes: BTreeMap::new(),

            runnable_pids: RunnablePids::new(),

            running: 0,
        }
    }

    fn init(&mut self) {
        self.add_idle_process_as_running();
    }

    fn add_idle_process_as_running(&mut self) {
        let idle = Process::idle();

        assert_eq!(idle.pid, 0, "Wrong PID for the idle process.");
        assert_eq!(
            idle.status,
            Status::Running,
            "The idle process should be running."
        );

        let r = self.processes.insert(idle.pid, idle);

        r.expect("Duplicated idle process.");
    }

    fn add_process_as_runnable(&mut self, p: Process) {
        let pid = p.id();

        let r = self.processes.insert(pid, p);

        assert!(r.is_none(), "Duplicated process with PID {}.", pid);
    }

    fn wake(&mut self, pid: Pid) {
        let p = self.process_as_mut(pid);
        let p = p.expect("No such process.");

        assert!(
            !matches!(p.status, Status::Running | Status::Runnable),
            "The process is already awake."
        );

        p.status = Status::Runnable;

        let priority = p.priority;

        self.runnable_pids.push(pid, priority);
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

    fn switch(&mut self) {
        Switcher(self).switch();
    }

    fn current_process_name(&self) -> &'static str {
        self.running_as_ref().name
    }

    fn current_pml4(&self) -> PhysFrame {
        let p = self.running_as_ref();

        let frame = PhysFrame::from_start_address(p.pml4.phys_addr());
        frame.expect("PML4 is not page-aligned.")
    }

    fn running_as_ref(&self) -> &Process {
        self.process_as_ref(self.running)
            .expect("Running process is not stored.")
    }

    fn running_as_mut(&mut self) -> &mut Process {
        self.process_as_mut(self.running)
            .expect("Running process is not stored.")
    }

    fn process_as_ref(&self, pid: Pid) -> Option<&Process> {
        self.processes.get(&pid)
    }

    fn process_as_mut(&mut self, pid: Pid) -> Option<&mut Process> {
        self.processes.get_mut(&pid)
    }
}

struct Sender<'a> {
    manager: &'a mut Scheduler,
    msg: PhysAddr,
    to: Pid,
}
impl<'a> Sender<'a> {
    fn new(manager: &'a mut Scheduler, msg: VirtAddr, to: Pid) -> Self {
        assert_ne!(manager.running, to, "Tried to send a message to self.");

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
        let p = self.manager.process_as_ref(self.to);
        let p = p.expect("The receiver does not exist.");

        [
            Some(ReceiveFrom::Id(self.manager.running)),
            Some(ReceiveFrom::Any),
        ]
        .contains(&p.receive_from)
    }

    fn copy_msg_and_wake(&mut self) {
        self.copy_msg();
        self.remove_msg_buf();
        self.wake_dst();
    }

    fn copy_msg(&self) {
        let dst_proc = self.manager.process_as_ref(self.to);
        let dst_proc = dst_proc.expect("The receiver does not exist.");

        let dst = dst_proc.msg_ptr;
        let dst = dst.expect("Message destination address is not specified.");

        unsafe { copy_msg(self.msg, dst, self.manager.running) }
    }

    fn remove_msg_buf(&mut self) {
        let p = self.manager.running_as_mut();

        p.msg_ptr = None;
        p.send_to = None;
    }

    fn wake_dst(&mut self) {
        let dst = self.manager.process_as_mut(self.to);
        let dst = dst.expect("The receiver does not exist.");

        dst.msg_ptr = None;
        dst.receive_from = None;

        self.manager.wake(self.to);
    }

    fn set_msg_buf_and_sleep(&mut self) {
        self.set_msg_buf();
        self.add_self_as_trying_to_send();
        self.mark_as_sending();
        self.sleep();
    }

    fn set_msg_buf(&mut self) {
        let p = self.manager.running_as_mut();

        if p.msg_ptr.is_none() {
            p.msg_ptr = Some(self.msg);
        } else {
            panic!("Message is already stored.");
        };
    }

    fn add_self_as_trying_to_send(&mut self) {
        let pid = self.manager.running;

        let dst = self.manager.process_as_mut(self.to);
        let dst = dst.expect("The receiver does not exist.");

        dst.pids_try_to_send_this_process.push_back(pid);
    }

    fn mark_as_sending(&mut self) {
        let p = self.manager.running_as_mut();

        p.send_to = Some(self.to);
    }

    fn sleep(&mut self) {
        let sender = self.manager.running_as_mut();

        sender.status = Status::Sending {
            to: self.to,
            message: self.msg,
        };

        let running = self.manager.running;

        let receiver = self.manager.process_as_mut(self.to);
        let receiver = receiver.expect("No such process.");

        receiver.pids_try_to_send_this_process.push_back(running);
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
            manager.running, from,
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
            let p = self.manager.process_as_ref(id);
            let p = p.expect("The sender does not exist.");

            p.send_to == Some(id)
        } else {
            let p = self.manager.running_as_ref();

            !p.pids_try_to_send_this_process.is_empty()
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
            let p = self.manager.running_as_mut();

            p.pids_try_to_send_this_process
                .pop_front()
                .expect("No process is waiting to send.")
        }
    }

    fn copy_msg(&self, src_slot_id: Pid) {
        let src_proc = self.manager.process_as_ref(src_slot_id);
        let src_proc = src_proc.expect("The sender does not exist.");

        let src = src_proc.msg_ptr;
        let src = src.expect("The message pointer of the sender is not set.");

        unsafe { copy_msg(src, self.msg_buf, src_slot_id) }
    }

    fn wake_sender(&mut self, src_pid: Pid) {
        let sender = self.manager.process_as_mut(src_pid);
        let sender = sender.expect("The sender does not exist.");

        sender.msg_ptr = None;
        sender.send_to = None;

        self.manager.wake(src_pid);
    }

    fn set_msg_buf_and_sleep(&mut self) {
        self.set_msg_buf();
        self.mark_as_receiving();
        self.sleep();
    }

    fn set_msg_buf(&mut self) {
        let p = self.manager.running_as_mut();

        if p.msg_ptr.is_none() {
            p.msg_ptr = Some(self.msg_buf);
        } else {
            panic!("Message is already stored.");
        };
    }

    fn mark_as_receiving(&mut self) {
        let p = self.manager.running_as_mut();

        p.receive_from = Some(self.from);
    }

    fn sleep(&mut self) {
        let receiver = self.manager.running_as_mut();

        assert!(
            receiver.msg_ptr.is_none(),
            "The message buffer is not empty."
        );

        receiver.status = Status::Receiving(self.from);
        receiver.msg_ptr = Some(self.msg_buf);
    }
}

struct Switcher<'a>(&'a mut Scheduler);
impl Switcher<'_> {
    fn switch(self) {
        if cfg!(feature = "qemu_test") {
            tests::process::count_switch();
        }

        self.switch_pml4();
    }

    fn switch_pml4(&self) {
        let (_, f) = Cr3::read();
        let pml4 = self.0.current_pml4();

        // SAFETY: The PML4 frame is correct one and flags are unchanged.
        unsafe { Cr3::write(pml4, f) }
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

fn lock() -> SpinlockGuard<'static, Scheduler> {
    SCHEDULER
        .try_lock()
        .expect("Failed to acquire the lock of `PROCESSES`.")
}

struct RunnablePids([VecDeque<Pid>; LEAST_PRIORITY.as_usize() + 1]);
impl RunnablePids {
    fn new() -> Self {
        Self(array_init(|_| VecDeque::new()))
    }

    fn push(&mut self, pid: Pid, priority: Priority) {
        self.0[priority.as_usize()].push_back(pid);
    }

    fn pop(&mut self) -> Option<Pid> {
        self.0.iter_mut().find_map(VecDeque::pop_front)
    }
}
