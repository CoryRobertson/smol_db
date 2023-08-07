use smol_db_common::db_packets::db_packet_response::DBPacketResponseError;
use std::io::Error;

#[derive(Debug)]
/// Enum that represents the possible outcomes of using the client
pub enum ClientError {
    /// Client was not able to connect to the database.
    UnableToConnect(Error),
    /// Client was unable to serialize the given data to be sent to the database.
    PacketSerializationError(Error),
    /// Client was unable to write to the socket, connection might be faulty.
    SocketWriteError(Error),
    /// Client was unable to read from the socket, connection might be faulty.
    SocketReadError(Error),
    /// Client was unable to deserialize the data from the server, the server might have stored a different type of data at the given location than was expected.
    PacketDeserializationError(Error),
    /// Client was successful in contacting the database, but the database returned an error, check the given error inside.
    DBResponseError(DBPacketResponseError),
    /// Client received the incorrect packet from a response, this should not happen.
    BadPacket,
}

impl PartialEq for ClientError {
    fn eq(&self, other: &Self) -> bool {
        match self {
            ClientError::UnableToConnect(_) => {
                matches!(other, ClientError::UnableToConnect(_))
            }
            ClientError::PacketSerializationError(_) => {
                matches!(other, ClientError::PacketSerializationError(_))
            }
            ClientError::SocketWriteError(_) => {
                matches!(other, ClientError::SocketWriteError(_))
            }
            ClientError::SocketReadError(_) => {
                matches!(other, ClientError::SocketReadError(_))
            }
            ClientError::PacketDeserializationError(_) => {
                matches!(other, ClientError::PacketDeserializationError(_))
            }
            ClientError::DBResponseError(_) => {
                matches!(other, ClientError::DBResponseError(_))
            }
            ClientError::BadPacket => {
                matches!(other, ClientError::BadPacket)
            }
        }
    }
}
