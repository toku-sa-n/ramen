// SPDX-License-Identifier: GPL-3.0-or-later

use {core::marker::PhantomData, x86_64::VirtAddr};

pub struct PageBox<T> {
    virt: VirtAddr,
    _marker: PhantomData<T>,
}
