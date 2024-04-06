//! Contains various error enums that a client may return on an operation with a database
use smol_db_common::db_packets::db_packet_response::DBPacketResponseError;
use smol_db_common::encryption::EncryptionError;
use std::io::Error;

#[derive(Debug)]
/// Enum that represents the possible outcomes of using the client
pub enum ClientError {
    /// SmolDbClient was not able to connect to the database.
    UnableToConnect(Error),
    /// SmolDbClient was unable to serialize the given data to be sent to the database.
    PacketSerializationError(Error),
    /// SmolDbClient was unable to write to the socket, connection might be faulty.
    SocketWriteError(Error),
    /// SmolDbClient was unable to read from the socket, connection might be faulty.
    SocketReadError(Error),
    /// SmolDbClient was unable to deserialize the data from the server, the server might have stored a different type of data at the given location than was expected.
    PacketDeserializationError(Error),
    /// SmolDbClient was successful in contacting the database, but the database returned an error, check the given error inside.
    DBResponseError(DBPacketResponseError),
    /// SmolDbClient received the incorrect packet from a response, this should not happen.
    BadPacket,
    /// Encryption failed either in decrypting a packet, or encrypting a packet
    PacketEncryptionError(EncryptionError),
    /// The server did not respond as expected when encryption was requested
    EncryptionSetupError,
    /// Generating a key pair produced an error
    KeyGenerationError(smol_db_common::prelude::Error),
}

impl PartialEq for ClientError {
    #[tracing::instrument]
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::UnableToConnect(_) => {
                matches!(other, Self::UnableToConnect(_))
            }
            Self::PacketSerializationError(_) => {
                matches!(other, Self::PacketSerializationError(_))
            }
            Self::SocketWriteError(_) => {
                matches!(other, Self::SocketWriteError(_))
            }
            Self::SocketReadError(_) => {
                matches!(other, Self::SocketReadError(_))
            }
            Self::PacketDeserializationError(_) => {
                matches!(other, Self::PacketDeserializationError(_))
            }
            Self::DBResponseError(_) => {
                matches!(other, Self::DBResponseError(_))
            }
            Self::BadPacket => {
                matches!(other, Self::BadPacket)
            }
            Self::PacketEncryptionError(_) => {
                matches!(other, Self::PacketEncryptionError(_))
            }
            Self::EncryptionSetupError => {
                matches!(other, Self::EncryptionSetupError)
            }
            Self::KeyGenerationError(_) => {
                matches!(other, Self::KeyGenerationError(_))
            }
        }
    }
}
