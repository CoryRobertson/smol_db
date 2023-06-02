//! Contains the struct and implementations for specific data points within a database.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct that contains the data that is to be put into a database.
pub struct DBData<T> {
    data: T,
    // TODO: the data field should really be a generic that implements Serialize and Deserialize
}

impl<T> DBData<T>
where for<'a> T: Serialize + Deserialize<'a>,
{
    /// Function to create a new DBData struct for a DBPacket::Write packet.
    pub fn new<'a>(data: T) -> Self {
        // TODO: eventually revise this with some amount of error checking
        Self { data }
    }

    /// Getter function for the data inside the DBData struct.
    pub fn get_data(&self) -> &T {
        &self.data
    }
}
