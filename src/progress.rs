use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Progress {
    progress: f64,
    instant: Instant,
    finished_time: Option<Duration>,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            instant: Instant::now(),
            finished_time: None,
        }
    }

    pub fn get_progress(&self) -> f64 {
        self.progress
    }

    pub fn set_progress(&mut self, prog: f64) {
        if (prog - 1.0).abs() < f64::EPSILON {
            self.finished_time = Some(self.instant.elapsed());
        }
        self.progress = prog;
    }

    pub fn reset(&mut self) {
        self.progress = 0.0;
        self.instant = Instant::now();
        self.finished_time = None;
    }

    pub fn get_elapsed_time(&self) -> f64 {
        if (self.progress - 1.0).abs() < f64::EPSILON {
            self.finished_time.unwrap().as_secs_f64()
        } else {
            self.instant.elapsed().as_secs_f64()
        }
    }

    pub fn get_remaining_time(&self) -> f64 {
        if (self.progress - 1.0).abs() < f64::EPSILON {
            return 0.0;
        }
        let time_diff = self.instant.elapsed().as_secs_f64();

        time_diff / self.progress - self.get_elapsed_time()
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}
