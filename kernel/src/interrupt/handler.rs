// SPDX-License-Identifier: GPL-3.0-or-later

use super::apic;
use crate::process;
use alloc::{collections::BTreeMap, vec::Vec};
use common::constant::INTERRUPT_STACK;
use spinning_top::Spinlock;

static NOTIFY_ON_INTERRUPT: Spinlock<BTreeMap<usize, Vec<i32>>> = Spinlock::new(BTreeMap::new());

pub extern "x86-interrupt" fn h_20(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    // Here, the stack pointer points the stack frame of the current task. By cloberring registers,
    // the state will be stored on the stack frame.
    //
    // SAFETY: This operation is safe. After calling the `switch` function, `rax` contains the address to the top of the stack frame of
    // the new process. It does not violate any memory safety.
    unsafe {
        asm!(
            "
            mov rsp, {}
            call {}
            call {}
            mov rsp, rax
        ", const INTERRUPT_STACK.as_u64(), sym apic::local::end_of_interrupt, sym process::manager::switch, out("rax") _, out("rbx") _, out("rcx") _, out("rdx") _, out("rsi") _, out("rdi") _,  out("r8") _, out("r9") _, out("r10") _, out("r11") _, out("r12") _, out("r13") _, out("r14") _, out("r15") _);
    }
}

pub extern "x86-interrupt" fn h_21(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    apic::local::end_of_interrupt();
    notify(0x21);
}

pub extern "x86-interrupt" fn h_2c(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    apic::local::end_of_interrupt();
    notify(0x2c);
}

pub fn notify_on_interrupt(vec: usize, pid: i32) {
    let mut l = NOTIFY_ON_INTERRUPT.lock();
    let a = l.entry(vec).or_insert_with(Vec::new);
    a.push(pid);
}

fn notify(vec: usize) {
    if let Some(a) = NOTIFY_ON_INTERRUPT.lock().get(&vec) {
        for pid in a {
            process::manager::notify(*pid);
        }
    }
}
