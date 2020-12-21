// SPDX-License-Identifier: GPL-3.0-or-later

use super::Process;
use alloc::collections::VecDeque;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
use x86_64::VirtAddr;

static MANAGER: Lazy<Spinlock<Manager>> = Lazy::new(|| Spinlock::new(Manager::new()));

pub struct Manager {
    tasks: VecDeque<Process>,
}
impl Manager {
    fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }

    pub fn switch_process(rsp: VirtAddr) -> VirtAddr {
        let mut m = MANAGER.lock();
        m.update_rsp_of_current_process(rsp);
        m.change_current_process();
        m.rsp_of_current_task()
    }

    fn update_rsp_of_current_process(&mut self, rsp: VirtAddr) {
        self.tasks[0].rsp = rsp;
    }

    fn change_current_process(&mut self) {
        self.tasks.rotate_left(1);
    }

    fn rsp_of_current_task(&self) -> VirtAddr {
        self.tasks[0].rsp
    }
}
