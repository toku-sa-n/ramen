// SPDX-License-Identifier: GPL-3.0-or-later

use super::Process;
use alloc::collections::VecDeque;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
use x86_64::VirtAddr;

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

    fn switch_process(rsp: u64) -> u64 {
        let mut m = MANAGER.lock();
        m.tasks[0].rsp = VirtAddr::new(rsp);

        let t = m.tasks.pop_front().unwrap();
        m.tasks.push_back(t);

        m.tasks[0].rsp.as_u64()
    }
}
