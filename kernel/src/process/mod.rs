mod context;
pub(crate) mod ipc;
mod page_table;
mod pid;
pub(crate) mod scheduler;
mod stack_frame;

use {
    self::context::Context,
    crate::{
        mem,
        mem::{allocator::kpbox::KpBox, paging},
    },
    alloc::collections::VecDeque,
    core::{cell::UnsafeCell, convert::TryInto},
    os_units::Bytes,
    stack_frame::StackFrame,
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
    scheduler::add(p);
}

pub(super) fn loader(f: fn() -> !) -> ! {
    f();
}

#[derive(Debug)]
pub(crate) struct Process {
    id: Pid,
    pml4: KpBox<PageTable>,
    stack_frame: KpBox<StackFrame>,

    msg_ptr: Option<PhysAddr>,

    context: Context,

    send_to: Option<Pid>,
    receive_from: Option<ReceiveFrom>,

    pids_try_to_send_this_process: VecDeque<Pid>,

    name: &'static str,
}
impl Process {
    #[allow(clippy::too_many_lines)]
    fn new(entry: VirtAddr, name: &'static str) -> Self {
        let pml4 = Self::generate_pml4();

        let mut tables = page_table::Collection::default();
        let stack = Self::generate_kernel_stack();
        let stack_bottom = stack.virt_addr() + stack.bytes().as_usize();
        let stack_frame = KpBox::from(StackFrame::kernel(entry, stack_bottom));
        tables.map_page_box(&stack);
        tables.map_page_box(&stack_frame);

        let context = Context::kernel(entry, tables.pml4_frame(), stack_bottom - 8_u64);

        Process {
            id: pid::generate(),
            pml4,
            stack_frame,

            context,

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

        let handler = crate::fs::get_handler(name);
        let raw = handler.content();

        unsafe {
            switch_pml4_do(&pml4, || mem::elf::map_to_current_address_space(raw)).unwrap();
        }

        let mut tables = page_table::Collection::default();
        let (_, entry) = tables.map_elf(name);

        let stack = Self::generate_kernel_stack();
        let stack_bottom = stack.virt_addr() + stack.bytes().as_usize();
        let stack_frame = KpBox::from(StackFrame::user(entry, stack_bottom));

        let context = Context::user(entry, tables.pml4_frame(), stack_bottom - 8_u64);

        Self {
            id: pid::generate(),
            pml4,
            stack_frame,

            context,

            msg_ptr: None,

            send_to: None,
            receive_from: None,

            pids_try_to_send_this_process: VecDeque::new(),
            name,
        }
    }

    fn id(&self) -> Pid {
        self.id
    }

    fn stack_frame_top_addr(&self) -> VirtAddr {
        self.stack_frame.virt_addr()
    }

    fn stack_frame_bottom_addr(&self) -> VirtAddr {
        let b = self.stack_frame.bytes();
        self.stack_frame_top_addr() + b.as_usize()
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

unsafe fn switch_pml4_do<T>(pml4: &KpBox<PageTable>, f: impl FnOnce() -> T) -> T {
    let (old_pml4, flags) = Cr3::read();

    let frame = PhysFrame::from_start_address(pml4.phys_addr());
    let frame = frame.expect("The address is not page-aligned.");

    unsafe {
        Cr3::write(frame, flags);
    }

    let r = f();

    unsafe {
        Cr3::write(old_pml4, flags);
    }

    r
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) enum ReceiveFrom {
    Any,
    Id(Pid),
}
