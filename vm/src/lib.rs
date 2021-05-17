#![no_std]

mod heap;
mod page_table;
mod process;

extern crate ralib as _;

#[no_mangle]
fn main() {
    heap::init();
}
