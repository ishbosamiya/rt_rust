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

    pub fn stop_progress(&mut self) {
        // set finished time, but do not set progress to 1.0
        self.finished_time = Some(self.instant.elapsed());
    }

    pub fn get_elapsed_duration(&self) -> Duration {
        if let Some(finished_time) = self.finished_time {
            // if progress has finished, return finished time
            finished_time
        } else {
            self.instant.elapsed()
        }
    }

    pub fn get_elapsed_time(&self) -> f64 {
        self.get_elapsed_duration().as_secs_f64()
    }

    pub fn get_remaining_time(&self) -> f64 {
        if let Some(_finished_time) = self.finished_time {
            // if progress has finished, remainging time is 0
            0.0
        } else {
            let time_diff = self.instant.elapsed().as_secs_f64();

            time_diff / self.progress - self.get_elapsed_time()
        }
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}
