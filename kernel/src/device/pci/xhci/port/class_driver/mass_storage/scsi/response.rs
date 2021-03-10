// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub(in crate::device::pci::xhci) struct Inquiry([u8; 36]);
impl Default for Inquiry {
    fn default() -> Self {
        Self([0; 36])
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(transparent)]
pub(in crate::device::pci::xhci) struct ReadCapacity10([u8; 8]);

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub(in crate::device::pci::xhci) struct Read10([u8; 32768]);
impl Default for Read10 {
    fn default() -> Self {
        Self([0; 32768])
    }
}
