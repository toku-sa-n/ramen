// SPDX-License-Identifier: GPL-3.0-or-later

pub mod doorbell;
pub mod hc_capability_registers;
pub mod hc_operational_registers;
pub mod runtime_base_registers;
pub mod usb_legacy_support_capability;

use {
    hc_capability_registers::HCCapabilityRegisters,
    hc_operational_registers::HCOperationalRegisters, runtime_base_registers::RuntimeBaseRegisters,
    usb_legacy_support_capability::UsbLegacySupportCapability,
};

pub struct Registers {
    usb_legacy_support_capability: Option<UsbLegacySupportCapability>,
    hc_capability_registers: HCCapabilityRegisters,
    hc_operational_registers: HCOperationalRegisters,
    runtime_base_registers: RuntimeBaseRegisters,
    doorbell_array: doorbell::Array,
}
