use core::marker::PhantomData;

pub trait AddrType {}

#[derive(Copy, Clone)]
pub struct Phys {}
impl AddrType for Phys {}

#[derive(Copy, Clone)]
pub struct Virt {}
impl AddrType for Virt {}

#[derive(Copy, Clone)]
pub struct Addr<T: AddrType> {
    addr: usize,
    _marker: PhantomData<fn() -> T>,
}

impl<T: AddrType> Addr<T> {
    pub const fn new(addr: usize) -> Self {
        Self {
            addr,
            _marker: PhantomData,
        }
    }

    pub fn offset(&self, offset: isize) -> Self {
        Self {
            addr: (self.addr as isize + offset) as _,
            ..*self
        }
    }
}

pub type PhysAddr = Addr<Phys>;
pub type VirtAddr = Addr<Virt>;

impl PhysAddr {
    pub fn as_usize(&self) -> usize {
        self.addr
    }

    pub fn as_mut_ptr(&self) -> *mut usize {
        self.addr as *mut _
    }
}

impl VirtAddr {
    pub fn as_usize(&self) -> usize {
        ((self.addr as isize) << 16 >> 16) as _
    }
}
