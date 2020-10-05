// SPDX-License-Identifier: GPL-3.0-or-later

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("*************");
    error!("*   PANIC   *");
    error!("*************");
    if let Some(location) = info.location() {
        error!(
            "Panic in {} at ({}, {}):{}",
            location.file(),
            location.line(),
            location.column(),
            info.message().unwrap_or(&format_args!(""))
        );
    }

    loop {
        x86_64::instructions::hlt();
    }
}
