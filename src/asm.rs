pub fn hlt() -> () {
    unsafe {
        asm!("HLT"::::"intel");
    }
}

pub fn load_eflags() -> i32 {
    let result: i32;
    unsafe {
        asm!("PUSHFD"::::"intel");
        asm!("POP EAX":"={EAX}"(result):::"intel");
    }
    result
}

pub fn store_eflags(eflags: i32) -> () {
    unsafe {
        asm!("PUSH EAX"::"EAX"(eflags)::"intel");
        asm!("POPFD"::::"intel");
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
