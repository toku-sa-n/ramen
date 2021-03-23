#![no_std]

#[no_mangle]
pub fn main() {}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
