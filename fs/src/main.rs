#![no_std]
#![no_main]

use fs::ProcessCollection;

#[no_mangle]
fn main() {
    ralib::init();

    let mut c = ProcessCollection::default();
    fs::init(&mut c);
}
