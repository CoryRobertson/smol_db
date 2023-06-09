use crate::db_data::DBData;
use crate::db_packets::db_location::DBLocation;
use crate::db_packets::db_packet_info::DBPacketInfo;
use crate::db_packets::db_settings::DBSettings;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A packet denoting the operation from client->server that the client wishes to do.
pub enum DBPacket {
    /// Read(db to operate on, key to read the db using)
    Read(DBPacketInfo, DBLocation),
    /// Write(db to operate on, key to write to the db using, data to write to the key location)
    Write(DBPacketInfo, DBLocation, DBData),
    /// CreateDB(db to create)
    CreateDB(DBPacketInfo, DBSettings),
    /// DeleteDB(db to delete)
    DeleteDB(DBPacketInfo),
    /// ListDB
    ListDB,
    /// ListDBContents(db to read from)
    ListDBContents(DBPacketInfo),

    //TODO: ChangeDBSetting takes a DBPacketInfo and a new DBSettings and replaces the old one.

    //TODO: SetAccessKey(hash string) sets the users access key, a hash of their password that is hashed from the client and sent to the server.
}

impl DBPacket {
    /// Creates a new Read DBPacket from a name of a database and location string to read from.
    pub fn new_read(dbname: &str, location: &str) -> DBPacket {
        DBPacket::Read(DBPacketInfo::new(dbname), DBLocation::new(location))
    }

    /// Creates a new Write DBPacket from a name of a database and location string to write to.
    pub fn new_write(dbname: &str, location: &str, data: &str) -> DBPacket {
        DBPacket::Write(
            DBPacketInfo::new(dbname),
            DBLocation::new(location),
            DBData::new(data.to_string()),
        )
    }

    /// Creates a new CreateDB DBPacket from a name of a database.
    pub fn new_create_db(dbname: &str, invalidation_time: Duration) -> DBPacket {
        DBPacket::CreateDB(
            DBPacketInfo::new(dbname),
            DBSettings::new(invalidation_time),
        )
    }

    /// Creates a new DeleteDB DBPacket from a name of a database.
    pub fn new_delete_db(dbname: &str) -> DBPacket {
        DBPacket::DeleteDB(DBPacketInfo::new(dbname))
    }

    pub fn new_list_db() -> DBPacket {
        DBPacket::ListDB
    }

    pub fn new_list_db_contents(db_name: &str) -> DBPacket {
        DBPacket::ListDBContents(DBPacketInfo::new(db_name))
    }

    /// Serializes a DBPacket into a string to be sent over the internet.
    pub fn serialize_packet(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }

    pub fn deserialize_packet(buf: &[u8]) -> serde_json::Result<Self> {
        serde_json::from_slice(buf)
    }
}
