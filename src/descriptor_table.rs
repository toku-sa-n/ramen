// See P.114

use crate::asm;

const VIRTUAL_ADDRESS_IDT: u64 = 0xFFFFFFFF80080000;
const LIMIT_INTERRUPT_DESCRIPTOR_TABLE: u32 = 0x000007FF;
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

pub fn init() -> () {
    init_idt();
    set_interruption();
}

fn init_idt() -> () {
    let interrupt_descriptor_table: *mut GateDescriptor =
        VIRTUAL_ADDRESS_IDT as *mut GateDescriptor;

    const SIZE_IDT_ENTRY: u32 = 16;
    for i in 0..=(LIMIT_INTERRUPT_DESCRIPTOR_TABLE / SIZE_IDT_ENTRY) {
        unsafe {
            (*interrupt_descriptor_table.offset(i as isize)).set_gate_descriptor(0, 0, 0);
        }
    }

    // LDIT instruction takes PHYSICAL address of idt.
    asm::load_interrupt_descriptor_table_register(
        LIMIT_INTERRUPT_DESCRIPTOR_TABLE,
        VIRTUAL_ADDRESS_IDT,
    );
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
            3 * 8,
            ACCESS_RIGHT_IDT,
        );
        (*interrupt_descriptor_table.offset(0x2C)).set_gate_descriptor(
            interrupt_handler!(interrupt_handler_2c) as u64,
            3 * 8,
            ACCESS_RIGHT_IDT,
        );
    }
}
