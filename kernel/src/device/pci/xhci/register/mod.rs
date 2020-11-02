// SPDX-License-Identifier: GPL-3.0-or-later

pub mod doorbell;
pub mod hc_capability_registers;
pub mod hc_operational_registers;
pub mod runtime_base_registers;
pub mod usb_legacy_support_capability;

use {
    hc_capability_registers::{HCCapabilityRegisters, MaxNumOfErst, NumberOfDeviceSlots},
    hc_operational_registers::HCOperationalRegisters,
    runtime_base_registers::RuntimeBaseRegisters,
    usb_legacy_support_capability::UsbLegacySupportCapability,
    x86_64::PhysAddr,
};

pub struct Registers {
    usb_legacy_support_capability: Option<UsbLegacySupportCapability>,
    hc_capability_registers: HCCapabilityRegisters,
    pub hc_operational_registers: HCOperationalRegisters,
    runtime_base_registers: RuntimeBaseRegisters,
    doorbell_array: doorbell::Array,
}
impl Registers {
    pub fn new(mmio_base: PhysAddr) -> Self {
        let hc_capability_registers = HCCapabilityRegisters::new(mmio_base);
        let usb_legacy_support_capability =
            UsbLegacySupportCapability::new(mmio_base, &hc_capability_registers);
        let hc_operational_registers =
            HCOperationalRegisters::new(mmio_base, &hc_capability_registers);
        let runtime_base_registers = RuntimeBaseRegisters::new(
            mmio_base,
            hc_capability_registers.offset_to_runtime_registers() as usize,
        );
        let doorbell_array = doorbell::Array::new(mmio_base, hc_capability_registers.db_off());

        Self {
            usb_legacy_support_capability,
            hc_capability_registers,
            hc_operational_registers,
            runtime_base_registers,
            doorbell_array,
        }
    }

    pub fn max_num_of_erst(&self) -> MaxNumOfErst {
        self.hc_capability_registers.max_num_of_erst()
    }

    pub fn num_of_device_slots(&self) -> NumberOfDeviceSlots {
        self.hc_capability_registers.number_of_device_slots()
    }

    pub fn transfer_hc_ownership_to_os(&mut self) {
        if let Some(ref mut usb_leg_sup_cap) = self.usb_legacy_support_capability {
            usb_leg_sup_cap.give_hc_ownership_to_os();
        }
    }

    pub fn reset_hc(&mut self) {
        self.hc_operational_registers.reset_hc();
    }

    pub fn wait_until_hc_is_ready(&self) {
        self.hc_operational_registers.wait_until_hc_is_ready();
    }

    pub fn init_num_of_slots(&mut self) {
        let num_of_slots = self.hc_capability_registers.number_of_device_slots();

        self.hc_operational_registers
            .set_num_of_device_slots(num_of_slots);
    }

    pub fn set_dcbaap(&mut self, addr: PhysAddr) {
        self.hc_operational_registers.set_dcbaa_ptr(addr);
    }

    pub fn set_command_ring_pointer(&mut self, addr: PhysAddr) {
        self.hc_operational_registers.set_command_ring_ptr(addr)
    }

    pub fn set_event_ring_segment_table_size(&mut self) {
        let max_num_of_erst = self.hc_capability_registers.max_num_of_erst();
        self.runtime_base_registers
            .set_event_ring_segment_table_size(max_num_of_erst.into());
    }

    pub fn set_event_ring_segment_table_addr(&mut self, addr: PhysAddr) {
        self.runtime_base_registers
            .set_event_ring_segment_table_addr(addr)
    }

    pub fn set_event_ring_dequeue_pointer(&mut self, addr: PhysAddr) {
        self.runtime_base_registers.set_event_ring_dequeue_ptr(addr)
    }

    pub fn notify_to_hc(&mut self) {
        self.doorbell_array.notify_to_hc();
    }
}
