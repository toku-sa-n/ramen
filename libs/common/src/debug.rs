// SPDX-License-Identifier: GPL-3.0-or-later

#[macro_export]
macro_rules! watch{
    ($value:expr)=>{
        unsafe{
            let val=$value as u64;
            asm!("mov rax, {:r}",in(reg) val);
        }
        loop{}
    };
}

#[macro_export]
macro_rules! stop {
    () => {
        watch!(0x55aa55aa55aa55aa);
    };
}

#[macro_export]
macro_rules! watch_if {
    ($cond:expr,$value:expr) => {
        if $cond {
            watch!($value);
        }
    };
}

#[macro_export]
macro_rules! stop_if {
    ($condition:expr) => {
        if $condition {
            stop!();
        }
    };
}
