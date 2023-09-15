//! Server encryption module
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use rsa::rand_core::OsRng;
use crate::encryption::BIT_LENGTH;

pub struct ServerKey{
    pri_key: RsaPrivateKey,
    pub_key: RsaPublicKey,
    rng: OsRng,
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
    pub fn encrypt(&mut self,client_pub_key: RsaPublicKey ,msg: &[u8]) -> rsa::Result<Vec<u8>> {
        client_pub_key.encrypt(&mut self.rng,Pkcs1v15Encrypt,msg)
    }

    /// Decrypt data using the servers private key encrypted with the servers public key
    /// This function is used when decrypting data sent from client -> server
    pub fn decrypt(&self, enc_data: &[u8]) -> rsa::Result<Vec<u8>> {
        self.pri_key.decrypt(Pkcs1v15Encrypt,enc_data)
    }

}