#![no_std]

pub mod mem;
pub mod vram;

use {
    core::ptr,
    mem::MemoryDescriptor,
    predefined_mmap::INIT_RSP,
    x86_64::{PhysAddr, VirtAddr},
};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Info {
    entry_addr: VirtAddr,
    vram_info: vram::Info,
    mem_map: mem::Map,
    rsdp: PhysAddr,
}

impl Info {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        entry_addr: VirtAddr,
        vram_info: vram::Info,
        mem_map: mem::Map,
        rsdp: PhysAddr,
    ) -> Self {
        Self {
            entry_addr,
            vram_info,
            mem_map,
            rsdp,
        }
    }

    #[must_use]
    pub fn entry_addr(&self) -> VirtAddr {
        self.entry_addr
    }

    #[must_use]
    pub fn vram(&self) -> vram::Info {
        self.vram_info
    }

    #[must_use]
    pub fn rsdp(&self) -> PhysAddr {
        self.rsdp
    }

    pub fn set(self) {
        unsafe {
            ptr::write(INIT_RSP.as_mut_ptr::<Self>(), self);
        }
    }

    #[must_use]
    pub fn get() -> Self {
        unsafe { ptr::read(INIT_RSP.as_mut_ptr::<Self>()) }
    }

    #[must_use]
    pub fn mem_map_mut(&mut self) -> &mut [MemoryDescriptor] {
        self.mem_map.as_mut_slice()
    }
}
