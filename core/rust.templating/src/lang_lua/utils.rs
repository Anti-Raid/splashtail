use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

pub struct AtomicInstant {
    base: Instant,
    offset: AtomicUsize,
}

impl AtomicInstant {
    /// Create a new AtomicInstant with the given base time.
    pub fn new(base: Instant) -> AtomicInstant {
        AtomicInstant {
            base,
            offset: AtomicUsize::new(0),
        }
    }

    /// Load the current time from the AtomicInstant.
    pub fn load(&self, order: Ordering) -> Instant {
        let offset_nanos = self.offset.load(order) as u64;
        let secs = offset_nanos / 1_000_000_000;
        let subsec_nanos = (offset_nanos % 1_000_000_000) as u32;
        let offset = Duration::new(secs, subsec_nanos);
        self.base + offset
    }

    /// Store the given time in the AtomicInstant.
    pub fn store(&self, val: Instant, order: Ordering) {
        let offset = val - self.base;
        let offset_nanos = offset.as_secs() * 1_000_000_000 + offset.subsec_nanos() as u64;
        self.offset.store(offset_nanos as usize, order);
    }
}
