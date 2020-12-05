use x86_64::instructions::port::Port;

// SPDX-License-Identifier: GPL-3.0-or-later

pub fn disable() {
    pic_init_mode();
    remap_pic();
    set_slave_offset();
    nonbuffer_mode();
    mask_pic();
}

fn pic_init_mode() {
    const PIC0_ICW1: u16 = 0x0020;
    const PIC1_ICW1: u16 = 0x00A0;

    unsafe {
        Port::new(PIC0_ICW1).write(0x11_u8);
        Port::new(PIC1_ICW1).write(0x11_u8);
    }
}

fn remap_pic() {
    const PIC0_ICW2: u16 = 0x0021;
    const PIC1_ICW2: u16 = 0x00A1;

    unsafe {
        Port::new(PIC0_ICW2).write(0x20_u8);
        Port::new(PIC1_ICW2).write(0x28_u8);
    }
}

fn set_slave_offset() {
    const PIC0_ICW3: u16 = 0x0021;
    const PIC1_ICW3: u16 = 0x00A1;

    unsafe {
        Port::new(PIC0_ICW3).write(4_u8);
        Port::new(PIC1_ICW3).write(2_u8);
    }
}

fn nonbuffer_mode() {
    const PIC0_ICW4: u16 = 0x0021;
    const PIC1_ICW4: u16 = 0x00A1;

    unsafe {
        Port::new(PIC0_ICW4).write(1_u8);
        Port::new(PIC1_ICW4).write(1_u8);
    }
}

fn mask_pic() {
    const PIC0_IMR: u16 = 0x0021;
    const PIC1_IMR: u16 = 0x00A1;

    // Safety: These operations are safe because `PIC0_IMR` and `PIC1_IMR` are the valid port numbers.
    unsafe {
        Port::new(PIC0_IMR).write(0xFF_u8);
        Port::new(PIC1_IMR).write(0xFF_u8);
    }
}
