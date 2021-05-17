#![no_std]

mod heap;
mod page_box;
mod process;

extern crate ralib as _;

#[no_mangle]
fn main() {
    heap::init();
}
