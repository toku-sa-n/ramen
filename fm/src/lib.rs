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
}
