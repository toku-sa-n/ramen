use core::marker::PhantomData;

trait AddrType {}

pub struct Phys {}
impl AddrType for Phys {}

pub struct Virt {}
impl AddrType for Virt {}

pub struct Addr<T: AddrType> {
    addr: usize,
    _marker: PhantomData<fn() -> T>,
}
