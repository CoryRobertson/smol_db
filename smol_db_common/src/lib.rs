

use std::collections::HashMap;
use serde::{Deserialize, Serialize};


#[derive(Serialize,Deserialize,Debug,Clone, Hash, PartialEq, Eq,PartialOrd,Ord)]
pub struct DBPacketInfo {
    dbname: String,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct DBLocation {
    location: String,
}

impl DBLocation {
    pub fn new(location: &str) -> Self {
        Self{ location: location.to_string() }
    }
    pub fn as_key(&self) -> &str {
        &self.location
    }
}

impl DBPacketInfo {
    pub fn new(dbname: &str) -> Self {
        Self{ dbname: dbname.to_string() }
    }
    pub fn get_db_name(&self) -> &str {
        &self.dbname
    }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum DBPacket {
    Read(DBPacketInfo, DBLocation),
    Write(DBPacketInfo, DBLocation),
    CreateDB(DBPacketInfo),
    DeleteDB(DBPacketInfo),
}

impl DBPacket {
    pub fn new_read(dbname: &str, location: &str) -> DBPacket {
        DBPacket::Read(DBPacketInfo::new(dbname),DBLocation::new(location))
    }
    pub fn new_write(dbname: &str, location: &str) -> DBPacket {
        DBPacket::Write(DBPacketInfo::new(dbname),DBLocation::new(location))
    }
    pub fn new_create_db(dbname: &str) -> DBPacket {
        DBPacket::CreateDB(DBPacketInfo::new(dbname))
    }
    pub fn new_delete_db(dbname: &str) -> DBPacket {
        DBPacket::DeleteDB(DBPacketInfo::new(dbname))
    }

    pub fn serialize_packet(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct DBContent {
    pub content: HashMap<String,String>,
}

impl DBContent {
    pub fn read_ser_data(data: String) -> serde_json::Result<Self> {
        serde_json::from_str(&data)
    }
    pub fn read_from_db(&self, key: &str) -> Option<&String> {
        self.content.get(key)
    }
}

#[allow(clippy::derivable_impls)] // This lint is allowed so we can later make default not simply have the default impl
impl Default for DBContent{
    fn default() -> Self {
        Self{ content: HashMap::default() }
    }
}
