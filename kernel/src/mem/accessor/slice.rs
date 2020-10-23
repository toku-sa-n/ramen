// SPDX-License-Identifier: GPL-3.0-or-later

use {
    core::{
        marker::PhantomData,
        mem::size_of,
        ops::{Deref, DerefMut},
        slice,
    },
    os_units::{Bytes, Size},
    x86_64::{PhysAddr, VirtAddr},
};

#[derive(Debug)]
pub struct Accessor<'a, T: 'a> {
    base: VirtAddr,
    len: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: 'a> Accessor<'a, T> {
    pub fn new(phys_base: PhysAddr, offset: Size<Bytes>, num_elements: usize) -> Self {
        let phys_base = phys_base + offset.as_usize();

        let base = super::map_pages(phys_base, Size::new(size_of::<T>() * num_elements));

        Self {
            base,
            len: num_elements,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    fn object_size(num_elements: usize) -> usize {
        size_of::<T>() * num_elements
    }
}

impl<'a, T: 'a> Deref for Accessor<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.base.as_ptr(), self.len) }
    }
}

impl<'a, T: 'a> DerefMut for Accessor<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.base.as_mut_ptr(), self.len) }
    }
}

impl<'a, T: 'a> Drop for Accessor<'a, T> {
    fn drop(&mut self) {
        super::unmap_pages(self.base, Size::new(Self::object_size(self.len)))
    }
}
