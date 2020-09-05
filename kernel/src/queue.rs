// SPDX-License-Identifier: GPL-3.0-or-later

const QUEUE_SIZE: usize = 128;
pub struct Queue<T: Copy> {
    data: [T; QUEUE_SIZE],
    next_idx_write: usize,
    next_idx_read: usize,
    size: usize,
}

impl<T: Copy> Queue<T> {
    pub fn new(init_data: T) -> Self {
        Self {
            data: [init_data; QUEUE_SIZE],
            next_idx_write: 0,
            next_idx_read: 0,
            size: 0,
        }
    }

    pub fn enqueue(&mut self, element: T) {
        if self.size == QUEUE_SIZE {
            return;
        }

        self.data[self.next_idx_write] = element;
        self.size += 1;
        self.next_idx_write = (self.next_idx_write + 1) % QUEUE_SIZE;
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }

        let return_value = self.data[self.next_idx_read];
        self.next_idx_read = (self.next_idx_read + 1) % QUEUE_SIZE;
        self.size -= 1;
        Some(return_value)
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
