// SPDX-License-Identifier: GPL-3.0-or-later

use qemu_exit::QEMUExit;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    syscalls::disable_interrupt();
    print_banner();
    if let Some(location) = info.location() {
        print_panic_location(location, info);
    }

    fini()
}

fn print_banner() {
    error!("*************");
    error!("*   PANIC   *");
    error!("*************");
}

fn print_panic_location(location: &core::panic::Location, info: &core::panic::PanicInfo) {
    error!(
        "Panic in {} at ({}, {}):{}",
        location.file(),
        location.line(),
        location.column(),
        info.message().unwrap_or(&format_args!(""))
    );
}

fn fini() -> ! {
    if cfg!(feature = "qemu_test") {
        qemu_exit::X86::new(0xf4, 33).exit_success();
    } else {
        loop {
            syscalls::halt()
        }
    }
}
