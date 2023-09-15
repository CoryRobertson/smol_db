//! Client encryption module

use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use rsa::rand_core::OsRng;
use crate::db_packets::db_packet::DBPacket;
use crate::encryption::BIT_LENGTH;

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

    pub fn encrypt_packet(&mut self, packet: &DBPacket) {
        let serialized_data = packet.serialize_packet()?;
    }

    /// Decrypt data sent from the server encrypted with client public key using client private key
    /// This function is used when decrypting data sent from server -> client
    pub fn decrypt(&self, msg: &[u8]) -> rsa::Result<Vec<u8>> {
        self.pri_key.decrypt(Pkcs1v15Encrypt,msg)
    }

    /// Encrypt data to be sent to the server using the servers public key
    /// This function is used when encrypting data sent from client -> server
    pub fn encrypt(&mut self, msg: &[u8]) -> rsa::Result<Vec<u8>> {
        self.server_pub_key.encrypt(&mut self.rng,Pkcs1v15Encrypt,msg)
    }

}