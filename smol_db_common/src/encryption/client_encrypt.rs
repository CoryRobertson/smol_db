//! Client encryption module

use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use rsa::rand_core::OsRng;
use crate::db_packets::db_packet::DBPacket;
use crate::encryption::{BIT_LENGTH, EncryptionError};
use crate::encryption::encrypted_data::EncryptedData;

pub struct ClientKey{
    pri_key: RsaPrivateKey,
    pub_key: RsaPublicKey,
    server_pub_key: RsaPublicKey,
    rng: OsRng,
}

impl ClientKey {
    pub fn new(server_pub_key: RsaPublicKey) -> Result<Self,rsa::Error> {
        let mut rng = OsRng::default();
        let pri_key = RsaPrivateKey::new(&mut rng,BIT_LENGTH)?;
        let pub_key = pri_key.to_public_key();
        Ok(Self { pri_key, pub_key, server_pub_key, rng })
    }

    /// Get the clients public key
    pub fn get_pub_key(&self) -> &RsaPublicKey {
        &self.pub_key
    }

    /// Encrypt a packet to be sent to the server
    pub fn encrypt_packet(&mut self, packet: &DBPacket) -> Result<DBPacket,EncryptionError> {
        let serialized_data = packet.serialize_packet().map_err(|err| EncryptionError::SerializationError)?;
        let encrypted_data = self.encrypt(serialized_data.as_bytes()).map_err(|err| EncryptionError::RSAError(err))?;
        let enc_struct = EncryptedData::new(encrypted_data.as_slice());
        Ok(DBPacket::Encrypted(enc_struct))
    }

    /// Decrypt a packet received from the server
    pub fn decrypt_packet(&self, packet: &EncryptedData) -> Result<DBPacket,EncryptionError> {
        crate::encryption::decrypt_packet(packet,&self.pri_key)
    }

    /// Decrypt data sent from the server encrypted with client public key using client private key
    /// This function is used when decrypting data sent from server -> client
    pub fn decrypt(&self, msg: &[u8]) -> rsa::Result<Vec<u8>> {
        self.pri_key.decrypt(Pkcs1v15Encrypt,msg)
    }

    /// Encrypt data to be sent to the server using the servers public key
    /// This function is used when encrypting data sent from client -> server
    pub fn encrypt(&mut self, msg: &[u8]) -> rsa::Result<Vec<u8>> {
        crate::encryption::encrypt(&self.server_pub_key, &mut self.rng, msg)
    }

}