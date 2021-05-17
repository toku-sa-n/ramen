#![no_std]

mod heap;
mod page_table;
mod process;
mod table_box;
mod table_collection;

extern crate ralib as _;

#[no_mangle]
fn main() {
    heap::init();
}
