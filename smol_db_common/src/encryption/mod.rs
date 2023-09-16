//! Encryption module for smol_db, used in smol_db_client and smol_db_server

use rsa::rand_core::OsRng;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

/// The length of bits an rsa key will be
pub(self) const BIT_LENGTH: usize = 2048;
pub mod client_encrypt;
pub mod encrypted_data;
pub mod server_encrypt;

#[derive(Debug)]
/// Error enum detailing types of encryption error
pub enum EncryptionError {
    SerializationError,
    RSAError(rsa::Error),
}

/// Encrypt a piece of data using a public key
pub(self) fn encrypt(key: &RsaPublicKey, mut rng: &mut OsRng, msg: &[u8]) -> rsa::Result<Vec<u8>> {
    key.encrypt(&mut rng, Pkcs1v15Encrypt, msg)
}

/// Decrypt a piece of data using a private key
pub(self) fn decrypt(pri_key: &RsaPrivateKey, enc_data: &[u8]) -> rsa::Result<Vec<u8>> {
    pri_key.decrypt(Pkcs1v15Encrypt, enc_data)
}
