//! Server encryption module
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use rsa::rand_core::OsRng;
use crate::db_packets::db_packet::DBPacket;
use crate::encryption::{BIT_LENGTH, EncryptionError};
use crate::encryption::encrypted_data::EncryptedData;

#[derive(Debug)]
pub struct ServerKey{
    pri_key: RsaPrivateKey,
    pub_key: RsaPublicKey,
    rng: OsRng,
}

impl Default for ServerKey {
    fn default() -> Self {
        ServerKey::new().unwrap()
    }
}

impl ServerKey {
    pub fn new() -> Result<Self,rsa::Error> {
        let mut rng = OsRng::default();
        let pri_key = RsaPrivateKey::new(&mut rng,BIT_LENGTH)?;
        let pub_key = pri_key.to_public_key();
        Ok(Self { pri_key, pub_key, rng })
    }

    /// Gets public key of server
    pub fn get_pub_key(&self) -> &RsaPublicKey {
        &self.pub_key
    }

    /// Encrypt data using the clients public key
    /// This function is used when encrypting data sent from server -> client
    fn encrypt(&mut self,client_pub_key: &RsaPublicKey ,msg: &[u8]) -> rsa::Result<Vec<u8>> {
        // client_pub_key.encrypt(&mut self.rng,Pkcs1v15Encrypt,msg)
        crate::encryption::encrypt(&client_pub_key, &mut self.rng, msg)
    }

    pub fn encrypt_packet(&mut self, packet: &DBPacket, client_pub_key: &RsaPublicKey) -> Result<DBPacket,EncryptionError> {
        let serialized_data = packet.serialize_packet().map_err(|err| EncryptionError::SerializationError)?;
        let encrypted_data = self.encrypt(client_pub_key,serialized_data.as_bytes()).map_err(|err| EncryptionError::RSAError(err))?;
        let enc_struct = EncryptedData::new(encrypted_data.as_slice());
        Ok(DBPacket::Encrypted(enc_struct))
    }

    pub fn decrypt_packet(&self, packet: &EncryptedData) -> Result<DBPacket,EncryptionError> {
        crate::encryption::decrypt_packet(packet,&self.pri_key)
    }

    /// Decrypt data using the servers private key encrypted with the servers public key
    /// This function is used when decrypting data sent from client -> server
    pub fn decrypt(&self, enc_data: &[u8]) -> rsa::Result<Vec<u8>> {
        crate::encryption::decrypt(&self.pri_key,enc_data)
    }

}