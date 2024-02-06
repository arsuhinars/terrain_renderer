use std::time::Instant;

pub struct TimeManager {
    instant: Instant,
    delta: f32,
}

impl TimeManager {
    pub fn new() -> TimeManager {
        TimeManager {
            instant: Instant::now(),
            delta: 0.0,
        }
    }

    pub fn update(&mut self) {
        let last_instant = self.instant;
        self.instant = Instant::now();
        self.delta = self.instant.duration_since(last_instant).as_millis() as f32 * 1000.0;
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }
}
