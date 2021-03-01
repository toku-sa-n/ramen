// SPDX-License-Identifier: GPL-3.0-or-later

use crate::qemu;

#[panic_handler]
fn panic(i: &core::panic::PanicInfo) -> ! {
    print_banner();
    print_info(i);

    fini()
}

fn print_banner() {
    error!("*************");
    error!("*   PANIC   *");
    error!("*************");
}

fn print_info(i: &core::panic::PanicInfo) {
    error!("{}", i);
}

fn fini() -> ! {
    if cfg!(feature = "qemu_test") {
        qemu::exit_failure();
    } else {
        loop {
            x86_64::instructions::nop();
        }
    }
}
