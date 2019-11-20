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
pub fn os_main(_argc: isize, _argv: *const *const u8) -> isize {
    for i in 0xa0000..0xb0000 {
        let ptr = unsafe { &mut *(i as *mut u8) };
        *ptr = 15;
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
