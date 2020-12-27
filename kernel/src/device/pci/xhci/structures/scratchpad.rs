// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::allocator::page_box::PageBox;
use x86_64::PhysAddr;

struct BufferArray(PageBox<PhysAddr>);
