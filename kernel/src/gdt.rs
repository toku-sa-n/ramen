// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::tss::TSS,
    conquer_once::spin::Lazy,
    x86_64::{
        instructions::{
            segmentation::{Segment, CS, DS, ES, FS, GS, SS},
            tables,
        },
        registers::model_specific::Star,
        structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
    },
};

pub(crate) static SELECTORS: Lazy<Selectors> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let kernel_code = gdt.add_entry(Descriptor::kernel_code_segment());
    let kernel_data = gdt.add_entry(Descriptor::kernel_data_segment());
    let user_data = gdt.add_entry(Descriptor::user_data_segment());
    let user_code = gdt.add_entry(Descriptor::user_code_segment());

    // SAFETY: This operation is safe because there is no instances of `MutexGuard` which wraps
    // `TSS`.
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(unsafe { &*TSS.data_ptr() }));

    Selectors {
        table: gdt,
        kernel_code,
        kernel_data,
        user_data,
        user_code,
        tss_selector,
    }
});

pub(crate) struct Selectors {
    table: GlobalDescriptorTable,
    pub(crate) kernel_data: SegmentSelector,
    pub(crate) kernel_code: SegmentSelector,
    pub(crate) user_code: SegmentSelector,
    pub(crate) user_data: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub(crate) fn init() {
    SELECTORS.table.load();
    unsafe {
        CS::set_reg(SELECTORS.kernel_code);
        DS::set_reg(SELECTORS.kernel_data);
        ES::set_reg(SELECTORS.kernel_data);
        FS::set_reg(SELECTORS.kernel_data);
        GS::set_reg(SELECTORS.kernel_data);
        SS::set_reg(SELECTORS.kernel_data);
        tables::load_tss(SELECTORS.tss_selector);
    }

    init_star();
}

fn init_star() {
    Star::write(
        SELECTORS.user_code,
        SELECTORS.user_data,
        SELECTORS.kernel_code,
        SELECTORS.kernel_data,
    )
    .unwrap();
}
