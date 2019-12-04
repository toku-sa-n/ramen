pub fn hlt() -> () {
    unsafe {
        asm!("HLT"::::"intel");
    }
}

pub fn sti() -> () {
    unsafe {
        asm!("STI"::::"intel");
    }
}

pub fn stihlt() -> () {
    unsafe {
        asm!(
            "STI
             HLT"
             :::: "intel");
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

// It might be true that the first line can be deleted because the lower bits of EDX are DX
// itself.
pub fn in8(port: i32) -> i32 {
    let result: i32;
    unsafe {
        asm!("MOV EAX,0"::::"intel");
        asm!("IN AL,DX":"={AL}"(result):"{DX}"(port)::"intel");
    }
    result
}

#[repr(C, packed)]
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

// Don't put these asm! in one! It doesn't work!
#[macro_export]
macro_rules! interrupt_handler{
    ($function_name:ident)=>{{
        #[naked]
        pub extern "C" fn handler_wrapper() -> () {
            unsafe{
                asm!("
                    PUSH ES
                    PUSH DS
                    PUSHAD
                    MOV EAX,ESP
                    PUSH EAX
                    MOV AX,SS
                    MOV DS,AX
                    MOV ES,AX"
                    ::::"intel","volatile"
                );
                asm!("CALL $0"::"r"($function_name as extern "C"  fn()->())::"intel");
                asm!("
                    POP EAX
                    POPAD
                    POP DS
                    POP ES
                    IRETD"
                    ::::"intel","volatile"
                );
            }
        }
        handler_wrapper
    }}
}
