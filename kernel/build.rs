fn main() {
    cc::Build::new()
        .file("src/boot/multiboot.s")
        .file("src/boot/main.s")
        .compile("boot.o");
}
