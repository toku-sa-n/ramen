use super::manager;

// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{tests, tss},
    x86_64::{registers::control::Cr3, VirtAddr},
};

pub(crate) fn switch() -> VirtAddr {
    if cfg!(feature = "qemu_test") {
        tests::process::count_switch();
    }

    change_current_process();
    switch_pml4();
    register_current_stack_frame_with_tss();
    manager::current_stack_frame_top_addr()
}

fn change_current_process() {
    manager::change_active_pid();
}

fn switch_pml4() {
    let (_, f) = Cr3::read();
    let pml4 = manager::current_pml4();

    // SAFETY: The PML4 frame is correct one and flags are unchanged.
    unsafe { Cr3::write(pml4, f) }
}

fn register_current_stack_frame_with_tss() {
    tss::set_interrupt_stack(manager::current_stack_frame_bottom_addr());
}
