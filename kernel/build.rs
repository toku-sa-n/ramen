fn main() {
    cc::Build::new()
        .file("src/interrupt/handler.s")
        .compile("handler");
}
