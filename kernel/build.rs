fn main() {
    cc::Build::new().file("src/initrd.s").compile("initrd");
}
