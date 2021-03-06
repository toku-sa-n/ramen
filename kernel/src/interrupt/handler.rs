use crate::{interrupt::apic::local, process};

extern "C" {
    fn syscall_prepare_arguments() -> u64;
}

#[no_mangle]
extern "C" fn h_20() -> u64 {
    local::end_of_interrupt();
    process::switch().as_u64()
}

#[no_mangle]
extern "C" fn h_80() -> u64 {
    let v = unsafe { syscall_prepare_arguments() };
    process::assign_to_rax(v);
    local::end_of_interrupt();
    process::switch().as_u64()
}

#[no_mangle]
extern "C" fn h_81() -> u64 {
    unsafe { syscall_prepare_arguments() };
    local::end_of_interrupt();
    process::switch().as_u64()
}
