// See P.114

// TODO: Methodificate set_segment_descriptor and set_gate_descriptor

use crate::asm;

const ADDRESS_INTERRUPT_DESCRIPTOR_TABLE: i32 = 0x0026f800;
const LIMIT_INTERRUPT_DESCRIPTOR_TABLE: i32 = 0x000007ff;
const ADDRESS_GATE_DESCRIPTOR_TABLE: i32 = 0x00270000;
const LIMIT_GATE_DESCRIPTOR_TABLE: i32 = 0x0000ffff;
const ADDRESS_BOOTPACK: i32 = 0x00280000;
const LIMIT_BOOTPACK: u32 = 0x0007ffff;
const ADDRESS_SYSTEM_READ_WRITE: i32 = 0x4092;
const ADDRESS_SYSTEM_READ_EXECUTE: i32 = 0x409a;

#[repr(C)]
struct SegmentDescriptor {
    limit_low: i16,
    base_low: i16,
    base_mid: i8,
    access_right: i8,
    limit_high: i8,
    base_high: i8,
}

#[repr(C)]
struct GateDescriptor {
    offset_low: i16,
    selector: i16,
    dw_count: i8,
    access_right: i8,
    offset_high: i16,
}

pub fn init_gdt_idt() -> () {
    let global_descriptor_table: *mut SegmentDescriptor =
        ADDRESS_GATE_DESCRIPTOR_TABLE as *mut SegmentDescriptor;

    for i in 0..8192 {
        unsafe {
            set_segment_descriptor(global_descriptor_table.offset(i), 0, 0, 0);
        }
    }

    unsafe {
        set_segment_descriptor(
            global_descriptor_table.offset(1),
            0xffffffff,
            0x00000000,
            ADDRESS_SYSTEM_READ_WRITE,
        );

        set_segment_descriptor(
            global_descriptor_table.offset(2),
            LIMIT_BOOTPACK,
            ADDRESS_BOOTPACK,
            ADDRESS_SYSTEM_READ_EXECUTE,
        );
    }

    asm::load_global_descriptor_table_register(
        LIMIT_GATE_DESCRIPTOR_TABLE,
        ADDRESS_GATE_DESCRIPTOR_TABLE,
    );

    let interrupt_descriptor_table: *mut GateDescriptor =
        ADDRESS_INTERRUPT_DESCRIPTOR_TABLE as *mut GateDescriptor;

    for i in 0..256 {
        unsafe {
            set_gate_descriptor(interrupt_descriptor_table.offset(i), 0, 0, 0);
        }
    }

    asm::load_interrupt_descriptor_table_register(
        LIMIT_INTERRUPT_DESCRIPTOR_TABLE,
        ADDRESS_INTERRUPT_DESCRIPTOR_TABLE,
    );
}

fn set_segment_descriptor(
    segment_descriptor: *mut SegmentDescriptor,
    limit: u32,
    base: i32,
    access_right: i32,
) -> () {
    let mut limit = limit;
    let mut access_right = access_right;
    if limit > 0xfffff {
        access_right |= 0x8000;
        limit /= 0x1000;
    }

    unsafe {
        (*segment_descriptor).limit_low = (limit & 0xffff) as i16;
        (*segment_descriptor).base_low = (base & 0xffff) as i16;
        (*segment_descriptor).base_mid = ((base >> 16) & 0xff) as i8;
        (*segment_descriptor).access_right = (access_right & 0xff) as i8;
        (*segment_descriptor).limit_high =
            (((limit >> 16) & 0x0f) as i32 | ((access_right >> 8) & 0xf0)) as i8;
        (*segment_descriptor).base_high = ((base >> 24) & 0xff) as i8;
    }
}

fn set_gate_descriptor(
    gate_descriptor: *mut GateDescriptor,
    offset: i32,
    selector: i32,
    access_right: i32,
) -> () {
    unsafe {
        (*gate_descriptor).offset_low = (offset & 0xffff) as i16;
        (*gate_descriptor).selector = selector as i16;
        (*gate_descriptor).dw_count = ((access_right >> 8) & 0xff) as i8;
        (*gate_descriptor).access_right = (access_right & 0xff) as i8;
        (*gate_descriptor).offset_high = ((offset >> 16) & 0xffff) as i16;
    }
}
