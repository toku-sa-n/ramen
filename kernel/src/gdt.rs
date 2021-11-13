// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::tss::TSS,
    conquer_once::spin::OnceCell,
    x86_64::{
        instructions::{
            segmentation::{Segment, CS, DS, ES, FS, GS, SS},
            tables,
        },
        registers::model_specific::Star,
        structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
    },
};

static GDT: OnceCell<GlobalDescriptorTable> = OnceCell::uninit();

static SELECTORS: OnceCell<Selectors> = OnceCell::uninit();

#[derive(Copy, Clone)]
struct Selectors {
    kernel_data: SegmentSelector,
    kernel_code: SegmentSelector,
    user_code: SegmentSelector,
    user_data: SegmentSelector,
    tss: SegmentSelector,
}

pub(crate) fn init() {
    let (gdt, selectors) = generate_gdt_and_selectors();

    GDT.init_once(|| gdt);
    SELECTORS.init_once(|| selectors);

    GDT.get().expect("GDT should be initialized.").load();

    unsafe {
        CS::set_reg(selectors.kernel_code);
        DS::set_reg(selectors.kernel_data);
        ES::set_reg(selectors.kernel_data);
        FS::set_reg(selectors.kernel_data);
        GS::set_reg(selectors.kernel_data);
        SS::set_reg(selectors.kernel_data);
        tables::load_tss(selectors.tss);
    }

    init_star();
}

pub(crate) fn kernel_code_selector() -> SegmentSelector {
    selectors().kernel_code
}

pub(crate) fn kernel_data_selector() -> SegmentSelector {
    selectors().kernel_data
}

pub(crate) fn user_code_selector() -> SegmentSelector {
    selectors().user_code
}

pub(crate) fn user_data_selector() -> SegmentSelector {
    selectors().user_data
}

fn generate_gdt_and_selectors() -> (GlobalDescriptorTable, Selectors) {
    let mut gdt = GlobalDescriptorTable::new();
    let kernel_code = gdt.add_entry(Descriptor::kernel_code_segment());
    let kernel_data = gdt.add_entry(Descriptor::kernel_data_segment());
    let user_data = gdt.add_entry(Descriptor::user_data_segment());
    let user_code = gdt.add_entry(Descriptor::user_code_segment());

    // SAFETY: This operation is safe because there is no instances of `MutexGuard` which wraps
    // `TSS`.
    let tss = gdt.add_entry(Descriptor::tss_segment(unsafe { &*TSS.data_ptr() }));

    let selectors = Selectors {
        kernel_data,
        kernel_code,
        user_code,
        user_data,
        tss,
    };

    (gdt, selectors)
}

fn init_star() {
    let selectors = SELECTORS.get().expect("The selectors are not initialized.");

    Star::write(
        selectors.user_code,
        selectors.user_data,
        selectors.kernel_code,
        selectors.kernel_data,
    )
    .unwrap();
}

fn selectors<'a>() -> &'a Selectors {
    SELECTORS.get().expect("The selectors are not recorded.")
}
