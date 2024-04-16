use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBKeyedListLocation {
    location: String,
    index: Option<usize>,
}

impl DBKeyedListLocation {
    pub fn new(index: usize, location: &str) -> Self {
        Self {
            location: location.to_string(),
            index: Some(index),
        }
    }

    pub fn get_index(&self) -> Option<usize> {
        self.index
    }

    pub fn get_key(&self) -> &str {
        self.location.as_str()
    }
}