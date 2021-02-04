// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub(in crate::device::pci::xhci) struct Inquiry([u8; 36]);
impl Default for Inquiry {
    fn default() -> Self {
        Self([0; 36])
    }
}
