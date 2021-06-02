use std::ops::Sub;

use derive_more::{Add, AddAssign, Sub, SubAssign};
use js_sys::Date;

#[derive(Clone, Debug, Copy, Default)]
pub struct Instant(f64);

impl Instant {
    pub fn now() -> Self {
        Self(Date::now())
    }
}

impl Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration(self.0 - rhs.0)
    }
}

#[derive(Clone, Debug, Copy, Add, AddAssign, Default, Sub, SubAssign, PartialEq, PartialOrd)]
pub struct Duration(f64);

impl Duration {
    pub(crate) fn as_secs_f32(&self) -> f32 {
        self.as_secs_f64() as f32
    }

    pub(crate) fn mul_f64(&self, rhs: f64) -> Duration {
        Self(self.0 * rhs)
    }

    pub(crate) fn as_secs_f64(&self) -> f64 {
        self.0 / 1000.0
    }

    pub(crate) fn from_secs_f64(secs: f64) -> Self {
        Self(secs * 1000.0)
    }
}

#[derive(Debug)]
pub struct Time {
    start: Instant,
    current_update: Instant,
    delta_since_previous: Duration,
}

impl Time {
    pub(crate) fn delta_seconds_f64(&self) -> f64 {
        self.delta_since_previous.as_secs_f64()
    }

    pub(crate) fn delta(&self) -> Duration {
        self.delta_since_previous
    }

    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            current_update: now,
            delta_since_previous: Duration::default(),
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.delta_since_previous = now - self.current_update;
        self.current_update = now;
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Timer {
    duration: Duration,
    elapsed: Duration,
    auto_reset: bool,
}

impl Timer {
    pub(crate) fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub(crate) fn duration(&self) -> Duration {
        self.duration
    }

    pub(crate) fn reset(&mut self) {
        self.elapsed = Duration::default();
    }

    pub(crate) fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }

    pub(crate) fn percent_left(&self) -> f32 {
        1.0 - self.percent()
    }

    pub(crate) fn new(duration: Duration, auto_reset: bool) -> Self {
        Self {
            duration,
            elapsed: Default::default(),
            auto_reset,
        }
    }

    pub(crate) fn from_seconds(duration: f64, auto_reset: bool) -> Self {
        Self::new(Duration::from_secs_f64(duration), auto_reset)
    }

    pub(crate) fn percent(&self) -> f32 {
        self.elapsed.as_secs_f32() / self.duration.as_secs_f32()
    }

    pub(crate) fn finished(&self) -> bool {
        self.elapsed > self.duration
    }

    pub(crate) fn tick(&mut self, delta: Duration) {
        self.elapsed += delta;
    }

    pub(crate) fn remaining_seconds(&self) -> f32 {
        (self.duration() - self.elapsed()).as_secs_f32()
    }
}
