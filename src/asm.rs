pub fn hlt() -> () {
    unsafe {
        asm!("HLT"::::"intel");
    }
}

pub fn load_eflags() -> i32 {
    let result: i32;
    unsafe {
        asm!("PUSHFD
              POP EAX":"={EAX}"(result):::"intel");
    }
    result
}

pub fn store_eflags(eflags: i32) -> () {
    unsafe {
        asm!("PUSH EAX
              POPFD"::"EAX"(eflags)::"intel");
    }
}

pub fn cli() -> () {
    unsafe {
        asm!("cli"::::"intel");
    }
}

pub fn out8(port: i32, data: i32) -> () {
    unsafe {
        asm!("OUT DX,AL"::"{DX}"(port),"{AL}"(data)::"intel");
    }
}

struct GdtrIdtrData {
    _limit: i16,
    _address: i32,
}

impl GdtrIdtrData {
    fn new(limit: i16, address: i32) -> GdtrIdtrData {
        GdtrIdtrData {
            _limit: limit,
            _address: address,
        }
    }
}

pub fn load_global_descriptor_table_register(limit: i32, address: i32) {
    unsafe {
        asm!("LGDT ($0)"::"r"(&GdtrIdtrData::new(limit as i16, address)));
    }
}

pub fn load_interrupt_descriptor_table_register(limit: i32, address: i32) {
    unsafe {
        asm!("LIDT ($0)"::"r"(&GdtrIdtrData::new(limit as i16, address)));
    }
}
