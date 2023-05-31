//! Contains the struct and implementations for specific data points within a database.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct that contains the data that is to be put into a database.
pub struct DBData {
    data: String,
    // TODO: the data field should really be a generic that implements Serialize and Deserialize
}

impl DBData {
    /// Function to create a new DBData struct for a DBPacket::Write packet.
    pub fn new(data: String) -> Self {
        // TODO: eventually revise this with some amount of error checking
        Self { data }
    }

    /// Getter function for the data inside the DBData struct.
    pub fn get_data(&self) -> &str {
        &self.data
    }
}
