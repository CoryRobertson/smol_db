//! Contains the implementation and structure of `DBStatistics`, used as a feature in a `DB`
#[cfg(feature = "statistics")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "statistics")]
use std::time::SystemTime;

const MIN_TIME_DIFFERENCE: f32 = 0.25;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
/// A struct representing the statistics stored from a `DB`
pub struct DBStatistics {
    /// The total number of requests that have been through the `DB`
    total_requests: u64,
    /// The average time between requests on the given `DB`, each request time must be larger than `MIN_TIME_DIFFERENCE`
    current_average_time: f32,
    // avg = ((current average time * num of reqs) + new time) / total number of reqs
}

impl DBStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the average time between requests from the given `DB`
    pub fn get_avg_time(&self) -> f32 {
        self.current_average_time
    }

    /// Returns the total number of requests the given `DB` has
    pub fn get_total_req(&self) -> u64 {
        self.total_requests
    }

    /// Adds the given system time to the average, provided it is below the `MIN_TIME_DIFFERENCE`
    /// If so, the `current_average_time` is updated as well as the `total_requests`
    pub fn add_new_time(&mut self, last_access_time: SystemTime) {
        self.add_avg_time(
            SystemTime::now()
                .duration_since(last_access_time)
                .unwrap()
                .as_secs_f32(),
        );
    }

    fn add_avg_time(&mut self, new_time_difference: f32) {
        if new_time_difference >= MIN_TIME_DIFFERENCE {
            let cur_avg = self.current_average_time;
            let cur_total = self.total_requests;
            let new_avg =
                cur_avg.mul_add(cur_total as f32, new_time_difference) / (cur_total as f32 + 1.0);
            self.current_average_time = new_avg;
        }
        self.total_requests += 1;
    }
}

impl Default for DBStatistics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            current_average_time: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "statistics")]
    use crate::statistics::DBStatistics;

    #[test]
    #[cfg(feature = "statistics")]
    fn test_avg() {
        let mut s = DBStatistics::default();

        let mut total;
        let mut sum = 0;
        let mut avg;

        for (index, num) in (0..10_000).into_iter().enumerate() {
            total = index + 1;
            sum += num;
            avg = sum as f32 / total as f32;
            s.add_avg_time(num as f32);
            assert!(
                (avg - s.get_avg_time()).abs() <= 0.2,
                "{}",
                format!("[{index}]: {} , {}", avg, s.get_avg_time())
            );
            assert_eq!(s.get_total_req(), (index + 1) as u64);
        }
    }
}
