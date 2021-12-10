// SPDX-License-Identifier: GPL-3.0-or-later

use {
    alloc::collections::BTreeSet, conquer_once::spin::Lazy, core::ops::DerefMut,
    spinning_top::Spinlock,
};

pub(crate) type Pid = i32;

static GENERATOR: Lazy<Spinlock<Generator>> = Lazy::new(|| Spinlock::new(Generator::new()));

pub(super) fn generate() -> Pid {
    lock_generator().generate()
}

fn lock_generator() -> impl DerefMut<Target = Generator> {
    GENERATOR.try_lock().expect("Failed to lock `GENERATOR`.")
}

#[derive(Default)]
struct Generator {
    used_ids: BTreeSet<Pid>,
}
impl Generator {
    fn new() -> Self {
        Self {
            used_ids: BTreeSet::new(),
        }
    }

    fn generate(&mut self) -> Pid {
        for i in 0..Pid::MAX {
            if !self.used_ids.contains(&i) {
                self.used_ids.insert(i);
                return i;
            }
        }

        panic!("No available Slot ID found.");
    }
}
