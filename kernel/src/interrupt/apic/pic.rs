// SPDX-License-Identifier: GPL-3.0-or-later

use crate::syscall;

const MASTER_CMD: u16 = 0x20;
const MASTER_DATA: u16 = 0x21;

const SLAVE_CMD: u16 = 0xa0;
const SLAVE_DATA: u16 = 0xa0;

const MASTER_ICW1: u16 = MASTER_CMD;
const SLAVE_ICW1: u16 = SLAVE_CMD;

const MASTER_ICW2: u16 = MASTER_DATA;
const SLAVE_ICW2: u16 = SLAVE_DATA;

const MASTER_ICW3: u16 = MASTER_DATA;
const SLAVE_ICW3: u16 = SLAVE_DATA;

const MASTER_ICW4: u16 = MASTER_DATA;
const SLAVE_ICW4: u16 = SLAVE_DATA;

pub fn disable() {
    pic_init_mode();
    remap_pic();
    set_slave_offset();
    nonbuffer_mode();
    mask_pic();
}

fn pic_init_mode() {
    unsafe {
        syscall::outb(MASTER_ICW1, 0x11);
        syscall::outb(SLAVE_ICW1, 0x11);
    }
}

fn remap_pic() {
    unsafe {
        syscall::outb(MASTER_ICW2, 0x20);
        syscall::outb(SLAVE_ICW2, 0x28);
    }
}

fn set_slave_offset() {
    unsafe {
        syscall::outb(MASTER_ICW3, 4);
        syscall::outb(SLAVE_ICW3, 2);
    }
}

fn nonbuffer_mode() {
    unsafe {
        syscall::outb(MASTER_ICW4, 1);
        syscall::outb(SLAVE_ICW4, 1);
    }
}

fn mask_pic() {
    unsafe {
        syscall::outb(MASTER_DATA, 0xFF);
        syscall::outb(SLAVE_DATA, 0xFF);
    }
}
