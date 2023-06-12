use std::sync::Arc;

use parking_lot::RwLock;

pub enum FillLevel {
    Empty,
    Partial,
    Full,
}

pub struct Buffer<T> {
    buffer: Arc<RwLock<buffer::Buffer<T>>>,
}

impl<T> Buffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(buffer::Buffer::new(capacity))),
        }
    }
    pub fn len(&self) -> usize {
        self.buffer.read().len()
    }
    pub fn is_empty(&self) -> bool {
        self.buffer.read().is_empty()
    }
    pub fn capacity(&self) -> usize {
        self.buffer.read().capacity()
    }
    pub fn fill_level(&self) -> FillLevel {
        self.buffer.read().fill_level()
    }
    pub fn push(&self, value: T) {
        self.buffer.write().push(value)
    }
}
impl<T: Clone> Buffer<T> {
    pub fn push_slice(&self, slice: &[T]) {
        let mut lock = self.buffer.write();
        for value in slice {
            lock.push(value.clone())
        }
    }
    pub fn head(&self) -> Option<T> {
        self.buffer.read().head()
    }
    pub fn snapshot(&self) -> Vec<T> {
        self.buffer.read().snapshot()
    }
    pub fn clear(&self) {
        self.buffer.write().clear()
    }
}

mod buffer {

    use super::*;

    pub struct Buffer<T> {
        buffer: Vec<T>,
        len: usize,
        pos: usize,
    }
    impl<T> Buffer<T> {
        pub fn new(capacity: usize) -> Self {
            let mut vec = Vec::with_capacity(capacity);
            #[allow(clippy::uninit_vec)]
            unsafe {
                vec.set_len(capacity);
            }
            Self {
                buffer: vec,
                len: 0,
                pos: 0,
            }
        }
        pub fn len(&self) -> usize {
            self.len
        }
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }
        pub fn capacity(&self) -> usize {
            self.buffer.capacity()
        }
        pub fn inc_pos(&mut self) {
            self.pos += 1;
            let capacity = self.capacity();
            if self.pos == capacity {
                self.pos = 0;
            }
            if self.len < capacity {
                self.len += 1;
            }
        }
        pub fn fill_level(&self) -> FillLevel {
            let capacity = self.capacity();
            match self.len() {
                0 => FillLevel::Empty,
                x if x == capacity => FillLevel::Full,
                _ => FillLevel::Partial,
            }
        }
        pub fn push(&mut self, value: T) {
            *&mut self.buffer[self.pos] = value;
            self.inc_pos();
        }
        pub fn clear(&mut self) {
            self.len = 0;
            self.pos = 0;
        }
    }
    impl<T: Clone> Buffer<T> {
        pub fn head(&self) -> Option<T> {
            match self.fill_level() {
                FillLevel::Empty => None,
                _ => self
                    .buffer
                    .get(if self.pos == 0 {
                        self.capacity() - 1
                    } else {
                        self.pos - 1
                    })
                    .cloned(),
            }
        }
        pub fn snapshot(&self) -> Vec<T> {
            let mut out = vec![];
            match self.fill_level() {
                FillLevel::Partial => out.append(&mut self.buffer[..self.pos].to_vec()),
                FillLevel::Full => {
                    out.append(&mut self.buffer[self.pos..self.capacity()].to_vec());
                    out.append(&mut self.buffer[..self.pos].to_vec());
                }
                _ => {}
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let buffer = Buffer::new(3);

        assert!(buffer.is_empty());
        assert_eq!(buffer.snapshot(), vec![]);

        buffer.push(1);
        assert_eq!(buffer.snapshot(), vec![1]);
        assert!(!buffer.is_empty());

        buffer.push_slice(&[2, 3]);
        assert_eq!(buffer.snapshot(), vec![1, 2, 3]);

        buffer.push_slice(&[4, 5]);
        assert_eq!(buffer.snapshot(), vec![3, 4, 5]);

        buffer.clear();
        assert!(buffer.is_empty());
    }
}
