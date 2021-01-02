// SPDX-License-Identifier: GPL-3.0-or-later

use core::{mem, ptr, str};
use x86_64::VirtAddr;

use crate::mem::allocator::page_box::PageBox;

pub struct Ustar {
    addr: VirtAddr,
}
impl Ustar {
    /// Safety: `addr` must be the valid address to the data of ustar.
    pub unsafe fn new(addr: VirtAddr) -> Self {
        Self { addr }
    }

    pub fn list(&self) {
        for m in self.iter() {
            info!("{}", m.name());
        }
    }

    pub fn content(&self, name: &str) -> Option<PageBox<[u8]>> {
        let m = self.iter().find(|&m| m.name() == name)?;
        let m_ptr = m as *const Meta;
        let cont_ptr = (m_ptr as usize + mem::size_of::<Meta>()) as *const u8;

        let b = PageBox::user_slice(0, m.filesize_as_dec());

        // SAFETY: Both the source and the destination pointers are valid and aligned.
        unsafe {
            ptr::copy(
                cont_ptr,
                b.virt_addr().as_mut_ptr() as *mut u8,
                m.filesize_as_dec(),
            )
        }
        Some(b)
    }

    fn iter(&self) -> impl Iterator<Item = &'static Meta> {
        // SAFETY: `Ustar::new` ensures `self.addr` is a valid address to the head of ustar data.
        unsafe { Iter::new(self.addr) }
    }
}

struct Iter {
    p: VirtAddr,
}
impl Iter {
    /// Safety: `p` must be a valid address to the head of ustar data.
    unsafe fn new(p: VirtAddr) -> Self {
        Self { p }
    }

    fn correct_magic_number(&self) -> bool {
        // SAFETY: This operation is safe as `self.p` is a valid address.
        unsafe {
            ptr::read_unaligned((self.p + 257_u64).as_ptr() as *const [u8; 5])
                == *"ustar".as_bytes()
        }
    }
}
impl Iterator for Iter {
    type Item = &'static Meta;

    fn next(&mut self) -> Option<Self::Item> {
        if self.correct_magic_number() {
            let meta: &Meta = unsafe { &*self.p.as_ptr() };
            self.p += (((meta.filesize_as_dec() + 511) / 512) + 1) * 512;
            Some(meta)
        } else {
            None
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
    fn name(&self) -> &str {
        str::from_utf8(&self.name).unwrap().trim_matches('\0')
    }

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
