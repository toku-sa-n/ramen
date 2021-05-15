#![no_std]

extern crate alloc;
extern crate ralib;

mod heap;
mod sync;

use conquer_once::spin::OnceCell;
use frame_manager::FrameManager;
use spinning_top::Spinlock;
use sync::sync;

static FRAME_MANAGER: OnceCell<Spinlock<FrameManager>> = OnceCell::uninit();

#[no_mangle]
pub fn main() {
    init();
}

fn init() {
    ralib::init();
    heap::init();
    sync();
}
