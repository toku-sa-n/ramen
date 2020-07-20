use crate::asm;
use core::mem;

#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    type_s_dpl_p: u8,
    limit_high_avl_l_db_g: u8,
    base_high: u8,
}

impl GdtEntry {
    const fn new(
        limit_low: u16,
        base_low: u16,
        base_mid: u8,
        type_s_dpl_p: u8,
        limit_high_avl_l_db_g: u8,
        base_high: u8,
    ) -> Self {
        Self {
            limit_low,
            base_low,
            base_mid,
            type_s_dpl_p,
            limit_high_avl_l_db_g,
            base_high,
        }
    }
}

#[repr(C, packed)]
struct Gdt {
    entries: [GdtEntry; Gdt::NUM_OF_ENTRIES_OF_GDT],
}

impl Gdt {
    const NUM_OF_ENTRIES_OF_GDT: usize = 3;

    const fn new(entries: [GdtEntry; Gdt::NUM_OF_ENTRIES_OF_GDT]) -> Self {
        Self { entries }
    }

    fn offset_of_code_segment(&self) -> u16 {
        0x10
    }

    fn offset_of_data_segment(&self) -> u16 {
        0x08
    }

    fn as_ptr(&self) -> *const Self {
        self as *const _
    }

    fn get_limit(&self) -> u16 {
        Self::NUM_OF_ENTRIES_OF_GDT as u16 * mem::size_of::<GdtEntry>() as u16 - 1
    }
}

static GDT: Gdt = Gdt::new([
    GdtEntry::new(0, 0, 0, 0, 0, 0),            // Null segment
    GdtEntry::new(0xFFFF, 0, 0, 0x92, 0xCF, 0), // Data segment
    GdtEntry::new(0xFFFF, 0, 0, 0x9A, 0xAF, 0), // Code segment
]);

pub fn init() -> () {
    unsafe {
        asm::lgdt(GDT.get_limit(), GDT.as_ptr() as u64);
        asm::set_code_segment(GDT.offset_of_code_segment());
        asm::set_data_segment(GDT.offset_of_data_segment());
    }
}
