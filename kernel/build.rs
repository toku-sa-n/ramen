fn main() {
    cc::Build::new()
        .file("src/interrupt/handler.s")
        .compile("handler");
    cc::Build::new().file("src/asm.s").compile("asm");
}
