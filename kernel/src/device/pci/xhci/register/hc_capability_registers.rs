// SPDX-License-Identifier: GPL-3.0-or-later

use {crate::mem::accessor::Accessor, bitfield::bitfield, os_units::Bytes, x86_64::PhysAddr};

pub struct HCCapabilityRegisters {
    cap_length: Accessor<CapabilityRegistersLength>,
    hcs_params_1: Accessor<StructuralParameters1>,
    hcs_params_2: Accessor<StructuralParameters2>,
    hc_cp_params_1: Accessor<HCCapabilityParameters1>,
    db_off: Accessor<DoorbellOffset>,
    rts_off: Accessor<RuntimeRegisterSpaceOffset>,
}

impl HCCapabilityRegisters {
    pub fn new(mmio_base: PhysAddr) -> Self {
        let cap_length = Accessor::new(mmio_base, Bytes::new(0));
        let hcs_params_1 = Accessor::new(mmio_base, Bytes::new(0x04));
        let hcs_params_2 = Accessor::new(mmio_base, Bytes::new(0x08));
        let hc_cp_params_1 = Accessor::new(mmio_base, Bytes::new(0x10));
        let db_off = Accessor::new(mmio_base, Bytes::new(0x14));
        let rts_off = Accessor::new(mmio_base, Bytes::new(0x18));

        let hci_version = Accessor::<HCInterfaceVersionNumber>::new(mmio_base, Bytes::new(0x2));
        info!("xHC version: {:X}", hci_version.get());

        Self {
            cap_length,
            hcs_params_1,
            hcs_params_2,
            hc_cp_params_1,
            db_off,
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

    pub fn db_off(&self) -> &DoorbellOffset {
        &*self.db_off
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
    struct StructuralParameters2(u32);
    u8, erst_max, _: 7, 4;
}

bitfield! {
    #[repr(transparent)]
    struct HCCapabilityParameters1(u32);
    xhci_extended_capabilities_pointer,_: 31,16;
}

#[repr(transparent)]
pub struct DoorbellOffset(u32);
impl DoorbellOffset {
    pub fn get(&self) -> u32 {
        self.0
    }
}

#[repr(transparent)]
struct RuntimeRegisterSpaceOffset(u32);
impl RuntimeRegisterSpaceOffset {
    fn get(&self) -> u32 {
        self.0 & !0x1f
    }
}
