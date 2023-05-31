use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct that describes a key to search with through the database.
pub struct DBLocation {
    location: String,
}

impl DBLocation {
    /// Function to create a new DBLocation struct from a given location.
    pub fn new(location: &str) -> Self {
        Self {
            location: location.to_string(),
        }
    }

    /// Function to retrieve the location as a key from the struct.
    pub fn as_key(&self) -> &str {
        &self.location
    }
}
