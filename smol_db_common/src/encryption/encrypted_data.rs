use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize, Clone, Debug)]
pub struct EncryptedData {
    data: Vec<u8>,
}

impl EncryptedData {
    pub fn new(data: &[u8]) -> Self {
        Self { data: data.to_vec() }
    }
    pub fn get_data(&self) -> &[u8] {
        self.data.as_slice()
    }
}