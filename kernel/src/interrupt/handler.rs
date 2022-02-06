use {
    crate::{interrupt::apic::local, process},
    x86_64::structures::idt::InterruptStackFrame,
};

pub(super) extern "x86-interrupt" fn h_20(_: InterruptStackFrame) {
    local::end_of_interrupt();
    process::switch();
}
