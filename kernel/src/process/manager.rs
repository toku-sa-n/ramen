// SPDX-License-Identifier: GPL-3.0-or-later

use super::Process;
use alloc::collections::VecDeque;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;

static MANAGER: Lazy<Spinlock<Manager>> = Lazy::new(|| Spinlock::new(Manager::new()));

struct Manager {
    tasks: VecDeque<Process>,
}
impl Manager {
    fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }
}
