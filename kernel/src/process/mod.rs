pub(crate) mod ipc;
mod page_table;
mod pid;
pub(crate) mod scheduler;
mod stack_frame;

use {
    crate::mem::allocator::kpbox::KpBox,
    alloc::collections::VecDeque,
    core::convert::TryInto,
    stack_frame::StackFrame,
    x86_64::{
        structures::paging::{PageSize, PageTable, PageTableFlags, PhysFrame, Size4KiB},
        PhysAddr, VirtAddr,
    },
};
pub(crate) use {
    pid::Pid,
    scheduler::{exit_process, switch},
};

pub(super) fn from_function(entry: fn(), name: &'static str) {
    let entry = VirtAddr::new((entry as usize).try_into().unwrap());
    push_process_to_queue(Process::new(entry, Privilege::Kernel, name));
}

pub(super) fn binary(name: &'static str, p: Privilege) {
    push_process_to_queue(Process::binary(name, p));
}

fn push_process_to_queue(p: Process) {
    let pid = p.id();

    scheduler::push(pid);
    scheduler::add(p);
}

pub(super) fn loader(f: fn()) -> ! {
    f();
    syscalls::exit();
}

#[derive(Debug)]
pub(crate) struct Process {
    id: Pid,
    _tables: page_table::Collection,
    pml4: PhysFrame,
    _stack: KpBox<[u8]>,
    stack_frame: KpBox<StackFrame>,
    _binary: Option<KpBox<[u8]>>,

    new_pml4: KpBox<PageTable>,

    msg_ptr: Option<PhysAddr>,

    send_to: Option<Pid>,
    receive_from: Option<ReceiveFrom>,

    pids_try_to_send_this_process: VecDeque<Pid>,

    name: &'static str,
}
impl Process {
    // No truncation from u64 to usize on the x86_64 platform.
    #[allow(clippy::cast_possible_truncation)]
    const STACK_SIZE: usize = Size4KiB::SIZE as usize * 4;

    #[allow(clippy::too_many_lines)]
    fn new(entry: VirtAddr, privilege: Privilege, name: &'static str) -> Self {
        let mut tables = page_table::Collection::default();
        let stack = KpBox::new_slice(0, Self::STACK_SIZE);
        let stack_bottom = stack.virt_addr() + stack.bytes().as_usize();
        let stack_frame = KpBox::from(match privilege {
            Privilege::Kernel => StackFrame::kernel(entry, stack_bottom),
            Privilege::User => StackFrame::user(entry, stack_bottom),
        });
        tables.map_page_box(&stack);
        tables.map_page_box(&stack_frame);

        let pml4 = tables.pml4_frame();

        let new_pml4 = Self::generate_pml4();

        Process {
            id: pid::generate(),
            _tables: tables,
            pml4,
            _stack: stack,
            stack_frame,
            _binary: None,

            new_pml4,

            msg_ptr: None,

            send_to: None,
            receive_from: None,

            pids_try_to_send_this_process: VecDeque::new(),
            name,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn binary(name: &'static str, privilege: Privilege) -> Self {
        let mut tables = page_table::Collection::default();
        let (content, entry) = tables.map_elf(name);

        let stack = KpBox::new_slice(0, Self::STACK_SIZE);
        let stack_bottom = stack.virt_addr() + stack.bytes().as_usize();
        let stack_frame = KpBox::from(match privilege {
            Privilege::Kernel => StackFrame::kernel(entry, stack_bottom),
            Privilege::User => StackFrame::user(entry, stack_bottom),
        });
        tables.map_page_box(&stack);
        tables.map_page_box(&stack_frame);

        let pml4 = tables.pml4_frame();

        let new_pml4 = Self::generate_pml4();

        Self {
            id: pid::generate(),
            _tables: tables,
            pml4,
            _stack: stack,
            stack_frame,
            _binary: Some(content),

            new_pml4,

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

        let frame = PhysFrame::from_start_address(pml4.phys_addr());
        let frame = frame.expect("Failed to generate a PML4.");

        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        pml4[510].set_frame(frame, flags);

        pml4
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Privilege {
    Kernel,
    User,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) enum ReceiveFrom {
    Any,
    Id(Pid),
}
