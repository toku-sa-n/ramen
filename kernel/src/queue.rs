// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::collections::vec_deque::VecDeque;

pub struct Queue<T: Copy> {
    queue: VecDeque<T>,
}

impl<T: Copy> Queue<T> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, element: T) {
        self.queue.push_back(element);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.queue.pop_front()
    }

    pub fn size(&self) -> usize {
        self.queue.len()
    }
}
