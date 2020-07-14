#![no_std]
#![feature(lang_items, start)]
#![no_main]

extern crate uefi;

use uefi::prelude::{Boot, Handle, Status, SystemTable};

#[start]
pub fn efi_main(image: Handle, system_table: SystemTable<Boot>) -> Status {
    loop {}
}

#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
