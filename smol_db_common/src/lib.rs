use serde::{Deserialize, Serialize};



#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct DBPacketInfo {
    dbname: String,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct DBLocation {
    location: String,
}

impl DBLocation {
    fn new(location: &str) -> Self {
        Self{ location: location.to_string() }
    } 
}

impl DBPacketInfo {
    fn new(dbname: &str) -> Self {
        Self{ dbname: dbname.to_string() }
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