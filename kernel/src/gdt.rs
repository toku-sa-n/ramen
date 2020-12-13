// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    tss::TSS,
    x86_64::{
        instructions::{segmentation, tables},
        structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        PrivilegeLevel,
    },
};
use conquer_once::spin::Lazy;

pub static GDT: Lazy<Gdt> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let kernel_code = gdt.add_entry(Descriptor::kernel_code_segment());
    let user_code = gdt.add_entry(Descriptor::user_code_segment());
    let user_data = gdt.add_entry(Descriptor::user_data_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

    Gdt {
        table: gdt,
        kernel_code,
        tss_selector,
        user_code,
        user_data,
    }
});

pub struct Gdt {
    table: GlobalDescriptorTable,
    kernel_code: SegmentSelector,
    tss_selector: SegmentSelector,
    user_code: SegmentSelector,
    user_data: SegmentSelector,
}

pub fn init() {
    GDT.table.load();
    unsafe {
        segmentation::set_cs(GDT.kernel_code);

        let null_seg = SegmentSelector::new(0, PrivilegeLevel::Ring0);
        segmentation::load_ds(null_seg);
        segmentation::load_es(null_seg);
        segmentation::load_fs(null_seg);
        segmentation::load_gs(null_seg);
        segmentation::load_ss(null_seg);
        tables::load_tss(GDT.tss_selector);
    }
}
