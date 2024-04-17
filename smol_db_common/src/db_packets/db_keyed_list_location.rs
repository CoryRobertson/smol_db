use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBKeyedListLocation {
    location: String,
    index: Option<usize>,
}

impl From<String> for DBKeyedListLocation {
    fn from(value: String) -> Self {
        Self::new(None, value.as_str())
    }
}
impl From<&String> for DBKeyedListLocation {
    fn from(value: &String) -> Self {
        Self::new(None, value)
    }
}
impl From<&str> for DBKeyedListLocation {
    fn from(value: &str) -> Self {
        Self::new(None, value)
    }
}

impl DBKeyedListLocation {
    #[must_use]
    pub fn new(index: Option<usize>, location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
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
