fn main() {
    cc::Build::new().file("src/asm.s").compile("asm");
    cc::Build::new().file("src/qemu.s").compile("qemu");
}
