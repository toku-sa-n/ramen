fn main() {
    cc::Build::new()
        .file("src/boot/main.s")
        .compile("bootstrap.o")
}
