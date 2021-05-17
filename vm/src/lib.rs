#![no_std]

mod heap;
mod process;

extern crate ralib as _;

#[no_mangle]
fn main() {
    heap::init();
}
