// SPDX-License-Identifier: GPL-3.0-or-later

use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use alloc::rc::Rc;
use futures_util::task::AtomicWaker;
use task::Task;

use crate::{
    device::pci::xhci::exchanger::transfer, mem::allocator::page_box::PageBox, multitask::task,
};

static WAKER: AtomicWaker = AtomicWaker::new();

async fn task(mut kbd: Keyboard) {
    loop {
        kbd.get_packet().await;
        kbd.print_buf();
        kbd.reset_buf();
    }
}

async fn waker_task() {
    WAKER.wake();
}

struct Keyboard {
    sender: transfer::Sender,
    task_collection: Rc<RefCell<task::Collection>>,
    buf: PageBox<[u8; 8]>,
}
impl Keyboard {
    fn new(sender: transfer::Sender, task_collection: Rc<RefCell<task::Collection>>) -> Self {
        Self {
            sender,
            task_collection,
            buf: PageBox::new([0; 8]),
        }
    }

    async fn get_packet(&mut self) {
        self.issue_normal_trb().await;
    }

    async fn issue_normal_trb(&mut self) {
        self.sender.issue_normal_trb(&self.buf).await;
    }

    async fn wait_until_packet_is_sent(&self) {
        PacketWaiterFuture::new(self).await
    }

    fn print_buf(&self) {
        info!("Keyboard packet: {:?}", self.buf);
    }

    fn reset_buf(&mut self) {
        *self.buf = [0; 8];
    }
}

struct PacketWaiterFuture<'a> {
    kbd: &'a Keyboard,
}
impl<'a> PacketWaiterFuture<'a> {
    fn new(kbd: &'a Keyboard) -> Self {
        Self { kbd }
    }
}
impl<'a> Future for PacketWaiterFuture<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        WAKER.register(cx.waker());
        let tasks = self.kbd.task_collection.clone();
        if *self.kbd.buf == [0; 8] {
            tasks
                .borrow_mut()
                .add_task_as_woken(Task::new(waker_task()));
            Poll::Pending
        } else {
            WAKER.take();
            Poll::Ready(())
        }
    }
}
