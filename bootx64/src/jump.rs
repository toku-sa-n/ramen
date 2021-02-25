// SPDX-License-Identifier: GPL-3.0-or-later

use common::{constant::INIT_RSP, kernelboot};

macro_rules! change_rsp{
    ($val:expr)=>{
        unsafe{
            asm!("mov rsp, {:r}",in(reg) $val,options(nomem,preserves_flags,nostack));
        }
    }
}

pub fn to_kernel(boot_info: kernelboot::Info) -> ! {
    disable_interruption();

    boot_info.set();

    change_rsp!(INIT_RSP.as_u64());

    let boot_info = kernelboot::Info::get();

    let kernel = unsafe {
        core::mem::transmute::<u64, fn(kernelboot::Info) -> !>(boot_info.entry_addr().as_u64())
    };

    kernel(boot_info)
}

fn disable_interruption() {
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
