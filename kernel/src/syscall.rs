// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;
use x86_64::{
    instructions::interrupts,
    registers::model_specific::{Efer, EferFlags, LStar},
    VirtAddr,
};

pub fn init() {
    enable();
    register();
}

fn enable() {
    // Safety: This operation is safe as this does not touch any unsafe things.
    unsafe { Efer::update(|e| *e |= EferFlags::SYSTEM_CALL_EXTENSIONS) }
}

fn register() {
    let addr = wrapper as usize;

    LStar::write(VirtAddr::new(addr.try_into().unwrap()));
}

fn wrapper() {
    unsafe {
        asm!(
            "
            cli
        push rcx    # Save rip
        push r11    # Save rflags
        "
        );

        syscall();

        asm!(
            "
        pop r11     # Restore rflags
        pop rcx     # Restore rip
        sti
        sysretq
        "
        );
    }
}

fn syscall() {
    info!("This is `syscall` function.");
}
