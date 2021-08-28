#![no_std]
#![no_main]

use xhci::Executor;

#[no_mangle]
fn main() {
    ralib::init();

    xhci::init();

    let mut executor = Executor::new();
    executor.run();
}
