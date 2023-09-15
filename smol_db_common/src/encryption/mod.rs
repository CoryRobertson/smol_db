//! Encryption module for smol_db, used in smol_db_client and smol_db_server

pub(self) const BIT_LENGTH: usize  = 2048;
pub mod server_encrypt;
pub mod client_encrypt;

pub enum EncryptionError {
    SerializationError,
    RSAError(rsa::Error),
}
