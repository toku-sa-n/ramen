#![no_std]

extern crate ralib;

mod heap;

#[no_mangle]
pub fn main() {
    ralib::init();
    heap::init();
}
