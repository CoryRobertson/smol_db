//! Contains the struct representing the content structure of a database, which is a hashmap.
use crate::db_data::DBData;
use crate::db_packets::db_keyed_list_location::DBKeyedListLocation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

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

    #[tracing::instrument(skip(self))]
    pub fn get_list_from_key(&self, key: &str) -> Option<&[String]> {
        self.keyed_list.get(key).map(Vec::as_slice)
    }

    #[tracing::instrument(skip(self))]
    pub fn get_data_from_list(&self, key: &DBKeyedListLocation) -> Option<&String> {
        self.keyed_list
            .get(key.get_key())
            .and_then(|list| key.get_index().and_then(|idx| list.get(idx)))
    }

    #[tracing::instrument(skip(self))]
    pub fn remove_data_from_list(&mut self, key: &DBKeyedListLocation) -> Option<String> {
        let removed = self.keyed_list.get_mut(key.get_key()).and_then(|list| {
            match key.get_index() {
                None => {
                    list.pop()
                }
                Some(idx) => {
                    if list.len() > idx {
                        Some(list.remove(idx))
                    } else {
                        warn!("Attempted to remove index from list when index was out of bounds from list length: {}, using index: {}", list.len(),idx);
                        None
                    }
                }
            }
        });

        // If a list is now empty we can remove the list from the database entirely, since its dead weight now
        // the second check is mainly a sanity check rather than needed
        if removed.is_none()
            || self
                .keyed_list
                .get(key.get_key())
                .is_some_and(Vec::is_empty)
        {
            info!("Database list deleted since it was made empty");
            self.keyed_list.remove(key.get_key());
        }

        removed
    }

    #[tracing::instrument(skip(self))]
    pub fn add_data_to_list(&mut self, key: &DBKeyedListLocation, data: DBData) {
        match self.keyed_list.get_mut(key.get_key()) {
            None => {
                info!("Created new list since it did not exist yet");
                self.keyed_list
                    .insert(key.get_key().to_string(), vec![data.to_string()]);
            }
            Some(list) => match key.get_index() {
                None => {
                    list.push(data.to_string());
                }
                Some(idx) => {
                    list.insert(idx, data.to_string());
                }
            },
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn clear_list(&mut self, key: &DBKeyedListLocation) -> bool {
        self.keyed_list.remove(key.get_key()).map_or_else(
            || {
                warn!("Could not clear list, as it did not exist");
                false
            },
            |list| {
                debug!("{:?}", list);
                true
            },
        )
    }

    #[tracing::instrument(skip(self))]
    pub fn get_length_of_list(&self, key: &DBKeyedListLocation) -> Option<usize> {
        self.keyed_list.get(key.get_key()).map(Vec::len)
    }

    /// Reads from the db using the key, returning an optional of either the retrieved content, or nothing.
    #[tracing::instrument(skip(self))]
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
            keyed_list: HashMap::default(),
        }
    }
}
