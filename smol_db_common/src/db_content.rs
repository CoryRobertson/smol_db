//! Contains the struct representing the content structure of a database, which is a hashmap.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::db_data::DBData;
use crate::db_packets::db_keyed_list_location::DBKeyedListLocation;

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Struct denoting the content structure itself of a database. Which is a hash map.
pub struct DBContent {
    pub content: HashMap<String, String>,
    keyed_list: HashMap<String, Vec<String>>,
}

impl DBContent {
    /// Reads serialized version of a `DBContent` struct from a string (read from a file most likely) into a `DBContent` struct itself.
    #[tracing::instrument]
    pub fn read_ser_data(data: &str) -> serde_json::Result<Self> {
        serde_json::from_str(data)
    }

    pub fn get_list_from_key(&self, key: &str) -> Option<&[String]> {
        self.keyed_list.get(key).map(|list| list.as_slice())
    }

    pub fn get_data_from_list(&self, key: &DBKeyedListLocation) -> Option<&String> {
        self.keyed_list.get(key.get_key()).and_then(|list| {
            key.get_index().and_then(|idx| {list.get(idx)})
        })
    }

    pub fn remove_data_from_list(&mut self, key: &DBKeyedListLocation) -> Option<String> {
        self.keyed_list.get_mut(key.get_key()).and_then(|list| {
            match key.get_index() {
                None => {
                    list.pop()
                }
                Some(idx) => {
                    if list.len() > idx {
                        Some(list.remove(idx))
                    } else {
                        None
                    }
                }
            }
        })
    }

    pub fn add_data_to_list(&mut self, key: &DBKeyedListLocation, data: DBData) -> bool {
        match self.keyed_list.get_mut(key.get_key()) {
            None => {
                self.keyed_list.insert(key.get_key().to_string(),vec![data.to_string()]);
                true
            }
            Some(list) => {
                match key.get_index() {
                    None => {
                        list.push(data.to_string());
                    }
                    Some(idx) => {
                        list.insert(idx,data.to_string());
                    }
                }
                true
            }
        }
    }


    /// Reads from the db using the key, returning an optional of either the retrieved content, or nothing.
    #[tracing::instrument]
    pub fn read_from_db(&self, key: &str) -> Option<&String> {
        self.content.get(key)
    }
}

#[allow(clippy::derivable_impls)] // This lint is allowed so we can later make default not simply have the default impl
impl Default for DBContent {
    /// Returns a default empty `HashMap`.
    #[tracing::instrument]
    fn default() -> Self {
        Self {
            content: HashMap::default(),
            keyed_list: Default::default(),
        }
    }
}
