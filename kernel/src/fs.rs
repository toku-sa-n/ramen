// SPDX-License-Identifier: GPL-3.0-or-later

use {
    core::{
        convert::{TryFrom, TryInto},
        mem, slice, str,
    },
    cstr_core::CStr,
    log::info,
    x86_64::VirtAddr,
};

pub(super) fn list_names() {
    for c in iter() {
        info!(
            "Name: {}, file size: {}, name size: {}",
            c.name(),
            c.header().file_size(),
            c.header().name_size()
        );
    }
}

pub(super) fn get_handler(name: &str) -> CpioArchievedFile {
    iter().find(|x| x.name() == name).expect("No such file.")
}

fn iter() -> impl Iterator<Item = CpioArchievedFile> {
    Iter::default()
}

fn initrd_addr() -> VirtAddr {
    extern "C" {
        static initrd: usize;
    }

    let a: *const usize = unsafe { &initrd };
    VirtAddr::new(a as _)
}

pub(super) struct CpioArchievedFile {
    ptr: VirtAddr,
}
impl CpioArchievedFile {
    pub(super) fn content(&self) -> &[u8] {
        let p = self.content_start().as_ptr();
        let sz: usize = self.header().file_size().try_into().unwrap();
        unsafe { slice::from_raw_parts(p, sz) }
    }

    unsafe fn new(ptr: VirtAddr) -> Self {
        assert_eq!(
            &ptr.as_ptr::<[u8; 6]>().read(),
            b"070707",
            "Invalid signature."
        );

        Self { ptr }
    }

    fn header(&self) -> CpioHeader {
        // SAFETY: The caller of the `new` method ensures that `self.header` is the correct
        // address.
        unsafe { self.ptr.as_ptr::<CpioHeader>().read() }
    }

    fn name(&self) -> &str {
        unsafe {
            let s = CStr::from_ptr(self.name_start().as_ptr()).to_str();
            s.expect("Failed to get the name of a file.")
        }
    }

    fn content_start(&self) -> VirtAddr {
        self.name_start() + usize::from(self.header().name_size())
    }

    fn name_start(&self) -> VirtAddr {
        self.ptr + mem::size_of::<CpioHeader>()
    }
}

struct Iter {
    ptr: VirtAddr,
}
impl Iterator for Iter {
    type Item = CpioArchievedFile;

    fn next(&mut self) -> Option<Self::Item> {
        let f = unsafe { CpioArchievedFile::new(self.ptr) };

        if f.name() == "TRAILER!!!" {
            None
        } else {
            self.ptr += mem::size_of::<CpioHeader>()
                + usize::try_from(u32::from(f.header().name_size()) + f.header().file_size())
                    .unwrap();
            Some(f)
        }
    }
}
impl Default for Iter {
    fn default() -> Self {
        Self { ptr: initrd_addr() }
    }
}

macro_rules! byte_array_to_str {
    ($slice:expr,$ty:ident) => {
        let s = str::from_utf8($slice).expect("Not the valid UTF-8");
        return $ty::from_str_radix(s, 8).expect("Radix is out of range.");
    };
}

#[repr(C, packed)]
struct CpioHeader {
    magic: [u8; 6],
    dev: [u8; 6],
    ino: [u8; 6],
    mode: [u8; 6],
    uid: [u8; 6],
    gid: [u8; 6],
    nlink: [u8; 6],
    rdev: [u8; 6],
    mtime: [u8; 11],
    namesize: [u8; 6],
    filesize: [u8; 11],
}
impl CpioHeader {
    fn name_size(&self) -> u16 {
        byte_array_to_str!(&self.namesize, u16);
    }

    fn file_size(&self) -> u32 {
        byte_array_to_str!(&self.filesize, u32);
    }
}
