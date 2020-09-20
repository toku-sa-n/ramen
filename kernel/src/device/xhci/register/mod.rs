// SPDX-License-Identifier: GPL-3.0-or-later

pub mod hc_capability_registers;
pub mod hc_operational_registers;
pub mod usb_legacy_support_capability;

pub trait Register {
    fn name() -> &'static str;
    fn new(base: x86_64::PhysAddr, offset: usize) -> Self;
}
