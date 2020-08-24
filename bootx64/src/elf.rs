use common_items::size::{Byte, Size};
use elf_rs::Elf;

pub fn sum_all_memory_region(elf: &Elf) -> Size<Byte> {
    match elf {
        Elf::Elf64(elf) => elf
            .program_header_iter()
            .fold(Size::new(0), |acc, header| acc + header.ph.memsz()),
        Elf::Elf32(_) => panic!("32bit kernel is not supported"),
    }
}
