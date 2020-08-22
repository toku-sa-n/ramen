use core::marker::PhantomData;
use core::ops::Add;

pub trait Unit {}

#[derive(Copy, Clone)]
pub struct Byte;
impl Unit for Byte {}

#[derive(Copy, Clone)]
pub struct NumOfPages;
impl Unit for NumOfPages {}

#[derive(Copy, Clone)]
pub struct Size<T: Unit> {
    num: usize,
    _marker: PhantomData<fn() -> T>,
}

impl<T: Unit> Size<T> {
    pub const fn new(num: usize) -> Self {
        Self {
            num,
            _marker: PhantomData,
        }
    }

    pub fn as_usize(&self) -> usize {
        self.num
    }
}

const BYTES_OF_PAGE: usize = 0x1000;

impl Size<Byte> {
    pub fn as_num_of_pages(&self) -> Size<NumOfPages> {
        Size::new((self.num + BYTES_OF_PAGE - 1) / BYTES_OF_PAGE)
    }
}

impl Size<NumOfPages> {
    pub fn as_bytes(&self) -> Size<Byte> {
        Size::new(self.num * BYTES_OF_PAGE)
    }
}

impl Add<usize> for Size<Byte> {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        Self::new(self.as_usize() + rhs)
    }
}

impl Add<u64> for Size<Byte> {
    type Output = Self;

    fn add(self, rhs: u64) -> Self {
        self + rhs as usize
    }
}

impl Add<usize> for Size<NumOfPages> {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        Self::new(self.as_usize() + rhs)
    }
}

impl Add<u64> for Size<NumOfPages> {
    type Output = Self;

    fn add(self, rhs: u64) -> Self {
        self + rhs as usize
    }
}
