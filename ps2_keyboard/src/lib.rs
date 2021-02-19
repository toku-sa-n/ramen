// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

use common::constant::{
    KEY_CMD_MODE, KEY_CMD_WRITE_MODE, KEY_STATUS_SEND_NOT_READY, PORT_KEY_CMD, PORT_KEY_DATA,
    PORT_KEY_STATUS,
};

pub fn main() {
    enable_keyboard();
    syscalls::notify_on_interrup(0x21, syscalls::getpid());

    loop {
        if syscalls::notify_exists() {
            let code = unsafe { syscalls::inb(PORT_KEY_DATA) };
            panic!("code: {}", code);
        }
    }
}

fn enable_keyboard() {
    wait_kbc_sendready();

    let port_key_cmd = PORT_KEY_CMD;
    unsafe { syscalls::outb(port_key_cmd, KEY_CMD_WRITE_MODE as u8) };

    wait_kbc_sendready();

    let port_key_data = PORT_KEY_DATA;
    unsafe { syscalls::outb(port_key_data, KEY_CMD_MODE as u8) };
}

fn wait_kbc_sendready() {
    loop {
        let port_key_status = PORT_KEY_STATUS;
        if unsafe { syscalls::inb(port_key_status) } & KEY_STATUS_SEND_NOT_READY == 0 {
            break;
        }
    }
}
