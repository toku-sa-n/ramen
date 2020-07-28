use crate::gop;
use crate::memory;
use core::mem;
use core::ptr;
use uefi::table::boot;

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

pub struct BootInfo {
    _vram_info: gop::VramInfo,
}

impl BootInfo {
    pub fn new(_vram_info: gop::VramInfo) -> Self {
        Self { _vram_info }
    }
}

const INIT_RSP: usize = 0xffff_ffff_800a_1000 - mem::size_of::<BootInfo>();

fn save_boot_info(boot_info: BootInfo) -> () {
    unsafe { ptr::write(INIT_RSP as *mut BootInfo, boot_info) }
}

fn jump_to_kernel(boot_info: BootInfo) -> ! {
    save_boot_info(boot_info);

    const ADDR_OF_KERNEL: usize = 0xffff_ffff_8000_0000;

    unsafe {
        asm!("mov rsp, rax
        jmp rdi",in("rax") INIT_RSP,in("rdi") ADDR_OF_KERNEL,options(nomem, preserves_flags, nostack,noreturn));
    }
}

pub fn bootx64<'a>(mem_map: &'a mut [boot::MemoryDescriptor], boot_info: BootInfo) -> ! {
    disable_interruption();

    memory::init_paging(mem_map);
    jump_to_kernel(boot_info);
}
