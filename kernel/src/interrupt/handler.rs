use crate::{interrupt::apic::local, process, syscall};

#[no_mangle]
pub extern "C" fn h_20() -> u64 {
    local::end_of_interrupt();
    process::switch().as_u64()
}

#[no_mangle]
pub extern "C" fn h_80() -> u64 {
    unsafe { syscall::prepare_arguments() }
    process::assign_rax_from_register();
    local::end_of_interrupt();
    process::switch().as_u64()
}
