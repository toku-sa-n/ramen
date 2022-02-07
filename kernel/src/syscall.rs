use {
    crate::{
        gdt,
        mem::{allocator, paging},
        process::{self, Pid},
    },
    core::{arch::asm, convert::TryInto, ffi::c_void, panic::PanicInfo, slice},
    num_traits::FromPrimitive,
    os_units::{Bytes, NumOfPages},
    terminal::print,
    x86_64::{
        registers::{
            model_specific::{Efer, EferFlags, LStar, Msr, Star},
            rflags::RFlags,
        },
        structures::paging::Size4KiB,
        PhysAddr, VirtAddr,
    },
};

const IA32_FMASK: Msr = Msr::new(0xc000_0084);

pub(super) fn init() {
    register_handler();

    register_segments_with_star();

    // SAFETY: `register_handler` registers a system call handler and `register_segments_with_start`
    // registers segment selectors.
    unsafe {
        enable_syscall_and_sysret();
    }

    disable_interrupts_on_syscall();
}

/// # Safety
///
/// The caller must ensure that the correct system call handler is registered with the LSTAR
/// register and segment selectors with STAR.
unsafe fn enable_syscall_and_sysret() {
    // SAFETY: The caller must register proper system call handler and segment selectors.
    unsafe {
        Efer::update(|efer| efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS));
    }
}

fn register_handler() {
    LStar::write(VirtAddr::new(
        (prepare_syscall as usize).try_into().unwrap(),
    ));
}

fn register_segments_with_star() {
    let r = Star::write(
        gdt::user_code_selector(),
        gdt::user_data_selector(),
        gdt::kernel_code_selector(),
        gdt::kernel_data_selector(),
    );
    r.expect("Failed to register segment registers with STAR.");
}

fn disable_interrupts_on_syscall() {
    // SAFETY: Disabling interrupts on a system call does not violate memory safety.
    unsafe {
        update_ia32_fmask(|mask| mask.insert(RFlags::INTERRUPT_FLAG));
    }
}

/// # Safety
///
/// See: [`x86_64::registers::rflags::write`].
unsafe fn update_ia32_fmask(f: impl FnOnce(&mut RFlags)) {
    let mut mask = read_ia32_fmask();

    f(&mut mask);

    // SAFETY: The caller must uphold the safety requirements.
    unsafe {
        write_ia32_fmask(mask);
    }
}

fn read_ia32_fmask() -> RFlags {
    // SAFETY: Reading from IA32_FMASK does not violate memory safety.
    let mask = unsafe { IA32_FMASK.read() };
    let mask = RFlags::from_bits(mask);
    mask.expect("Invalid rflags.")
}

/// # Safety
///
/// See [`x86_64::registers::rflag::write`].
unsafe fn write_ia32_fmask(mask: RFlags) {
    // SAFETY: The caller must uphold the safety requirements.
    unsafe {
        let mut reg = IA32_FMASK;

        reg.write(mask.bits());
    }
}

#[naked]
unsafe extern "sysv64" fn prepare_syscall() {
    unsafe {
        asm!(
            "
            push rcx
            push r11

            push rbp
            mov rbp, rsp

            push rdi
            push rsi
            push rdx
            push rax

            call current_kernel_stack_bottom

            mov rcx, rax

            pop rax
            pop rdx
            pop rsi
            pop rdi

            mov rsp, rcx

            mov rcx, rdx
            mov rdx, rsi
            mov rsi, rdi
            mov rdi, rax

            call select_proper_syscall

            mov rsp, rbp
            pop rbp

            pop r11
            pop rcx

            sysretq",
            options(noreturn)
        );
    }
}

#[no_mangle]
#[allow(clippy::too_many_arguments)]
unsafe extern "sysv64" fn select_proper_syscall(idx: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    if let Some(t) = FromPrimitive::from_u64(idx) {
        // SAFETY: At least the index is correct. The caller must ensure that
        // the all arguments are correctly passed.
        unsafe { select_proper_syscall_unchecked(t, a1, a2, a3) }
    } else {
        panic!("Unrecognized system call index: {}", idx)
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
unsafe fn select_proper_syscall_unchecked(ty: syscalls::Ty, a1: u64, a2: u64, a3: u64) -> u64 {
    match ty {
        syscalls::Ty::AllocatePages => {
            sys_allocate_pages(NumOfPages::new(a1.try_into().unwrap())).as_u64()
        }
        syscalls::Ty::DeallocatePages => {
            sys_deallocate_pages(VirtAddr::new(a1), NumOfPages::new(a2.try_into().unwrap()))
        }
        syscalls::Ty::MapPages => {
            sys_map_pages(PhysAddr::new(a1), Bytes::new(a2.try_into().unwrap())).as_u64()
        }
        syscalls::Ty::UnmapPages => {
            sys_unmap_pages(VirtAddr::new(a1), Bytes::new(a2.try_into().unwrap()))
        }
        syscalls::Ty::TranslateAddress => sys_translate_address(VirtAddr::new(a1)).as_u64(),
        // SAFETY: The caller must ensure that `a2` is the correct pointer to the string.
        syscalls::Ty::Write => unsafe {
            sys_write(
                a1.try_into().unwrap(),
                a2 as *const _,
                a3.try_into().unwrap(),
            )
            .try_into()
            .unwrap()
        },
        syscalls::Ty::Send => sys_send(VirtAddr::new(a1), a2.try_into().unwrap()),
        syscalls::Ty::ReceiveFromAny => sys_receive_from_any(VirtAddr::new(a1)),
        syscalls::Ty::ReceiveFrom => sys_receive_from(VirtAddr::new(a1), a2.try_into().unwrap()),
        // SAFETY: The caller must ensure that `a1` is the correct pointer to the panic
        // information.
        syscalls::Ty::Panic => unsafe { sys_panic(a1 as *const PanicInfo<'_>) },
        _ => unreachable!("This sytem call should not be handled by the kernel itself."),
    }
}

fn sys_allocate_pages(num_of_pages: NumOfPages<Size4KiB>) -> VirtAddr {
    allocator::allocate_pages_for_user(num_of_pages).unwrap_or_else(VirtAddr::zero)
}

fn sys_deallocate_pages(virt: VirtAddr, pages: NumOfPages<Size4KiB>) -> u64 {
    allocator::deallocate_pages(virt, pages);
    0
}

fn sys_map_pages(start: PhysAddr, bytes: Bytes) -> VirtAddr {
    crate::mem::map_pages_for_user(start, bytes)
}

fn sys_unmap_pages(start: VirtAddr, bytes: Bytes) -> u64 {
    crate::mem::unmap_pages(start, bytes);
    0
}

fn sys_translate_address(v: VirtAddr) -> PhysAddr {
    paging::translate_addr(v).unwrap_or_else(PhysAddr::zero)
}

/// # Safety
///
/// `buf` must be valid.
unsafe fn sys_write(fildes: i32, buf: *const c_void, nbyte: u32) -> i32 {
    if fildes == 1 {
        let buf: *const u8 = buf.cast();

        // SAFETY: The caller ensures that `buf` is valid.
        let s = unsafe { slice::from_raw_parts(buf, nbyte.try_into().unwrap()) };
        let s = core::str::from_utf8(s);

        if let Ok(s) = s {
            print!("{}", s);

            nbyte.try_into().unwrap()
        } else {
            0
        }
    } else {
        unimplemented!("Not stdout");
    }
}

fn sys_send(m: VirtAddr, to: Pid) -> u64 {
    process::ipc::send(m, to);
    0
}

fn sys_receive_from_any(m: VirtAddr) -> u64 {
    process::ipc::receive_from_any(m);
    0
}

fn sys_receive_from(m: VirtAddr, from: Pid) -> u64 {
    process::ipc::receive_from(m, from);
    0
}

unsafe fn sys_panic(i: *const PanicInfo<'_>) -> ! {
    let name = process::scheduler::current_process_name();

    // SAFETY: The caller must ensure that `i` is the correct pointer to the panic information.
    panic!("The process {} paniced: {}", name, unsafe { &*i });
}
