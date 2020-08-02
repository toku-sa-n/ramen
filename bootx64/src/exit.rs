use crate::gop;
use crate::mem::paging;
use core::mem::size_of;
use core::ptr;
use uefi::table::boot;

pub struct BootInfo {
    _vram_info: gop::VramInfo,
}

impl BootInfo {
    pub fn new(_vram_info: gop::VramInfo) -> Self {
        Self { _vram_info }
    }
}

const INIT_RSP: usize = 0xffff_ffff_800a_1000 - size_of::<BootInfo>();

pub fn bootx64<'a>(mem_map: &'a mut [boot::MemoryDescriptor], boot_info: BootInfo) -> ! {
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

fn jump_to_kernel(boot_info: BootInfo) -> ! {
    save_boot_info(boot_info);

    unsafe {
        asm!("mov rsp, rax
        jmp rdi",in("rax") INIT_RSP,in("rdi") fetch_entry_address(),options(nomem, preserves_flags, nostack,noreturn));
    }
}

fn save_boot_info(boot_info: BootInfo) -> () {
    unsafe { ptr::write(INIT_RSP as *mut BootInfo, boot_info) }
}

fn fetch_entry_address() -> u64 {
    unsafe { ptr::read(0xffff_ffff_8000_0000 as *const u64) }
}
