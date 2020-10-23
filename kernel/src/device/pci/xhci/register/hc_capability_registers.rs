// SPDX-License-Identifier: GPL-3.0-or-later

use {crate::mem::accessor::single_object::Accessor, bitfield::bitfield, x86_64::PhysAddr};

pub struct HCCapabilityRegisters<'a> {
    cap_length: Accessor<'a, CapabilityRegistersLength>,
    hcs_params_1: Accessor<'a, StructuralParameters1>,
    hc_cp_params_1: Accessor<'a, HCCapabilityParameters1>,
    rts_off: Accessor<'a, RuntimeRegisterSpaceOffset>,
}

impl<'a> HCCapabilityRegisters<'a> {
    pub fn new(mmio_base: PhysAddr) -> Self {
        let cap_length = Accessor::new(mmio_base, 0);
        let hcs_params_1 = Accessor::new(mmio_base, 0x04);
        let hc_cp_params_1 = Accessor::new(mmio_base, 0x10);
        let rts_off = Accessor::new(mmio_base, 0x18);

        let hci_version = Accessor::<'a, HCInterfaceVersionNumber>::new(mmio_base, 0x2);
        info!("xHC version: {:X}", hci_version.get());

        Self {
            cap_length,
            hcs_params_1,
            hc_cp_params_1,
            rts_off,
        }
    }

    pub fn number_of_device_slots(&self) -> u32 {
        self.hcs_params_1.number_of_device_slots()
    }

    pub fn xhci_capability_ptr(&self) -> u32 {
        self.hc_cp_params_1.xhci_extended_capabilities_pointer()
    }

    pub fn len(&self) -> usize {
        self.cap_length.get()
    }

    pub fn offset_to_runtime_registers(&self) -> u32 {
        info!("RTSOFF: {}", self.rts_off.get());
        self.rts_off.get()
    }
}

#[repr(transparent)]
struct CapabilityRegistersLength(u8);

impl CapabilityRegistersLength {
    fn get(&self) -> usize {
        self.0 as _
    }
}

#[repr(transparent)]
struct HCInterfaceVersionNumber(u16);
impl HCInterfaceVersionNumber {
    fn get(&self) -> u16 {
        self.0
    }
}

bitfield! {
    #[repr(transparent)]
    struct StructuralParameters1(u32);
    number_of_device_slots, _: 7, 0;
}

bitfield! {
    #[repr(transparent)]
    struct HCCapabilityParameters1(u32);
    xhci_extended_capabilities_pointer,_: 31,16;
}

#[repr(transparent)]
struct RuntimeRegisterSpaceOffset(u32);
impl RuntimeRegisterSpaceOffset {
    fn get(&self) -> u32 {
        self.0 & !0x1f
    }
}
