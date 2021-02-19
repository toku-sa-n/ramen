// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

use common::constant::{KEY_STATUS_SEND_NOT_READY, PORT_KEY_CMD, PORT_KEY_DATA, PORT_KEY_STATUS};
use vek::Vec2;

const KEY_CMD_SEND_TO_MOUSE: u8 = 0xD4;
const MOUSE_CMD_ENABLE: u8 = 0xF4;

pub fn main() {
    Device::enable();
    syscalls::notify_on_interrup(0x2c, syscalls::getpid());

    let mut device = Device::new();

    loop {
        if syscalls::notify_exists() {
            let packet = unsafe { syscalls::inb(PORT_KEY_DATA) };
            handle_packet(&mut device, packet);
        }
    }
}

fn handle_packet(device: &mut Device, packet: u8) {
    device.put_data(packet);
    if device.three_packets_available() {
        parse_packets(device);
    }
}

fn parse_packets(device: &mut Device) {
    device.parse_packets();
    device.print_click_info();
    device.print_speed();
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
        wait_kbc_sendready();

        let port_key_cmd = PORT_KEY_CMD;
        unsafe { syscalls::outb(port_key_cmd, KEY_CMD_SEND_TO_MOUSE) };

        wait_kbc_sendready();

        let port_key_data = PORT_KEY_DATA;
        unsafe { syscalls::outb(port_key_data, MOUSE_CMD_ENABLE) };
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
            panic!("Left button pressed");
        }

        if self.buttons.center {
            panic!("Scroll wheel pressed");
        }

        if self.buttons.right {
            panic!("Right button pressed");
        }
    }

    fn print_speed(&self) {
        panic!("Speed: {}", self.speed());
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

fn wait_kbc_sendready() {
    loop {
        let port_key_status = PORT_KEY_STATUS;
        if unsafe { syscalls::inb(port_key_status) } & KEY_STATUS_SEND_NOT_READY == 0 {
            break;
        }
    }
}
