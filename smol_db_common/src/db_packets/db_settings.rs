use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Struct describing settings used when creating a db.
pub struct DBSettings {
    /// The duration to wait before removing the given db from the cache.
    invalidation_time: Duration,
}

impl DBSettings {
    /// Returns a new DBSettings given a duration
    pub fn new(invalidation_time: Duration) -> Self {
        Self { invalidation_time }
    }

    /// Returns the invalidation time duration
    pub fn get_invalidation_time(&self) -> Duration {
        self.invalidation_time
    }
}
