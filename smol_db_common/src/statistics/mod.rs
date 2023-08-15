use std::time::SystemTime;
use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize,Deserialize,Clone)]
#[non_exhaustive]
pub struct DBStatistics {
    total_requests: u64,
    current_average_time: f32,
    // avg = ((current average time * num of reqs) + new time) / total number of reqs
}

impl DBStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_avg_time(&self) -> f32 {
        self.current_average_time
    }

    pub fn get_total_req(&self) -> u64 {
        self.total_requests
    }

    pub fn add_new_time(&mut self, last_access_time: SystemTime) {
        self.add_avg_time(SystemTime::now().duration_since(last_access_time).unwrap().as_secs_f32());
    }

    fn add_avg_time(&mut self, new_time: f32) {
        let cur_avg = self.current_average_time;
        let cur_total = self.total_requests;
        let new_avg = ((cur_avg * cur_total as f32) + new_time) / (cur_total as f32 + 1.0);
        self.current_average_time = new_avg;
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
    use crate::statistics::DBStatistics;

    #[test]
    fn test_avg() {
        let mut s = DBStatistics::default();

        let mut total;
        let mut sum = 0;
        let mut avg;

        for (index,num) in (0..10_000).into_iter().enumerate() {
            total = index + 1;
            sum += num;
            avg = sum as f32 / total as f32;
            s.add_avg_time(num as f32);
            assert!((avg - s.get_avg_time()).abs() <= 0.2, "{}", format!("[{index}]: {} , {}", avg,s.get_avg_time()));
        }
    }
}