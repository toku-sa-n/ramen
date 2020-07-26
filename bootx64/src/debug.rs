#[macro_export]
#[allow(unused_macros)]
macro_rules! watch{
    ($value:expr)=>{
        unsafe{
            let val=$value as u64;
            asm!("mov rax, {:r}",in(reg) val);
        }
        loop{}
    };
}

#[allow(unused_macros)]
macro_rules! stop {
    () => {
        watch!(0x55aa55aa55aa55aa);
    };
}
