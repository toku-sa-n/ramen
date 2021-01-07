// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
    collections,
    manager::{self, Message},
};

pub fn exit() -> ! {
    free_stack();
    manager::set_temporary_stack_frame();
    send_exit_message();
    cause_timer_interrupt();
}

fn free_stack() {
    collections::process::handle_running_mut(|p| {
        p.stack = None;
        p.stack_frame = None;
    });
}

fn send_exit_message() {
    let id = collections::woken_pid::pop();
    manager::send_message(Message::Exit(id));
}

fn cause_timer_interrupt() -> ! {
    unsafe { asm!("int 0x20", options(noreturn)) }
}
