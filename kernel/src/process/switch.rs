// SPDX-License-Identifier: GPL-3.0-or-later

use core::sync::atomic::Ordering;

use super::{
    collections::{self, woken_pid},
    context::Context,
    Process, State,
};
use crate::{tests, tss::TSS};
use x86_64::VirtAddr;

#[allow(clippy::too_many_lines)]
pub(crate) fn switch() {
    if cfg!(feature = "qemu_test") {
        tests::process::count_switch();
    }

    let old_context = collections::process::handle_running(|p| VirtAddr::from_ptr(&p.context));

    if collections::process::handle_running(|p| p.state == State::Current) {
        if collections::woken_pid::next() == collections::woken_pid::CURRENT.load(Ordering::Relaxed)
        {
            return;
        }

        collections::process::handle_running_mut(|p| {
            p.state = State::Ready;
            woken_pid::push(p.id);
        });
    }

    let id = woken_pid::pop();

    woken_pid::CURRENT.store(id, Ordering::Relaxed);

    let new_context = collections::process::handle_running(|p| {
        log::info!("Process name: {}", p.name);
        VirtAddr::from_ptr(&p.context)
    });

    register_current_stack_frame_with_tss();

    Context::switch_context(old_context.as_mut_ptr(), new_context.as_mut_ptr());
}

fn register_current_stack_frame_with_tss() {
    TSS.lock().interrupt_stack_table[0] = current_kernel_stack_bottom();
}

fn current_kernel_stack_bottom() -> VirtAddr {
    collections::process::handle_running(Process::kernel_stack_bottom)
}
