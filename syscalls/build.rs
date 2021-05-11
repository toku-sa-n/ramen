fn main() {
    cc::Build::new().file("src/asm.s").compile("asm");
}
