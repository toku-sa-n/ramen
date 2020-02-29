// See P.114

use crate::asm;

const VIRTUAL_ADDRESS_IDT: u32 = 0xC0080000;
const LIMIT_INTERRUPT_DESCRIPTOR_TABLE: i32 = 0x000007ff;
const ACCESS_RIGHT_IDT: i32 = 0x008e;

#[repr(C, packed)]
struct GateDescriptor {
    offset_low: i16,
    selector: i16,
    dw_count: i8,
    access_right: i8,
    offset_high: i16,
}

impl GateDescriptor {
    fn set_gate_descriptor(&mut self, offset: i32, selector: i32, access_right: i32) -> () {
        (*self).offset_low = (offset & 0xffff) as i16;
        (*self).selector = selector as i16;
        (*self).dw_count = ((access_right >> 8) & 0xff) as i8;
        (*self).access_right = (access_right & 0xff) as i8;
        (*self).offset_high = ((offset >> 16) & 0xffff) as i16;
    }
}

pub fn init() -> () {
    init_idt();
    set_interruption();
}

fn init_idt() -> () {
    let interrupt_descriptor_table: *mut GateDescriptor =
        VIRTUAL_ADDRESS_IDT as *mut GateDescriptor;

    for i in 0..=(LIMIT_INTERRUPT_DESCRIPTOR_TABLE / 8) {
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
            interrupt_handler!(interrupt_handler_21) as i32,
            2 * 8,
            ACCESS_RIGHT_IDT,
        );
        (*interrupt_descriptor_table.offset(0x2c)).set_gate_descriptor(
            interrupt_handler!(interrupt_handler_2c) as i32,
            2 * 8,
            ACCESS_RIGHT_IDT,
        );
    }
}
