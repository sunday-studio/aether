//! Debounce and triggers for push. Used by engine to batch rapid edits.

use std::time::{Duration, Instant};

const DEBOUNCE: Duration = Duration::from_secs(3);

/// Tracks last write time for debounced push.
#[derive(Default)]
pub struct SyncScheduler {
    last_write: Option<Instant>,
}

impl SyncScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_local_write(&mut self) {
        self.last_write = Some(Instant::now());
    }

    /// True if a write occurred and DEBOUNCE has elapsed.
    pub fn should_flush(&self) -> bool {
        self.last_write
            .map(|t| t.elapsed() >= DEBOUNCE)
            .unwrap_or(false)
    }

    pub fn clear(&mut self) {
        self.last_write = None;
    }
}
