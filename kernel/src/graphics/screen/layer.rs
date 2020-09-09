// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Coord, TwoDimensionalVec};
use alloc::vec::Vec;

struct Layer {
    buf: Vec<Vec<u8>>,
    top_left: Coord<usize>,
    len: TwoDimensionalVec<usize>,
}
