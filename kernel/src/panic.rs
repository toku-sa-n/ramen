// SPDX-License-Identifier: GPL-3.0-or-later

use crate::qemu;
use core::{fmt::Write, format_args};
use log::error;
use uart_16550::SerialPort;
use x86_64::instructions::interrupts;

#[panic_handler]
fn panic(i: &core::panic::PanicInfo<'_>) -> ! {
    interrupts::disable();

    print_banner();
    print_info(i);

    fini()
}

fn print_banner() {
    error!("*************");
    error!("*   PANIC   *");
    error!("*************");
}

fn print_info(i: &core::panic::PanicInfo<'_>) {
    let mut s = unsafe { SerialPort::new(0x3f8) };
    s.init();
    s.write_fmt(format_args!("{}\n", i)).unwrap();

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
