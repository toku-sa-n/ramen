#![no_std]
#![no_main]

use common::kernelboot;

#[no_mangle]
pub extern "win64" fn os_main(mut boot_info: kernelboot::Info) -> ! {
    ramen::init(&mut boot_info);
    ramen::cause_timer_interrupt();
}
