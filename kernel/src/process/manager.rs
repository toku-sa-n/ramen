// SPDX-License-Identifier: GPL-3.0-or-later

use crate::tss::TSS;

use super::Process;
use alloc::collections::VecDeque;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
use x86_64::VirtAddr;

static MANAGER: Lazy<Spinlock<Manager>> = Lazy::new(|| Spinlock::new(Manager::new()));

pub fn add_process(p: Process) {
    MANAGER.lock().add_process(p);
}

pub fn switch_process() -> VirtAddr {
    MANAGER.lock().switch_process()
}

struct Manager {
    processes: VecDeque<Process>,
}
impl Manager {
    fn new() -> Self {
        Self {
            processes: VecDeque::new(),
        }
    }

    fn add_process(&mut self, p: Process) {
        self.processes.push_back(p)
    }

    fn switch_process(&mut self) -> VirtAddr {
        self.change_current_process();
        self.register_current_stack_frame_with_tss();
        self.current_stack_frame_top_addr()
    }

    fn change_current_process(&mut self) {
        self.processes.rotate_left(1);
    }

    fn register_current_stack_frame_with_tss(&self) {
        TSS.lock().interrupt_stack_table[0] = self.current_stack_frame_bottom_addr();
    }

    fn current_stack_frame_top_addr(&self) -> VirtAddr {
        self.current_process().stack_frame_top_addr()
    }

    fn current_stack_frame_bottom_addr(&self) -> VirtAddr {
        self.current_process().stack_frame_bottom_addr()
    }

    fn current_process(&self) -> &Process {
        &self.processes[0]
    }
}
