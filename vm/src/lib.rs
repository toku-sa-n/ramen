#![no_std]

mod heap;

extern crate ralib as _;

#[no_mangle]
fn main() {
    heap::init();
}
