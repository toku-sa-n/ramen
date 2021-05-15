#![no_std]

extern crate alloc;
extern crate ralib;

mod heap;

use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use core::convert::TryInto;
use frame_manager::{FrameManager, Frames};
use message::Message;
use os_units::NumOfPages;
use spinning_top::Spinlock;
use x86_64::PhysAddr;

static FRAME_MANAGER: OnceCell<Spinlock<FrameManager>> = OnceCell::uninit();

#[no_mangle]
pub fn main() {
    init();
}

fn init() {
    ralib::init();
    heap::init();
    sync_with_sysproc();
}

fn sync_with_sysproc() {
    start_sync_with_sysproc();
    init_frame_manager();
}

fn start_sync_with_sysproc() {
    let m = receive_from_sysproc();

    assert_eq!(
        m.body.0,
        fm_message::Ty::StartInitialization as _,
        "Failed to receive the Start Initialization message from the sysproc."
    );
}

fn init_frame_manager() {
    let mut m;
    let mut v = Vec::new();

    while {
        m = receive_from_sysproc();

        m.body.0 != fm_message::Ty::EndInitialization as u64
    } {
        let start = PhysAddr::new(m.body.1);
        let num_of_pages = NumOfPages::new(m.body.2.try_into().unwrap());
        let available = match m.body.3 {
            0 => false,
            1 => true,
            _ => unreachable!("`available` is neither 0 nor 1."),
        };

        let frames = if available {
            Frames::new_for_available(start, num_of_pages)
        } else {
            Frames::new_for_used(start, num_of_pages)
        };

        v.push(frames);
    }

    let r = FRAME_MANAGER.try_init_once(|| Spinlock::new(v.into()));
    r.expect("Failed to initialize `FRAME_MANAGER`.");
}

fn receive_from_sysproc() -> Message {
    syscalls::receive_from(5)
}
