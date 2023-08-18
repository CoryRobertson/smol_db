//! Contains the implementation and structure of `DBStatistics`, used as a feature in a `DB`
use crate::statistics::previous_time_diff::PreviousTimeDifferences;
use crate::statistics::time_of_usage::UsageTimeList;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

mod previous_time_diff;
mod time_of_usage;
pub(self) const MIN_TIME_DIFFERENCE: f32 = 0.25;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
/// A struct representing the statistics stored from a `DB`
/// Uses rolling average for access time
pub struct DBStatistics {
    /// The total number of requests that have been through the `DB`
    total_requests: u64,
    /// The average time between requests on the given `DB`, each request time must be larger than `MIN_TIME_DIFFERENCE`
    #[serde(default)]
    rolling_average: PreviousTimeDifferences,
    /// List of system times recorded at each request, stores a maximum number of system times, but does not have a `MIN_TIME_DIFFERENCE`
    #[serde(default)]
    usage_time_list: UsageTimeList,
}

impl DBStatistics {
    pub fn new(rolling_average_length: u32, usage_list_length: usize) -> Self {
        Self {
            total_requests: 0,
            rolling_average: PreviousTimeDifferences::new(rolling_average_length),
            usage_time_list: UsageTimeList::new(usage_list_length),
        }
    }

    /// Returns the average time between requests from the given `DB`
    pub fn get_avg_time(&self) -> f32 {
        self.rolling_average.get_rolling_average()
    }

    /// Returns the total number of requests the given `DB` has
    pub fn get_total_req(&self) -> u64 {
        self.total_requests
    }

    /// Returns a list of system times that were recorded at a request time in this statistics struct
    pub fn get_usage_time_list(&self) -> &Vec<DateTime<Local>> {
        self.usage_time_list.get_list()
    }

    /// Adds the given system time to the average, provided it is below the `MIN_TIME_DIFFERENCE`
    /// If so, the `current_average_time` is updated as well as the `total_requests`
    pub fn add_new_time(&mut self, last_access_time: SystemTime) {
        if let Ok(dur) = SystemTime::now().duration_since(last_access_time) {
            self.rolling_average.add_new_time(dur);
            self.usage_time_list.add_time(last_access_time);
            self.total_requests += 1;
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for DBStatistics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            rolling_average: PreviousTimeDifferences::default(),
            usage_time_list: UsageTimeList::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "statistics")]
    use crate::statistics::DBStatistics;
    use std::time::Duration;

    #[test]
    fn test_avg() {
        let mut s = DBStatistics::new(10_000, 10);

        let mut total;
        let mut sum = 0;
        let mut avg;

        for (index, num) in (0..10_000).into_iter().enumerate() {
            total = index + 1;
            sum += num;
            avg = sum as f32 / total as f32;
            s.rolling_average
                .add_new_time(Duration::from_secs_f32(num as f32));
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
