// SPDX-License-Identifier: GPL-3.0-or-later

use {
    core::{
        marker::PhantomData,
        mem::size_of,
        ops::{Deref, DerefMut},
    },
    os_units::Size,
    x86_64::{PhysAddr, VirtAddr},
};

pub struct Accessor<'a, T: 'a> {
    base: VirtAddr,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: 'a> Accessor<'a, T> {
    pub fn new(phys_base: PhysAddr, offset: usize) -> Self {
        use x86_64::structures::paging::MapperAllSizes;
        let phys_base = phys_base + offset;
        debug!("phys_base: {:?}", phys_base);

        let base = super::map_pages(phys_base, Size::new(size_of::<T>()));

        debug!(
            "{:?} -> {:?}",
            base,
            crate::mem::paging::pml4::PML4.lock().translate_addr(base)
        );

        Self {
            base,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Deref for Accessor<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.base.as_ptr() }
    }
}

impl<'a, T: 'a> DerefMut for Accessor<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.base.as_mut_ptr() }
    }
}

impl<'a, T: 'a> Drop for Accessor<'a, T> {
    fn drop(&mut self) {
        super::unmap_pages(self.base, Size::new(size_of::<T>()))
    }
}
