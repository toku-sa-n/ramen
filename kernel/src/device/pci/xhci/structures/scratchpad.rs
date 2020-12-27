// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::allocator::page_box::PageBox;
use conquer_once::spin::OnceCell;
use x86_64::PhysAddr;

static BUFFER_ARRAY: OnceCell<BufferArray> = OnceCell::uninit();

struct BufferArray(PageBox<PhysAddr>);
