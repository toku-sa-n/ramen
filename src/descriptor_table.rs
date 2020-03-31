// See P.114

use crate::addresses::*;

const LIMIT_INTERRUPT_DESCRIPTOR_TABLE: u16 = 0x000007FF;
const LIMIT_GDT: u16 = 8 * 3 - 1;
const ACCESS_RIGHT_IDT: u32 = 0x008E;

#[repr(C, packed)]
struct GateDescriptor {
    offset_low: u16,
    selector: u16,
    dw_count: u8,
    access_right: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl GateDescriptor {
    fn set_gate_descriptor(&mut self, offset: u64, selector: u16, access_right: u32) -> () {
        (*self).offset_low = (offset & 0xFFFF) as u16;
        (*self).selector = selector;
        (*self).dw_count = ((access_right >> 8) & 0xFF) as u8;
        (*self).access_right = (access_right & 0xFF) as u8;
        (*self).offset_mid = ((offset >> 16) & 0xFFFF) as u16;
        (*self).offset_high = ((offset >> 32) & 0xFFFFFFFF) as u32;
        (*self).reserved = 0;
    }
}

#[repr(C, packed)]
struct SegmentDescriptor {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    p_dpl_s_type: u8,
    flags_and_limit_high: u8,
    base_high: u8,
}

enum SegmentType {
    NullSegment,
    CodeSegment,
    DataSegment,
}

impl SegmentDescriptor {
    fn set_segment_descriptor(&mut self, seg_type: SegmentType) -> () {
        match seg_type {
            SegmentType::NullSegment => {
                (*self).limit_low = 0;
                (*self).base_low = 0;
                (*self).base_mid = 0;
                (*self).p_dpl_s_type = 0;
                (*self).flags_and_limit_high = 0;
                (*self).base_high = 0;
            }
            SegmentType::CodeSegment => {
                (*self).limit_low = 0xFFFF;
                (*self).base_low = 0x0000;
                (*self).base_mid = 0x00;
                (*self).p_dpl_s_type = 0x9A;
                (*self).flags_and_limit_high = 0xAF;
                (*self).base_high = 0x00;
            }
            SegmentType::DataSegment => {
                (*self).limit_low = 0xFFFF;
                (*self).base_low = 0x0000;
                (*self).base_mid = 0x00;
                (*self).p_dpl_s_type = 0x92;
                (*self).flags_and_limit_high = 0xCF;
                (*self).base_high = 0x00;
            }
        }
    }
}

pub fn init() -> () {
    init_idt();
    init_gdt();
    set_interruption();
}

fn init_idt() -> () {
    let interrupt_descriptor_table: *mut GateDescriptor =
        VIRTUAL_ADDRESS_IDT as *mut GateDescriptor;

    const SIZE_IDT_ENTRY: u16 = 16;
    for i in 0..=(LIMIT_INTERRUPT_DESCRIPTOR_TABLE / SIZE_IDT_ENTRY) {
        unsafe {
            (*interrupt_descriptor_table.offset(i as isize)).set_gate_descriptor(0, 0, 0);
        }
    }

    lidt(LIMIT_INTERRUPT_DESCRIPTOR_TABLE, VIRTUAL_ADDRESS_IDT);
}

fn init_gdt() -> () {
    let gdt: *mut SegmentDescriptor = VIRTUAL_ADDRESS_GDT as *mut SegmentDescriptor;

    const SIZE_GDT_ENTRY: u16 = 8;
    for i in 0..=((LIMIT_GDT + 1) / SIZE_GDT_ENTRY) {
        unsafe {
            (*gdt.offset(i as isize)).set_segment_descriptor(SegmentType::NullSegment);
        }
    }

    unsafe {
        (*gdt.offset(1)).set_segment_descriptor(SegmentType::DataSegment);
        (*gdt.offset(2)).set_segment_descriptor(SegmentType::CodeSegment);
    }

    lgdt(LIMIT_GDT, VIRTUAL_ADDRESS_GDT);
}

fn set_interruption() {
    use crate::interrupt::interrupt_handler_21;
    use crate::interrupt::interrupt_handler_2c;
    use crate::interrupt_handler;

    let interrupt_descriptor_table: *mut GateDescriptor =
        VIRTUAL_ADDRESS_IDT as *mut GateDescriptor;
    unsafe {
        (*interrupt_descriptor_table.offset(0x21)).set_gate_descriptor(
            interrupt_handler!(interrupt_handler_21) as u64,
            2 * 8,
            ACCESS_RIGHT_IDT,
        );
        (*interrupt_descriptor_table.offset(0x2C)).set_gate_descriptor(
            interrupt_handler!(interrupt_handler_2c) as u64,
            2 * 8,
            ACCESS_RIGHT_IDT,
        );
    }
}

#[repr(C, packed)]
struct GdtrIdtrData {
    _limit: u16,
    _address: u64,
}

impl GdtrIdtrData {
    fn new(limit: u16, address: u64) -> Self {
        Self {
            _limit: limit,
            _address: address,
        }
    }
}

fn lidt(limit: u16, address: u64) {
    unsafe {
        asm!("LIDT ($0)"::"r"(&GdtrIdtrData::new(limit, address)));
    }
}

fn lgdt(limit: u16, address: u64) {
    unsafe {
        asm!("LGDT ($0)"::"r"(&GdtrIdtrData::new(limit,address)));
    }
}
