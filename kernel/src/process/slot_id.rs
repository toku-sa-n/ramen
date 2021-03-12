// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::collections::BTreeSet;
use core::ops::DerefMut;
use spinning_top::Spinlock;

pub(crate) type SlotId = u64;

static GENERATOR: Spinlock<Generator> = Spinlock::new(Generator::new());

pub(super) fn generate() -> SlotId {
    lock_generator().generate()
}

fn lock_generator() -> impl DerefMut<Target = Generator> {
    GENERATOR.try_lock().expect("Failed to lock `GENERATOR`.")
}

#[derive(Default)]
struct Generator {
    used_ids: BTreeSet<SlotId>,
}
impl Generator {
    const fn new() -> Self {
        Self {
            used_ids: BTreeSet::new(),
        }
    }

    fn generate(&mut self) -> SlotId {
        for i in 0..SlotId::MAX {
            if !self.used_ids.contains(&i) {
                self.used_ids.insert(i);
                return i;
            }
        }

        panic!("No available Slot ID found.");
    }
}
