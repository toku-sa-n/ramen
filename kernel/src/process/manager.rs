// SPDX-License-Identifier: GPL-3.0-or-later

use crate::gdt::GDT;

use super::Process;
use alloc::collections::VecDeque;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
use x86_64::VirtAddr;

pub static MANAGER: Lazy<Spinlock<Manager>> = Lazy::new(|| Spinlock::new(Manager::new()));

pub struct Manager {
    processes: VecDeque<Process>,
}
impl Manager {
    pub fn switch_process(rsp: VirtAddr) -> VirtAddr {
        let mut m = MANAGER.lock();
        m.update_rsp_of_current_process(rsp);
        m.change_current_process();
        m.rsp_of_current_task()
    }

    pub fn add_process(&mut self, p: Process) {
        self.processes.push_back(p)
    }

    fn new() -> Self {
        Self {
            processes: VecDeque::new(),
        }
    }

    fn update_rsp_of_current_process(&mut self, rsp: VirtAddr) {
        self.processes[0].rsp = rsp;
    }

    fn change_current_process(&mut self) {
        self.processes.rotate_left(1);
    }

    fn rsp_of_current_task(&self) -> VirtAddr {
        self.processes[0].rsp
    }
}
