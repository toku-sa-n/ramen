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

pub fn cli() -> () {
    unsafe {
        asm!("cli"::::"intel");
    }
}

pub fn out8(port: u32, data: u32) -> () {
    unsafe {
        asm!("OUT DX,AL"::"{DX}"(port),"{AL}"(data)::"intel");
    }
}

// It might be true that the first line can be deleted because the lower bits of EDX are DX
// itself.
pub fn in8(port: u32) -> u32 {
    let result: u32;
    unsafe {
        asm!("MOV EDX,$0"::"r"(port)::"intel");
        asm!("MOV EAX,0"::::"intel");
        asm!("IN AL,DX":"={AL}"(result):::"intel");
    }
    result
}

#[repr(C, packed)]
struct GdtrIdtrData {
    _limit: i16,
    _address: u64,
}

impl GdtrIdtrData {
    fn new(limit: i16, address: u64) -> Self {
        Self {
            _limit: limit,
            _address: address,
        }
    }
}

pub fn load_interrupt_descriptor_table_register(limit: u32, address: u64) {
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
            // In 64-bit mode, ES, DS, and SS segment registers are not used.
            // It's not necessary to push these registers.
            unsafe{
                asm!("
                    PUSH RAX
                    PUSH RCX
                    PUSH RDX
                    PUSH RBX
                    PUSH RSP
                    PUSH RBP
                    PUSH RSI
                    PUSH RDI
                    PUSH R8
                    PUSH R9
                    PUSH R10
                    PUSH R11
                    PUSH R12
                    PUSH R13
                    PUSH R14
                    PUSH R15
                    "
                    ::::"intel","volatile"
                );
                asm!("CALL $0"::"r"($function_name as extern "C"  fn()->())::"intel");
                asm!("
                    POP R15
                    POP R14
                    POP R13
                    POP R12
                    POP R11
                    POP R10
                    POP R9
                    POP R8
                    POP RDI
                    POP RSI
                    POP RBP
                    POP RSP
                    POP RBX
                    POP RDX
                    POP RCX
                    POP RAX
                    IRETQ"
                    ::::"intel","volatile"
                );
            }
        }
        handler_wrapper
    }}
}
