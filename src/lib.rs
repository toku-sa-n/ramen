#![no_std]
#![feature(asm)]
#![feature(start)]

#[no_mangle]
fn hlt() -> () {
    unsafe {
        asm!("hlt");
    }
}

#[no_mangle]
#[start]
pub extern "C" fn os_main() -> ! {
    loop {
        hlt()
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        hlt()
    }
}
