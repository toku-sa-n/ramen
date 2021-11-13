use {
    crate::{interrupt::apic::local, process},
    x86_64::structures::idt::InterruptStackFrame,
};

extern "C" {
    fn syscall_prepare_arguments() -> u64;
}

pub(super) extern "x86-interrupt" fn h_20(_: InterruptStackFrame) {
    local::end_of_interrupt();
    process::switch();
}

pub(super) extern "x86-interrupt" fn h_80(_: InterruptStackFrame) {
    let v = unsafe { syscall_prepare_arguments() };
    process::assign_to_rax(v);
    local::end_of_interrupt();
    process::switch();
}

pub(super) extern "x86-interrupt" fn h_81(_: InterruptStackFrame) {
    unsafe { syscall_prepare_arguments() };
    local::end_of_interrupt();
    process::switch();
}
