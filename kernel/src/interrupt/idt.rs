// SPDX-License-Identifier: GPL-3.0-or-later

use bit_field::BitField;
use conquer_once::spin::Lazy;
use core::{convert::TryInto, mem};
use x86_64::{
    instructions::{segmentation, tables},
    structures::DescriptorTablePointer,
    PrivilegeLevel, VirtAddr,
};

extern "C" {
    fn h_20_asm();
    fn h_80_asm();
    fn h_81();
}

static IDT: Lazy<Idt> = Lazy::new(|| {
    let mut idt = Idt::new();

    idt.0[0x20].set_handler(h_20_asm);
    idt.0[0x20].set_stack_index(0);

    idt.0[0x80].set_handler(h_80_asm);
    idt.0[0x80].set_stack_index(0);
    idt.0[0x80].set_privilege_level(PrivilegeLevel::Ring3);

    idt.0[0x81].set_handler(h_81);
    idt.0[0x81].set_stack_index(0);
    idt.0[0x81].set_privilege_level(PrivilegeLevel::Ring3);

    idt
});

#[repr(C, align(16))]
#[derive(Debug)]
struct Idt([Entry; 256]);
impl Idt {
    const fn new() -> Self {
        Self([Entry::new(); 256])
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Entry {
    offset_low: u16,
    segment_selector: u16,
    options: Options,
    offset_mid: u16,
    offset_high: u32,
    _rsvd: u32,
}
impl Entry {
    const fn new() -> Self {
        Self {
            offset_low: 0,
            segment_selector: 0,
            options: Options::new(),
            offset_mid: 0,
            offset_high: 0,
            _rsvd: 0,
        }
    }

    fn set_handler(&mut self, f: unsafe extern "C" fn()) {
        let addr = f as usize;
        self.offset_low = addr.get_bits(0..16).try_into().unwrap();
        self.offset_mid = addr.get_bits(16..32).try_into().unwrap();
        self.offset_high = addr.get_bits(32..64).try_into().unwrap();

        self.segment_selector = segmentation::cs().0;
        self.options.set_present();
    }

    fn set_stack_index(&mut self, index: usize) {
        assert!(index < 7, "The Interrupt Stack Index must be less than 8.");
        self.options.set_stack_index_unchecked(index);
    }

    fn set_privilege_level(&mut self, l: PrivilegeLevel) {
        self.options.set_privilege_level(l);
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
struct Options(u16);
impl Options {
    const fn new() -> Self {
        Self(0b1110_0000_0000)
    }

    fn set_stack_index_unchecked(&mut self, index: usize) {
        self.0.set_bits(0..=2, (index + 1).try_into().unwrap());
    }

    fn set_privilege_level(&mut self, l: PrivilegeLevel) {
        self.0.set_bits(13..=14, l as _);
    }

    fn set_present(&mut self) {
        self.0.set_bit(15, true);
    }
}

pub(crate) fn init() {
    unsafe {
        tables::lidt(&DescriptorTablePointer {
            limit: (mem::size_of::<Idt>() - 1).try_into().unwrap(),
            base: VirtAddr::from_ptr(&*IDT),
        })
    }

    log::info!("IDT: {:X?}", IDT.0[0x20]);
}
