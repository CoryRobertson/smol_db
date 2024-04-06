//! Client encryption module

use crate::db_packets::db_packet::DBPacket;
use crate::encryption::encrypted_data::EncryptedData;
use crate::encryption::{decrypt, EncryptionError, BIT_LENGTH};
use crate::prelude::{DBPacketResponseError, DBSuccessResponse};
use rsa::rand_core::OsRng;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use tracing::{error, info};

#[derive(Debug)]
/// A client rsa key pair, along with a server public key used for end to end encryption
pub struct ClientKey {
    pri_key: RsaPrivateKey,
    pub_key: RsaPublicKey,
    server_pub_key: RsaPublicKey,
    rng: OsRng,
}

impl ClientKey {
    #[tracing::instrument]
    pub fn new(server_pub_key: RsaPublicKey) -> Result<Self, rsa::Error> {
        info!("Generating client key from server public key");
        let mut rng = OsRng;
        let pri_key = RsaPrivateKey::new(&mut rng, BIT_LENGTH)?;
        let pub_key = pri_key.to_public_key();

        Ok(Self {
            pri_key,
            pub_key,
            server_pub_key,
            rng,
        })
    }

    /// Get the clients public key
    #[tracing::instrument]
    pub fn get_pub_key(&self) -> &RsaPublicKey {
        &self.pub_key
    }

    /// Encrypt a packet to be sent to the server
    #[tracing::instrument]
    pub fn encrypt_packet(&mut self, packet: &DBPacket) -> Result<DBPacket, EncryptionError> {
        let serialized_data = packet
            .serialize_packet()
            .map_err(|_| EncryptionError::SerializationError)?;
        let encrypted_data = self
            .encrypt(serialized_data.as_bytes())
            .map_err(EncryptionError::RSAError)?;
        let enc_struct = EncryptedData::new(encrypted_data.as_slice());
        Ok(DBPacket::Encrypted(enc_struct))
    }

    /// Decrypt a packet received from the server on the client
    #[tracing::instrument(skip_all)]
    pub fn decrypt_server_packet(
        &self,
        server_db_response: &[u8],
    ) -> Result<Result<DBSuccessResponse<String>, DBPacketResponseError>, EncryptionError> {
        let msg = decrypt(&self.pri_key, server_db_response).map_err(EncryptionError::RSAError)?;
        match serde_json::from_slice(&msg) {
            Ok(packet) => {
                info!("Successfully decrypted packet");
                Ok(packet)
            }
            Err(e) => {
                error!("Error deserializing encrypted packet from server: {}", e);
                Err(EncryptionError::SerializationError)
            }
        }
    }

    /// Decrypt data sent from the server encrypted with client public key using client private key
    /// This function is used when decrypting data sent from server -> client
    #[tracing::instrument]
    pub fn decrypt(&self, msg: &[u8]) -> rsa::Result<Vec<u8>> {
        self.pri_key.decrypt(Pkcs1v15Encrypt, msg)
    }

    /// Encrypt data to be sent to the server using the servers public key
    /// This function is used when encrypting data sent from client -> server
    #[tracing::instrument]
    pub fn encrypt(&mut self, msg: &[u8]) -> rsa::Result<Vec<u8>> {
        crate::encryption::encrypt(&self.server_pub_key, &mut self.rng, msg)
    }
}
