// SPDX-License-Identifier: GPL-3.0-or-later

#[no_mangle]
extern "C" fn exit_process() -> ! {
    super::set_temporary_stack_frame();
    // TODO: Call this. Currently this calling will cause a panic because the `KBox` is not mapped
    // to this process.
    // super::collections::process::remove(super::manager::getpid().into());

    super::collections::woken_pid::pop();
    cause_timer_interrupt();
}

fn cause_timer_interrupt() -> ! {
    extern "C" {
        fn cause_timer_interrupt_asm() -> !;
    }

    unsafe { cause_timer_interrupt_asm() }
}
