// SPDX-License-Identifier: GPL-3.0-or-later

use core::marker::PhantomData;
use core::ops::{Add, Sub};

pub trait Unit {}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Byte;
impl Unit for Byte {}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NumOfPages;
impl Unit for NumOfPages {}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

    pub const fn as_usize(&self) -> usize {
        self.num
    }
}

const BYTES_OF_PAGE: usize = 0x1000;

impl Size<Byte> {
    pub fn as_num_of_pages(self) -> Size<NumOfPages> {
        Size::new((self.num + BYTES_OF_PAGE - 1) / BYTES_OF_PAGE)
    }
}

impl Size<NumOfPages> {
    #[must_use]
    pub const fn as_bytes(self) -> Size<Byte> {
        Size::new(self.num * BYTES_OF_PAGE)
    }
}

impl Add<usize> for Size<Byte> {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        Self::new(self.as_usize() + rhs)
    }
}

impl Add<usize> for Size<NumOfPages> {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        Self::new(self.as_usize() + rhs)
    }
}

impl Sub<usize> for Size<Byte> {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self {
        Self::new(self.as_usize() - rhs)
    }
}

impl Sub<usize> for Size<NumOfPages> {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self {
        Self::new(self.as_usize() - rhs)
    }
}
