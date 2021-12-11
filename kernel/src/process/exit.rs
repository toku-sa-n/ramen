// SPDX-License-Identifier: GPL-3.0-or-later

use {super::manager, x86_64::software_interrupt};

pub(crate) fn exit_process() -> ! {
    super::set_temporary_stack_frame();
    // TODO: Call this. Currently this calling will cause a panic because the `KBox` is not mapped
    // to this process.
    // super::collections::process::remove(super::manager::getpid().into());

    manager::pop();
    cause_timer_interrupt();
}

fn cause_timer_interrupt() -> ! {
    unsafe {
        software_interrupt!(0x20);
    }

    unreachable!();
}
