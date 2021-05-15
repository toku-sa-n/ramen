use core::convert::TryInto;

use crate::mem::allocator::phys;
use frame_manager::Frames;
use message::Message;

pub(crate) fn main() {
    sync_with_fm();
    loop {
        let _ = syscalls::receive_from_any();
    }
}

fn sync_with_fm() {
    start_sync_with_fm();
    send_frame_ranges();
}

fn start_sync_with_fm() {
    let b = message::Body(fm_message::Ty::StartInitialization as _, 0, 0, 0, 0);

    send_to_fm(b);
}

fn send_frame_ranges() {
    for f in &phys::frames() {
        send_frame_range(f);
    }
}

fn send_frame_range(f: &Frames) {
    let start = f.start().as_u64();
    let num: u64 = f.num_of_pages().as_usize().try_into().unwrap();
    let availble = f.available();

    let b = message::Body(fm_message::Ty::AddFrames as _, start, num, availble as _, 0);
    send_to_fm(b);
}

fn send_to_fm(b: message::Body) {
    let h = message::Header::default();
    let m = Message::new(h, b);

    syscalls::send(m, 4);
}
