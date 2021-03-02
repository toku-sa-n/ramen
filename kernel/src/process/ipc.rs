// SPDX-License-Identifier: GPL-3.0-or-later

use super::collections;
use crate::{mem, mem::accessor::Single};
use mem::paging::pml4::PML4;
use message::Message;
use x86_64::{structures::paging::Translate, PhysAddr, VirtAddr};

pub(crate) fn send(msg: VirtAddr, to: i32) {
    Sender::new(msg, to).send()
}

pub(crate) fn receive(msg_buf: VirtAddr) {
    Receiver::new(msg_buf).receive()
}

struct Sender {
    msg: PhysAddr,
    to: i32,
}
impl Sender {
    fn new(msg: VirtAddr, to: i32) -> Self {
        let msg = PML4
            .lock()
            .translate_addr(msg)
            .expect("Failed to get the physical address of a message.");

        Self { msg, to }
    }

    fn send(self) {
        if Self::is_receiver_waiting() {
            self.copy_msg_and_wake();
        } else {
            self.set_msg_buf_and_sleep();
        }
    }

    fn is_receiver_waiting() -> bool {
        collections::process::handle_running(|p| p.waiting_message() && p.msg_ptr.is_some())
    }

    fn copy_msg_and_wake(&self) {
        self.copy_msg();
        Self::remove_msg_buf();
        self.wake_dst();
    }

    fn copy_msg(&self) {
        let dst = collections::process::handle_running(|p| p.msg_ptr);
        let dst = dst.expect("Message destination address is not specified.");

        unsafe { copy_msg(self.msg, dst) }
    }

    fn remove_msg_buf() {
        collections::process::handle_running_mut(|p| p.msg_ptr = None)
    }

    fn wake_dst(&self) {
        collections::process::handle_mut(self.to.into(), |p| p.flags -= super::Flags::RECEIVING);
        collections::woken_pid::push(self.to.into());
    }

    fn set_msg_buf_and_sleep(&self) {
        self.set_msg_buf();
        Self::mark_as_sending();
        sleep();
    }

    fn set_msg_buf(&self) {
        collections::process::handle_running_mut(|p| {
            if p.msg_ptr.is_none() {
                p.msg_ptr = Some(self.msg);
            } else {
                panic!("Message is already stored.");
            }
        })
    }

    fn mark_as_sending() {
        collections::process::handle_running_mut(|p| p.flags |= super::Flags::SENDING)
    }
}

struct Receiver {
    msg_buf: PhysAddr,
}
impl Receiver {
    fn new(msg_buf: VirtAddr) -> Self {
        let msg_buf = virt_to_phys(msg_buf);
        Self { msg_buf }
    }

    fn receive(self) {
        if self.is_sender_waiting() {
            self.copy_msg_and_wake();
        } else {
            self.set_msg_buf_and_sleep();
        }
    }

    fn is_sender_waiting(&self) -> bool {
        collections::process::handle_running(|p| !p.pids_try_to_send_this_process.is_empty())
    }

    fn copy_msg_and_wake(&self) {
        let src_pid = self.src_pid();

        self.copy_msg(src_pid);
        Self::wake_sender(src_pid);
    }

    fn src_pid(&self) -> super::Id {
        collections::process::handle_running_mut(|p| {
            p.pids_try_to_send_this_process
                .pop_front()
                .expect("No process is waiting to send.")
        })
        .into()
    }

    fn copy_msg(&self, src_pid: super::Id) {
        let src = collections::process::handle(src_pid, |p| p.msg_ptr).expect("Process not found.");

        unsafe { copy_msg(src, self.msg_buf) }
    }

    fn wake_sender(src_pid: super::Id) {
        collections::process::handle_mut(src_pid, |p| p.flags -= super::Flags::SENDING);
        collections::woken_pid::push(src_pid);
    }

    fn set_msg_buf_and_sleep(&self) {
        self.set_msg_buf();
        Self::mark_as_receiving();
        sleep();
    }

    fn set_msg_buf(&self) {
        collections::process::handle_running_mut(|p| {
            if p.msg_ptr.is_none() {
                p.msg_ptr = Some(self.msg_buf);
            } else {
                panic!("Message is already stored.");
            }
        })
    }

    fn mark_as_receiving() {
        collections::process::handle_running_mut(|p| p.flags |= super::Flags::RECEIVING)
    }
}

/// # Safety
///
/// `src` and `dst` must be the correct addresses where a message is located and copied.
unsafe fn copy_msg(src: PhysAddr, dst: PhysAddr) {
    let src: Single<Message> = mem::accessor::kernel(src);
    let mut dst = mem::accessor::kernel(dst);

    dst.write(src.read());
}

fn virt_to_phys(v: VirtAddr) -> PhysAddr {
    PML4.lock()
        .translate_addr(v)
        .expect("Failed to convert a virtual address to physical one.")
}

fn sleep() {
    super::block_running();
}
