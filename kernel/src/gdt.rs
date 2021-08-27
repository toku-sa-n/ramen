// SPDX-License-Identifier: GPL-3.0-or-later

use crate::tss::TSS;
use conquer_once::spin::Lazy;
use x86_64::{
    instructions::{
        segmentation::{Segment, CS, DS, ES, FS, GS, SS},
        tables,
    },
    registers::model_specific::Star,
    structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
};

pub(crate) static GDT: Lazy<Gdt> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let kernel_code = gdt.add_entry(Descriptor::kernel_code_segment());
    let kernel_data = gdt.add_entry(Descriptor::kernel_data_segment());
    let user_data = gdt.add_entry(Descriptor::user_data_segment());
    let user_code = gdt.add_entry(Descriptor::user_code_segment());

    // SAFETY: This operation is safe because there is no instances of `MutexGuard` which wraps
    // `TSS`.
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(unsafe { &*TSS.data_ptr() }));

    Gdt {
        table: gdt,
        kernel_code,
        kernel_data,
        user_data,
        user_code,
        tss_selector,
    }
});

pub(crate) struct Gdt {
    table: GlobalDescriptorTable,
    pub(crate) kernel_data: SegmentSelector,
    pub(crate) kernel_code: SegmentSelector,
    pub(crate) user_code: SegmentSelector,
    pub(crate) user_data: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub(crate) fn init() {
    GDT.table.load();
    unsafe {
        CS::set_reg(GDT.kernel_code);
        DS::set_reg(GDT.kernel_data);
        ES::set_reg(GDT.kernel_data);
        FS::set_reg(GDT.kernel_data);
        GS::set_reg(GDT.kernel_data);
        SS::set_reg(GDT.kernel_data);
        tables::load_tss(GDT.tss_selector);
    }

    init_star();
}

fn init_star() {
    Star::write(
        GDT.user_code,
        GDT.user_data,
        GDT.kernel_code,
        GDT.kernel_data,
    )
    .unwrap();
}
