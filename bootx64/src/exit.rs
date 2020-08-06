use crate::common_items;
use crate::mem::paging;
use core::mem::size_of;
use core::ptr;
use uefi::table::boot;

const INIT_RSP: usize = 0xffff_ffff_800a_1000 - size_of::<common_items::BootInfo>();

pub fn bootx64<'a>(
    mem_map: &'a mut [boot::MemoryDescriptor],
    boot_info: common_items::BootInfo,
) -> ! {
    disable_interruption();

    paging::init(mem_map);
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
    save_boot_info(boot_info);

    unsafe {
        asm!("mov rsp, rax
        jmp rdi",in("rax") INIT_RSP,in("rdi") fetch_entry_address(),options(nomem, preserves_flags, nostack,noreturn));
    }
}

fn save_boot_info(boot_info: common_items::BootInfo) -> () {
    unsafe { ptr::write(INIT_RSP as *mut common_items::BootInfo, boot_info) }
}

fn fetch_entry_address() -> u64 {
    unsafe { ptr::read(0xffff_ffff_8000_0000 as *const u64) }
}
