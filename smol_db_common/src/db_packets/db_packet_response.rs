#![allow(deprecated)]
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[deprecated]
/// Represents the various types of responses that accessing the database can be.
pub enum DBPacketResponse<T> {
    /// DBPacketResponse is a response type for a DBPacket request
    SuccessNoData,
    /// SuccessNoData represents when the operation was successful, but no response data was necessary to be replied back.
    SuccessReply(T),
    /// Error represents any issue when interacting with a database in general, it contains a further description of the error inside.
    Error(DBPacketResponseError),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
/// Represents the various types of successful responses that accessing the database can be.
pub enum DBSuccessResponse<T> {
    /// SuccessNoData represents when the operation was successful, but no response data was necessary to be replied back.
    SuccessNoData,
    /// SuccessReply represents when the operation was successful, and there is data to be replied back
    SuccessReply(T),
}

impl<T> From<DBSuccessResponse<T>> for Option<T> {
    #[tracing::instrument(skip_all)]
    fn from(value: DBSuccessResponse<T>) -> Self {
        match value {
            DBSuccessResponse::SuccessNoData => None,
            DBSuccessResponse::SuccessReply(data) => Some(data),
        }
    }
}

#[allow(dead_code)]
impl<T> DBSuccessResponse<T> {
    #[tracing::instrument(skip_all)]
    pub fn into_option(self) -> Option<T> {
        match self {
            Self::SuccessNoData => None,
            Self::SuccessReply(data) => Some(data),
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn as_option(&self) -> Option<&T> {
        match self {
            Self::SuccessNoData => None,
            Self::SuccessReply(data) => Some(data),
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn as_option_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::SuccessNoData => None,
            Self::SuccessReply(data) => Some(data),
        }
    }
}

impl<T> Display for DBSuccessResponse<T>
where
    T: Display,
{
    #[tracing::instrument(skip_all)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SuccessNoData => {
                write!(f, "SuccessNoData")
            }
            Self::SuccessReply(reply) => {
                write!(f, "SuccessReply: {}", reply)
            }
        }
    }
}

#[allow(deprecated)]
impl<T> Display for DBPacketResponse<T>
where
    T: Display,
{
    #[tracing::instrument(skip_all)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SuccessNoData => {
                write!(f, "SuccessNoData")
            }
            Self::SuccessReply(reply) => {
                write!(f, "SuccessReply: {}", reply)
            }
            Self::Error(err) => {
                write!(f, "Error: {}", err)
            }
        }
    }
}

impl Display for DBPacketResponseError {
    #[tracing::instrument(skip_all)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq)]
/// Represents the various types of errors that can occur when an error is returned in a db packet response
pub enum DBPacketResponseError {
    /// BadPacket represents a packet that was improperly handled, these should be reported immediately and should never happen under proper circumstances.
    BadPacket,
    /// DBNotFound represents a request to a database that does not exist.
    DBNotFound,
    /// DBFileSystemError represents an issue loading or reading the file that contains a given database, not necessarily it not existing.
    DBFileSystemError,
    /// ValueNotFound represents when a given value in a database does not exist.
    ValueNotFound,
    /// DBAlreadyExists represents when attempting to create a database fails because that database already exists either as a file or in the db list.
    DBAlreadyExists,
    /// An error occurred during serialization, specifically not during deserialization, but during serialization. This should never happen.
    SerializationError,
    /// An error occurred during deserialization, data could have been dropped during transmission, or an unexpected or malformed packet was received.
    DeserializationError,
    /// The client issuing the command does not have the required permissions to this data or operation
    InvalidPermissions,
    /// A user was attempted to be read, and was not found
    UserNotFound,
}

#[allow(deprecated)]
impl<T> DBPacketResponse<T> {
    /// Convert the response from the database to a result
    pub fn as_result(&self) -> Result<Option<&T>, &DBPacketResponseError> {
        match self {
            Self::SuccessNoData => Ok(None),
            Self::SuccessReply(data) => Ok(Some(data)),
            Self::Error(err) => Err(err),
        }
    }

    /// Consume the response and convert into a result
    pub fn into_result(self) -> Result<Option<T>, DBPacketResponseError> {
        match self {
            Self::SuccessNoData => Ok(None),
            Self::SuccessReply(data) => Ok(Some(data)),
            Self::Error(err) => Err(err),
        }
    }
}
