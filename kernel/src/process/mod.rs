mod context;
pub(crate) mod ipc;
mod page_table;
mod pid;
mod priority;
mod receive_from;
pub(crate) mod scheduler;

use {
    self::{
        context::Context,
        priority::{Priority, LEAST_PRIORITY},
        receive_from::ReceiveFrom,
    },
    crate::{
        mem,
        mem::{
            allocator::{allocate_pages_for_user, kpbox::KpBox},
            paging,
        },
    },
    alloc::collections::VecDeque,
    core::{cell::UnsafeCell, convert::TryInto},
    os_units::{Bytes, NumOfPages},
    static_assertions::const_assert,
    x86_64::{
        registers::control::Cr3,
        structures::paging::{PageSize, PageTable, PageTableFlags, PhysFrame, Size4KiB},
        PhysAddr, VirtAddr,
    },
};
pub(crate) use {pid::Pid, scheduler::switch};

// No truncation from u64 to usize on the x86_64 platform.
#[allow(clippy::cast_possible_truncation)]
const STACK_SIZE: usize = Size4KiB::SIZE as usize * 4;
const STACK_GUARD_SIZE: Bytes = Bytes::new(4096);
const STACK_MAGIC: &str = "Oh god! What a man!";

const_assert!(STACK_GUARD_SIZE.as_usize() + STACK_MAGIC.as_bytes().len() <= STACK_SIZE);

pub(super) fn from_function(entry: fn(), name: &'static str) {
    let entry = VirtAddr::new((entry as usize).try_into().unwrap());
    push_process_to_queue(Process::new(entry, name));
}

pub(super) fn binary(name: &'static str) {
    push_process_to_queue(Process::binary(name));
}

fn push_process_to_queue(p: Process) {
    let pid = p.id();

    scheduler::push(pid);
    scheduler::add_process_as_runnable(p);
}

#[derive(Debug)]
pub(crate) struct Process {
    pid: Pid,
    pml4: KpBox<PageTable>,
    context: Context,
    kernel_stack: KpBox<UnsafeCell<[u8; STACK_SIZE]>>,
    priority: Priority,
    msg_ptr: Option<PhysAddr>,
    send_to: Option<Pid>,
    receive_from: Option<ReceiveFrom>,
    pids_try_to_send_this_process: VecDeque<Pid>,
    name: &'static str,
}
impl Process {
    fn idle() -> Self {
        Self {
            pid: 0,
            pml4: Self::generate_pml4(),
            context: Context::default(),
            kernel_stack: Self::generate_kernel_stack(),
            priority: LEAST_PRIORITY,
            msg_ptr: None,
            send_to: None,
            receive_from: None,
            pids_try_to_send_this_process: VecDeque::new(),
            name: "idle",
        }
    }

    #[allow(clippy::too_many_lines)]
    fn new(entry: VirtAddr, name: &'static str) -> Self {
        let pml4 = Self::generate_pml4();

        let pml4_frame = PhysFrame::from_start_address(pml4.phys_addr());
        let pml4_frame = pml4_frame.expect("PML4 is not page-aligned.");

        let kernel_stack = Self::generate_kernel_stack();

        let stack_bottom = kernel_stack.virt_addr() + kernel_stack.bytes().as_usize();

        let context = Context::kernel(entry, pml4_frame, stack_bottom - 8_u64);

        Process {
            pid: pid::generate(),
            pml4,

            context,
            kernel_stack,
            priority: Priority::new(0),

            msg_ptr: None,

            send_to: None,
            receive_from: None,

            pids_try_to_send_this_process: VecDeque::new(),
            name,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn binary(name: &'static str) -> Self {
        let pml4 = Self::generate_pml4();

        let pml4_frame = PhysFrame::from_start_address(pml4.phys_addr());
        let pml4_frame = pml4_frame.expect("PML4 is not page-aligned.");

        let handler = crate::fs::get_handler(name);
        let raw = handler.content();

        let kernel_stack = Self::generate_kernel_stack();

        unsafe {
            switch_pml4_do(pml4_frame, || {
                let entry = mem::elf::map_to_current_address_space(raw).unwrap();

                let stack_size = NumOfPages::<Size4KiB>::new(5);

                let stack_top = allocate_pages_for_user(NumOfPages::new(5)).unwrap();

                let context = Context::user(
                    entry,
                    pml4_frame,
                    stack_top + stack_size.as_bytes().as_usize() - 8_u64,
                );

                Self {
                    pid: pid::generate(),
                    pml4,

                    context,
                    kernel_stack,
                    priority: Priority::new(0),

                    msg_ptr: None,

                    send_to: None,
                    receive_from: None,

                    pids_try_to_send_this_process: VecDeque::new(),
                    name,
                }
            })
        }
    }

    fn id(&self) -> Pid {
        self.pid
    }

    fn generate_pml4() -> KpBox<PageTable> {
        let mut pml4 = KpBox::<PageTable>::default();

        for i in 0..510 {
            pml4[i].set_unused();
        }

        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        let addr = pml4.phys_addr();

        pml4[510].set_addr(addr, flags);
        pml4[511] = paging::level_4_table()[511].clone();

        pml4
    }

    fn generate_kernel_stack() -> KpBox<UnsafeCell<[u8; STACK_SIZE]>> {
        let mut stack = KpBox::from(UnsafeCell::from([0; STACK_SIZE]));

        for (i, c) in STACK_MAGIC.as_bytes().iter().enumerate() {
            stack.get_mut()[STACK_GUARD_SIZE.as_usize() + i] = *c;
        }

        stack
    }
}

unsafe fn switch_pml4_do<T>(pml4: PhysFrame, f: impl FnOnce() -> T) -> T {
    let (old_pml4, flags) = Cr3::read();

    unsafe {
        Cr3::write(pml4, flags);
    }

    let r = f();

    unsafe {
        Cr3::write(old_pml4, flags);
    }

    r
}
