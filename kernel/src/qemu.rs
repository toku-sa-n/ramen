use qemu_exit::QEMUExit;

const IO_BASE: u16 = 0xf4;
const EXIT_SUCCESS_CODE: u32 = 33;

pub(crate) fn exit_success() -> ! {
    exit_handler().exit_success();
}

pub(crate) fn exit_failure() -> ! {
    exit_handler().exit_failure();
}

fn exit_handler() -> qemu_exit::X86 {
    qemu_exit::X86::new(IO_BASE, EXIT_SUCCESS_CODE)
}
