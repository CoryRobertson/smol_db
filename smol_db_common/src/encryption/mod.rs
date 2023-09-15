//! Encryption module for smol_db, used in smol_db_client and smol_db_server

use rsa::rand_core::OsRng;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use crate::db_packets::db_packet::DBPacket;
use crate::encryption::encrypted_data::EncryptedData;

pub(self) const BIT_LENGTH: usize  = 2048;
pub mod server_encrypt;
pub mod client_encrypt;
pub mod encrypted_data;


#[derive(Debug)]
pub enum EncryptionError {
    SerializationError,
    RSAError(rsa::Error),
}

pub fn encrypt(key: &RsaPublicKey, mut rng: &mut OsRng, msg: &[u8]) -> rsa::Result<Vec<u8>> {
    key.encrypt(&mut rng,Pkcs1v15Encrypt,msg)
}
pub fn decrypt(pri_key: &RsaPrivateKey, enc_data: &[u8]) -> rsa::Result<Vec<u8>> {
    pri_key.decrypt(Pkcs1v15Encrypt,enc_data)
}

pub fn decrypt_packet(packet: &EncryptedData, key: &RsaPrivateKey) -> Result<DBPacket, EncryptionError> {

    let msg = decrypt(key,packet.get_data()).map_err(|err| EncryptionError::RSAError(err))?;
    match serde_json::from_slice::<DBPacket>(&msg) {
        Ok(packet) => {
            Ok(packet)
        }
        Err(_) => {
            Err(EncryptionError::SerializationError)
        }
    }


}