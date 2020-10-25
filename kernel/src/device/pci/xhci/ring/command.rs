// SPDX-License-Identifier: GPL-3.0-or-later

use super::Raw;

struct CommandRing {
    raw: Raw,
    enqueue_ptr: usize,
    len: usize,
}
impl CommandRing {
    fn new(len: usize) -> Self {
        Self {
            raw: Raw::new(len),
            enqueue_ptr: 0,
            len,
        }
    }
}
