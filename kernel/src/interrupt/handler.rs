use crate::{interrupt::apic::local, process};

#[no_mangle]
pub extern "C" fn h_20() -> u64 {
    local::end_of_interrupt();
    process::switch().as_u64()
}
