// SPDX-License-Identifier: GPL-3.0-or-later

use super::{collections, get_slot_id, ReceiveFrom, SlotId};
use crate::{mem, mem::accessor::Single};
use mem::paging::pml4::PML4;
use message::Message;
use x86_64::{structures::paging::Translate, PhysAddr, VirtAddr};

pub(crate) fn send(msg: VirtAddr, to: SlotId) {
    Sender::new(msg, to).send();
}

pub(crate) fn receive_from_any(msg_buf: VirtAddr) {
    Receiver::new_from_any(msg_buf).receive();
}

pub(crate) fn receive_from(msg_buf: VirtAddr, from: SlotId) {
    Receiver::new_from(msg_buf, from).receive();
}

struct Sender {
    msg: PhysAddr,
    to: SlotId,
}
impl Sender {
    fn new(msg: VirtAddr, to: SlotId) -> Self {
        assert_ne!(get_slot_id(), to, "Tried to send a message to self.");

        let msg = virt_to_phys(msg);

        Self { msg, to }
    }

    fn send(self) {
        if self.is_receiver_waiting() {
            self.copy_msg_and_wake();
        } else {
            self.set_msg_buf_and_sleep();
        }
    }

    fn is_receiver_waiting(&self) -> bool {
        collections::process::handle(self.to, |p| {
            [Some(ReceiveFrom::Id(get_slot_id())), Some(ReceiveFrom::Any)].contains(&p.receive_from)
        })
    }

    fn copy_msg_and_wake(&self) {
        self.copy_msg();
        Self::remove_msg_buf();
        self.wake_dst();
    }

    fn copy_msg(&self) {
        let dst = collections::process::handle(self.to, |p| p.msg_ptr);
        let dst = dst.expect("Message destination address is not specified.");

        unsafe { copy_msg(self.msg, dst, get_slot_id()) }
    }

    fn remove_msg_buf() {
        collections::process::handle_running_mut(|p| {
            p.msg_ptr = None;
            p.send_to = None;
        });
    }

    fn wake_dst(&self) {
        collections::process::handle_mut(self.to, |p| {
            p.msg_ptr = None;
            p.receive_from = None;
        });
        collections::woken_pid::push(self.to);
    }

    fn set_msg_buf_and_sleep(&self) {
        self.set_msg_buf();
        self.add_self_as_trying_to_send();
        self.mark_as_sending();
        sleep();
    }

    fn set_msg_buf(&self) {
        collections::process::handle_running_mut(|p| {
            if p.msg_ptr.is_none() {
                p.msg_ptr = Some(self.msg);
            } else {
                panic!("Message is already stored.");
            }
        });
    }

    fn add_self_as_trying_to_send(&self) {
        let pid = get_slot_id();
        collections::process::handle_mut(self.to, |p| {
            p.pids_try_to_send_this_process.push_back(pid);
        });
    }

    fn mark_as_sending(&self) {
        collections::process::handle_running_mut(|p| p.send_to = Some(self.to));
    }
}

struct Receiver {
    msg_buf: PhysAddr,
    from: ReceiveFrom,
}
impl Receiver {
    fn new_from_any(msg_buf: VirtAddr) -> Self {
        let msg_buf = virt_to_phys(msg_buf);

        Self {
            msg_buf,
            from: ReceiveFrom::Any,
        }
    }

    fn new_from(msg_buf: VirtAddr, from: SlotId) -> Self {
        assert_ne!(get_slot_id(), from, "Tried to receive a message from self.");

        let msg_buf = virt_to_phys(msg_buf);

        Self {
            msg_buf,
            from: ReceiveFrom::Id(from),
        }
    }

    fn receive(self) {
        if self.is_sender_waiting() {
            self.copy_msg_and_wake();
        } else {
            self.set_msg_buf_and_sleep();
        }
    }

    fn is_sender_waiting(&self) -> bool {
        use collections::process::{handle, handle_running};

        if let ReceiveFrom::Id(id) = self.from {
            handle(id, |p| p.send_to == Some(id))
        } else {
            handle_running(|p| !p.pids_try_to_send_this_process.is_empty())
        }
    }

    fn copy_msg_and_wake(&self) {
        let src_pid = self.src_pid();

        self.copy_msg(src_pid);
        Self::wake_sender(src_pid);
    }

    fn src_pid(&self) -> SlotId {
        if let ReceiveFrom::Id(id) = self.from {
            id
        } else {
            collections::process::handle_running_mut(|p| {
                p.pids_try_to_send_this_process
                    .pop_front()
                    .expect("No process is waiting to send.")
            })
        }
    }

    fn copy_msg(&self, src_slot_id: SlotId) {
        let src = collections::process::handle(src_slot_id, |p| p.msg_ptr);
        let src = src.expect("The message pointer of the sender is not set.");

        unsafe { copy_msg(src, self.msg_buf, src_slot_id) }
    }

    fn wake_sender(src_pid: SlotId) {
        collections::process::handle_mut(src_pid, |p| {
            p.msg_ptr = None;
            p.send_to = None;
        });
        collections::woken_pid::push(src_pid);
    }

    fn set_msg_buf_and_sleep(&self) {
        self.set_msg_buf();
        self.mark_as_receiving();
        sleep();
    }

    fn set_msg_buf(&self) {
        collections::process::handle_running_mut(|p| {
            if p.msg_ptr.is_none() {
                p.msg_ptr = Some(self.msg_buf);
            } else {
                panic!("Message is already stored.");
            }
        });
    }

    fn mark_as_receiving(&self) {
        collections::process::handle_running_mut(|p| p.receive_from = Some(self.from));
    }
}

/// # Safety
///
/// `src` and `dst` must be the correct addresses where a message is located and copied.
unsafe fn copy_msg(src: PhysAddr, dst: PhysAddr, sender_slot_id: SlotId) {
    let mut src: Single<Message> = mem::accessor::new(src);
    let mut dst = mem::accessor::new(dst);

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
