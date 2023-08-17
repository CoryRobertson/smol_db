//! Module containing a struct that records the time measured at every request
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use chrono::{DateTime, Local};

#[derive(Debug, Serialize, Deserialize, Clone)]
/// A list of times that the database carrying this statistics struct has had users connect at
/// The most recent connection time is at the end of the list
pub(super) struct UsageTimeList {
    list: Vec<DateTime<Local>>,
    max_list_length: usize,
}

impl UsageTimeList {
    pub fn new(max_list_length: usize) -> Self {
        Self {
            list: vec![],
            max_list_length,
        }
    }

    /// Add a `SystemTime` to the list, removing the oldest entry if the length exceeds the maximum length
    pub fn add_time(&mut self, time: SystemTime) {
        self.list.push(time.into());
        if self.list.len() > self.max_list_length {
            self.list.remove(0);
        }
    }

    /// Return the list of `SystemTime` that have been recorded
    pub fn get_list(&self) -> &Vec<DateTime<Local>> {
        &self.list
    }

    /// Return the maximum number of stored system times
    #[allow(dead_code)]
    pub fn get_max_length(&self) -> usize {
        self.max_list_length
    }
}

impl Default for UsageTimeList {
    fn default() -> Self {
        Self::new(30)
    }
}
