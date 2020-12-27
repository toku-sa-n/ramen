// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{device::pci::xhci, mem::allocator::page_box::PageBox};
use conquer_once::spin::OnceCell;
use x86_64::PhysAddr;

static BUFFER_ARRAY: OnceCell<BufferArray> = OnceCell::uninit();

struct BufferArray(PageBox<PhysAddr>);

fn max_scratchpad_buffers() -> u32 {
    xhci::handle_registers(|r| r.capability.hcs_params_2.read().max_scratchpad_bufs())
}
