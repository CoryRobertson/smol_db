use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// A struct that describes the name of a database to be searched through.
pub struct DBPacketInfo {
    dbname: String,
}

impl Display for DBPacketInfo {
    #[tracing::instrument(skip_all)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dbname)
    }
}

impl From<String> for DBPacketInfo {
    fn from(value: String) -> Self {
        Self::new(value.as_str())
    }
}
impl From<&String> for DBPacketInfo {
    fn from(value: &String) -> Self {
        Self::new(value)
    }
}
impl From<&str> for DBPacketInfo {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl DBPacketInfo {
    /// Function to create a new `DBPacketInfo` struct with the given name
    #[must_use]
    pub fn new(dbname: &str) -> Self {
        Self {
            dbname: dbname.to_string(),
        }
    }

    /// Function to retrieve the name from the `DBPacketInfo` struct.
    #[must_use]
    pub fn get_db_name(&self) -> &str {
        &self.dbname
    }
}
