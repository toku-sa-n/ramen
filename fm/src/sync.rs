use alloc::vec::Vec;
use core::convert::TryInto;
use frame_manager::Frames;
use message::Message;
use os_units::NumOfPages;
use spinning_top::Spinlock;
use x86_64::PhysAddr;

pub(super) fn sync() {
    Syncer::default().sync();
}

#[derive(Default)]
struct Syncer(Vec<Frames>);
impl Syncer {
    fn sync(mut self) {
        Self::receive_start_initialization();
        self.receive_memory_map();
        self.init_global_frame_manager();
    }

    fn receive_start_initialization() {
        let m = receive_from_sysproc();

        assert_eq!(
            m.body.0,
            fm_message::Ty::StartSync as _,
            "Failed to receive the Start Initialization message from the sysproc."
        );
    }

    fn receive_memory_map(&mut self) {
        let mut m;

        while {
            m = receive_from_sysproc();

            !Self::sync_finished(m)
        } {
            self.push_frame(m);
        }
    }

    fn push_frame(&mut self, m: Message) {
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

        self.0.push(frames);
    }

    fn sync_finished(m: Message) -> bool {
        m.body.0 == fm_message::Ty::EndSync as u64
    }

    fn init_global_frame_manager(self) {
        let r = super::FRAME_MANAGER.try_init_once(|| Spinlock::new(self.0.into()));
        r.expect("Failed to initialize `FRAME_MANAGER`.");
    }
}

fn receive_from_sysproc() -> Message {
    syscalls::receive_from(5)
}
