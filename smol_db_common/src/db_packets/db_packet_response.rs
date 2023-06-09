use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
/// This enum represents the various types of responses that accessing the database can be.
pub enum DBPacketResponse<T> {
    /// DBPacketResponse is a response type for a DBPacket request
    SuccessNoData,
    /// SuccessNoData represents when the operation was successful, but no response data was necessary to be replied back.
    SuccessReply(T),
    /// Error represents any issue when interacting with a database in general, it contains a further description of the error inside.
    Error(DBPacketResponseError),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// This enum represents the various types of errors that can occur when an error is returned in a db packet response
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

    DeserializationError,

    InvalidPermissions,

    UserNotFound,
}

impl<T> DBPacketResponse<T> {
    /// Convert the response from the database to a result
    pub fn as_result(&self) -> Result<Option<&T>, &DBPacketResponseError> {
        match self {
            DBPacketResponse::SuccessNoData => Ok(None),
            DBPacketResponse::SuccessReply(data) => Ok(Some(data)),
            DBPacketResponse::Error(err) => Err(err),
        }
    }

    /// Consume the response and convert into a result
    pub fn into_result(self) -> Result<Option<T>, DBPacketResponseError> {
        match self {
            DBPacketResponse::SuccessNoData => Ok(None),
            DBPacketResponse::SuccessReply(data) => Ok(Some(data)),
            DBPacketResponse::Error(err) => Err(err),
        }
    }
}
