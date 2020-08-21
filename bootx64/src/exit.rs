use crate::common_items;
use crate::common_items::constant::KERNEL_ADDR;
use crate::mem::paging;
use crate::x86_64::addr::VirtAddr;
use core::ptr;
use uefi::table::boot;

pub fn bootx64<'a>(
    mem_map: &'a mut [boot::MemoryDescriptor],
    boot_info: common_items::BootInfo,
) -> ! {
    disable_interruption();

    paging::init(mem_map, &boot_info.vram());
    jump_to_kernel(boot_info);
}

fn disable_interruption() -> () {
    // Use `nop` because some machines go wrong when continuously doing `out`.
    unsafe {
        asm!(
            "mov al,0xff
            out 0x21,al
            nop
            out 0xa1,al
            cli"
        );
    }
}

fn jump_to_kernel(boot_info: common_items::BootInfo) -> ! {
    boot_info.set();

    unsafe {
        asm!("mov rsp, rax
        jmp rdi",in("rax") common_items::INIT_RSP,in("rdi") fetch_entry_address().as_u64(),options(nomem, preserves_flags, nostack,noreturn));
    }
}

fn fetch_entry_address() -> VirtAddr {
    VirtAddr::new(unsafe { ptr::read(KERNEL_ADDR.as_ptr::<u64>()) })
}
