//! Contains the implementation and structure of `DBStatistics`, used as a feature in a `DB`
#[cfg(feature = "statistics")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "statistics")]
use std::time::SystemTime;
use crate::statistics::previous_time_diff::PreviousTimeDifferences;

mod previous_time_diff;
pub(self) const MIN_TIME_DIFFERENCE: f32 = 0.25;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
#[cfg(feature = "statistics")]
/// A struct representing the statistics stored from a `DB`
/// Uses rolling average for access time
pub struct DBStatistics {
    /// The total number of requests that have been through the `DB`
    total_requests: u64,
    /// The average time between requests on the given `DB`, each request time must be larger than `MIN_TIME_DIFFERENCE`
    #[serde(default)]
    rolling_average: PreviousTimeDifferences,
}

#[cfg(feature = "statistics")]
impl DBStatistics {
    pub fn new(rolling_average_length: u32) -> Self {
        Self{ total_requests: 0, rolling_average: PreviousTimeDifferences::new(rolling_average_length) }
    }

    /// Returns the average time between requests from the given `DB`
    pub fn get_avg_time(&self) -> f32 {
        self.rolling_average.get_rolling_average()
    }

    /// Returns the total number of requests the given `DB` has
    pub fn get_total_req(&self) -> u64 {
        self.total_requests
    }

    /// Adds the given system time to the average, provided it is below the `MIN_TIME_DIFFERENCE`
    /// If so, the `current_average_time` is updated as well as the `total_requests`
    pub fn add_new_time(&mut self, last_access_time: SystemTime) {
        if let Ok(dur) = SystemTime::now().duration_since(last_access_time) {
            self.rolling_average.add_new_time(dur);
            self.total_requests += 1;
        }
    }

}

#[cfg(feature = "statistics")]
#[allow(clippy::derivable_impls)]
impl Default for DBStatistics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            rolling_average: PreviousTimeDifferences::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    #[cfg(feature = "statistics")]
    use crate::statistics::DBStatistics;

    #[test]
    #[cfg(feature = "statistics")]
    fn test_avg() {
        let mut s = DBStatistics::new(10_000);

        let mut total;
        let mut sum = 0;
        let mut avg;

        for (index, num) in (0..10_000).into_iter().enumerate() {
            total = index + 1;
            sum += num;
            avg = sum as f32 / total as f32;
            s.rolling_average.add_new_time(Duration::from_secs_f32(num as f32));
            s.total_requests += 1;
            assert!(
                (avg - s.get_avg_time()).abs() <= 0.5,
                "{}",
                format!("[{index}]: {} , {}", avg, s.get_avg_time())
            );
            assert_eq!(s.get_total_req(), (index + 1) as u64);
        }
    }
}
