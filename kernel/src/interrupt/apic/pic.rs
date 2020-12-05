// SPDX-License-Identifier: GPL-3.0-or-later

use x86_64::instructions::port::PortWriteOnly;

const MASTER_CMD: PortWriteOnly<u8> = PortWriteOnly::new(0x20);
const MASTER_DATA: PortWriteOnly<u8> = PortWriteOnly::new(0x21);

const SLAVE_CMD: PortWriteOnly<u8> = PortWriteOnly::new(0xa0);
const SLAVE_DATA: PortWriteOnly<u8> = PortWriteOnly::new(0xa0);

const MASTER_ICW1: PortWriteOnly<u8> = MASTER_CMD;
const SLAVE_ICW1: PortWriteOnly<u8> = SLAVE_CMD;

const MASTER_ICW2: PortWriteOnly<u8> = MASTER_DATA;
const SLAVE_ICW2: PortWriteOnly<u8> = SLAVE_DATA;

const MASTER_ICW3: PortWriteOnly<u8> = MASTER_DATA;
const SLAVE_ICW3: PortWriteOnly<u8> = SLAVE_DATA;

const MASTER_ICW4: PortWriteOnly<u8> = MASTER_DATA;
const SLAVE_ICW4: PortWriteOnly<u8> = SLAVE_DATA;

pub fn disable() {
    pic_init_mode();
    remap_pic();
    set_slave_offset();
    nonbuffer_mode();
    mask_pic();
}

fn pic_init_mode() {
    let mut m = MASTER_ICW1;
    let mut s = SLAVE_ICW1;

    unsafe {
        m.write(0x11_u8);
        s.write(0x11_u8);
    }
}

fn remap_pic() {
    let mut m = MASTER_ICW2;
    let mut s = SLAVE_ICW2;

    unsafe {
        m.write(0x20_u8);
        s.write(0x28_u8);
    }
}

fn set_slave_offset() {
    let mut m = MASTER_ICW3;
    let mut s = SLAVE_ICW3;

    unsafe {
        m.write(4_u8);
        s.write(2_u8);
    }
}

fn nonbuffer_mode() {
    let mut m = MASTER_ICW4;
    let mut s = SLAVE_ICW4;

    unsafe {
        m.write(1_u8);
        s.write(1_u8);
    }
}

fn mask_pic() {
    let mut m = MASTER_DATA;
    let mut s = SLAVE_DATA;

    unsafe {
        m.write(0xFF_u8);
        s.write(0xFF_u8);
    }
}
