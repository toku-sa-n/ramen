// SPDX-License-Identifier: GPL-3.0-or-later

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    syscalls::disable_interrupt();
    print_banner();
    if let Some(location) = info.location() {
        print_panic_location(location, info);
    }

    loop {
        syscalls::halt();
    }
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
