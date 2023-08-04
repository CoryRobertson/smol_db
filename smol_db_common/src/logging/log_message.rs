use std::fmt::{Display, Formatter};

/// The text component to a log entry
pub struct LogMessage(String);

impl LogMessage {
    pub fn new(text: &str) -> Self {
        Self(text.to_string())
    }
}

impl Display for LogMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
