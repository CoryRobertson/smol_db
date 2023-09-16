//! Encryption module for smol_db, used in smol_db_client and smol_db_server

use rsa::rand_core::OsRng;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

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
// response type from server to client -> Result<DBSuccessResponse<String>, ClientError>
// response type from client to server -> DBPacket

