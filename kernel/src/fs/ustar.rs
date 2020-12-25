// SPDX-License-Identifier: GPL-3.0-or-later

use core::{ptr, str};
use x86_64::VirtAddr;

pub struct UStar {
    addr: VirtAddr,
}
impl UStar {
    /// Safety: `addr` must be the valid address to the data of UStar.
    pub unsafe fn new(addr: VirtAddr) -> Self {
        Self { addr }
    }

    pub fn list(&self) {
        let mut p = self.addr;
        while unsafe {
            ptr::read_unaligned((p + 257_u64).as_ptr() as *const [u8; 5]) == *"ustar".as_bytes()
        } {
            let meta: Meta = unsafe { ptr::read_unaligned(p.as_ptr()) };
            info!("{}", str::from_utf8(&meta.name).unwrap());
            p += (((meta.filesize_as_dec() + 511) / 512) + 1) * 512;
        }
    }
}

#[repr(C, packed)]
struct Meta {
    name: [u8; 100],
    mode: [u8; 8],
    owner: [u8; 8],
    group: [u8; 8],
    size: [u8; 12],
    modified_time: [u8; 12],
    checksum: [u8; 8],
    ty: [u8; 1],
    linked_file_name: [u8; 100],
    magic: [u8; 6],
    version: [u8; 2],
    owner_name: [u8; 32],
    group_name: [u8; 32],
    device_major_number: [u8; 8],
    device_minor_number: [u8; 8],
    filename_prefix: [u8; 155],
    _rsvd: [u8; 12],
}
impl Meta {
    fn filesize_as_dec(&self) -> usize {
        let mut sz: usize = 0;

        // The last byte of `size` field is always 0 (u8), not 0 (char).
        for d in 0..self.size.len() - 1 {
            sz *= 8;
            sz += usize::from(self.size[d] - b'0');
        }
        sz
    }
}
