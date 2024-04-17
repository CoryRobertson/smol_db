use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBKeyedListLocation {
    location: String,
    index: Option<usize>,
}

impl DBKeyedListLocation {
    #[must_use]
    pub fn new(index: Option<usize>, location: &str) -> Self {
        Self {
            location: location.to_string(),
            index,
        }
    }

    #[must_use]
    pub fn get_index(&self) -> Option<usize> {
        self.index
    }

    #[must_use]
    pub fn get_key(&self) -> &str {
        self.location.as_str()
    }
}
