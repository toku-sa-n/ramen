use message::Message;

pub(crate) fn main() {
    sync_with_fm();
    loop {
        let _ = syscalls::receive_from_any();
    }
}

fn sync_with_fm() {
    start_sync_with_fm();
}

fn start_sync_with_fm() {
    let h = message::Header::default();
    let b = message::Body(fm_message::Ty::StartInitialization as _, 0, 0, 0, 0);
    let m = Message::new(h, b);

    syscalls::send(m, 4);
}
