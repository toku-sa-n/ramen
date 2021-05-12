use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../build/initrd.cpio");
    println!("cargo:rerun-if-changed=src/initrd.s");
    touch_initrd().expect("Failed to `touch` the initrd cpio file.");
    cc::Build::new().file("src/initrd.s").compile("initrd");
    cc::Build::new()
        .file("src/interrupt/handler.s")
        .compile("handler");
    cc::Build::new().file("src/asm.s").compile("asm");
    cc::Build::new().file("src/qemu.s").compile("qemu");
}

fn touch_initrd() -> Result<(), std::io::Error> {
    Command::new("mkdir")
        .arg("-p")
        .arg("../build/")
        .spawn()?
        .wait()?;
    Command::new("touch")
        .arg("../build/initrd.cpio")
        .spawn()?
        .wait()?;
    Ok(())
}
