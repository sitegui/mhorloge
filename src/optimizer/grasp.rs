use rand::rngs::SmallRng;
use rand::Rng;
use std::collections::VecDeque;

/// A helper type used to select randomly over the first N values
pub struct Grasp<T> {
    values: VecDeque<T>,
    size: usize,
}

impl<T> Grasp<T> {
    pub fn new(values: VecDeque<T>, size: usize) -> Self {
        Grasp { values, size }
    }

    pub fn pop(&mut self, rng: &mut SmallRng) -> Option<T> {
        if self.values.is_empty() {
            None
        } else {
            let size = self.size.min(self.values.len());
            let index = rng.gen_range(0..size);
            self.values.remove(index)
        }
    }
}
