// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::graphics::screen::cursor::Cursor,
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
    let mut cursor = Cursor::new();

    while let Some(packet) = packet_stream.next().await {
        handle_packet(&mut device, &mut cursor, packet);
    }
}

fn handle_packet(device: &mut Device, cursor: &mut Cursor, packet: u8) {
    device.put_data(packet);
    if device.three_packets_available() {
        parse_packets(device, cursor);
    }
}

fn parse_packets(device: &mut Device, cursor: &mut Cursor) {
    device.parse_packets();
    device.print_click_info();
    cursor.move_offset(device.speed());
}

pub fn enqueue_packet(packet: u8) {
    if queue().push(packet).is_ok() {
        WAKER.wake();
    } else {
        warn!("MOUSE_PACKET_QUEUE is full.")
    }
}

fn queue() -> &'static ArrayQueue<u8> {
    MOUSE_PACKET_QUEUE
        .try_get()
        .expect("MOUSE_PACKET_QUEUE is not initialized.")
}

struct Device {
    buf: Buf,
    speed: Vec2<i32>,
    buttons: MouseButtons,
}
impl Device {
    const fn new() -> Self {
        Self {
            buf: Buf::new(),
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

    fn three_packets_available(&self) -> bool {
        self.buf.full()
    }

    fn put_data(&mut self, packet: u8) {
        self.buf.put(packet)
    }

    fn speed(&self) -> Vec2<i32> {
        self.speed
    }

    fn parse_packets(&mut self) {
        self.buttons = self.buf.buttons_info();
        self.speed = self.buf.speed();
        self.clear_stack();
    }

    fn clear_stack(&mut self) {
        self.buf.clear()
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
}

struct Buf {
    packets: [u8; 3],
    phase: DevicePhase,
}
impl Buf {
    const fn new() -> Self {
        Self {
            packets: [0; 3],
            phase: DevicePhase::Init,
        }
    }

    fn full(&self) -> bool {
        self.phase == DevicePhase::ThreeData
    }

    fn put(&mut self, packet: u8) {
        if self.phase == DevicePhase::Init {
            self.check_ack_packet(packet)
        } else if self.phase != DevicePhase::ThreeData {
            self.push_packet(packet)
        }
    }

    fn push_packet(&mut self, packet: u8) {
        self.packets[self.phase as usize - 1] = packet;
        self.phase = self.phase.next().unwrap();
    }

    fn check_ack_packet(&mut self, packet: u8) {
        if Self::is_ack_packet(packet) {
            self.phase = DevicePhase::NoData
        }
    }

    fn is_ack_packet(packet: u8) -> bool {
        packet == 0xfa
    }

    fn clear(&mut self) {
        self.phase = DevicePhase::NoData
    }

    fn speed(&self) -> Vec2<i32> {
        Vec2::new(self.speed_x(), self.speed_y())
    }

    fn speed_x(&self) -> i32 {
        let mut speed = self.packets[1].into();
        if self.speed_x_is_negative() {
            speed -= 256;
        }
        speed
    }

    fn speed_x_is_negative(&self) -> bool {
        self.packets[0] & 0x10 != 0
    }

    fn speed_y(&self) -> i32 {
        let mut speed: i32 = self.packets[2].into();
        if self.speed_y_is_negative() {
            speed -= 256;
        }
        -speed
    }

    fn speed_y_is_negative(&self) -> bool {
        self.packets[0] & 0x20 != 0
    }

    fn buttons_info(&self) -> MouseButtons {
        MouseButtons::purse_data(self.packets[0])
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
        WAKER.register(&cx.waker());
        match queue().pop() {
            Some(packet) => {
                WAKER.take();
                Poll::Ready(Some(packet))
            }
            None => Poll::Pending,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum DevicePhase {
    Init,
    NoData,
    OneData,
    TwoData,
    ThreeData,
}
impl DevicePhase {
    fn next(self) -> Option<Self> {
        match self {
            Self::Init => Some(Self::NoData),
            Self::NoData => Some(Self::OneData),
            Self::OneData => Some(Self::TwoData),
            Self::TwoData => Some(Self::ThreeData),
            Self::ThreeData => None,
        }
    }
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
