// Bounded time of the GC increment.
// Deterministically measured in synthetic steps.
pub struct BoundedTime {
    steps: usize,
    limit: usize,
}

impl BoundedTime {
    pub fn new(limit: usize) -> BoundedTime {
        BoundedTime { steps: 0, limit }
    }

    pub fn tick(&mut self) {
        self.steps += 1;
    }

    pub fn advance(&mut self, amount: usize) {
        self.steps += amount;
    }

    pub fn is_over(&self) -> bool {
        self.steps > self.limit
    }
}
