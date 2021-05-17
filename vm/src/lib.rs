#![no_std]

mod frame;
mod heap;
mod page_table;
mod process;

extern crate alloc;
extern crate ralib as _;

const PID: i32 = 4;

#[no_mangle]
fn main() {
    heap::init();
}
