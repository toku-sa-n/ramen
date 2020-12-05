// SPDX-License-Identifier: GPL-3.0-or-later

pub mod apic;
mod handler;
pub mod idt;
pub mod pic;

const PIC0_ICW1: u16 = 0x0020;
const PIC0_OCW2: u16 = 0x0020;
const PIC0_IMR: u16 = 0x0021;
const PIC0_ICW2: u16 = 0x0021;
const PIC0_ICW3: u16 = 0x0021;
const PIC0_ICW4: u16 = 0x0021;
const PIC1_ICW1: u16 = 0x00A0;
const PIC1_OCW2: u16 = 0x00A0;
const PIC1_IMR: u16 = 0x00A1;
const PIC1_ICW2: u16 = 0x00A1;
const PIC1_ICW3: u16 = 0x00A1;
const PIC1_ICW4: u16 = 0x00A1;
