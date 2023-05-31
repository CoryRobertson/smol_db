use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// A struct that describes the name of a database to be searched through.
pub struct DBPacketInfo {
    dbname: String,
}

impl DBPacketInfo {
    /// Function to create a new DBPacketInfo struct with the given name
    pub fn new(dbname: &str) -> Self {
        Self {
            dbname: dbname.to_string(),
        }
    }

    /// Function to retrieve the name from the DBPacketInfo struct.
    pub fn get_db_name(&self) -> &str {
        &self.dbname
    }
}
