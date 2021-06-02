use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Time {
    start: Instant,
    last_update: Instant,
}

impl Time {
    pub(crate) fn delta_seconds_f64(&self) -> f64 {
        todo!()
    }
}

impl Time {
    pub(crate) fn delta(&self) -> Duration {
        todo!()
    }
}

impl Time {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last_update: now,
        }
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Timer {}

impl Timer {
    pub(crate) fn elapsed(&self) -> Duration {
        todo!()
    }
}

impl Timer {
    pub(crate) fn duration(&self) -> Duration {
        todo!()
    }
}

impl Timer {
    pub(crate) fn reset(&self) {
        todo!()
    }
}

impl Timer {
    pub(crate) fn set_duration(&self, p0: Duration) {
        todo!()
    }
}

impl Timer {
    pub(crate) fn percent_left(&self) -> f32 {
        todo!()
    }
}

impl Timer {
    pub(crate) fn new(p0: Duration, p1: bool) -> Timer {
        todo!()
    }
}

impl Timer {
    pub(crate) fn from_seconds(p0: f64, p1: bool) -> Timer {
        todo!()
    }
}

impl Timer {
    pub(crate) fn percent(&self) -> f32 {
        todo!()
    }
}

impl Timer {
    pub(crate) fn finished(&self) -> bool {
        todo!()
    }
}

impl Timer {
    pub(crate) fn tick(&self, p0: Duration) {
        todo!()
    }
}

impl Timer {
    pub(crate) fn remaining_seconds(&self) -> f32 {
        (self.duration() - self.elapsed()).as_secs_f32()
    }
}
