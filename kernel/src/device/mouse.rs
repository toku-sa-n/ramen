// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::graphics::screen::cursor::MouseCursor,
    common::constant::{PORT_KEY_CMD, PORT_KEY_DATA},
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
    vek::Vec2,
};

static MOUSE_PACKET_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

const KEY_CMD_SEND_TO_MOUSE: u8 = 0xD4;
const MOUSE_CMD_ENABLE: u8 = 0xF4;

pub async fn task() {
    PacketStream::init_queue();
    Device::enable();
    let mut packet_stream = PacketStream;

    let mut device = Device::new();
    let mut cursor = MouseCursor::new();

    while let Some(packet) = packet_stream.next().await {
        device.put_data(packet);
        if device.data_available() {
            device.purse_data();
            device.print_click_info();
            cursor.move_offset(device.get_speed());
        }
    }
}

pub fn enqueue_packet(packet: u8) {
    match MOUSE_PACKET_QUEUE.try_get() {
        Ok(queue) => {
            if queue.push(packet).is_ok() {
                WAKER.wake();
            } else {
                warn!("MOUSE_PACKET_QUEUE is full.")
            }
        }
        Err(_) => panic!("MOUSE_PACKET_QUEUE is not initialized."),
    }
}

struct Device {
    data_from_device: [u8; 3],
    phase: DevicePhase,

    speed: Vec2<i32>,

    buttons: MouseButtons,
}

impl Device {
    const fn new() -> Self {
        Self {
            data_from_device: [0; 3],
            phase: DevicePhase::Init,
            speed: Vec2::new(0, 0),
            buttons: MouseButtons::new(),
        }
    }

    fn enable() {
        super::keyboard::wait_kbc_sendready();

        let mut port_key_cmd = PORT_KEY_CMD;
        unsafe { port_key_cmd.write(KEY_CMD_SEND_TO_MOUSE) };

        super::keyboard::wait_kbc_sendready();

        let mut port_key_data = PORT_KEY_DATA;
        unsafe { port_key_data.write(MOUSE_CMD_ENABLE) };
    }

    fn data_available(&self) -> bool {
        self.phase == DevicePhase::ThreeData
    }

    fn put_data(&mut self, data: u8) {
        match self.phase {
            DevicePhase::Init => {
                let is_correct_startup = data == 0xfa;
                if is_correct_startup {
                    self.phase = DevicePhase::NoData
                }
            }

            DevicePhase::NoData => {
                if Self::is_correct_first_byte_from_device(data) {
                    self.data_from_device[0] = data;
                    self.phase = DevicePhase::OneData;
                }
            }
            DevicePhase::OneData => {
                self.data_from_device[1] = data;
                self.phase = DevicePhase::TwoData;
            }
            DevicePhase::TwoData => {
                self.data_from_device[2] = data;
                self.phase = DevicePhase::ThreeData;
            }
            DevicePhase::ThreeData => {}
        }
    }

    // To sync phase, and data sent from mouse device
    fn is_correct_first_byte_from_device(data: u8) -> bool {
        data & 0xC8 == 0x08
    }

    fn clear_stack(&mut self) {
        self.phase = DevicePhase::NoData;
    }

    fn purse_data(&mut self) {
        self.buttons = MouseButtons::purse_data(self.data_from_device[0]);
        self.speed.x = i32::from(self.data_from_device[1]);
        self.speed.y = i32::from(self.data_from_device[2]);

        if self.data_from_device[0] & 0x10 != 0 {
            self.speed.x -= 256;
        }

        if self.data_from_device[0] & 0x20 != 0 {
            self.speed.y -= 256;
        }

        self.speed.y = -self.speed.y;

        self.clear_stack();
    }

    fn print_click_info(&self) {
        if self.buttons.left {
            info!("Left button pressed");
        }

        if self.buttons.center {
            info!("Scroll wheel pressed");
        }

        if self.buttons.right {
            info!("Right button pressed");
        }
    }

    fn get_speed(&self) -> Vec2<i32> {
        Vec2::new(self.speed.x, self.speed.y)
    }
}

struct PacketStream;

impl PacketStream {
    fn init_queue() {
        MOUSE_PACKET_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("MOUSE_PACKET_QUEUE is already initialized.")
    }
}

impl Stream for PacketStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let queue = MOUSE_PACKET_QUEUE
            .try_get()
            .expect("MOUSE_PACKET_QUEUE is not initialized");
        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(packet) => {
                WAKER.take();
                Poll::Ready(Some(packet))
            }
            Err(_) => Poll::Pending,
        }
    }
}

#[derive(PartialEq, Eq)]
enum DevicePhase {
    Init,
    NoData,
    OneData,
    TwoData,
    ThreeData,
}

struct MouseButtons {
    left: bool,
    center: bool,
    right: bool,
}

impl MouseButtons {
    const fn new() -> Self {
        Self {
            left: false,
            right: false,
            center: false,
        }
    }

    fn purse_data(data: u8) -> Self {
        Self {
            left: data & 0x01 != 0,
            right: data & 0x02 != 0,
            center: data & 0x04 != 0,
        }
    }
}
