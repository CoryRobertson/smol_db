//! Contains the struct and implementations for specific data points within a database.
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct that contains the data that is to be put into a database.
pub struct DBData {
    data: String,
}

impl Display for DBData {
    #[tracing::instrument(skip_all)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl From<String> for DBData {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
impl From<&str> for DBData {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}
impl From<&String> for DBData {
    fn from(value: &String) -> Self {
        Self::from(value.to_string())
    }
}

impl DBData {
    /// Function to create a new `DBData` struct for a `DBPacket::Write` packet.
    #[must_use]
    pub const fn new(data: String) -> Self {
        Self { data }
    }

    /// Getter function for the data inside the `DBData` struct.
    #[tracing::instrument]
    pub fn get_data(&self) -> &str {
        &self.data
    }
}
