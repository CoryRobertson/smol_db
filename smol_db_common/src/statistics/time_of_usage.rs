//! Module containing a struct that records the time measured at every request
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

const MIN_TIME_DIFFERENCE: i64 = 1;

#[derive(Debug, Serialize, Deserialize, Clone)]
/// A list of times that the database carrying this statistics struct has had users connect at
/// The most recent connection time is at the end of the list
pub(super) struct UsageTimeList {
    list: Vec<DateTime<Local>>,
    max_list_length: usize,
}

impl UsageTimeList {
    pub const fn new(max_list_length: usize) -> Self {
        Self {
            list: vec![],
            max_list_length,
        }
    }

    /// Add a `SystemTime` to the list, removing the oldest entry if the length exceeds the maximum length
    /// Does not add the new time if the time since the last entry and the added entry is less than `MIN_TIME_DIFFERENCE`
    pub fn add_time(&mut self, time: SystemTime) {
        if let Some(date) = self.list.last() {
            let added_date: DateTime<Local> = time.into();
            // early return if the added time is not long enough since the previous time
            if (added_date.timestamp() - date.timestamp()).abs() < MIN_TIME_DIFFERENCE {
                return;
            }
        }
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
