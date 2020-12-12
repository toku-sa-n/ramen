// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

use crate::mem::{allocator::page_box::PageBox, paging::pml4::PML4};
use alloc::vec::Vec;
use spinning_top::Spinlock;
use x86_64::{
    instructions::interrupts,
    structures::paging::{
        page_table::PageTableEntry, PageSize, PageTable, PageTableFlags, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

static QUEUE: Spinlock<Vec<Process>> = Spinlock::new(Vec::new());

struct Process {
    pml4: PageBox<PageTable>,
    rip: VirtAddr,
    rsp: VirtAddr,
    stack: PageBox<[u8]>,
}
impl Process {
    fn new(entry_addr: VirtAddr) -> Self {
        let stack = PageBox::new_slice(0, Size4KiB::SIZE.try_into().unwrap());
        Self {
            pml4: Pml4Creator::new().create(),
            rip: entry_addr,
            rsp: stack.virt_addr() + stack.bytes().as_usize(),
            stack,
        }
    }

    fn init_stack(&mut self) {
        interrupts::disable();

        let rsp: u64;
        unsafe {
            asm!("
            # Save the stack pointer.
            mov rcx, rsp

            # Jump to the stack of this process.
            mov rsp, {}

            # Save registers
            push {} # rip
            push 0  # rbp
            push 0  # r15
            push 0  # r14
            push 0  # r13
            push 0  # r12
            push 0  # r11
            push 0  # r10
            push 0  # r9
            push 0  # r8
            push 0  # rdi
            push 0  # rsi
            push 0  # rdx
            push 0  # rcx
            push 0  # rax

            # Return the current rsp
            mov {}, rsp

            # Restore rsp
            mov rsp, rcx
            ", in(reg) self.rsp.as_u64(),in(reg) self.rip.as_u64(),out(reg) rsp);
        }

        self.rsp = VirtAddr::new(rsp);
    }
}

struct Pml4Creator {
    pml4: PageBox<PageTable>,
}
impl Pml4Creator {
    fn new() -> Self {
        Self {
            pml4: PageBox::new(PageTable::new()),
        }
    }

    fn create(mut self) -> PageBox<PageTable> {
        self.enable_recursive_mapping();
        self.map_kernel_regions();
        self.pml4
    }

    fn enable_recursive_mapping(&mut self) {
        let a = self.pml4.phys_addr();
        self.pml4[511].set_addr(a, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
    }

    fn map_kernel_regions(&mut self) {
        // Kernel region starts from `0xffff_ffff_8000_0000`.
        let p3 = PML4.lock().level_4_table()[510].addr();
        self.pml4[510].set_addr(p3, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
    }
}
