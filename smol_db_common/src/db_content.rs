//! Contains the struct representing the content structure of a database, which is a hashmap.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Struct denoting the content structure itself of a database. Which is a hash map.
pub struct DBContent<T> {
    pub content: HashMap<String, T>,
}

impl<T> DBContent<T>
    where for<'a> T: Serialize + Deserialize<'a>,
{
    /// Reads serialized version of a DBContent struct from a string (read from a file most likely) into a DBContent struct itself.
    pub fn read_ser_data(data: String) -> serde_json::Result<Self> {
        serde_json::from_str(&data)
    }

    /// Reads from the db using the key, returning an optional of either the retrieved content, or nothing.
    pub fn read_from_db(&self, key: &str) -> Option<&T> {
        self.content.get(key)
    }
}

#[allow(clippy::derivable_impls)] // This lint is allowed so we can later make default not simply have the default impl
impl<T> Default for DBContent<T> {
    /// Returns a default empty HashMap.
    fn default() -> Self {
        Self {
            content: HashMap::default(),
        }
    }
}
