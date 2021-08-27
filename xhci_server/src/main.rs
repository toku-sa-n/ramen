#![no_std]
#![no_main]

use xhci_server::Executor;

#[no_mangle]
fn main() {
    ralib::init();

    xhci_server::init();

    let mut executor = Executor::new();
    executor.run();
}
