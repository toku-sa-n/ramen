// SPDX-License-Identifier: GPL-3.0-or-later

pub mod hc_capability_registers;
pub mod hc_operational_registers;
pub mod usb_legacy_support_capability;

use {
    core::{
        marker::PhantomData,
        ops::{Deref, DerefMut},
    },
    x86_64::VirtAddr,
};

pub trait Register {
    fn name() -> &'static str;
    fn new(base: x86_64::PhysAddr, offset: usize) -> Self;
}

struct Accessor<'a, T: 'a + Register> {
    base: VirtAddr,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: 'a + Register> Deref for Accessor<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.base.as_ptr() }
    }
}

impl<'a, T: 'a + Register> DerefMut for Accessor<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.base.as_mut_ptr() }
    }
}
