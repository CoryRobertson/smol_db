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

impl DBData {
    /// Function to create a new `DBData` struct for a `DBPacket::Write` packet.
    pub const fn new(data: String) -> Self {
        Self { data }
    }

    /// Getter function for the data inside the `DBData` struct.
    #[tracing::instrument]
    pub fn get_data(&self) -> &str {
        &self.data
    }
}
