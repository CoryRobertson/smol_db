//! Module containing a private struct for calculating rolling average of access times by the statistics struct
use crate::statistics::MIN_TIME_DIFFERENCE;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct PreviousTimeDifferences {
    list: Vec<Duration>,
    rolling_average_max: u32,
}

impl PreviousTimeDifferences {
    pub const fn new(rolling_average_max: u32) -> Self {
        Self {
            list: vec![],
            rolling_average_max,
        }
    }

    #[allow(dead_code)]
    #[tracing::instrument]
    pub fn get_limit(&self) -> u32 {
        self.rolling_average_max
    }

    /// Adds a new time to the rolling average
    #[tracing::instrument]
    pub fn add_new_time(&mut self, time: Duration) {
        if time.as_secs_f32() >= MIN_TIME_DIFFERENCE {
            self.list.push(time);
            if self.list.len() > self.rolling_average_max as usize {
                self.list.remove(0);
            }
        }
    }

    #[tracing::instrument]
    pub fn get_rolling_average(&self) -> f32 {
        if self.list.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.list.iter().map(Duration::as_secs_f32).sum();
        sum / self.list.len() as f32
    }
}

impl Default for PreviousTimeDifferences {
    #[tracing::instrument]
    fn default() -> Self {
        Self::new(100)
    }
}
