// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::graphics::screen::Screen,
    common::constant::{
        KEY_CMD_MODE, KEY_CMD_WRITE_MODE, KEY_STATUS_SEND_NOT_READY, PORT_KEY_CMD, PORT_KEY_DATA,
        PORT_KEY_STATUS,
    },
    conquer_once::spin::OnceCell,
    core::{
        pin::Pin,
        task::{Context, Poll},
    },
    crossbeam_queue::ArrayQueue,
    futures_util::{
        stream::{Stream, StreamExt},
        task::AtomicWaker,
    },
    rgb::RGB8,
    vek::Vec2,
};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub fn enqueue_scancode(code: u8) {
    match SCANCODE_QUEUE.try_get() {
        Ok(queue) => {
            if queue.push(code).is_ok() {
                WAKER.wake();
            } else {
                warn!("SCANCODE_QUEUE is full.");
            }
        }
        Err(_) => panic!("SCANCODE_QUEUE is not initialized."),
    }
}

struct ScancodeStream;

impl ScancodeStream {
    fn init_queue() {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("SCANCODE_QUEUE is already initialized.")
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("SCANCODE_QUEUE is not initialized");

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(code) => {
                WAKER.take();
                Poll::Ready(Some(code))
            }
            Err(_) => Poll::Pending,
        }
    }
}

pub async fn task() {
    ScancodeStream::init_queue();

    enable_keyboard();

    let mut scancode_stream = ScancodeStream;

    while let Some(code) = scancode_stream.next().await {
        Screen::draw_rectangle(
            RGB8::new(0, 0x84, 0x84),
            Vec2::new(0, 16),
            Vec2::new(15, 31),
        );
        info!("{:} pressed.", code as char);
    }
}

fn enable_keyboard() {
    wait_kbc_sendready();

    let mut port_key_cmd = PORT_KEY_CMD;
    unsafe { port_key_cmd.write(KEY_CMD_WRITE_MODE as u8) };

    wait_kbc_sendready();

    let mut port_key_data = PORT_KEY_DATA;
    unsafe { port_key_data.write(KEY_CMD_MODE as u8) };
}

pub(super) fn wait_kbc_sendready() {
    loop {
        let mut port_key_status = PORT_KEY_STATUS;
        if unsafe { port_key_status.read() } & KEY_STATUS_SEND_NOT_READY == 0 {
            break;
        }
    }
}
