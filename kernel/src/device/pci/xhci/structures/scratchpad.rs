// SPDX-License-Identifier: GPL-3.0-or-later

use super::dcbaa;
use crate::{device::pci::xhci, mem::allocator::page_box::PageBox};
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use core::convert::TryInto;
use os_units::Bytes;
use x86_64::PhysAddr;

static SCRATCHPAD: OnceCell<Scratchpad> = OnceCell::uninit();

pub fn init() {
    if Scratchpad::exists() {
        init_static();
    }
}

fn init_static() {
    let mut scratchpad = Scratchpad::new();
    scratchpad.init();
    scratchpad.register_with_dcbaa();

    SCRATCHPAD.init_once(|| scratchpad)
}

struct Scratchpad {
    arr: PageBox<[PhysAddr]>,
    bufs: Vec<PageBox<[u8]>>,
}
impl Scratchpad {
    fn new() -> Self {
        let len: usize = Self::num_of_buffers().try_into().unwrap();

        Self {
            arr: PageBox::new_slice(PhysAddr::zero(), len),
            bufs: Vec::new(),
        }
    }

    fn exists() -> bool {
        Self::num_of_buffers() > 0
    }

    fn init(&mut self) {
        self.allocate_buffers();
        self.write_buffer_addresses();
    }

    fn register_with_dcbaa(&self) {
        dcbaa::register(0, self.arr.phys_addr());
    }

    fn allocate_buffers(&mut self) {
        for _ in 0..Self::num_of_buffers() {
            let b = PageBox::new_slice(0, Self::page_size().as_usize());
            self.bufs.push(b);
        }
    }

    fn write_buffer_addresses(&mut self) {
        for (x, buf) in self.arr.iter_mut().zip(self.bufs.iter()) {
            *x = buf.phys_addr();
        }
    }

    fn num_of_buffers() -> u32 {
        xhci::handle_registers(|r| r.capability.hcs_params_2.read().max_scratchpad_bufs())
    }

    fn page_size() -> Bytes {
        xhci::handle_registers(|r| r.operational.page_size.read().as_bytes())
    }
}
