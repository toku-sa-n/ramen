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
#[repr(C, packed)]
pub(in crate::device::pci::xhci) struct ReadFormatCapacities {
    pub header: ReadFormatCapacitiesHeader,
    pub descriptors: [CapacityDescriptor; 31],
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
pub(in crate::device::pci::xhci) struct ReadFormatCapacitiesHeader {
    _rsvd: [u8; 3],
    pub list_len: u8,
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(transparent)]
pub(in crate::device::pci::xhci) struct CapacityDescriptor([u8; 8]);
