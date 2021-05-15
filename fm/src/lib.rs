#![no_std]

extern crate ralib;

mod heap;

#[no_mangle]
pub fn main() {
    init();
}

fn init() {
    ralib::init();
    heap::init();
    sync_with_sysproc();
}

fn sync_with_sysproc() {
    start_sync_with_sysproc();
}

fn start_sync_with_sysproc() {
    let m = syscalls::receive_from(5);

    assert_eq!(
        m.body.0,
        fm_message::Ty::StartInitialization as _,
        "Failed to receive the Start Initialization message from the sysproc."
    );
}
