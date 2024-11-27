use std::time::{Duration, Instant};

/// [Stopwatch] is a simple utility for keeping track of time over a duration.
///
/// Useful for building a game loop, or measuring wall clock time for debugging.
pub struct Stopwatch {
    // All in milliseconds.
    start: Instant,
    last: u128,
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

impl Stopwatch {
    pub fn new() -> Self {
        let start = Instant::now();
        let ms = start.elapsed().as_millis();

        Self { start, last: ms }
    }

    pub fn lap(&mut self) -> Duration {
        let ms = self.start.elapsed().as_millis();
        let diff = ms - self.last;

        self.last = ms;

        Duration::from_millis(diff as u64)
    }

    pub fn rewind(&mut self, duration: Duration) {
        self.last -= duration.as_millis();
    }
}
