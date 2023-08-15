use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize,Deserialize)]
#[non_exhaustive]
pub struct DBStatistics {
    total_requests: AtomicU64,
    current_average_time: Mutex<f32>,
    client_list: Vec<String>,
    // avg = ((current average time * num of reqs) + new time) / total number of reqs
}

impl DBStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_avg_time(&self) -> f32 {
        self.current_average_time.lock().unwrap().clone()
    }

    pub fn get_total_req(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    pub fn add_new_time(&self, last_access_time: SystemTime) {
        self.add_avg_time(SystemTime::now().duration_since(last_access_time).unwrap().as_secs_f32());
    }

    fn add_avg_time(&self, new_time: f32) {
        let mut cur_avg = self.current_average_time.lock().unwrap();
        let cur_total = self.total_requests.fetch_add(1,Ordering::Relaxed);
        let new_avg = ((*cur_avg * cur_total as f32) + new_time) / (cur_total as f32 + 1.0);
        *cur_avg = new_avg;
    }

}

impl Clone for DBStatistics {
    fn clone(&self) -> Self {
        Self {
            total_requests: self.total_requests.load(Ordering::Relaxed).into(),
            current_average_time: Mutex::new(*self.current_average_time.lock().unwrap()),
            client_list: self.client_list.clone(),
        }
    }
}

impl Default for DBStatistics {
    fn default() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            current_average_time: Mutex::new(0.0),
            client_list: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::statistics::DBStatistics;

    #[test]
    fn test_avg() {
        let s = DBStatistics::default();

        let mut total;
        let mut sum = 0;
        let mut avg;

        for (index,num) in (0..10_000).into_iter().enumerate() {
            total = index + 1;
            sum += num;
            avg = sum as f32 / total as f32;
            s.add_avg_time(num as f32);
            assert!((avg - s.get_avg_time()).abs() <= 0.2, "{}", format!("{} , {}", avg,s.get_avg_time()));
        }
    }
}