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
pub fn os_main() -> isize {
    for i in 0xa0000..0xb0000 {
        unsafe {
            *(&mut *(i as *mut u8)) = 15;
        }
    }

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
