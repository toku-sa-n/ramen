// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
    collections::{self, woken_pid},
    Process,
};
use crate::{tests, tss::TSS};
use x86_64::{registers::control::Cr3, VirtAddr};

pub(crate) fn switch() -> VirtAddr {
    if cfg!(feature = "qemu_test") {
        tests::process::count_switch();
    }

    change_current_process();
    switch_pml4();
    register_current_stack_frame_with_tss();
    current_stack_frame_top_addr()
}

fn change_current_process() {
    woken_pid::change_active_pid();
}

fn switch_pml4() {
    let (_, f) = Cr3::read();
    let pml4 = collections::process::handle_running(|p| p.pml4);

    // SAFETY: The PML4 frame is correct one and flags are unchanged.
    unsafe { Cr3::write(pml4, f) }
}

fn register_current_stack_frame_with_tss() {
    TSS.lock().interrupt_stack_table[0] = current_stack_frame_bottom_addr();
}

fn current_stack_frame_top_addr() -> VirtAddr {
    collections::process::handle_running(Process::stack_frame_top_addr)
}

fn current_stack_frame_bottom_addr() -> VirtAddr {
    collections::process::handle_running(Process::stack_frame_bottom_addr)
}
