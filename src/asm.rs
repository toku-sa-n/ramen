#[no_mangle]
pub fn hlt() -> () {
    unsafe {
        asm!("HLT"::::"intel");
    }
}

#[no_mangle]
pub fn load_eflags() -> i32 {
    let result: i32;
    unsafe {
        asm!("PUSHFD"::::"intel");
        asm!("POP EAX":"={EAX}"(result):::"intel");
    }
    result
}

#[no_mangle]
pub fn store_eflags(eflags: i32) -> () {
    unsafe {
        asm!("PUSH EAX"::"EAX"(eflags)::"intel");
        asm!("POPFD"::::"intel");
    }
}

#[no_mangle]
pub fn cli() -> () {
    unsafe {
        asm!("cli"::::"intel");
    }
}

#[no_mangle]
pub fn out8(port: i32, data: i32) -> () {
    unsafe {
        asm!("OUT DX,AL"::"{DX}"(port),"{AL}"(data)::"intel");
    }
}
