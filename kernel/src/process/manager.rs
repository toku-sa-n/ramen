// SPDX-License-Identifier: GPL-3.0-or-later

use crate::tss::TSS;

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
    pub fn switch_process() -> VirtAddr {
        let mut m = MANAGER.lock();
        m.change_current_process();
        m.register_new_stack_frame_with_tss();
        m.next_stack_frame_addr()
    }

    pub fn add_process(&mut self, p: Process) {
        self.processes.push_back(p)
    }

    fn new() -> Self {
        Self {
            processes: VecDeque::new(),
        }
    }

    fn change_current_process(&mut self) {
        self.processes.rotate_left(1);
    }

    fn next_stack_frame_addr(&self) -> VirtAddr {
        self.processes[0].stack_frame.virt_addr()
    }

    fn register_new_stack_frame_with_tss(&self) {
        TSS.lock().privilege_stack_table[0] = self.processes[0].stack_frame_bottom_addr();
    }
}
