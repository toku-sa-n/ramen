use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../build/initrd.cpio");
    println!("cargo:rerun-if-changed=src/initrd.s");
    touch_initrd().expect("Failed to `touch` the initrd cpio file.");
    cc::Build::new().file("src/initrd.s").compile("initrd");
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
