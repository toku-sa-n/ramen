// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        mem::{allocator, paging::pml4::PML4},
        process::{self, exit_process, SlotId},
    },
    core::{convert::TryInto, ffi::c_void, panic::PanicInfo, slice},
    num_traits::FromPrimitive,
    os_units::{Bytes, NumOfPages},
    terminal::print,
    x86_64::{
        structures::paging::{Size4KiB, Translate},
        PhysAddr, VirtAddr,
    },
};

#[no_mangle]
#[allow(clippy::too_many_arguments)]
unsafe extern "C" fn select_proper_syscall(idx: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    if let Some(t) = FromPrimitive::from_u64(idx) {
        select_proper_syscall_unchecked(t, a1, a2, a3)
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
        syscalls::Ty::Exit => sys_exit(),
        syscalls::Ty::TranslateAddress => sys_translate_address(VirtAddr::new(a1)).as_u64(),
        syscalls::Ty::Write => sys_write(
            a1.try_into().unwrap(),
            a2 as *const _,
            a3.try_into().unwrap(),
        )
        .try_into()
        .unwrap(),
        syscalls::Ty::Send => sys_send(VirtAddr::new(a1), a2.try_into().unwrap()),
        syscalls::Ty::ReceiveFromAny => sys_receive_from_any(VirtAddr::new(a1)),
        syscalls::Ty::ReceiveFrom => sys_receive_from(VirtAddr::new(a1), a2.try_into().unwrap()),
        syscalls::Ty::Panic => sys_panic(a1 as *const PanicInfo<'_>),
        _ => unreachable!("This sytem call should not be handled by the kernel itself."),
    }
}

fn sys_allocate_pages(num_of_pages: NumOfPages<Size4KiB>) -> VirtAddr {
    allocator::allocate_pages(num_of_pages).unwrap_or_else(VirtAddr::zero)
}

fn sys_deallocate_pages(virt: VirtAddr, pages: NumOfPages<Size4KiB>) -> u64 {
    allocator::deallocate_pages(virt, pages);
    0
}

fn sys_map_pages(start: PhysAddr, bytes: Bytes) -> VirtAddr {
    crate::mem::map_pages(start, bytes)
}

fn sys_unmap_pages(start: VirtAddr, bytes: Bytes) -> u64 {
    crate::mem::unmap_pages(start, bytes);
    0
}

fn sys_exit() -> ! {
    unsafe {
        asm!(
            "
            mov rsp, {}
            call {}
            "
            ,
            const 0xffff_ffff_c000_0000_u64 - (0x1000 * 16 / 2),
            sym exit_process,
            options(noreturn)
        )
    }
}

fn sys_translate_address(v: VirtAddr) -> PhysAddr {
    PML4.lock().translate_addr(v).unwrap_or_else(PhysAddr::zero)
}

/// # Safety
///
/// `buf` must be valid.
unsafe fn sys_write(fildes: i32, buf: *const c_void, nbyte: u32) -> i32 {
    if fildes == 1 {
        let buf: *const u8 = buf.cast();

        // SAFETY: The caller ensures that `buf` is valid.
        let s = slice::from_raw_parts(buf, nbyte.try_into().unwrap());
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

fn sys_send(m: VirtAddr, to: SlotId) -> u64 {
    process::ipc::send(m, to);
    0
}

fn sys_receive_from_any(m: VirtAddr) -> u64 {
    process::ipc::receive_from_any(m);
    0
}

fn sys_receive_from(m: VirtAddr, from: SlotId) -> u64 {
    process::ipc::receive_from(m, from);
    0
}

unsafe fn sys_panic(i: *const PanicInfo<'_>) -> ! {
    let name = process::current_name();

    panic!("The process {} paniced: {}", name, *i);
}
