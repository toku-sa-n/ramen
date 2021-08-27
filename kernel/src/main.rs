#![no_std]
#![no_main]

use common::kernelboot;

#[no_mangle]
extern "win64" fn os_main(mut boot_info: kernelboot::Info) -> ! {
    kernel::init(&mut boot_info);
    kernel::cause_timer_interrupt();
}
