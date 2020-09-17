// SPDX-License-Identifier: GPL-3.0-or-later

struct UsbLegacySupportCapabilityRegister;

impl UsbLegacySupportCapabilityRegister {
    fn get_hc_bios_owned_semaphore() -> bool {}
    fn get_hc_os_owned_semaphore() -> bool {}
    fn set_hc_os_owned_semaphore() {}
}
