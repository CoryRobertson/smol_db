use std::fmt::{Display, Formatter};

/// The type of a log entry
/// Error represents a log message that needs attention
/// Warn represents a log message that might or might not matter
/// Info represents simple logging info for record keeping purposes
pub enum LogLevel {
    Error,
    Warn,
    Info,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => {
                write!(f, "Error")
            }
            LogLevel::Warn => {
                write!(f, "Warn")
            }
            LogLevel::Info => {
                write!(f, "Info")
            }
        }
    }
}
