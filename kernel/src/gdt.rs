// SPDX-License-Identifier: GPL-3.0-or-later
use crate::x86_64::instructions::segmentation;
use crate::x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use crate::x86_64::PrivilegeLevel;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref GDT: Gdt = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());

        Gdt::new(gdt, code_selector)
    };
}

pub struct Gdt {
    table: GlobalDescriptorTable,
    code_selector: SegmentSelector,
}

impl Gdt {
    fn new(table: GlobalDescriptorTable, code_selector: SegmentSelector) -> Self {
        Self {
            table,
            code_selector,
        }
    }
}

pub fn init() -> () {
    GDT.table.load();
    unsafe {
        segmentation::set_cs(GDT.code_selector);

        let null_seg = SegmentSelector::new(0, PrivilegeLevel::Ring0);
        segmentation::load_ds(null_seg);
        segmentation::load_es(null_seg);
        segmentation::load_fs(null_seg);
        segmentation::load_gs(null_seg);
        segmentation::load_ss(null_seg);
    }
}
