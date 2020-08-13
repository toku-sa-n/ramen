use crate::common_items::addr::{Addr, Virt};
use crate::common_items::size::{Byte, Size};
use core::ptr;
use uefi::proto::media::file;

struct KernelHeader {
    _entry_addr: Addr<Virt>,
    memory_bytes: Size<Byte>,
}

impl KernelHeader {
    fn new_from_slice(slice: &[u8; 16]) -> Self {
        unsafe {
            Self {
                _entry_addr: ptr::read(slice.as_ptr() as *const _),
                memory_bytes: ptr::read(slice.as_ptr().offset(8) as *const _),
            }
        }
    }
}

pub fn get(root_dir: &mut file::Directory) -> Size<Byte> {
    let mut handler = super::get_kernel_handler(root_dir);

    let mut header = [0u8; 16];

    handler
        .read(&mut header)
        .expect("Failed to read kernel header")
        .unwrap();

    let header = KernelHeader::new_from_slice(&header);

    header.memory_bytes
}
