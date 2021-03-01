// SPDX-License-Identifier: GPL-3.0-or-later

use common::constant::INTERRUPT_STACK;

// Do not define this as a function as the function cannot return.
macro_rules! change_stack {
    () => {
        const RSP: u64 = INTERRUPT_STACK.as_u64();
        unsafe {
            asm!("
            mov rsp, {}
            ", const RSP);
        }
    };
}

pub fn exit() -> ! {
    change_stack!();
    super::set_temporary_stack_frame();
    // TODO: Call this. Currently this calling will cause a panic because the `KBox` is not mapped
    // to this process.
    // super::collections::process::remove(super::manager::getpid().into());

    super::collections::woken_pid::pop();
    cause_timer_interrupt();
}

fn cause_timer_interrupt() -> ! {
    unsafe { asm!("int 0x20", options(noreturn)) }
}
