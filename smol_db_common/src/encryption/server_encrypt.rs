//! Server encryption module
use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::rand_core::OsRng;
use crate::db_packets::db_packet::DBPacket;
use crate::encryption::{BIT_LENGTH, decrypt, EncryptionError};
use crate::encryption::encrypted_data::EncryptedData;
use crate::prelude::{DBPacketResponseError, DBSuccessResponse};

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

    pub fn encrypt_packet(&mut self, packet: &String, client_pub_key: &RsaPublicKey) -> Result<EncryptedData,EncryptionError> {
        let encrypted_data = self.encrypt(client_pub_key,packet.as_bytes()).map_err(|err| EncryptionError::RSAError(err))?;
        let enc_struct = EncryptedData::new(encrypted_data.as_slice());
        Ok(enc_struct)
    }

    /// Decrypt a packet send from the client to the server on the server side
    pub fn decrypt_client_packet(&self,client_packet: &EncryptedData) -> Result<DBPacket, EncryptionError> {

        let msg = decrypt(&self.pri_key, client_packet.get_data()).map_err(|err| EncryptionError::RSAError(err))?;
        match serde_json::from_slice::<DBPacket>(&msg) {
            Ok(packet) => {
                Ok(packet)
            }
            Err(_) => {
                Err(EncryptionError::SerializationError)
            }
        }
    }

    /// Decrypt data using the servers private key encrypted with the servers public key
    /// This function is used when decrypting data sent from client -> server
    pub fn decrypt(&self, enc_data: &[u8]) -> rsa::Result<Vec<u8>> {
        crate::encryption::decrypt(&self.pri_key,enc_data)
    }

}