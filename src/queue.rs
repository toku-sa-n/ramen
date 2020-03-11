const QUEUE_SIZE: usize = 128;
pub struct Queue {
    data: [u32; QUEUE_SIZE],
    next_idx_write: usize,
    next_idx_read: usize,
    size: usize,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            data: [0; QUEUE_SIZE],
            next_idx_write: 0,
            next_idx_read: 0,
            size: 0,
        }
    }

    pub fn enqueue(&mut self, element: u32) -> () {
        if self.size == QUEUE_SIZE {
            return;
        }

        self.data[self.next_idx_write] = element;
        self.size += 1;
        self.next_idx_write = (self.next_idx_write + 1) % QUEUE_SIZE;
    }

    pub fn dequeue(&mut self) -> Option<u32> {
        if self.size == 0 {
            return None;
        }

        let return_value: u32 = self.data[self.next_idx_read];
        self.next_idx_read = (self.next_idx_read + 1) % QUEUE_SIZE;
        self.size -= 1;
        return Some(return_value);
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
